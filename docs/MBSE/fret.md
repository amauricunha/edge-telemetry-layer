# Guia de Cadastramento e Sentenças FRETish (NASA FRET)
## Projeto: Edge Telemetry Layer (ESP32-S3 & Arduino UNO R3)

Este documento estabelece **100% de paridade e rastreabilidade com a Especificação de Requisitos de Sistema (`docs/mestrado/srs.md`)**, contendo todos os 30 requisitos do sistema (REQ-SYS-01 a REQ-SYS-30), os critérios de aceitação AC-01 a AC-08 e os fluxos das quatro threads principais da arquitetura.

Cada requisito está formalizado na gramática **FRETish (em inglês normatizado)** aceita pelo parser ANTLR 4 da ferramenta **NASA FRET**, com nomes de componentes higienizados (sem `::`) e com o mapeamento tipado de variáveis.

---

## Regras Essenciais de Sintaxe FRETish (ANTLR 4)

> **Unidades de tempo:** O parser exige os tokens lexicais completos em maiúsculas:
> `MILLISECOND`, `MICROSECOND`, `SECOND`, `MINUTE`, `HOUR`, `TICK`.
> Abreviações como `ms`, `us`, `s` causam erro `mismatched input 'ms' expecting {...}`.

> **Predicados Booleanos:** Para variáveis `Boolean` no `satisfy`, **nunca** escreva `= TRUE`, `= FALSE` ou `= "TRUE"`.
> O parser trata `TRUE` como uma variável desconhecida adicional, não como literal.
> **Correto:** `satisfy sd_fallback_active` | **Errado:** `satisfy sd_fallback_active = TRUE`

> **Nomes de componentes:** Proibido `::`. Use `_` como separador: `mcal_twai`, não `mcal::twai`.

> **Parent Requirement ID:** O campo espera o **Req ID exato de um requisito já existente no projeto**, com **underscores** (não hífens). `REQ_SYS_01` é válido; `REQ-SYS-01` causa erro de parse. O campo não afeta a verificação de realizabilidade — serve apenas para rastreabilidade hierárquica no relatório.

---

## Como cadastrar cada requisito no FRET:

> **Atenção:** O FRET **não possui campo "Título"**. Os campos reais do modal são:
> `Requirement ID`, `Parent Requirement ID`, `Project`, `Rationale`, `Comments` e `Requirement Description`.
> O título descritivo vai dentro do campo **Rationale**, junto com o texto em Português.

1. Abra o FRET e acesse/crie o projeto **`EdgeTelemetryLayer`**.
2. Clique em **`CREATE`** no menu superior.
3. No modal, preencha:
   - **Requirement ID:** Cole o valor de `ID` (ex: `REQ_EMU_005`) — underscores, sem hífens.
   - **Parent Requirement ID:** Deixe vazio (a rastreabilidade com o SRS é feita pela tag `[REQ-SYS-XX]` no Rationale).
   - **Project:** `EdgeTelemetryLayer`
   - **Rationale:** Cole o título descritivo + texto em Português no seguinte formato:
     ```
     [REQ-SYS-08 / AC-02] Latência Média de Resposta OBD-II

     O emulador deve responder com latência média estritamente menor que 10 ms (Critério de Aceitação AC-02).
     ```
4. No campo **Requirement Description**, cole **apenas** o texto da caixa `FRETish Text`:
   ```
   in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy mean_obd_latency_ms < 10
   ```
   - O editor colorirá automaticamente: **vermelho** (Scope), **laranja** (Condition), **verde** (Component), **azul** (Timing), **roxo** (Response).
   - Texto sem coloração = erro de sintaxe — consulte as Regras acima.
5. Clique em **`CREATE`**.
6. Abra a aba **`Variable Mapping`** e configure o **Role** (`Input`, `Output` ou `Internal`) e o **Type** (`Boolean`, `Integer`, `Double`) conforme detalhado em cada bloco.
7. Clique em **`Realizability`** para executar a prova formal (resultado esperado: **`Realizable: True`**).

---

## Sobre Variáveis de Valores Físicos Calculados (Perfis e Senoide)

O emulador Arduino calcula `speed_kmh`, `rpm`, `throttle_pct`, `load_pct`, `maf_g_s` a partir de uma tabela senoidal pré-computada em `PROGMEM`, modulada pelo `active_profile`.
1. **As equações trigonométricas contínuas não entram no FRET** porque os model checkers (NuSMV/JKind) operam sobre lógica proposicional e aritmética linear finita.
2. **As grandezas físicas e invariantes de segurança ENTRAM no FRET:** As variáveis (`speed_kmh`, `rpm`, `throttle_pct`, `load_pct`, `maf_g_s`) são formalizadas em [`REQ_EMU_003`] através de contratos de invariantes de faixa física válida (`>= 0`) e deadline de CPU (`within 1 MILLISECOND`).
3. O `active_profile` governa tanto a dinâmica física em [`REQ_EMU_003`] quanto a emissão dos frames DBC em [`REQ_EMU_004`] (`when active_profile >= 1`).

---

