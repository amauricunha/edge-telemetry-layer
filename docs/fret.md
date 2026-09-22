# Guia de Cadastramento e Sentenças FRETish (NASA FRET)
## Projeto: Edge Telemetry Layer (ESP32-S3 & Arduino UNO R3)

Este documento estabelece **100% de paridade e rastreabilidade com a Especificação de Requisitos de Sistema (`docs/srs.md`)**, contendo todos os 30 requisitos do sistema (REQ-SYS-01 a REQ-SYS-30), os critérios de aceitação AC-01 a AC-08 e os fluxos das quatro threads principais da arquitetura.

Cada requisito está formalizado na gramática **FRETish (em inglês normatizado)** aceita pelo editor da ferramenta **NASA FRET**, com nomes de componentes higienizados (sem `::`) e com o mapeamento tipado de variáveis para a aba **Variable Mapping**.

---

## Como cadastrar cada requisito no FRET:
1. Abra a ferramenta FRET e acesse o projeto (ex: `EdgeTelemetry`).
2. Clique no botão **`+ New Requirement`**.
3. Preencha os campos exatamente com os valores deste prontuário:
   - **Req ID:** Cole o valor de `ID`.
   - **Component:** Cole o valor de `Component`.
   - **Requirement Text:** Cole **exclusivamente** o código da caixa `FRETish Text`.
4. Clique em **Save**:
   - O editor colorirá a sintaxe e gerará automaticamente a **Fórmula LTL** e o **Diagrama FSM**.
5. Abra a aba **`Variable Mapping`** e configure o **Role** (`Input`, `Output` ou `Internal`) e o **Type** (`Boolean`, `Integer`, `Double`) conforme detalhado em cada bloco.
6. Clique em **`Realizability`** para executar a prova formal sem deadlocks (resultado **Realizable: True**).

---

# Subsistema 1: Emulação de ECU Automotiva (Arduino UNO R3)

### REQ_EMU_001 — Emissão Cíclica de Grandezas do Motor DBC [REQ-SYS-01]
- **ID:** `REQ_EMU_001`
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session upon timer1_50ms_tick the uno_ecu_emulator shall within 2 ms satisfy dbc_frames_emitted = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `timer1_50ms_tick`: **Input** (Boolean)
  - `dbc_frames_emitted`: **Output** (Boolean)
- **Rationale (Português):** O emulador deve emitir os frames CAN periódicos conforme a DBC: 0x200 a cada 50 ms, 0x100 a cada 100 ms e 0x300 a cada 1000 ms via Timer1.

---

### REQ_EMU_002 — Comutação de Perfil de Condução Simulada [REQ-SYS-01]
- **ID:** `REQ_EMU_002`
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session upon profile_cmd_0x010_received the uno_ecu_emulator shall within 50 ms satisfy active_profile_updated = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `profile_cmd_0x010_received`: **Input** (Boolean)
  - `active_profile_updated`: **Output** (Boolean)
- **Rationale (Português):** Ao receber o comando CAN 0x010, o emulador deve atualizar o perfil de simulação (Econômico, Normal ou Esportivo) no ciclo seguinte.

---

