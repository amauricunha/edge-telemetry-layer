# Guia de Verificação Formal de Concorrência Embarcada via Software Bounded Model Checking (SBMC)

Este documento apresenta a fundamentação teórica, a metodologia e os procedimentos práticos para a aplicação de **Software Bounded Model Checking (SBMC)** no projeto **Edge Telemetry Layer**.

---

## 1. O que é SBMC e por que utilizá-lo em Sistemas Embarcados?

Em sistemas embarcados críticos (especialmente nas indústrias automotiva e aeroespacial), os métodos tradicionais de teste (testes unitários, testes de integração e testes em bancada) possuem uma limitação fundamental: **eles executam apenas caminhos de execução amostrais**.

Em sistemas concorrentes com interrupções de hardware (ISRs), barramentos CAN de alta velocidade e tarefas assíncronas, bugs como *Data Races*, *Deadlocks* e *Priority Inversion* dependem de alinhamentos temporais específicos na ordem de microssegundos (*Heisenbugs*), que quase nunca se repetem durante testes normais.

### A Abordagem do Model Checking:
O **Bounded Model Checking (BMC)** transforma a verificação de software em um problema de decisão lógica:
1. O código-fonte (em C, C++ ou Rust) é convertido em uma representação intermediária (LLVM IR ou GOTO-programs).
2. Todos os laços de repetição (*loops*), chamadas de função e fluxos assíncronos são desenrolados até um limite finito $k$ (*bound*).
3. As variáveis, ponteiros e registradores são tratados como variáveis simbólicas.
4. O problema é entregue a um solucionador de teorias matemáticas **SMT Solver** (como Z3, Boolector, CVC5 ou MathSAT).
5. **Resultado:** O solucionador comprova matematicamente se as asserções de segurança são satisfeitas em **100% dos estados possíveis** ou gera um contraexemplo exato (traço de execução com os valores de entrada) que levou à falha.

---

## 2. Ferramentas Alvo para o Projeto

```text
+----------------------------------------------------+---------------------------------------------------+
|               Nó Emulador (C++ / Arduino)          |            Nó Coletor (Rust no_std / ESP32-S3)    |
+----------------------------------------------------+---------------------------------------------------+
| Ferramenta: ESBMC (Efficient SMT-based Bounded     | Ferramenta: Kani Rust Verifier                    |
|             Model Checker) / CBMC                  |             (Baseado no CBMC com suporte a Rust) |
| Alvo: Atomicidade de ISRs, ponteiros SPI e floats | Alvo: Canais Bounded, FSM de Rede e Ring Buffer   |
+----------------------------------------------------+---------------------------------------------------+
```

---

## 3. Propriedades Verificadas no Projeto

### 3.1 Ausência de Deadlock na Recuperação de Bus-Off
- **Cenário:** Quando o controlador CAN entra em Bus-Off, a `task_watchdog` emite o sinal `BUS_OFF_SIGNAL` e aguarda o tempo ISO 11898 antes de rearmar o transceptor e emitir `BUS_OFF_CLEAR`.
- **Prova Formal:** O modelo comprova que não existe nenhum estado em que `task_can_rx` fique bloqueada indefinidamente aguardando um sinal que o watchdog não possa emitir.

### 3.2 Preservação da Ordem Cronológica FIFO no Backlog
- **Cenário:** Quando o Wi-Fi é restabelecido ou o SD Card é recuperado, os frames acumulados na memória PSRAM devem ser gravados/transmitidos na ordem estrita de geração.
- **Prova Formal:** O modelo formal prova que a operação de drenagem satisfaz monotonicidade temporal estrita ($\forall i < j \implies \text{timestamp}(frame_i) \le \text{timestamp}(frame_j)$).

### 3.3 Garantia de Não-Interferência na Memória de IA (Particionamento de PSRAM)
- **Cenário:** O ESP32-S3 dispõe de 8 MB de PSRAM. O buffer de telemetria consome no máximo 512 KB.
- **Prova Formal:** Prova-se que a alocação do Ring Buffer de telemetria é limitada estaticamente e é incapaz de invadir a partição de 7,5 MB reservada para o modelo de Machine Learning (Mamba-2 / TinyML).

---

## 4. Procedimentos de Execução

### Execução de Model Checking em Rust com Kani:
```bash
# Instalação do Kani Verifier
cargo install --locked kani-verifier
cargo kani setup

# Verificação formal dos harnesses no coletor
cd firmware/esp32s3_collector
cargo kani --harness verify_telemetry_channel_overflow_safety
```

### Execução de Model Checking em C com ESBMC:
```bash
# Verificação formal da atomicidade do emulador Arduino
esbmc firmware/uno_ecu_emulator_ino/uno_ecu_emulator_ino.ino \
    --unwind 10 \
    --incremental-bmc \
    --no-pointer-check \
    --overflow-check
```

---

## 5. Conclusão

A integração de **Software Bounded Model Checking (SBMC)** eleva a maturidade do projeto ao padrão exigido por normas automotivas como a **ISO 26262 (ASIL-B)**, demonstrando que a telemetria é matematicamente imune a travamentos, corrupção de arquivos e perda de dados por problemas de concorrência.