```
[SISTEMA: Edge Telemetry Layer] (Total: 48 Requisitos)
   │
   ├── [COMPONENTE 1: uno_ecu_emulator] (8 requisitos)
   │     └── Nó Emulador de ECU (ATmega328P + MCP2515 Bare-Metal C++)
   │           └── [Subsistema 1] [Emulador ECU] Emulação Dinâmica e Física DBC [REQ_EMU_001 a REQ_EMU_008]
   │
   └── [COMPONENTE 2: esp32s3_collector] (40 requisitos)
         └── Nó Coletor e Gateway de Borda Integrado (Firmware Rust no_std Embassy)
               ├── [Subsistema 2] [Camada MCAL] Aquisição e Recepção Passiva TWAI [REQ_CAN_001 a REQ_CAN_006]
               ├── [Subsistema 3] [Camada APP] Diagnóstico Ativo OBD-II ISO 15765-4 [REQ_OBD_001 a REQ_OBD_004]
               ├── [Subsistema 4] [Camada APP] Estruturação Tabular CSV e Buffers SRAM [REQ_LOG_001 a REQ_LOG_009]
               ├── [Subsistema 5] [Camada BSW] Persistência em MicroSD FAT32 via SPI2 [REQ_SD_001 a REQ_SD_005]
               ├── [Subsistema 6] [Camada BSW] Conectividade em Nuvem Wi-Fi e MQTT [REQ_COM_001 a REQ_COM_005]
               ├── [Subsistema 7] [Camada APP] FSM de Conectividade e Fallback Offline [REQ_FSM_001 a REQ_FSM_002]
               ├── [Subsistema 8] [Camada BSW] Controle Remoto de Bancada e Replay [REQ_CMD_001 a REQ_CMD_002]
               └── [Subsistema 9] [Camada BSW] Resiliência Bus-Off e Watchdog [REQ_REC_001 a REQ_REC_007]
```

# Subsistema 1: Emulação de ECU Automotiva (Arduino UNO R3)

### REQ_EMU_001 — Inicialização do MCP2515 e Perfil Padrão [REQ-SYS-01]
- **ID:** `REQ_EMU_001`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & mcp2515_hardware_up the uno_ecu_emulator shall within 100 MILLISECOND satisfy can_bus_operational & active_profile = 2
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean) — reinicialização ou energização da placa ATmega328P
  - `mcp2515_hardware_up`: **Input** (Boolean) — módulo MCP2515 e transceptor energizados e respondendo via SPI
  - `can_bus_operational`: **Output** (Boolean) — barramento inicializado a 500 kbps (CAN_OK), modo MCP_NORMAL ativo e LED de status D4 aceso
  - `active_profile`: **Output** (Integer) — Perfil ativo de física (1=Eco, **2=Normal**, 3=Sport)
- **Rationale (Português):** Durante o setup(), ao detectar o hardware do MCP2515 via SPI, o emulador deve configurar o modo normal a 500 kbps, sinalizar o LED de status operacional e definir o perfil ativo como **2 (Normal)** antes de habilitar os timers de interrupção. Confirmado no firmware: `volatile uint8_t perfil_atual = 2`.

---

### REQ_EMU_002 — Comutação de Perfil de Condução Simulada [REQ-SYS-01]
- **ID:** `REQ_EMU_002`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session upon profile_cmd_0x010_received the uno_ecu_emulator shall within 50 MILLISECOND satisfy active_profile = commanded_profile
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `profile_cmd_0x010_received`: **Input** (Boolean)
  - `commanded_profile`: **Input** (Integer) — Byte 0 do comando CAN (1=Eco, 2=Normal, 3=Sport)
  - `active_profile`: **Output** (Integer) — Perfil ativo de física (1=Eco, 2=Normal, 3=Sport)
- **Rationale (Português):** Ao receber o comando CAN 0x010, o emulador deve atualizar o perfil de simulação ativo (`active_profile = commanded_profile`, onde 1=Econômico, 2=Normal, 3=Esportivo) no próximo ciclo de física do Timer1 (50 ms).

---

### REQ_EMU_003 — Atualização do Modelo Físico da ECU [REQ-SYS-01]
- **ID:** `REQ_EMU_003`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session when active_profile >= 1 upon timer1_50ms_tick the uno_ecu_emulator shall within 1 MILLISECOND satisfy physics_model_updated & speed_kmh >= 0 & rpm >= 0 & throttle_pct >= 0 & load_pct >= 0 & maf_g_s >= 0 & coolant_temp_c >= -40
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `active_profile`: **Input** (Integer) — Perfil de condução ativo (1=Eco, 2=Normal, 3=Sport)
  - `timer1_50ms_tick`: **Input** (Boolean) — Interrupção periódica do Timer1 a cada 50 ms
  - `physics_model_updated`: **Output** (Boolean) — Flag de conclusão do cálculo
  - `speed_kmh`: **Output** (Double) — Velocidade simulada (km/h)
  - `rpm`: **Output** (Integer) — Rotação do motor (RPM)
  - `throttle_pct`: **Output** (Double) — Posição do acelerador (%)
  - `load_pct`: **Output** (Double) — Carga calculada do motor (%)
  - `maf_g_s`: **Output** (Double) — Fluxo de ar MAF (g/s)
  - `coolant_temp_c`: **Output** (Double) — Temperatura do líquido de arrefecimento (°C, nominal 87°C)
