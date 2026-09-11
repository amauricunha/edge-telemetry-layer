# SPEC — Fase 1: Arquitetura de Aquisição de Telemetria CAN/OBD-II em Bancada

**Projeto:** Edge Telemetry Layer — Telemetry Layer  
**Fase:** F1 — Bancada CAN + OBD-II  
**Equipe:** Amauri Cunha · Henrique Alves · Larissa Lima · Wissam Melo  
**Orientador:** Prof. Anderson Bittar  
**Versão:** 1.0  
**Status:** 🟡 Em desenvolvimento  

---

## Índice

1. [Objetivo e Escopo](#1-objetivo-e-escopo)
2. [Arquitetura do Sistema](#2-arquitetura-do-sistema)
3. [Módulo 1 — Emulador de ECU (Arduino UNO R3)](#3-módulo-1--emulador-de-ecu-arduino-uno-r3)
4. [Módulo 2 — Nó de Coleta (ESP32-S3)](#4-módulo-2--nó-de-coleta-esp32-s3)
5. [Protocolo de Comunicação CAN](#5-protocolo-de-comunicação-can)
6. [Estrutura do Dataset Gerado](#6-estrutura-do-dataset-gerado)
7. [Critérios de Aceitação (Gate F1)](#7-critérios-de-aceitação-gate-f1)
8. [Setup de Hardware](#8-setup-de-hardware)
9. [Estrutura de Repositório](#9-estrutura-de-repositório)
10. [Dependências e Toolchain](#10-dependências-e-toolchain)
11. [Plano de Tarefas](#11-plano-de-tarefas)
12. [Riscos e Mitigações](#12-riscos-e-mitigações)

---

## 1. Objetivo e Escopo

### 1.1 O que a Fase 1 entrega

A Fase 1 entrega a **camada de aquisição e estruturação de dados** do projeto. Ao final, o sistema deve ser capaz de:

- Emular uma ECU automotiva em bancada (Arduino UNO R3), transmitindo frames CAN conforme especificação DBC e respondendo a solicitações OBD-II
- Coletar esses dados no ESP32-S3 via barramento CAN a 500 Kbps
- Estruturá-los em CSV com carimbo de tempo e metadados de sessão
- Persistir localmente no SD Card (fallback offline) e publicar via MQTT quando Wi-Fi disponível
- Gerar um dataset validado que serve de base para as fases futuras.

### 1.2 O que está **fora** do escopo da Fase 1

| Fora de escopo | Fase planejada |
|---|---|
| Inferência / TinyML | F3 |
| ISO 26262 | F3 |
| Diver Coaching | F3/F4 |
| Validação em veículo real (Renault Sandero) | F5 |
| Fault injection / HIL avançado | F4 |

### 1.3 Relação com o artigo eTech

O artigo submetido à Revista eTech SENAI **documenta exclusivamente esta fase**. As fases F2–F5 aparecem apenas na seção de Trabalhos Futuros de forma genérica.

---

## 2. Arquitetura do Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                        BANCADA HIL F1                           │
│                                                                 │
│  ┌──────────────────┐   CAN 500 Kbps   ┌───────────────────┐  │
│  │  Arduino UNO R3  │◄────────────────►│    ESP32-S3       │  │
│  │  + MCP2515       │                  │    (TWAI nativo)  │  │
│  │  + TJA1050       │                  │                   │  │
│  │                  │                  │  ┌─────────────┐  │  │
│  │  • Emite DBC     │                  │  │ Embassy     │  │  │
│  │  • Responde      │                  │  │ async tasks │  │  │
│  │    0x7DF→0x7E8   │                  │  └──────┬──────┘  │  │
│  │  • Recebe 0x010  │                  │         │         │  │
│  └──────────────────┘                  │  ┌──────▼──────┐  │  │
│                                        │  │  SD Card    │  │  │
│                                        │  │  (FAT32)    │  │  │
│                                        │  └─────────────┘  │  │
│                                        │         │         │  │
│                                        │  ┌──────▼──────┐  │  │
│                                        │  │  Wi-Fi /    │  │  │
│                                        │  │  MQTT       │  │  │
│                                        │  └─────────────┘  │  │
│                                        └───────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.1 Princípios de design

- **Separação de responsabilidades:** cada módulo tem uma função única e bem definida
- **Fail-safe por default:** se Wi-Fi falhar, dados vão para SD. Se SD falhar, log de erro e continuidade
- **Sem alocação dinâmica em runtime:** todos os buffers são estáticos (`heapless::Vec`, arrays com tamanho em compile-time)
- **Testabilidade por camadas:** cada camada pode ser testada isoladamente com mocks

---

## 3. Módulo 1 — Emulador de ECU (Arduino UNO R3)

### 3.1 Responsabilidades

| Função | Descrição |
|---|---|
| Emissão passiva DBC | Transmite frames CAN periódicos simulando ECU real |
| Resposta OBD-II | Monitora 0x7DF e responde com PIDs no 0x7E8 |
| Troca de perfil | Recebe ID 0x010 e altera os valores simulados |

### 3.2 Frames CAN emitidos (especificação DBC mínima)

| CAN ID | Sinal | Período | Bytes | Faixa de valores |
|---|---|---|---|---|
| `0x100` | `VEHICLE_SPEED` | 100 ms | 2 | 0–250 km/h |
| `0x100` | `ENGINE_RPM` | 100 ms | 2 | 0–8000 rpm |
| `0x200` | `THROTTLE_POS` | 50 ms | 1 | 0–100% |
| `0x200` | `ENGINE_LOAD` | 50 ms | 1 | 0–100% |
| `0x300` | `COOLANT_TEMP` | 1000 ms | 1 | -40–215 °C (offset +40) |

#### Perfis de simulação

Ativados via comando CAN ID `0x010`, byte 0:

| Byte 0 | Perfil | Velocidade | RPM | Throttle |
|---|---|---|---|---|
| `0x01` | Econômico | 40–80 km/h senoidal | 1200–2500 | 10–30% |
| `0x02` | Normal | 60–120 km/h senoidal | 2000–4000 | 20–60% |
| `0x03` | Esportivo | 80–160 km/h senoidal | 3500–6500 | 50–100% |

> **Implementação:** usar tabelas de lookup senoidais pré-calculadas em `PROGMEM` para evitar `sin()` em runtime. O perfil define amplitude e offset da senoide.

### 3.3 Resposta OBD-II (ISO 15765-4)

O emulador escuta o ID funcional `0x7DF` e responde no `0x7E8`.

**Formato da solicitação (8 bytes):**
```
Byte 0: comprimento (0x02)
Byte 1: modo (0x01 = dados em tempo real)
Byte 2: PID solicitado
Bytes 3-7: 0x00 (padding)
```

**Formato da resposta (8 bytes):**
```
Byte 0: comprimento (0x03 ou 0x04 conforme PID)
Byte 1: modo + 0x40 (ex: 0x41)
Byte 2: PID espelhado
Byte 3[+4]: valor codificado conforme fórmula do PID
Bytes restantes: 0x00 (padding)
```

**PIDs suportados pelo emulador:**

| PID | Parâmetro | Fórmula de codificação | Bytes de dados |
|---|---|---|---|
| `0x04` | Engine Load | `A = load_pct * 255 / 100` | 1 |
| `0x05` | Coolant Temp | `A = temp_celsius + 40` | 1 |
| `0x0C` | Engine RPM | `A = (rpm * 4) >> 8`, `B = (rpm * 4) & 0xFF` | 2 |
| `0x0D` | Vehicle Speed | `A = speed_kmh` | 1 |
| `0x10` | MAF Air Flow Rate | `A = (maf_g_s * 100) >> 8`, `B = (maf_g_s * 100) & 0xFF` | 2 |
| `0x11` | Throttle Position | `A = throttle_pct * 255 / 100` | 1 |

### 3.4 Alocação de timers (ATmega328P)

| Timer | Uso | Período | Justificativa |
|---|---|---|---|
| **Timer1** (16-bit) | Geração de valores senoidais + emissão DBC | 50 ms (20 Hz) | Frequência suficiente para simular dinâmica veicular |
| **Timer2** (8-bit) | Polling de RX no MCP2515 (0x7DF + 0x010) | 1 ms | Prioridade máxima; tempo de resposta OBD < 10 ms |
| **loop()** | Resposta OBD-II após detecção | — | Executa após sinalização por flag da ISR do Timer2 |

> **Regra de ouro:** Timer2 tem prioridade de interrupção maior que Timer1. Nunca bloquear dentro de uma ISR por mais de 50 µs.

### 3.5 Firmware UNO R3 — pseudocódigo

```cpp
// Arquivo: uno_ecu_emulator/src/main.cpp

// --- Variáveis globais de estado ---
volatile uint8_t  perfil_atual = PERFIL_NORMAL;
volatile float    speed_kmh    = 60.0;
volatile float    rpm          = 2000.0;
volatile uint8_t  throttle_pct = 30;
volatile uint8_t  load_pct     = 25;
volatile float    maf_g_s      = 4.5;  // g/s (idle ~2–4, WOT ~50–80)
volatile bool     obd_request_pending = false;
volatile uint8_t  obd_pid_requested   = 0x00;

// --- ISR Timer2 (1 ms): polling CAN RX ---
ISR(TIMER2_COMPA_vect) {
    if (mcp2515.checkReceive()) {
        CAN_FRAME frame;
        mcp2515.readMsgBuf(&frame);
        if (frame.id == 0x7DF) {
            obd_pid_requested   = frame.data[2];
            obd_request_pending = true;
        }
        if (frame.id == 0x010) {
            perfil_atual = frame.data[0];
        }
    }
}

// --- ISR Timer1 (50 ms): atualização de valores e emissão DBC ---
ISR(TIMER1_COMPA_vect) {
    atualizar_valores_senoide(perfil_atual);
    emitir_frame_can(0x100, encode_speed_rpm(speed_kmh, rpm));
    emitir_frame_can(0x200, encode_throttle_load(throttle_pct, load_pct));
}

// --- loop(): resposta OBD-II ---
void loop() {
    if (obd_request_pending) {
        obd_request_pending = false;
        uint8_t resposta[8] = {0};
        montar_resposta_obd(obd_pid_requested, resposta);
        mcp2515.sendMsgBuf(0x7E8, 0, 8, resposta);
    }
}
```

### 3.6 Limitações documentadas do UNO R3

| Limitação | Valor esperado | Impacto | Aceitabilidade |
|---|---|---|---|
| Jitter de resposta OBD-II | ±1–3 ms | Variação no timestamp de resposta | ✅ Aceitável para bancada HIL |
| Fidelidade senoidal | ~8 bits de resolução | Valores discretizados em steps | ✅ Suficiente para simular perfis de condução |
| Overhead de CPU a 500 Kbps | ~60–70% | Sem margem para tarefas adicionais | ✅ Documentado; não adicionar funcionalidades |
| Upgrade path (F5) | — | Para validação em veículo real, substituir por STM32F103 (bxCAN nativo, 72 MHz) | — |

---

## 4. Módulo 2 — Nó de Coleta (ESP32-S3)

### 4.1 Linguagem e runtime

- **Linguagem:** Rust (stable, `no_std` onde possível)
- **Runtime:** [Embassy](https://embassy.dev) — async/await para sistemas embarcados
- **Target:** `xtensa-esp32s3-none-elf`
- **HAL:** `esp-hal` 0.18+

### 4.2 Camadas de software (AUTOSAR-inspired)

```
┌─────────────────────────────────────────────┐
│  APPLICATION LAYER                          │
│  ┌─────────────────┐  ┌──────────────────┐  │
│  │  obd_poller     │  │  session_logger  │  │
│  │  (crate::obd)   │  │  (crate::logger) │  │
│  └────────┬────────┘  └────────┬─────────┘  │
├───────────┼────────────────────┼────────────┤
│  RTE (Runtime Environment)     │            │
│  embassy::channel<T, 32>       │            │
│  ┌─────────────────────────────▼──────────┐ │
│  │  Channel<TelemetryFrame, 32>           │ │
│  └──────────────────────┬─────────────────┘ │
├─────────────────────────┼────────────────────┤
│  BSW (Basic Software)   │                   │
│  ┌──────────┐  ┌────────▼──────┐  ┌───────┐ │
│  │ bsw_com  │  │  bsw_mem      │  │bsw_   │ │
│  │ (Wi-Fi   │  │  (SD FAT32    │  │diag   │ │
│  │  MQTT)   │  │   + fallback) │  │(CAN   │ │
│  └──────────┘  └───────────────┘  │recov) │ │
├────────────────────────────────────┴───────┤ │
│  MCAL (Microcontroller Abstraction Layer)  │ │
│  esp-hal: TWAI · SPI · GPIO · DMA         │ │
└───────────────────────────────────────────┘
```

### 4.3 Tarefas Embassy e responsabilidades

| Tarefa | Prioridade | Responsabilidade |
|---|---|---|
| `task_can_rx` | Alta | Lê frames do TWAI via DMA → publica em `channel_rx` |
| `task_obd_poller` | Média | A cada 100 ms envia `0x7DF` → aguarda `0x7E8` com timeout 50 ms |
| `task_logger` | Média | Consome `channel_rx` → serializa CSV → grava SD ou publica MQTT |
| `task_wifi` | Baixa | Gerencia conexão Wi-Fi; notifica `task_logger` sobre disponibilidade |
| `task_watchdog` | Máxima | Feed do watchdog HW a cada 5 s; se `task_logger` travar → reset controlado |

### 4.4 Fluxo de dados detalhado

```
TWAI (CAN HW)
     │
     ▼ IRQ / DMA
task_can_rx ──────────────────────────────────────────────────────────────►
     │                                                                     │
     │ TelemetryFrame { timestamp_ms, can_id, data[8] }                   │
     ▼                                                                     │
Channel<TelemetryFrame, 32>                                               │
     │                                                                     │
     ├──────────► task_logger                                              │
     │                 │                                                   │
     │                 ├── Wi-Fi OK? ──► MQTT publish("/telemetry/raw")    │
     │                 │                                                   │
     │                 └── Wi-Fi OFF ──► SD write(session_YYYYMMDD.csv)    │
     │                                                                     │
task_obd_poller ──────────────────────────────────────────────────────────┘
     │ (envia 0x7DF a cada 100 ms, resultado entra no mesmo channel)
     │
     └── timeout 50 ms → log OBD_TIMEOUT no SD, continua
```

### 4.5 Estrutura de módulos Rust

```
esp32s3-telemetry/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point; inicialização de periféricos; spawn de tasks
│   ├── types.rs             # TelemetryFrame, SessionConfig, OBD_PID enum
│   ├── mcal/
│   │   ├── mod.rs
│   │   ├── twai.rs          # Wrapper TWAI: init, send, recv via DMA
│   │   └── spi_sd.rs        # Wrapper SPI para SD Card
│   ├── bsw/
│   │   ├── mod.rs
│   │   ├── bsw_mem.rs       # SD Card: open/write/flush; buffer circular 4 KB
│   │   ├── bsw_com.rs       # Wi-Fi connect; MQTT client (rumqttc)
│   │   └── bsw_diag.rs      # Bus-Off detect; auto-recovery; watchdog feed
│   ├── app/
│   │   ├── mod.rs
│   │   ├── obd_poller.rs    # task_obd_poller: envio 0x7DF e parse 0x7E8
│   │   ├── logger.rs        # task_logger: serializa CSV, decide SD vs MQTT
│   │   └── csv_writer.rs    # Serialização de TelemetryFrame → linha CSV
│   └── config.rs            # Constantes: CAN_BAUDRATE, OBD_POLL_MS, SD_FLUSH_INTERVAL_S
```

### 4.6 Gerenciamento de memória e Particionamento para Edge AI

| Região | Alocação | Tamanho | Conteúdo |
|---|---|---|---|
| SRAM interna | Estática (compile-time) | ~58 KB heap + buffers | Stack Embassy + Channel RTE + buffers TWAI/SPI |
| PSRAM (8 MB) | Partição Telemetria | **512 KB** (~28.400 frames) | Ring Buffer FIFO com política *Drop-Oldest* na RAM |
| PSRAM (8 MB) | **Partição Edge AI (Fase 2)** | **> 7,5 MB (93,75%)** | **Isolada e reservada para modelos de ML (TinyML / Mamba-2)** |
| Flash (16 MB) | Firmware + DBC table | ~200–400 KB | Código binário em Rust `no_std` + tabelas |

> **Regra de Isolamento de Memória:** A telemetria é matematicamente delimitada a 512 KB, impedindo qualquer interferência ou *Out-Of-Memory* na partição de inferência de IA.

### 4.7 Transição Atômica de Sessões e Datetime Epoch Real

1. **Transição Atômica de Arquivos CSV:**
   - Ao receber comando de troca de sessão (`ECO`, `SPT`, `NOR`, `RESET`), o firmware executa `flush_sync()` imediato do `SD_BUFFER` no arquivo antigo, garantindo que o novo arquivo ($S\_XXXX\text{.CSV}$) inicie **estritamente com o Header CSV na linha 1 e a linha BOOT na linha 2**.
2. **Sincronização Datetime Epoch (ms):**
   - Suporte a timestamp Unix absoluto em milissegundos (`EPOCH_BASE_MS`), aceito nos comandos MQTT (ex: `ECO,10,1724498400000`) ou comando `TIME,epoch`.
   - Compatibilidade nativa com InfluxDB, Grafana e Telegraf.

### 4.8 Comandos Avançados de Bancada e Replay sob Demanda (MQTT)

| Comando (`/coach/command`) | Função | Resposta / Tópico |
| :--- | :--- | :--- |
| `RESET,ECO,10,[epoch]` | Formata o SD, purga buffers de RAM e inicia Sessão 1 (`S_0001.CSV`) | Publica `SESSION_START` em `/system/status` |
| `SPT,10` / `NOR,10` | Inicia nova sessão no modo indicado com timer de 10 min | Publica `SESSION_START` em `/system/status` |
| `STOP` | Interrompe imediatamente a sessão ativa com flush para o SD | Publica `SESSION_STOPPED` em `/system/status` |
| `LIST_SESSIONS` | Lista todos os arquivos de sessão existentes no SD Card | Publica JSON com lista em `/system/status` |
| `REPLAY,N` | Reenvia o histórico da sessão $N$ em blocos CSV delimitados por `\n` | Transmite no `/telemetry/replay` (backfilling no InfluxDB) |

### 4.9 Resiliência e Bus-Off Recovery (bsw_diag.rs)

```rust
// Lógica de auto-recovery de Bus-Off (erro TWAI 259)
pub async fn monitor_bus_off(twai: &mut TwaiDriver) {
    loop {
        if twai.is_bus_off() {
            log::warn!("TWAI Bus-Off detectado. Aguardando 128 ms...");
            Timer::after(Duration::from_millis(128)).await;
            twai.recover().await;  // re-habilita sem reiniciar o SoC
            log::info!("TWAI recuperado.");
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
```

### 4.10 Verificação Formal de Concorrência (SBMC)
O firmware do coletor e o emulador C++ são verificados formalmente via **Software Bounded Model Checking (SBMC)** conforme a especificação `specs/sbmc_formal_verification_spec.md` (critérios FV-01 a FV-08), provando matematicamente a ausência de *deadlocks*, *data races*, *buffer overflows* e garantindo a preservação da ordem FIFO.

---

## 5. Protocolo de Comunicação CAN

### 5.1 Parâmetros do barramento

| Parâmetro | Valor |
|---|---|
| Baudrate | 500 Kbps |
| Terminação | 2× 120 Ω (uma em cada ponta do barramento) |
| Protocolo | CAN 2.0B (frames de 11 bits de ID) |
| Transceiver UNO R3 | MCP2515 + TJA1050 |
| Transceiver ESP32-S3 | TWAI nativo (GPIO configurável) |

### 5.2 Mapeamento de CAN IDs

| ID | Direção | Conteúdo | Emissor |
|---|---|---|---|
| `0x100` | UNO R3 → ESP32-S3 | `[speed_H, speed_L, rpm_H, rpm_L, 0, 0, 0, 0]` | Emulador |
| `0x200` | UNO R3 → ESP32-S3 | `[throttle, load, 0, 0, 0, 0, 0, 0]` | Emulador |
| `0x300` | UNO R3 → ESP32-S3 | `[coolant_temp, 0, 0, 0, 0, 0, 0, 0]` | Emulador |
| `0x7DF` | ESP32-S3 → UNO R3 | Solicitação OBD-II funcional | Nó de coleta |
| `0x7E8` | UNO R3 → ESP32-S3 | Resposta OBD-II com PID solicitado | Emulador |
| `0x010` | ESP32-S3 → UNO R3 | `[perfil, 0, 0, 0, 0, 0, 0, 0]` | Nó de coleta |

### 5.3 Parser DBC no ESP32-S3

O parser extrai sinais dos frames CAN conforme a DBC mínima da Seção 3.2:

```rust
pub fn parse_frame(frame: &CanFrame) -> Option<TelemetrySignals> {
    match frame.id() {
        0x100 => Some(TelemetrySignals {
            speed_kmh:  ((frame.data[0] as u16) << 8 | frame.data[1] as u16) as f32,
            rpm:        ((frame.data[2] as u16) << 8 | frame.data[3] as u16) as f32,
            ..Default::default()
        }),
        0x200 => Some(TelemetrySignals {
            throttle_pct: frame.data[0] as f32 * 100.0 / 255.0,
            load_pct:     frame.data[1] as f32 * 100.0 / 255.0,
            ..Default::default()
        }),
        _ => None,
    }
}
```

---

## 6. Estrutura do Dataset Gerado

### 6.1 Formato do arquivo CSV

**Nome do arquivo:** `session_YYYYMMDD_HHMMSS.csv`  
**Localização no SD:** `/telemetry/YYYYMM/session_YYYYMMDD_HHMMSS.csv`  
**Encoding:** UTF-8, separador vírgula, sem BOM

### 6.2 Schema das colunas

> **Nota de escopo:** o dataset da Fase 1 cobre **6 variáveis OBD-II** acessíveis via polling padrão ISO 15765-4. Três variáveis adicionais originalmente planejadas — pressão de freio, ângulo do volante e aceleração lateral — dependem de IDs proprietários do barramento CAN do Renault Sandero (não padronizados pelo OBD-II) e serão incorporadas na Fase 5 (validação em veículo real), após engenharia reversa do DBC do veículo. Ver Seção 6.5.

| Coluna | Tipo | Unidade | PID OBD-II | Valores possíveis | Descrição |
|---|---|---|---|---|---|
| `timestamp_ms` | `uint64` | ms | — | 0 – ∞ | Milissegundos desde início da sessão |
| `source` | `string` | — | — | `CAN_DBC`, `OBD_PID` | Origem do dado |
| `can_id` | `string` | hex | — | `0x100`, `0x7E8`, etc. | CAN ID do frame de origem |
| `speed_kmh` | `float` | km/h | `0x0D` | 0.0 – 250.0 | Velocidade do veículo |
| `rpm` | `float` | rpm | `0x0C` | 0.0 – 8000.0 | Rotações por minuto do motor |
| `throttle_pct` | `float` | % | `0x11` | 0.0 – 100.0 | Posição do acelerador (intenção do motorista) |
| `engine_load_pct` | `float` | % | `0x04` | 0.0 – 100.0 | Carga calculada do motor |
| `maf_g_s` | `float` | g/s | `0x10` | 0.0 – 655.35 | Fluxo de massa de ar (proxy direto de consumo) |
| `coolant_temp_c` | `float` | °C | `0x05` | -40.0 – 215.0 | Temperatura do líquido de arrefecimento (contexto mecânico) |
| `session_label` | `string` | — | — | `Economico`, `Normal`, `Esportivo` | Perfil de condução ativo |
| `obd_latency_ms` | `float` \| `null` | ms | — | 0.0 – 999.9 \| `null` | Latência 0x7DF→0x7E8 (apenas linhas `OBD_PID`) |

### 6.3 Exemplo de linhas CSV

```csv
timestamp_ms,source,can_id,speed_kmh,rpm,throttle_pct,engine_load_pct,maf_g_s,coolant_temp_c,session_label,obd_latency_ms
0,CAN_DBC,0x100,60.5,2100.0,25.1,28.3,,,Normal,
100,CAN_DBC,0x200,60.5,2100.0,25.1,28.3,,,Normal,
200,OBD_PID,0x7E8,60.0,2100.0,25.5,28.0,12.4,87.0,Normal,4.2
300,OBD_PID,0x7E8,60.0,,,,8.1,,Normal,3.8
400,CAN_DBC,0x100,61.2,2150.0,26.0,29.1,,,Normal,
```

> **Leitura das linhas:** frames `CAN_DBC` populam apenas os campos decodificados do respectivo ID (0x100: speed+rpm; 0x200: throttle+load); os demais ficam vazios. Frames `OBD_PID` populam apenas o campo do PID respondido no ciclo (round-robin entre os 6 PIDs) — por isso `maf_g_s` e `coolant_temp_c` raramente aparecem na mesma linha.

### 6.4 Volume esperado do dataset (por cenário de 10 min)

| Fonte | Frequência | PIDs / ciclo | Amostras/min | Amostras em 10 min |
|---|---|---|---|---|
| CAN_DBC (0x100 + 0x200) | 10 Hz cada | — | 1.200 | 12.000 |
| OBD_PID (polling 0x7DF round-robin) | 10 Hz | 6 PIDs (0x0C,0x0D,0x11,0x04,0x10,0x05) | 600 | 6.000 |
| **Total por cenário** | — | — | — | **~18.000 linhas** |
| **Total 3 cenários** | — | — | — | **~54.000 linhas** |

> **Atualização em relação à versão anterior:** a adição do MAF (PID 0x10) ao round-robin de polling aumentou o volume de ~12.000 para ~18.000 amostras por cenário. O round-robin agora cicla por 6 PIDs em vez de 5, aumentando o período de retorno de cada PID de 500 ms para 600 ms — ainda dentro do budget de latência aceitável para análise de comportamento de condução.

Tamanho estimado em disco: ~8–12 MB por sessão completa (11 colunas vs. 10 anteriores).

### 6.5 Variáveis planejadas não incluídas na Fase 1 — justificativa técnica

As três variáveis abaixo constavam do planejamento original mas **não fazem parte do dataset da Fase 1**. A decisão é técnica, não de omissão, e está documentada aqui para rastreabilidade:

| Variável | Origem | Por que não está na F1 | Quando entra |
|---|---|---|---|
| Pressão / estado do freio | CAN Raw (módulo ABS/ESC) | ID proprietário do Sandero — não padronizado pelo OBD-II. Requer captura com analisador CAN no veículo real e engenharia reversa do DBC | Fase 5 (veículo real) |
| Ângulo do volante | CAN Raw (módulo EPS/ESP) | Idem — ID proprietário. Requer acesso ao DBC do Sandero ou documentação técnica do fabricante | Fase 5 (veículo real) |
| Aceleração lateral (G lateral) | CAN Raw (módulo ESP) | Idem — ID proprietário. Alternativa: sensor IMU externo (MPU-6050 via I2C) plugado ao ESP32-S3, independente do CAN | Fase 5 ou IMU externo em F1-bis |

**Impacto no modelo Mamba-2 (Fase 2):** o modelo será treinado sem aceleração lateral e ângulo de volante. Isso limita a detecção de manobras abruptas e curvas violentas. O MAF adicionado nesta revisão compensa parcialmente, fornecendo um proxy de consumo que complementa throttle e carga do motor. A Fase 5 expandirá o feature set com as variáveis CAN Raw do veículo real, permitindo fine-tuning do modelo.

---

## 7. Critérios de Aceitação (Gate F1)

O Gate F1 é a condição de saída que autoriza o início da Fase 2 (pipeline ML). **Todos os critérios devem ser atendidos.**

| ID | Critério | Threshold | Como verificar |
|---|---|---|---|
| **AC-01** | Taxa de recepção de frames CAN | ≥ 99% dos frames esperados | Contador de frames no firmware vs. frames teóricos no período |
| **AC-02** | Latência média de resposta OBD-II | < 10 ms | Timer de software ESP32-S3; análise do CSV coluna `obd_latency_ms` |
| **AC-03** | Jitter de resposta OBD-II (desvio padrão) | < 3 ms | `std(obd_latency_ms)` via script Python |
| **AC-04** | Integridade do CSV | 100% linhas sem campos vazios indevidos | Script de validação Python (ver Seção 9) |
| **AC-05** | Uso de SRAM (ESP32-S3) | < 200 KB | `esp-idf heap monitor` durante run de 30 min |
| **AC-06** | Fallback SD automático | Log continua no SD em desconexão Wi-Fi forçada | Desligar roteador durante coleta; verificar CSV gerado |
| **AC-07** | Bus-Off recovery | Sistema continua sem reinício do SoC após desconexão temporária do cabo CAN | Desconectar e reconectar o cabo CAN; verificar log de recovery |
| **AC-08** | Dataset mínimo gerado (Dual-Source) | ≥ 72.000 amostras (3 cenários × ~24.000: 31 Hz DBC + 10 Hz OBD-II) | `wc -l S_*.CSV` |

---

## 8. Setup de Hardware

### 8.1 Lista de materiais (BOM)

| Componente | Quantidade | Custo estimado (R$) | Observações |
|---|---|---|---|
| ESP32-S3 DevKit-C (Espressif) | 1 | 70–90 | Com PSRAM 8 MB |
| Arduino UNO R3 | 1 | 40–60 | Original ou clone |
| Módulo MCP2515 + TJA1050 | 1 | 15–25 | Módulo já montado com cristal 8 MHz |
| Módulo microSD (SPI) | 1 | 8–15 | Para o ESP32-S3 |
| Cartão microSD 32 GB classe 10 | 1 | 20–30 | FAT32 formatado |
| Resistores de terminação 120 Ω | 2 | < 1 | Para as extremidades do barramento CAN |
| Jumpers macho-macho / macho-fêmea | 20 | 5–10 | |
| Protoboard 830 pontos | 1 | 15–20 | |
| Fonte de bancada 5V/2A (ou USB) | 1 | — | Pode usar USB da bancada |
| **Total estimado** | | **~R$ 175–250** | |

### 8.2 Pinagem — ESP32-S3 ↔ MCP2515 / SD Card

#### CAN Bus (TWAI — via transceiver externo se necessário)

| ESP32-S3 GPIO | Função TWAI | Destino |
|---|---|---|
| GPIO 4 | TX (CAN_TX) | TJA1050 TXD |
| GPIO 5 | RX (CAN_RX) | TJA1050 RXD |
| 3.3V | — | TJA1050 VCC |
| GND | — | TJA1050 GND |

> **Nota:** o ESP32-S3 possui transceiver TWAI em nível lógico. O TJA1050 converte para nível de barramento CAN diferencial (CANH/CANL).

#### SD Card (SPI)

| ESP32-S3 GPIO | Função SPI | SD Card pino |
|---|---|---|
| GPIO 10 | CS (Chip Select) | CS |
| GPIO 11 | MOSI | DI |
| GPIO 12 | CLK | CLK |
| GPIO 13 | MISO | DO |
| 3.3V | — | VCC |
| GND | — | GND |

#### UNO R3 ↔ MCP2515

| Arduino PIN | Função SPI | MCP2515 pino |
|---|---|---|
| D10 | CS | CS |
| D11 | MOSI | SI |
| D12 | MISO | SO |
| D13 | SCK | SCK |
| D2 | INT | INT |
| 5V | — | VCC |
| GND | — | GND |

#### Barramento CAN

| Sinal | UNO R3 (via TJA1050) | ESP32-S3 (via TJA1050) | Terminação |
|---|---|---|---|
| CANH | TJA1050 CANH | TJA1050 CANH | 120 Ω entre CANH e CANL |
| CANL | TJA1050 CANL | TJA1050 CANL | 120 Ω entre CANH e CANL |

### 8.3 Diagrama de conexões

```
                    ┌──────────────┐
                    │  Arduino     │
                    │  UNO R3      │
                    │              │
                    │  D10 ──────► CS   ┐
                    │  D11 ──────► SI   ├── MCP2515 ──► TJA1050 ──┐
                    │  D12 ◄────── SO   │                          │
                    │  D13 ──────► SCK  ┘                     CANH─┤───[120Ω]───┐
                    │  D2  ◄────── INT                        CANL─┤            │
                    └──────────────┘                               │            │
                                                                   │            │
                    ┌──────────────┐                               │            │
                    │  ESP32-S3    │                               │            │
                    │              │                               │            │
                    │  GPIO4 ────► TX   ┐                     CANH─┤            │
                    │  GPIO5 ◄──── RX   ├── TJA1050 ──────── CANL─┘            │
                    │              │   ┘                                   [120Ω]
                    │  GPIO10 ───► CS  ┐                                        │
                    │  GPIO11 ───► DI  ├── SD Card                             GND
                    │  GPIO12 ◄─── DO  │
                    │  GPIO13 ───► CLK ┘
                    └──────────────┘
```

---

## 9. Estrutura de Repositório

```
safemamba-edge/
├── README.md
├── SPEC_F1.md                          ← este documento
│
├── firmware/
│   ├── uno_ecu_emulator/               ← Arduino UNO R3
│   │   ├── uno_ecu_emulator.ino        ← ou src/main.cpp (PlatformIO)
│   │   ├── can_profiles.h              ← tabelas de lookup senoidais
│   │   ├── obd_responses.h             ← codificação de PIDs
│   │   └── platformio.ini
│   │
│   └── esp32s3_collector/              ← ESP32-S3 (Rust)
│       ├── Cargo.toml
│       ├── .cargo/config.toml          ← target xtensa-esp32s3-none-elf
│       ├── src/
│       │   ├── main.rs
│       │   ├── types.rs
│       │   ├── config.rs
│       │   ├── mcal/
│       │   │   ├── mod.rs
│       │   │   ├── twai.rs
│       │   │   └── spi_sd.rs
│       │   ├── bsw/
│       │   │   ├── mod.rs
│       │   │   ├── bsw_mem.rs
│       │   │   ├── bsw_com.rs
│       │   │   └── bsw_diag.rs
│       │   └── app/
│       │       ├── mod.rs
│       │       ├── obd_poller.rs
│       │       ├── logger.rs
│       │       └── csv_writer.rs
│       └── build.rs
│
├── tools/
│   ├── validate_csv.py                 ← verifica integridade do dataset (AC-04)
│   ├── plot_session.py                 ← gera gráficos para o artigo (Figura 6)
│   ├── compute_metrics.py              ← calcula AC-01 a AC-08 automaticamente
│   └── requirements.txt               ← pandas, matplotlib, numpy
│
├── dataset/
│   └── README.md                       ← instruções de como gerar; dados não commitados (gitignore)
│
└── docs/
    ├── BOM.md                          ← lista de materiais
    ├── pinagem.md                      ← diagrama de conexões detalhado
    └── DBC_minimal.dbc                 ← especificação DBC dos frames emitidos
```

### 9.1 Script de validação do dataset (tools/validate_csv.py)

```python
"""
validate_csv.py — Verifica critérios de aceitação AC-01, AC-03, AC-04, AC-08
Uso: python validate_csv.py session_20250630_143022.csv
"""
import sys
import pandas as pd
import numpy as np

def validate(filepath: str):
    df = pd.read_csv(filepath)

    resultados = {}

    # AC-04: Integridade — campos obrigatórios não podem ser nulos
    obrigatorios = ['timestamp_ms', 'source', 'can_id', 'speed_kmh', 'rpm',
                    'throttle_pct', 'engine_load_pct', 'maf_g_s', 'session_label']
    nulos = df[obrigatorios].isnull().sum().sum()
    resultados['AC-04_integridade'] = {
        'status': '✅ OK' if nulos == 0 else f'❌ FALHA ({nulos} nulos)',
        'valor': f'{nulos} campos nulos'
    }

    # AC-03: Jitter OBD-II
    obd_rows = df[df['source'] == 'OBD_PID']['obd_latency_ms'].dropna()
    if len(obd_rows) > 0:
        media    = obd_rows.mean()
        desvio   = obd_rows.std()
        resultados['AC-02_latencia_media'] = {
            'status': '✅ OK' if media < 10 else f'❌ FALHA',
            'valor': f'{media:.2f} ms'
        }
        resultados['AC-03_jitter'] = {
            'status': '✅ OK' if desvio < 3 else f'❌ FALHA',
            'valor': f'{desvio:.2f} ms'
        }

    # AC-08: Volume mínimo por sessão (meta: ≥ 18.000 por cenário; ≥ 54.000 total nos 3)
    total_linhas = len(df)
    resultados['AC-08_volume'] = {
        'status': '✅ OK' if total_linhas >= 18000 else '⚠️  ABAIXO DO MÍNIMO (sessão única)',
        'valor': f'{total_linhas} amostras (meta por sessão: ≥ 18.000)'
    }

    # Relatório
    print(f"\n{'='*55}")
    print(f"  RELATÓRIO DE VALIDAÇÃO — {filepath.split('/')[-1]}")
    print(f"{'='*55}")
    for criterio, r in resultados.items():
        print(f"  {criterio:<35} {r['status']}")
        print(f"    └─ {r['valor']}")
    print(f"{'='*55}\n")

if __name__ == '__main__':
    validate(sys.argv[1])
```

---

## 10. Dependências e Toolchain

### 10.1 Firmware Arduino UNO R3

```ini
; platformio.ini
[env:uno]
platform  = atmelavr
board     = uno
framework = arduino
lib_deps  =
    coryjfowler/MCP_CAN_lib @ ^1.5.0
```

### 10.2 Firmware ESP32-S3 (Rust)

```toml
# Cargo.toml
[package]
name    = "esp32s3-collector"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-hal        = { version = "0.18", features = ["esp32s3"] }
embassy-executor = { version = "0.5", features = ["nightly"] }
embassy-time   = { version = "0.3" }
embedded-sdmmc = { version = "0.7" }
heapless       = { version = "0.8" }
rumqttc        = { version = "0.24", default-features = false }
defmt          = { version = "0.3" }
defmt-rtt      = { version = "0.4" }
panic-probe    = { version = "0.3" }

[profile.release]
opt-level = "s"    # otimização para tamanho
lto       = true
```

```toml
# .cargo/config.toml
[build]
target = "xtensa-esp32s3-none-elf"

[target.xtensa-esp32s3-none-elf]
runner  = "espflash flash --monitor"
rustflags = ["-C", "link-arg=-Tlinkall.x"]
```

#### Instalação do toolchain Rust para Xtensa

```bash
# Instalar espup (gerenciador do toolchain Xtensa)
cargo install espup
espup install

# Ativar o toolchain no shell
source ~/export-esp.sh

# Verificar
cargo +esp build --release
```

### 10.3 Ferramentas Python (tools/)

```txt
# requirements.txt
pandas>=2.0
numpy>=1.24
matplotlib>=3.7
pyserial>=3.5      # para monitor serial opcional
```

---

## 11. Plano de Tarefas

### Semana 1 — Hardware e emulador

| ID | Tarefa | Responsável | Entrega |
|---|---|---|---|
| T01 | Montar bancada física (BOM completa, cabos, terminação) | Todos | Bancada funcionando, continuidade verificada com multímetro |
| T02 | Firmware UNO R3: emissão DBC (0x100, 0x200, 0x300) | Henrique / Amauri | UNO emite frames, visualizados com Serial Monitor do MCP2515 |
| T03 | Firmware UNO R3: resposta OBD-II 0x7DF → 0x7E8 | Henrique / Amauri | Respostas corretas verificadas com analisador CAN (ou segundo UNO como listener) |
| T04 | Firmware UNO R3: recepção de 0x010 (troca de perfil) | Henrique | Perfil altera valores emitidos conforme tabela de perfis |

### Semana 2 — ESP32-S3 recepção CAN

| ID | Tarefa | Responsável | Entrega |
|---|---|---|---|
| T05 | Setup toolchain Rust ESP32-S3 (espup + Embassy) | Wissam | `cargo build --release` sem erros |
| T06 | `mcal/twai.rs`: inicialização TWAI a 500 Kbps, leitura de frames | Wissam | Frames do UNO R3 lidos e impressos via `defmt` |
| T07 | `types.rs`: definição de `TelemetryFrame`, `TelemetrySignals` | Larissa | Structs compilando, `Default` implementado |
| T08 | `app/obd_poller.rs`: envio 0x7DF e recepção 0x7E8 com timeout | Wissam | OBD polling funcionando, latência medida via defmt |

### Semana 3 — Persistência e estruturação

| ID | Tarefa | Responsável | Entrega |
|---|---|---|---|
| T09 | `mcal/spi_sd.rs`: init SD Card, mount FAT32 | Larissa | SD montado, arquivo de teste criado |
| T10 | `bsw/bsw_mem.rs`: buffer circular 4 KB + write CSV + flush 10 s | Larissa | Linhas CSV gravadas corretamente, verificadas em PC |
| T11 | `app/csv_writer.rs`: serialização `TelemetryFrame` → linha CSV | Larissa | Output de CSV válido conforme schema Seção 6.2 |
| T12 | `bsw/bsw_com.rs`: Wi-Fi connect + MQTT publish | Wissam | Frames publicados no tópico `/telemetry/raw` |

### Semana 4 — Integração, fallback e validação

| ID | Tarefa | Responsável | Entrega |
|---|---|---|---|
| T13 | `bsw/bsw_diag.rs`: Bus-Off detection + auto-recovery | Amauri | Recovery sem reinício verificado em teste de cabo desconectado |
| T14 | `app/logger.rs`: lógica SD ↔ MQTT com fallback automático | Amauri | Fallback verificado: Wi-Fi desligado → dados no SD |
| T15 | Coleta dos 3 cenários (C1/C2/C3 × 10 min cada) | Todos | 3 arquivos CSV gerados, ≥ 54.000 amostras totais (≥ 18.000 por sessão) |
| T16 | Execução de `validate_csv.py` e `compute_metrics.py` | Larissa | Todos os critérios AC-01 a AC-08 atendidos |
| T17 | Geração dos gráficos para o artigo (`plot_session.py`) | Larissa | Figuras 5 e 6 do artigo prontas |
| T18 | Documentação: README, pinagem, DBC_minimal.dbc | Henrique | Repositório documentado para replicabilidade |

### Cronograma visual

```
         Semana 1    Semana 2    Semana 3    Semana 4
         ──────────  ──────────  ──────────  ──────────
Hardware  ████████
Emulador  ████████
TWAI/OBD              ████████
SD/CSV                            ████████
Wi-Fi/MQTT                        ████████
Integração                                  ████████
Coleta                                      ████████
Validação                                   ████████
```

---

## 12. Riscos e Mitigações

| ID | Risco | Probabilidade | Impacto | Mitigação |
|---|---|---|---|---|
| R01 | Toolchain Rust para Xtensa com problemas de compilação | Média | Alto | Reservar Semana 1 completa para setup. Fallback: usar `esp-idf` com C se Rust travar além de 3 dias |
| R02 | Jitter OBD-II acima de 3 ms (AC-03 falha) | Média | Médio | Documentar como limitação do UNO R3; ajustar threshold de aceitação para 5 ms se necessário, com justificativa técnica |
| R03 | SD Card com escrita lenta (FAT32 fragmentation) | Baixa | Médio | Formatar como FAT32 antes de cada sessão; usar buffer circular de 4 KB + flush em bloco de 512 B |
| R04 | Bus-Off persistente por cabo com impedância errada | Baixa | Alto | Verificar terminação 120 Ω com multímetro antes de ligar; testar com cabo curto (< 30 cm) na bancada |
| R05 | embassy-net incompatível com versão do esp-hal | Média | Alto | Fixar versões exatas no Cargo.toml; consultar matrix de compatibilidade do repositório esp-rs |
| R06 | Prazo: implementação excede 4 semanas | Média | Alto | Tasks T01–T08 são críticas para o artigo; T12–T14 podem ser simplificadas (SD-only sem MQTT) se houver pressão de prazo |

---

## Apêndice A — Tópicos MQTT

| Tópico | QoS | Payload | Retenção |
|---|---|---|---|
| `/telemetry/raw` | 0 | JSON com campos do schema CSV | Não |
| `/coach/session/label` | 1 | `"Economico"` / `"Normal"` / `"Esportivo"` | Sim |
| `/system/status` | 0 | `{"sd_ok": true, "wifi_ok": true, "bus_off_count": 0}` | Não |

## Apêndice B — Glossário

| Termo | Definição |
|---|---|
| **CAN** | Controller Area Network — protocolo serial automotivo (ISO 11898) |
| **DBC** | Database CAN — arquivo de descrição de sinais CAN (Vector/CANdb++) |
| **ECU** | Electronic Control Unit — unidade eletrônica de controle |
| **Embassy** | Runtime async/await para sistemas embarcados em Rust |
| **HAL** | Hardware Abstraction Layer — camada de abstração de hardware |
| **HIL** | Hardware-in-the-Loop — bancada de simulação com hardware real |
| **MCAL** | Microcontroller Abstraction Layer (AUTOSAR) |
| **OBD-II** | On-Board Diagnostics II — interface de diagnóstico veicular |
| **PID** | Parameter ID — identificador de parâmetro OBD-II |
| **TWAI** | Two-Wire Automotive Interface — controlador CAN nativo do ESP32 |
| **SDV** | Software Defined Vehicle — Veículo Definido por Software |
| **BSW** | Basic Software (AUTOSAR) |
| **RTE** | Runtime Environment (AUTOSAR) |

---

*Documento vivo — atualizar com resultados reais ao longo da implementação.*  
*Versão 1.0 — Junho 2025*