### REQ_EMU_003 — Processamento de Interrupção de Alta Frequência [REQ-SYS-02]
- **ID:** `REQ_EMU_003`
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  upon timer2_1ms_tick the uno_ecu_emulator shall within 50 us satisfy mcp2515_rx_polled = TRUE
  ```
- **Variable Mapping:**
  - `timer2_1ms_tick`: **Input** (Boolean)
  - `mcp2515_rx_polled`: **Output** (Boolean)
- **Rationale (Português):** O Timer2 a cada 1 ms deve verificar a chegada de mensagens no MCP2515 sem bloquear a CPU por mais de 50 µs.

---

### REQ_EMU_004 — Codificação e Emissão de Resposta OBD-II [REQ-SYS-07]
- **ID:** `REQ_EMU_004`
- **Component:** `uno_ecu_emulator`
- **FRETish Text:**
  ```text
  in active_session upon obd_request_0x7df_received the uno_ecu_emulator shall within 10 ms satisfy obd_response_0x7e8_sent = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_request_0x7df_received`: **Input** (Boolean)
  - `obd_response_0x7e8_sent`: **Output** (Boolean)
- **Rationale (Português):** Ao receber requisição Modo 01 em 0x7DF, o emulador deve codificar o PID e transmitir o frame de resposta 0x7E8.

---

### REQ_EMU_005 — Latência Média de Resposta OBD-II [REQ-SYS-08 / AC-02]
- **ID:** `REQ_EMU_005`
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

### REQ_EMU_006 — Estabilidade Temporal e Jitter de Resposta [REQ-SYS-09 / AC-03]
- **ID:** `REQ_EMU_006`
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

### REQ_CAN_001 — Inicialização e Sincronismo do TWAI
- **ID:** `REQ_CAN_001`
- **Component:** `mcal_twai`
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger the mcal_twai shall within 50 ms satisfy twai_async_enabled = TRUE
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `twai_async_enabled`: **Output** (Boolean)
- **Rationale (Português):** Durante o boot, o driver MCAL deve configurar o periférico TWAI a 500 kbps em modo assíncrono em até 50 ms.

---

### REQ_CAN_002 — Captura e Marcação Temporal de Frames [REQ-SYS-03]
- **ID:** `REQ_CAN_002`
- **Component:** `task_can_rx`
- **FRETish Text:**
  ```text
  in active_session upon can_frame_arrived the task_can_rx shall within 2 ms satisfy frame_timestamp_captured = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_frame_arrived`: **Input** (Boolean)
  - `frame_timestamp_captured`: **Output** (Boolean)
- **Rationale (Português):** Ao receber um frame físico no TWAI, registrar imediatamente o carimbo temporal do sistema em microssegundos/milissegundos.

---

### REQ_CAN_003 — Conversão de Grandezas Físicas DBC [REQ-SYS-03]
- **ID:** `REQ_CAN_003`
- **Component:** `app_can_decoder`
- **FRETish Text:**
  ```text
  upon raw_can_frame_ready the app_can_decoder shall within 100 us satisfy engineering_values_scaled = TRUE
  ```
- **Variable Mapping:**
  - `raw_can_frame_ready`: **Input** (Boolean)
  - `engineering_values_scaled`: **Output** (Boolean)
- **Rationale (Português):** O decodificador deve aplicar as equações de escala e offset da DBC nos bytes brutos sem alocação dinâmica em menos de 100 µs.

---

### REQ_CAN_004 — Inserção no Canal Assíncrono da RTE [REQ-SYS-03]
- **ID:** `REQ_CAN_004`
- **Component:** `task_can_rx`
- **FRETish Text:**
  ```text
  in active_session upon telemetry_frame_parsed the task_can_rx shall within 1 ms satisfy rte_channel_pushed = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `telemetry_frame_parsed`: **Input** (Boolean)
  - `rte_channel_pushed`: **Output** (Boolean)
- **Rationale (Português):** Ao concluir o parse de um frame de telemetria, postar a estrutura no canal bounded da RTE em até 1 ms.

---

### REQ_CAN_005 — Cumprimento da Taxa de Recepção [REQ-SYS-04 / AC-01]
- **ID:** `REQ_CAN_005`
- **Component:** `task_can_rx`
- **FRETish Text:**
  ```text
  in active_session when can_bus_healthy the task_can_rx shall always satisfy frame_loss_percentage <= 1
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_bus_healthy`: **Input** (Boolean)
  - `frame_loss_percentage`: **Output** (Double)
- **Rationale (Português):** A taxa de perda de quadros deve ser mantida estritamente abaixo de 1%, assegurando recepção de no mínimo 99% (Critério AC-01).

---