- **Rationale (Português):** A cada ciclo de 50 ms do Timer1 sob perfil ativo válido (`active_profile >= 1`), a CPU do emulador deve calcular as 6 grandezas físicas simuladas do motor (`speed_kmh`, `rpm`, `throttle_pct`, `load_pct`, `maf_g_s`, `coolant_temp_c`) a partir da tabela senoidal modulada pelo perfil em menos de 1 ms de tempo de execução da ISR, assegurando invariantes válidos.

---

### REQ_EMU_004 — Emissão Cíclica de Grandezas do Motor DBC [REQ-SYS-01]
- **ID:** `REQ_EMU_004`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session when active_profile >= 1 & can_bus_operational upon physics_model_updated the uno_ecu_emulator shall within 2 MILLISECOND satisfy dbc_frames_emitted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `active_profile`: **Input** (Integer) — Perfil ativo selecionado
  - `can_bus_operational`: **Input** (Boolean) — Barramento CAN operacional e inicializado
  - `physics_model_updated`: **Input** (Boolean) — Gerado após a conclusão do cálculo físico em REQ_EMU_003
  - `dbc_frames_emitted`: **Output** (Boolean)
- **Rationale (Português):** Após a conclusão do cálculo do modelo de física (`physics_model_updated`), com o barramento CAN operacional, o emulador deve codificar e transmitir os frames CAN periódicos conforme a DBC (0x200 a cada 50 ms, 0x100 a cada 100 ms e 0x300 a cada 1000 ms) via SPI/MCP2515 em até 2 ms.

---

### REQ_EMU_005 — Processamento de Interrupção de Alta Frequência [REQ-SYS-02]
- **ID:** `REQ_EMU_005`
- **Parent Requirement ID:** REQ_SYS_02
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  when can_bus_operational upon timer2_1ms_tick the uno_ecu_emulator shall within 50 MICROSECOND satisfy mcp2515_rx_polled
  ```
- **Variable Mapping:**
  - `can_bus_operational`: **Input** (Boolean) — Barramento CAN operacional
  - `timer2_1ms_tick`: **Input** (Boolean)
  - `mcp2515_rx_polled`: **Output** (Boolean)
- **Rationale (Português):** Estando o barramento operacional, o Timer2 a cada 1 ms deve verificar a chegada de mensagens no MCP2515 sem bloquear a CPU por mais de 50 µs.

---

### REQ_EMU_006 — Codificação e Emissão de Resposta OBD-II [REQ-SYS-07]
- **ID:** `REQ_EMU_006`
- **Parent Requirement ID:** REQ_SYS_07
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session upon obd_request_0x7df_received the uno_ecu_emulator shall within 10 MILLISECOND satisfy obd_response_0x7e8_sent
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_request_0x7df_received`: **Input** (Boolean)
  - `obd_response_0x7e8_sent`: **Output** (Boolean)
- **Rationale (Português):** Ao receber requisição Modo 01 em 0x7DF, o emulador deve codificar o PID e transmitir o frame de resposta 0x7E8.

---

### REQ_EMU_007 — Latência Média de Resposta OBD-II [REQ-SYS-08 / AC-02]
- **ID:** `REQ_EMU_007`
- **Parent Requirement ID:** REQ_SYS_08
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy mean_obd_latency_ms < 10
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_benchmark_running`: **Input** (Boolean)
  - `mean_obd_latency_ms`: **Output** (Double)
- **Rationale (Português):** O emulador deve responder com latência média estritamente menor que 10 ms (Critério de Aceitação AC-02).

---

### REQ_EMU_008 — Estabilidade Temporal e Jitter de Resposta [REQ-SYS-09 / AC-03]
- **ID:** `REQ_EMU_008`
- **Parent Requirement ID:** REQ_SYS_09
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy latency_jitter_std_ms < 3
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_benchmark_running`: **Input** (Boolean)
  - `latency_jitter_std_ms`: **Output** (Double)
- **Rationale (Português):** O desvio padrão da latência de resposta OBD-II deve permanecer inferior a 3 ms durante os ensaios (Critério AC-03).

---

# Subsistema 2: Aquisição e Recepção Passiva CAN (ESP32-S3 TWAI)
### REQ_CAN_001 — Inicialização e Sincronismo do TWAI [REQ-SYS-03]
- **ID:** `REQ_CAN_001`
- **Parent Requirement ID:** `REQ_SYS_03`
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & mcal_twai_up the esp32s3_collector shall within 50 MILLISECOND satisfy twai_async_enabled
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `mcal_twai_up`: **Input** (Boolean)
  - `twai_async_enabled`: **Output** (Boolean)
- **Rationale (Português):** Durante o boot, se o gatilho de inicialização e o periférico TWAI estiverem ativos, o sistema integrado deve confirmar o modo assíncrono habilitado em até 50 ms.

