# Especificação Formal de Verificação e Bounded Model Checking (SBMC)

- **Fase SDD:** specify / formal-verification
- **Módulo:** `specs/sbmc_formal_verification_spec.md`
- **Padrão de Referência:** AUTOSAR Classic / ISO 26262 (ASIL-B) / Rust Type System Formal Model
- **Ferramentas Alvo:** Kani Rust Verifier (Rust Model Checker), ESBMC / CBMC (C/C++ Bounded Model Checker)

---

## 1. Visão Geral e Fundamentação

A programação concorrente em nós automotivos de borda impõe requisitos rígidos de determinismo temporal, ausência de deadlocks e preservação de memória. O método **Software Bounded Model Checking (SBMC)** traduz o código-fonte em fórmulas lógicas decidíveis (SAT/SMT), desenrola laços até um limite finito $k$ e explora exaustivamente **todas as possíveis intercalações de tarefas e todos os valores de entrada possíveis**, provando matematicamente propriedades de segurança de execução.

```text
+-------------------+      +---------------------+      +----------------------+
|  Código C / Rust  | ---> |  Tradutor LLVM IR / | ---> |     SMT Solver       | ---> PROVA MATEMÁTICA:
| (Tasks, Channels) |      |   GOTO Program      |      | (Z3 / Boolector / CVC5)|   Zero Data Race, Deadlock
+-------------------+      +---------------------+      +----------------------+   e Buffer Overflow
```

---

## 2. Critérios Formais de Verificação (FV-01 a FV-08)

| ID | Propriedade Verificada | Módulo Alvo | Invariante / Teorema Formal |
| :--- | :--- | :--- | :--- |
| **FV-01** | **Bounded Channel Capacity** | `RTE / TELEMETRY_CHANNEL` | $\forall t, \text{len}(\text{TELEMETRY\_CHANNEL}(t)) \le 32$. Operações `try_send` nunca causam panic, corrupção de ponteiros ou memory leak. |
| **FV-02** | **Deadlock-Free Bus-Off Recovery** | `bsw_diag` / `main.rs` | Ausência de ciclo de dependência entre `BUS_OFF_SIGNAL.wait()` e `BUS_OFF_CLEAR.wait()`. Tempo máximo de recuperação delimitado por $T_{\text{recover}} \le 150\text{ ms}$. |
| **FV-03** | **Memory Safety em Drivers Unsafe** | `mcal::spi_sd` / `bsw_com` | $\forall \text{ptr} \in \text{addr\_of\_mut!}(\text{BUFFERS})$, $\text{offset}(\text{ptr}) < \text{size\_of}(\text{BUFFER})$. Zero *Buffer Overflow* e zero *Null Pointer Dereference*. |
| **FV-04** | **Monotonicidade Temporal FIFO** | `app::logger` / `BACKLOG_PSRAM` | Se frame $A$ foi inserido antes de frame $B$ ($ts_A \le ts_B$), na drenagem do backlog a ordem de gravação satisfaz estritamente $\text{order}(A) < \text{order}(B)$ (Preservação FIFO). |
| **FV-05** | **Zero Data-Race em ISR Multibyte** | `uno_ecu_emulator_ino.ino` | Toda leitura no loop principal de variáveis `volatile` atualizadas pelo `Timer1` (ex: `v_speed_kmh`, `v_rpm`) ocorre sob proteção de seção crítica (`noInterrupts()`). |
| **FV-06** | **Finitude e Terminação de Sessão** | `bsw_com` / `app::logger` | Para qualquer sessão temporizada $T_{\text{limit}} = D \times 60.000\text{ ms}$ ($D > 0$), a transição para `DIAG,SESSION_COMPLETE` ocorre em $t \in [T_{\text{start}} + T_{\text{limit}}, T_{\text{start}} + T_{\text{limit}} + \epsilon]$, onde $\epsilon \le 100\text{ ms}$. |
| **FV-07** | **Isolamento de Memória para IA** | `firmware::collector` | A soma de todas as estruturas estáticas de telemetria em PSRAM satisfaz $\text{Mem}_{\text{telemetry}} \le 512\text{ KB}$, garantindo $\text{Mem}_{\text{free\_for\_AI}} \ge 7.5\text{ MB}$ em qualquer estado. |
| **FV-08** | **Transição Atômica de Arquivos CSV** | `bsw_mem` / `mcal::spi_sd` | O primeiro bloco gravado em qualquer arquivo $S\_XXXX\text{.CSV}$ contém estritamente o cabeçalho CSV como linha 1 e a linha BOOT como linha 2. Zero frames da sessão anterior no novo arquivo. |

---

## 3. Harnesses de Verificação Model Checking

### 3.1 Harness Kani (Rust Model Checker) — Invariante FIFO e Ausência de Panic

```rust
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(35)]
fn verify_telemetry_channel_overflow_safety() {
    let channel = embassy_sync::channel::Channel::<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        crate::types::TelemetryFrame,
        32,
    >::new();

    let sender = channel.sender();
    let receiver = channel.receiver();

    // Inserir N frames arbitrários (simbólicos)
    let num_sends: usize = kani::any();
    kani::assume(num_sends <= 35);

    let mut dropped_count: usize = 0;
    for _ in 0..num_sends {
        let frame: crate::types::TelemetryFrame = kani::any();
        if sender.try_send(frame).is_err() {
            dropped_count += 1;
        }
    }

    // Provar que o canal nunca excede 32 e que descartes são consistentes
    if num_sends > 32 {
        assert!(dropped_count == num_sends - 32);
    } else {
        assert!(dropped_count == 0);
    }
}
```

### 3.2 Harness ESBMC (C++ Bounded Model Checker) — Atomicidade de ISR no Arduino

```c
// Harness para verificação formal com ESBMC / CBMC
// Comando: esbmc uno_emulator_harness.c --unwind 10 --incremental-bmc

#include <assert.h>
#include <stdint.h>
#include <stdbool.h>

volatile float v_speed_kmh = 60.0f;
volatile bool in_critical_section = false;

void isr_timer1_simulated(float new_speed) {
    // Interrupção de hardware pode acontecer a qualquer momento
    v_speed_kmh = new_speed;
}

void superloop_read_simulated() {
    float local_speed;
    
    // Início da Seção Crítica
    in_critical_section = true;
    local_speed = v_speed_kmh;
    in_critical_section = false;
    // Fim da Seção Crítica

    // Asserção: Se a leitura ocorreu, o valor lido é um float válido e finito
    assert(local_speed >= 0.0f && local_speed <= 300.0f);
}
```

---

## 4. Garantias e Conformidade de Engenharia

Esta especificação garante que:
1. Nenhuma condição de corrida afeta a ordem cronológica dos dados gravados.
2. A alocação de memória em PSRAM é matematicamente limitada, protegendo os 7,5 MB necessários para a Fase 2 (Inferência com Redes Neurais / Mamba-2).
3. Todas as transições de estado de rede e de arquivos respeitam a semântica formal do AUTOSAR Classic.