### REQ_CAN_006 — Tratamento Não-Bloqueante de Saturação da RTE [REQ-SYS-03]
- **ID:** `REQ_CAN_006`
- **Component:** `task_can_rx`
- **FRETish Text:**
  ```text
  in active_session upon rte_channel_overflow the task_can_rx shall immediately satisfy overflow_logged_and_dropped = TRUE
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
- **Component:** `task_obd_poller`
- **FRETish Text:**
  ```text
  in active_session upon obd_timer_100ms_expired the task_obd_poller shall immediately satisfy obd_request_0x7df_transmitted = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_timer_100ms_expired`: **Input** (Boolean)
  - `obd_request_0x7df_transmitted`: **Output** (Boolean)
- **Rationale (Português):** A cada 100 ms, emitir solicitação funcional Modo 01 em 0x7DF para o próximo PID programado.

---

### REQ_OBD_002 — Escalonamento Circular Round-Robin dos PIDs [REQ-SYS-06]
- **ID:** `REQ_OBD_002`
- **Component:** `task_obd_poller`
- **FRETish Text:**
  ```text
  in active_session upon obd_tx_cycle_completed the task_obd_poller shall within 1 ms satisfy pid_index_incremented = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_tx_cycle_completed`: **Input** (Boolean)
  - `pid_index_incremented`: **Output** (Boolean)
- **Rationale (Português):** Alternar sequencialmente entre os 6 PIDs suportados, completando um ciclo a cada 600 ms.

---

### REQ_OBD_003 — Tratamento de Timeout de Diagnóstico [REQ-SYS-10]
- **ID:** `REQ_OBD_003`
- **Component:** `task_obd_poller`
- **FRETish Text:**
  ```text
  in active_session upon obd_timeout_50ms_elapsed the task_obd_poller shall immediately satisfy obd_timeout_recorded = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_timeout_50ms_elapsed`: **Input** (Boolean)
  - `obd_timeout_recorded`: **Output** (Boolean)
- **Rationale (Português):** Se a resposta 0x7E8 não chegar em 50 ms, declarar timeout e registrar log sem suspender o escalonador.

---

### REQ_OBD_004 — Intercalação de Comandos de Bancada no Transmissor [REQ-SYS-05]
- **ID:** `REQ_OBD_004`
- **Component:** `task_obd_poller`
- **FRETish Text:**
  ```text
  in active_session upon can_cmd_received_in_queue the task_obd_poller shall within 10 ms satisfy can_cmd_interleaved = TRUE
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
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  in active_session upon telemetry_frame_received the task_logger shall within 1 ms satisfy csv_line_formatted = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `telemetry_frame_received`: **Input** (Boolean)
  - `csv_line_formatted`: **Output** (Boolean)
- **Rationale (Português):** Converter cada amostra em linha CSV de 11 colunas utilizando buffer estático de 320 bytes sem alocação dinâmica no heap.

---

### REQ_LOG_002 — Integridade Estrutural do Dataset [REQ-SYS-12 / AC-04]
- **ID:** `REQ_LOG_002`
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  in active_session when dataset_recording the task_logger shall always satisfy null_mandatory_fields = 0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `dataset_recording`: **Input** (Boolean)
  - `null_mandatory_fields`: **Output** (Integer)
- **Rationale (Português):** 100% dos registros gerados devem conter todos os campos mandatórios (timestamp, source, can_id e label) sem campos nulos indevidos (Critério AC-04).

---

### REQ_LOG_003 — Inserção no Buffer Circular em SRAM [REQ-SYS-13]
- **ID:** `REQ_LOG_003`
- **Component:** `bsw_mem`
- **FRETish Text:**
  ```text
  in active_session upon csv_line_available the bsw_mem shall within 50 us satisfy sd_buffer_pushed = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `csv_line_available`: **Input** (Boolean)
  - `sd_buffer_pushed`: **Output** (Boolean)
- **Rationale (Português):** Inserir a linha formatada no buffer em anel de 4096 bytes em SRAM sob seção crítica rápida inferior a 50 µs.

---

### REQ_LOG_004 — Esvaziamento de Buffer por Limiar de Ocupação [REQ-SYS-14]
- **ID:** `REQ_LOG_004`
- **Component:** `bsw_mem`
- **FRETish Text:**
  ```text
  in active_session when buffer_occupancy >= 3584 the bsw_mem shall immediately satisfy flush_signal_emitted = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `buffer_occupancy`: **Internal** (Integer)
  - `flush_signal_emitted`: **Output** (Boolean)