---
### REQ_CAN_002 — Captura e Marcação Temporal de Frames [REQ-SYS-03]
- **ID:** `REQ_CAN_002`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon can_frame_arrived the esp32s3_collector shall within 2 MILLISECOND satisfy frame_timestamp_captured
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_frame_arrived`: **Input** (Boolean)
  - `frame_timestamp_captured`: **Output** (Boolean)
- **Rationale (Português):** Ao receber um frame físico no TWAI, registrar imediatamente o carimbo temporal do sistema em microssegundos/milissegundos.

---
### REQ_CAN_003 — Conversão de Grandezas Físicas DBC [REQ-SYS-03]
- **ID:** `REQ_CAN_003`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon raw_can_frame_ready the esp32s3_collector shall within 100 MICROSECOND satisfy engineering_values_scaled
  ```
- **Variable Mapping:**
  - `raw_can_frame_ready`: **Input** (Boolean)
  - `engineering_values_scaled`: **Output** (Boolean)
- **Rationale (Português):** O decodificador deve aplicar as equações de escala e offset da DBC nos bytes brutos sem alocação dinâmica em menos de 100 µs.

---
### REQ_CAN_004 — Inserção no Canal Assíncrono da RTE [REQ-SYS-03]
- **ID:** `REQ_CAN_004`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon telemetry_frame_parsed the esp32s3_collector shall within 1 MILLISECOND satisfy rte_channel_pushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `telemetry_frame_parsed`: **Input** (Boolean)
  - `rte_channel_pushed`: **Output** (Boolean)
- **Rationale (Português):** Ao concluir o parse de um frame de telemetria, postar a estrutura no canal bounded da RTE em até 1 ms.

---
### REQ_CAN_005 — Cumprimento da Taxa de Recepção [REQ-SYS-04 / AC-01]
- **ID:** `REQ_CAN_005`
- **Parent Requirement ID:** REQ_SYS_04
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session when can_bus_healthy the esp32s3_collector shall always satisfy frame_loss_percentage <= 1
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_bus_healthy`: **Input** (Boolean)
  - `frame_loss_percentage`: **Output** (Double)
- **Rationale (Português):** A taxa de perda de quadros deve ser mantida estritamente abaixo de 1%, assegurando recepção de no mínimo 99% (Critério AC-01).

---
### REQ_CAN_006 — Tratamento Não-Bloqueante de Saturação da RTE [REQ-SYS-03]
- **ID:** `REQ_CAN_006`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon rte_channel_overflow the esp32s3_collector shall immediately satisfy overflow_logged_and_dropped
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `rte_channel_overflow`: **Input** (Boolean)
  - `overflow_logged_and_dropped`: **Output** (Boolean)
- **Rationale (Português):** Caso o canal da RTE esteja cheio (32 amostras), descartar o pacote excedente e registrar advertência sem travar a recepção.

---

# Subsistema 3: Diagnóstico Ativo OBD-II ISO 15765-4
### REQ_OBD_001 — Polling Cíclico de Diagnóstico a 10 Hz [REQ-SYS-05]
- **ID:** `REQ_OBD_001`
- **Parent Requirement ID:** REQ_SYS_05
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon obd_timer_100ms_expired the esp32s3_collector shall immediately satisfy obd_request_0x7df_transmitted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_timer_100ms_expired`: **Input** (Boolean)
  - `obd_request_0x7df_transmitted`: **Output** (Boolean)
- **Rationale (Português):** A cada 100 ms, emitir solicitação funcional Modo 01 em 0x7DF para o próximo PID programado.

---

### REQ_OBD_002 — Escalonamento Circular Round-Robin dos PIDs [REQ-SYS-06]
- **ID:** `REQ_OBD_002`
- **Parent Requirement ID:** REQ_SYS_06
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon obd_tx_cycle_completed the esp32s3_collector shall within 1 MILLISECOND satisfy pid_index_incremented
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_tx_cycle_completed`: **Input** (Boolean)
  - `pid_index_incremented`: **Output** (Boolean)
- **Rationale (Português):** Alternar sequencialmente entre os 6 PIDs suportados, completando um ciclo a cada 600 ms.

---

### REQ_OBD_003 — Tratamento de Timeout de Diagnóstico [REQ-SYS-10]
- **ID:** `REQ_OBD_003`
- **Parent Requirement ID:** REQ_SYS_10
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon obd_timeout_50ms_elapsed the esp32s3_collector shall immediately satisfy obd_timeout_recorded
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_timeout_50ms_elapsed`: **Input** (Boolean)
  - `obd_timeout_recorded`: **Output** (Boolean)
- **Rationale (Português):** Se a resposta 0x7E8 não chegar em 50 ms, declarar timeout e registrar log sem suspender o escalonador.

---