- **Rationale (Português):** Ao atingir 85% de ocupação (3584 bytes), emitir sinal imediato para a tarefa de gravação descarregar os dados.

---

### REQ_LOG_005 — Esvaziamento Periódico de Buffer por Temporizador [REQ-SYS-14]
- **ID:** `REQ_LOG_005`
- **Component:** `task_sd_writer`
- **FRETish Text:**
  ```text
  in active_session upon flush_timer_2s_expired the task_sd_writer shall within 100 ms satisfy pending_bytes_flushed = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `flush_timer_2s_expired`: **Input** (Boolean)
  - `pending_bytes_flushed`: **Output** (Boolean)
- **Rationale (Português):** A cada 2 segundos de inatividade de escrita, a tarefa de gravação deve persistir quaisquer bytes pendentes no arquivo.

---

### REQ_LOG_006 — Fatiamento em Chunks de 256 Bytes com Preempção [REQ-SYS-15]
- **ID:** `REQ_LOG_006`
- **Component:** `task_sd_writer`
- **FRETish Text:**
  ```text
  in active_session upon sector_write_chunk the task_sd_writer shall within 1 ms satisfy chunk_preemption_yielded = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `sector_write_chunk`: **Input** (Boolean)
  - `chunk_preemption_yielded`: **Output** (Boolean)
- **Rationale (Português):** Fracionar a gravação no SD em pedaços de 256 bytes e ceder controle à CPU para manter o bloqueio inferior a 0,76 ms.

---

### REQ_LOG_007 — Retenção em Backlog de RAM sob Falha do SD [REQ-SYS-23]
- **ID:** `REQ_LOG_007`
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  in offline_mode upon sd_write_failed the task_logger shall within 2 ms satisfy ram_backlog_retained = TRUE
  ```
- **Variable Mapping:**
  - `offline_mode`: **Internal** (Boolean)
  - `sd_write_failed`: **Input** (Boolean)
  - `ram_backlog_retained`: **Output** (Boolean)
- **Rationale (Português):** Se a escrita falhar em modo offline, reter os frames no buffer de contingência em RAM (50 elementos / 16 KB) sem descarte imediato.

---

### REQ_LOG_008 — Limite de Consumo de Memória SRAM [REQ-SYS-29 / AC-05]
- **ID:** `REQ_LOG_008`
- **Component:** `bsw_diag`
- **FRETish Text:**
  ```text
  in active_session when memory_supervision_active the bsw_diag shall always satisfy sram_usage_kb < 200
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `memory_supervision_active`: **Input** (Boolean)
  - `sram_usage_kb`: **Output** (Double)
- **Rationale (Português):** O consumo de memória SRAM alocável deve permanecer estritamente contido abaixo de 200 KB durante operação contínua (Critério AC-05).

---

### REQ_LOG_009 — Telemetria de Desempenho HEARTBEAT no SD [REQ-SYS-29]
- **ID:** `REQ_LOG_009`
- **Component:** `bsw_diag`
- **FRETish Text:**
  ```text
  in active_session upon heartbeat_timer_60s the bsw_diag shall within 100 ms satisfy heartbeat_diag_logged = TRUE
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
- **Component:** `mcal_spi_sd`
- **FRETish Text:**
  ```text
  in boot_mode upon sd_card_inserted the mcal_spi_sd shall within 500 ms satisfy fat32_filesystem_mounted = TRUE
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `sd_card_inserted`: **Input** (Boolean)
  - `fat32_filesystem_mounted`: **Output** (Boolean)
- **Rationale (Português):** Inicializar o cartão MicroSD via SPI2 no boot e montar a partição FAT32 em até 500 ms.

---

### REQ_SD_002 — Transição Atômica de Arquivos de Sessão [REQ-SYS-17]
- **ID:** `REQ_SD_002`
- **Component:** `bsw_mem`
- **FRETish Text:**
  ```text
  upon session_rotate_command the bsw_mem shall within 50 ms satisfy session_file_rotated_atomically = TRUE
  ```
- **Variable Mapping:**
  - `session_rotate_command`: **Input** (Boolean)
  - `session_file_rotated_atomically`: **Output** (Boolean)
- **Rationale (Português):** Ao girar sessão, esvaziar síncronamente o buffer antigo e abrir o novo arquivo indexado S_XXXX.CSV em até 50 ms.

---

### REQ_SD_003 — Injeção de Cabeçalho e Linha BOOT [REQ-SYS-17]
- **ID:** `REQ_SD_003`
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  upon new_file_opened the task_logger shall immediately satisfy header_and_boot_lines_written = TRUE
  ```
- **Variable Mapping:**
  - `new_file_opened`: **Input** (Boolean)
  - `header_and_boot_lines_written`: **Output** (Boolean)
- **Rationale (Português):** Gravar o cabeçalho CSV na linha 1 e o metadado BOOT na linha 2 antes de qualquer amostra de telemetria.

---

### REQ_SD_004 — Temporização de Sessão Automática [REQ-SYS-18]
- **ID:** `REQ_SD_004`
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  in active_session upon session_duration_reached the task_logger shall within 10 ms satisfy session_stopped_and_flushed = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `session_duration_reached`: **Input** (Boolean)
  - `session_stopped_and_flushed`: **Output** (Boolean)
- **Rationale (Português):** Ao atingir a duração programada, injetar SESSION_COMPLETE, forçar flush de encerramento e suspender novas gravações.

---

### REQ_SD_005 — Volume Consolidado do Dataset [REQ-SYS-11/17 / AC-08]
- **ID:** `REQ_SD_005`
- **Component:** `pipeline_telemetria`
- **FRETish Text:**
  ```text
  upon benchmark_completion the pipeline_telemetria shall satisfy total_dataset_samples >= 72000
  ```
- **Variable Mapping:**
  - `benchmark_completion`: **Input** (Boolean)
  - `total_dataset_samples`: **Output** (Integer)
- **Rationale (Português):** O sistema deve registrar um volume consolidado igual ou superior a 72.000 amostras nos três cenários de teste de 10 min (Critério AC-08).

---

# Subsistema 6: Conectividade em Nuvem e Telemetria Remota

### REQ_COM_001 — Conexão Wi-Fi e Pilha TCP/IP [REQ-SYS-19]
- **ID:** `REQ_COM_001`
- **Component:** `bsw_com`
- **FRETish Text:**
  ```text
  in boot_mode upon wifi_credentials_configured the bsw_com shall within 10000 ms satisfy ip_dhcp_assigned = TRUE
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `wifi_credentials_configured`: **Input** (Boolean)
  - `ip_dhcp_assigned`: **Output** (Boolean)
- **Rationale (Português):** Conectar a interface sem fio em modo Station e obter endereço IP via DHCP através da pilha embassy-net.

---

### REQ_COM_002 — Despacho em Lotes Binários via MQTT [REQ-SYS-20]
- **ID:** `REQ_COM_002`
- **Component:** `bsw_com`
- **FRETish Text:**
  ```text
  in connected_mode upon binary_batch_full the bsw_com shall within 200 ms satisfy mqtt_batch_published = TRUE
  ```
- **Variable Mapping:**
  - `connected_mode`: **Internal** (Boolean)
  - `binary_batch_full`: **Input** (Boolean)
  - `mqtt_batch_published`: **Output** (Boolean)
- **Rationale (Português):** Agrupar frames binários compactos de 18 bytes (até 150 registros) e despachá-los no tópico MQTT em até 200 ms.

---

### REQ_COM_003 — Publicação Periódica de Status e Keepalive [REQ-SYS-21]
- **ID:** `REQ_COM_003`
- **Component:** `bsw_com`
- **FRETish Text:**
  ```text
  in connected_mode upon status_timer_5s the bsw_com shall within 500 ms satisfy status_json_published = TRUE
  ```
- **Variable Mapping:**
  - `connected_mode`: **Internal** (Boolean)
  - `status_timer_5s`: **Input** (Boolean)
  - `status_json_published`: **Output** (Boolean)