### REQ_OBD_004 — Intercalação de Comandos de Bancada no Transmissor [REQ-SYS-05]
- **ID:** `REQ_OBD_004`
- **Parent Requirement ID:** REQ_SYS_05
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon can_cmd_received_in_queue the esp32s3_collector shall within 10 MILLISECOND satisfy can_cmd_interleaved
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_cmd_received_in_queue`: **Input** (Boolean)
  - `can_cmd_interleaved`: **Output** (Boolean)
- **Rationale (Português):** Intercalar o envio de comandos CAN (ex: 0x010) entre as janelas de polling OBD sem violar a periodicidade nominal de 100 ms.

---

# Subsistema 4: Estruturação de Dados e Bufferização em Memória

### REQ_LOG_001 — Serialização Determinística em CSV [REQ-SYS-11]
- **ID:** `REQ_LOG_001`
- **Parent Requirement ID:** REQ_SYS_11
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon telemetry_frame_received the esp32s3_collector shall within 1 MILLISECOND satisfy csv_line_formatted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `telemetry_frame_received`: **Input** (Boolean)
  - `csv_line_formatted`: **Output** (Boolean)
- **Rationale (Português):** Converter cada amostra em linha CSV de 11 colunas utilizando buffer estático de 320 bytes sem alocação dinâmica no heap.

---

### REQ_LOG_002 — Integridade Estrutural do Dataset [REQ-SYS-12 / AC-04]
- **ID:** `REQ_LOG_002`
- **Parent Requirement ID:** REQ_SYS_12
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session when dataset_recording the esp32s3_collector shall always satisfy null_mandatory_fields = 0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `dataset_recording`: **Input** (Boolean)
  - `null_mandatory_fields`: **Output** (Integer)
- **Rationale (Português):** 100% dos registros gerados devem conter todos os campos mandatórios (timestamp, source, can_id e label) sem campos nulos indevidos (Critério AC-04).

---

### REQ_LOG_003 — Inserção no Buffer Circular em SRAM [REQ-SYS-13]
- **ID:** `REQ_LOG_003`
- **Parent Requirement ID:** REQ_SYS_13
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon csv_line_available the esp32s3_collector shall within 50 MICROSECOND satisfy sd_buffer_pushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `csv_line_available`: **Input** (Boolean)
  - `sd_buffer_pushed`: **Output** (Boolean)
- **Rationale (Português):** Inserir a linha formatada no buffer em anel de 4096 bytes em SRAM sob seção crítica rápida inferior a 50 µs.

---

### REQ_LOG_004 — Esvaziamento de Buffer por Limiar de Ocupação [REQ-SYS-14]
- **ID:** `REQ_LOG_004`
- **Parent Requirement ID:** REQ_SYS_14
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session when buffer_occupancy >= 3584 the esp32s3_collector shall immediately satisfy flush_signal_emitted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `buffer_occupancy`: **Internal** (Integer)
  - `flush_signal_emitted`: **Output** (Boolean)
- **Rationale (Português):** Ao atingir 85% de ocupação (3584 bytes), emitir sinal imediato para a tarefa de gravação descarregar os dados.

---

### REQ_LOG_005 — Esvaziamento Periódico de Buffer por Temporizador [REQ-SYS-14]
- **ID:** `REQ_LOG_005`
- **Parent Requirement ID:** REQ_SYS_14
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon flush_timer_2s_expired the esp32s3_collector shall within 100 MILLISECOND satisfy pending_bytes_flushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `flush_timer_2s_expired`: **Input** (Boolean)
  - `pending_bytes_flushed`: **Output** (Boolean)
- **Rationale (Português):** A cada 2 segundos de inatividade de escrita, a tarefa de gravação deve persistir quaisquer bytes pendentes no arquivo.

---

### REQ_LOG_006 — Fatiamento em Chunks de 256 Bytes com Preempção [REQ-SYS-15]
- **ID:** `REQ_LOG_006`
- **Parent Requirement ID:** REQ_SYS_15
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon sector_write_chunk the esp32s3_collector shall within 1 MILLISECOND satisfy chunk_preemption_yielded
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `sector_write_chunk`: **Input** (Boolean)
  - `chunk_preemption_yielded`: **Output** (Boolean)
- **Rationale (Português):** Fracionar a gravação no SD em pedaços de 256 bytes e ceder controle à CPU para manter o bloqueio inferior a 0,76 ms.

---

### REQ_LOG_007 — Retenção em Backlog de RAM sob Falha do SD [REQ-SYS-23]
- **ID:** `REQ_LOG_007`
- **Parent Requirement ID:** REQ_SYS_23
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in offline_mode upon sd_write_failed the esp32s3_collector shall within 2 MILLISECOND satisfy ram_backlog_retained
  ```
- **Variable Mapping:**
  - `offline_mode`: **Internal** (Boolean)
  - `sd_write_failed`: **Input** (Boolean)
  - `ram_backlog_retained`: **Output** (Boolean)
- **Rationale (Português):** Se a escrita falhar em modo offline, reter os frames no buffer de contingência em RAM (50 elementos / 16 KB) sem descarte imediato.

---

### REQ_LOG_008 — Limite de Consumo de Memória SRAM [REQ-SYS-29 / AC-05]
- **ID:** `REQ_LOG_008`
- **Parent Requirement ID:** REQ_SYS_29
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session when memory_supervision_active the esp32s3_collector shall always satisfy sram_usage_kb < 200
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `memory_supervision_active`: **Input** (Boolean)
  - `sram_usage_kb`: **Output** (Double)
- **Rationale (Português):** O consumo de memória SRAM alocável deve permanecer estritamente contido abaixo de 200 KB durante operação contínua (Critério AC-05).

---