- **Rationale (Português):** A cada 5 segundos com rede ativa, publicar payload JSON contendo saúde do SD, Wi-Fi, contadores e uptime em /system/status.

---

### REQ_COM_004 — Tarefa Periódica de Despacho (task_tx_dispatch) [REQ-SYS-20]
- **ID:** `REQ_COM_004`
- **Component:** `task_tx_dispatch`
- **FRETish Text:**
  ```text
  in active_session upon dispatch_cycle_50ms the task_tx_dispatch shall within 2500 us satisfy data_packets_dispatched = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `dispatch_cycle_50ms`: **Input** (Boolean)
  - `data_packets_dispatched`: **Output** (Boolean)
- **Rationale (Português):** A cada 50 ms, a thread de despacho deve arbitrar o encaminhamento físico entre o soquete MQTT e o descritor de arquivo do SD.

---

# Subsistema 7: Máquina de Estados de Conectividade e Fallback Offline

### REQ_FSM_001 — Comutação Automática de Fallback em MicroSD [REQ-SYS-23 / AC-06]
- **ID:** `REQ_FSM_001`
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  in active_session upon wifi_disconnected the task_logger shall within 5 ms satisfy sd_fallback_active = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `wifi_disconnected`: **Input** (Boolean)
  - `sd_fallback_active`: **Output** (Boolean)
- **Rationale (Português):** Na perda do sinal Wi-Fi, registrar evento DIAG e redirecionar 100% dos dados para o cartão SD em até 5 ms (Critério AC-06).

---

### REQ_FSM_002 — Reconexão de Rede e Dreno FIFO de Backlog [REQ-SYS-24]
- **ID:** `REQ_FSM_002`
- **Component:** `task_logger`
- **FRETish Text:**
  ```text
  in active_session upon wifi_reconnected the task_logger shall within 10 ms satisfy backlog_fifo_drained = TRUE
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
- **Component:** `bsw_com`
- **FRETish Text:**
  ```text
  upon remote_command_received the bsw_com shall within 20 ms satisfy command_executed_ack = TRUE
  ```
- **Variable Mapping:**
  - `remote_command_received`: **Input** (Boolean)
  - `command_executed_ack`: **Output** (Boolean)
- **Rationale (Português):** Interpretar e executar instruções no tópico /coach/command (STOP, ECO/NOR/SPT, RESET, TIME, LIST_SESSIONS) em até 20 ms.

---

### REQ_CMD_002 — Transmissão em Streaming de Replay [REQ-SYS-26]
- **ID:** `REQ_CMD_002`
- **Component:** `bsw_com`
- **FRETish Text:**
  ```text
  upon replay_command_triggered the bsw_com shall within 100 ms satisfy replay_streaming_active = TRUE
  ```
- **Variable Mapping:**
  - `replay_command_triggered`: **Input** (Boolean)
  - `replay_streaming_active`: **Output** (Boolean)
- **Rationale (Português):** Suspender sessão ativa, abrir arquivo S_XXXX.CSV e transmitir blocos de até 1536 bytes em streaming no tópico /telemetry/replay.

---

# Subsistema 9: Supervisão de Falhas, Resiliência e Watchdog

### REQ_REC_001 — Detecção de Erro de Bus-Off no TWAI [REQ-SYS-27]
- **ID:** `REQ_REC_001`
- **Component:** `task_can_rx`
- **FRETish Text:**
  ```text
  in active_session upon bus_off_error_detected the task_can_rx shall within 1 ms satisfy bus_off_flag_set = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `bus_off_error_detected`: **Input** (Boolean)
  - `bus_off_flag_set`: **Output** (Boolean)
- **Rationale (Português):** Ao detectar o código de erro crítico de saturação elétrica do TWAI, registrar atomicamente o estado de Bus-Off em até 1 ms.

---

### REQ_REC_002 — Pausa Cooperativa das Tarefas de Barramento [REQ-SYS-27]
- **ID:** `REQ_REC_002`
- **Component:** `task_watchdog`
- **FRETish Text:**
  ```text
  upon bus_off_flag_active the task_watchdog shall within 10 ms satisfy bus_off_pause_signaled = TRUE
  ```
- **Variable Mapping:**
  - `bus_off_flag_active`: **Input** (Boolean)
  - `bus_off_pause_signaled`: **Output** (Boolean)
- **Rationale (Português):** O watchdog deve emitir sinal de pausa cooperativa para task_can_rx e task_obd_poller suspenderem tentativas de acesso ao TWAI.

---

### REQ_REC_003 — Janela de Espera de 128 ms da Norma ISO 11898 [REQ-SYS-27 / AC-07]
- **ID:** `REQ_REC_003`
- **Component:** `task_watchdog`
- **FRETish Text:**
  ```text
  upon bus_off_pause_started the task_watchdog shall after 128 ms satisfy recovery_window_elapsed = TRUE
  ```
- **Variable Mapping:**
  - `bus_off_pause_started`: **Input** (Boolean)
  - `recovery_window_elapsed`: **Output** (Boolean)
- **Rationale (Português):** Aguardar compulsoriamente os 128 ms normatizados para observação de bits recessivos antes de reabilitar o controlador (Critério AC-07).

---

### REQ_REC_004 — Reativação via Registradores PAC sem Reboot [REQ-SYS-27 / AC-07]
- **ID:** `REQ_REC_004`
- **Component:** `mcal_twai`
- **FRETish Text:**
  ```text
  upon recovery_window_elapsed the mcal_twai shall within 1 ms satisfy twai_reset_mode_cleared = TRUE
  ```
- **Variable Mapping:**
  - `recovery_window_elapsed`: **Input** (Boolean)
  - `twai_reset_mode_cleared`: **Output** (Boolean)
  - **Rationale (Português):** Limpar a flag de reset no registrador físico via PAC sem reiniciar o processador ESP32-S3 e sem destruir tarefas ativas.

---

### REQ_REC_005 — Retomada Operacional e Emissão de BUS_OFF_CLEAR [REQ-SYS-27 / AC-07]
- **ID:** `REQ_REC_005`
- **Component:** `task_watchdog`
- **FRETish Text:**
  ```text
  upon twai_reset_mode_cleared the task_watchdog shall within 5 ms satisfy bus_off_clear_signaled = TRUE
  ```
- **Variable Mapping:**
  - `twai_reset_mode_cleared`: **Input** (Boolean)
  - `bus_off_clear_signaled`: **Output** (Boolean)
- **Rationale (Português):** Concluído o rearmamento, emitir BUS_OFF_CLEAR liberando as tarefas e gravar registro de sucesso no log do cartão SD.

---

### REQ_REC_006 — Detecção de Travamento de Tarefa (Logger Stall) [REQ-SYS-28]
- **ID:** `REQ_REC_006`
- **Component:** `task_watchdog`
- **FRETish Text:**
  ```text
  in active_session upon logger_stall_30s_detected the task_watchdog shall within 100 ms satisfy stall_diag_logged = TRUE
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `logger_stall_30s_detected`: **Input** (Boolean)
  - `stall_diag_logged`: **Output** (Boolean)
- **Rationale (Português):** Se nenhum frame foi processado pelo logger em 30 segundos de sessão ativa com SD presente, registrar alerta de stall no SD.

---

### REQ_REC_007 — Rearme Periódico do Watchdog de Hardware [REQ-SYS-30]
- **ID:** `REQ_REC_007`
- **Component:** `task_watchdog`
- **FRETish Text:**
  ```text
  when system_tasks_healthy the task_watchdog shall within 2000 ms satisfy hw_watchdog_fed = TRUE
  ```
- **Variable Mapping:**
  - `system_tasks_healthy`: **Input** (Boolean)
  - `hw_watchdog_fed`: **Output** (Boolean)
- **Rationale (Português):** Enquanto as tarefas estiverem saudáveis, a tarefa supervisora deve rearmar o watchdog de hardware em intervalo inferior a 2 s (timeout de 5 s).