### REQ_LOG_009 — Telemetria de Desempenho HEARTBEAT no SD [REQ-SYS-29]
- **ID:** `REQ_LOG_009`
- **Parent Requirement ID:** REQ_SYS_29
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon heartbeat_timer_60s the esp32s3_collector shall within 100 MILLISECOND satisfy heartbeat_diag_logged
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `heartbeat_timer_60s`: **Input** (Boolean)
  - `heartbeat_diag_logged`: **Output** (Boolean)
- **Rationale (Português):** A cada 60 segundos em sessão ativa, gravar linha DIAG,HEARTBEAT com métricas de heap e contadores de frames no cartão SD.

---

# Subsistema 5: Persistência em Cartão MicroSD e Gestão de Sessões

### REQ_SD_001 — Montagem do Sistema de Arquivos FAT32 [REQ-SYS-16]
- **ID:** `REQ_SD_001`
- **Parent Requirement ID:** REQ_SYS_16
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & mcal_spi_sd_up & sd_card_inserted the esp32s3_collector shall within 500 MILLISECOND satisfy fat32_filesystem_mounted
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `mcal_spi_sd_up`: **Input** (Boolean) — barramento SPI2 e módulo leitor de SD energizados e prontos
  - `sd_card_inserted`: **Input** (Boolean) — cartão físico inserido no slot
  - `fat32_filesystem_mounted`: **Output** (Boolean) — partição FAT32 montada com sucesso
- **Rationale (Português):** Durante o boot, se o módulo leitor SPI estiver operacional e o cartão MicroSD estiver fisicamente inserido, o sistema deve inicializar o barramento e montar a partição FAT32 em até 500 ms, assegurando que a mídia de armazenamento esteja pronta e formatada antes do início da telemetria.

---

### REQ_SD_002 — Transição Atômica de Arquivos de Sessão [REQ-SYS-17]
- **ID:** `REQ_SD_002`
- **Parent Requirement ID:** REQ_SYS_17
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon session_rotate_command the esp32s3_collector shall within 50 MILLISECOND satisfy session_file_rotated_atomically
  ```
- **Variable Mapping:**
  - `session_rotate_command`: **Input** (Boolean)
  - `session_file_rotated_atomically`: **Output** (Boolean)
- **Rationale (Português):** Ao girar sessão, esvaziar síncronamente o buffer antigo e abrir o novo arquivo indexado S_XXXX.CSV em até 50 ms.

---

### REQ_SD_003 — Injeção de Cabeçalho e Linha BOOT [REQ-SYS-17]
- **ID:** `REQ_SD_003`
- **Parent Requirement ID:** REQ_SYS_17
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon new_file_opened the esp32s3_collector shall immediately satisfy header_and_boot_lines_written
  ```
- **Variable Mapping:**
  - `new_file_opened`: **Input** (Boolean)
  - `header_and_boot_lines_written`: **Output** (Boolean)
- **Rationale (Português):** Gravar o cabeçalho CSV na linha 1 e o metadado BOOT na linha 2 antes de qualquer amostra de telemetria.

---

### REQ_SD_004 — Temporização de Sessão Automática [REQ-SYS-18]
- **ID:** `REQ_SD_004`
- **Parent Requirement ID:** REQ_SYS_18
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon session_duration_reached the esp32s3_collector shall within 10 MILLISECOND satisfy session_stopped_and_flushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `session_duration_reached`: **Input** (Boolean)
  - `session_stopped_and_flushed`: **Output** (Boolean)
- **Rationale (Português):** Ao atingir a duração programada, injetar SESSION_COMPLETE, forçar flush de encerramento e suspender novas gravações.

---

### REQ_SD_005 — Volume Consolidado do Dataset [REQ-SYS-11/17 / AC-08]
- **ID:** `REQ_SD_005`
- **Parent Requirement ID:** REQ_SYS_18
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon benchmark_completion the esp32s3_collector shall satisfy total_dataset_samples >= 72000
  ```
- **Variable Mapping:**
  - `benchmark_completion`: **Input** (Boolean)
  - `total_dataset_samples`: **Output** (Integer)
- **Rationale (Português):** O sistema deve registrar um volume consolidado igual ou superior a 72.000 amostras nos três cenários de teste de 10 min (Critério AC-08).

---

# Subsistema 6: Conectividade em Nuvem e Telemetria Remota

### REQ_COM_001 — Conexão Wi-Fi e Pilha TCP/IP [REQ-SYS-19]
- **ID:** `REQ_COM_001`
- **Parent Requirement ID:** REQ_SYS_19
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & wifi_credentials_configured the esp32s3_collector shall within 10000 MILLISECOND satisfy ip_dhcp_assigned
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `wifi_credentials_configured`: **Input** (Boolean)
  - `ip_dhcp_assigned`: **Output** (Boolean)
- **Rationale (Português):** Durante a inicialização, validar as credenciais, conectar a interface sem fio em modo Station e obter endereço IP via DHCP através da pilha embassy-net.

---

### REQ_COM_005 — Estabelecimento de Conexão com o Broker MQTT [REQ-SYS-19]
- **ID:** `REQ_COM_005`
- **Parent Requirement ID:** REQ_SYS_19
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in boot_mode upon ip_dhcp_assigned & mqtt_credentials_valid the esp32s3_collector shall within 5000 MILLISECOND satisfy mqtt_broker_connected
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `ip_dhcp_assigned`: **Input** (Boolean)
  - `mqtt_credentials_valid`: **Input** (Boolean)
  - `mqtt_broker_connected`: **Output** (Boolean)
- **Rationale (Português):** Condicionado à obtenção do IP, o sistema deve validar as credenciais do MQTT e estabelecer a conexão TCP/IP com o broker em até 5000 ms, concluindo a verificação de rede ativa.

---

### REQ_COM_002 — Despacho em Lotes Binários via MQTT [REQ-SYS-20]
- **ID:** `REQ_COM_002`
- **Parent Requirement ID:** REQ_SYS_20
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in connected_mode upon binary_batch_full the esp32s3_collector shall within 200 MILLISECOND satisfy mqtt_batch_published
  ```
- **Variable Mapping:**
  - `connected_mode`: **Internal** (Boolean)
  - `binary_batch_full`: **Input** (Boolean)
  - `mqtt_batch_published`: **Output** (Boolean)
- **Rationale (Português):** Agrupar frames binários compactos de 18 bytes (até 150 registros) e despachá-los no tópico MQTT em até 200 ms.

---

### REQ_COM_003 — Publicação Periódica de Status e Keepalive [REQ-SYS-21]
- **ID:** `REQ_COM_003`
- **Parent Requirement ID:** REQ_SYS_21
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in connected_mode upon status_timer_5s the esp32s3_collector shall within 500 MILLISECOND satisfy status_json_published
  ```
- **Variable Mapping:**
  - `connected_mode`: **Internal** (Boolean)
  - `status_timer_5s`: **Input** (Boolean)
  - `status_json_published`: **Output** (Boolean)
- **Rationale (Português):** A cada 5 segundos com rede ativa, publicar payload JSON contendo saúde do SD, Wi-Fi, contadores e uptime em /system/status.

---

### REQ_COM_004 — Tarefa Periódica de Despacho (BSW Com) [REQ-SYS-20]
- **ID:** `REQ_COM_004`
- **Parent Requirement ID:** REQ_SYS_20
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon dispatch_cycle_50ms the esp32s3_collector shall within 2500 MICROSECOND satisfy data_packets_dispatched
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `dispatch_cycle_50ms`: **Input** (Boolean)
  - `data_packets_dispatched`: **Output** (Boolean)
- **Rationale (Português):** A cada ciclo de despacho de telemetria, a camada de comunicação (`bsw_com`) deve arbitrar e despachar os lotes binários acumulados no canal MQTT sem exceder 2,5 ms de tempo de execução da CPU.
- **Nota de Implementação:** No firmware em Rust (`firmware/esp32s3_collector/src/bsw/bsw_com.rs`), o despacho é executado pela tarefa assíncrona `task_wifi`, consumindo frames do canal estático `MQTT_TX_CHANNEL` (capacidade 500) em lotes de até 150 registros.

---

# Subsistema 7: Máquina de Estados de Conectividade e Fallback Offline

### REQ_FSM_001 — Comutação Automática de Fallback em MicroSD [REQ-SYS-23 / AC-06]
- **ID:** `REQ_FSM_001`
- **Parent Requirement ID:** REQ_SYS_23
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon wifi_disconnected the esp32s3_collector shall within 5 MILLISECOND satisfy sd_fallback_active
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `wifi_disconnected`: **Input** (Boolean)
  - `sd_fallback_active`: **Output** (Boolean)
- **Rationale (Português):** Na perda do sinal Wi-Fi, registrar evento DIAG e redirecionar 100% dos dados para o cartão SD em até 5 ms (Critério AC-06).

---

### REQ_FSM_002 — Reconexão de Rede e Dreno FIFO de Backlog [REQ-SYS-24]
- **ID:** `REQ_FSM_002`
- **Parent Requirement ID:** REQ_SYS_24
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon wifi_reconnected the esp32s3_collector shall within 10 MILLISECOND satisfy backlog_fifo_drained
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `wifi_reconnected`: **Input** (Boolean)
  - `backlog_fifo_drained`: **Output** (Boolean)
- **Rationale (Português):** Ao restabelecer a rede, descarregar integralmente as amostras retidas no backlog de RAM para o SD na ordem estrita de chegada.

---

# Subsistema 8: Controle Remoto e Streaming de Replay

### REQ_CMD_001 — Execução de Comandos Remotos de Bancada [REQ-SYS-25]
- **ID:** `REQ_CMD_001`
- **Parent Requirement ID:** REQ_SYS_25
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon remote_command_received the esp32s3_collector shall within 20 MILLISECOND satisfy command_executed_ack
  ```
- **Variable Mapping:**
  - `remote_command_received`: **Input** (Boolean)
  - `command_executed_ack`: **Output** (Boolean)
- **Rationale (Português):** Interpretar e executar instruções no tópico /coach/command (STOP, ECO/NOR/SPT, RESET, TIME, LIST_SESSIONS) em até 20 ms.

---

### REQ_CMD_002 — Transmissão em Streaming de Replay [REQ-SYS-26]
- **ID:** `REQ_CMD_002`
- **Parent Requirement ID:** REQ_SYS_26
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon replay_command_triggered the esp32s3_collector shall within 100 MILLISECOND satisfy replay_streaming_active
  ```
- **Variable Mapping:**
  - `replay_command_triggered`: **Input** (Boolean)
  - `replay_streaming_active`: **Output** (Boolean)
- **Rationale (Português):** Suspender sessão ativa, abrir arquivo S_XXXX.CSV e transmitir blocos de até 1536 bytes em streaming no tópico /telemetry/replay.

---

# Subsistema 9: Supervisão de Falhas, Resiliência e Watchdog

### REQ_REC_001 — Detecção de Erro de Bus-Off no TWAI [REQ-SYS-27]
- **ID:** `REQ_REC_001`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon bus_off_error_detected the esp32s3_collector shall within 1 MILLISECOND satisfy bus_off_flag_set
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `bus_off_error_detected`: **Input** (Boolean)
  - `bus_off_flag_set`: **Output** (Boolean)
- **Rationale (Português):** Ao detectar o código de erro crítico de saturação elétrica do TWAI, registrar atomicamente o estado de Bus-Off em até 1 ms.

---

### REQ_REC_002 — Pausa Cooperativa das Tarefas de Barramento [REQ-SYS-27]
- **ID:** `REQ_REC_002`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon bus_off_flag_active the esp32s3_collector shall within 10 MILLISECOND satisfy bus_off_pause_signaled
  ```
- **Variable Mapping:**
  - `bus_off_flag_active`: **Input** (Boolean)
  - `bus_off_pause_signaled`: **Output** (Boolean)
- **Rationale (Português):** O watchdog deve emitir sinal de pausa cooperativa para task_can_rx e task_obd_poller suspenderem tentativas de acesso ao TWAI.

---

### REQ_REC_003 — Janela de Espera de 128 ms da Norma ISO 11898 [REQ-SYS-27 / AC-07]
- **ID:** `REQ_REC_003`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon bus_off_pause_started the esp32s3_collector shall after 128 MILLISECOND satisfy recovery_window_elapsed
  ```
- **Variable Mapping:**
  - `bus_off_pause_started`: **Input** (Boolean)
  - `recovery_window_elapsed`: **Output** (Boolean)
- **Rationale (Português):** Aguardar compulsoriamente os 128 ms normatizados para observação de bits recessivos antes de reabilitar o controlador (Critério AC-07).

---

### REQ_REC_004 — Reativação via Registradores PAC sem Reboot [REQ-SYS-27 / AC-07]
- **ID:** `REQ_REC_004`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon recovery_window_elapsed the esp32s3_collector shall within 1 MILLISECOND satisfy twai_reset_mode_cleared
  ```
- **Variable Mapping:**
  - `recovery_window_elapsed`: **Input** (Boolean)
  - `twai_reset_mode_cleared`: **Output** (Boolean)
  - **Rationale (Português):** Limpar a flag de reset no registrador físico via PAC sem reiniciar o processador ESP32-S3 e sem destruir tarefas ativas.

---

### REQ_REC_005 — Retomada Operacional e Emissão de BUS_OFF_CLEAR [REQ-SYS-27 / AC-07]
- **ID:** `REQ_REC_005`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  upon twai_reset_mode_cleared the esp32s3_collector shall within 5 MILLISECOND satisfy bus_off_clear_signaled
  ```
- **Variable Mapping:**
  - `twai_reset_mode_cleared`: **Input** (Boolean)
  - `bus_off_clear_signaled`: **Output** (Boolean)
- **Rationale (Português):** Concluído o rearmamento, emitir BUS_OFF_CLEAR liberando as tarefas e gravar registro de sucesso no log do cartão SD.

---

### REQ_REC_006 — Detecção de Travamento de Tarefa (Logger Stall) [REQ-SYS-28]
- **ID:** `REQ_REC_006`
- **Parent Requirement ID:** REQ_SYS_28
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  in active_session upon logger_stall_30s_detected the esp32s3_collector shall within 100 MILLISECOND satisfy stall_diag_logged
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `logger_stall_30s_detected`: **Input** (Boolean)
  - `stall_diag_logged`: **Output** (Boolean)
- **Rationale (Português):** Se nenhum frame foi processado pelo logger em 30 segundos de sessão ativa com SD presente, registrar alerta de stall no SD.

---

### REQ_REC_007 — Rearme Periódico do Watchdog de Hardware [REQ-SYS-30]
- **ID:** `REQ_REC_007`
- **Parent Requirement ID:** REQ_SYS_30
- **Component:** `esp32s3_collector`
- **FRETish Text:**
  ```text
  when system_tasks_healthy the esp32s3_collector shall within 2000 MILLISECOND satisfy hw_watchdog_fed
  ```
- **Variable Mapping:**
  - `system_tasks_healthy`: **Input** (Boolean)
  - `hw_watchdog_fed`: **Output** (Boolean)
- **Rationale (Português):** Enquanto as tarefas estiverem saudáveis, a tarefa supervisora deve rearmar o watchdog de hardware em intervalo inferior a 2 s (timeout de 5 s).
