# -*- coding: utf-8 -*-
import json

VAR_DICT = {
    # ================= UNO ECU EMULATOR (24 vars) =================
    ("uno_ecu_emulator", "active_profile"): (
        "Output", "integer",
        "Perfil ativo de dinâmica veicular no emulador de ECU (1=Eco, 2=Normal, 3=Sport)."
    ),
    ("uno_ecu_emulator", "active_session"): (
        "Internal", "boolean",
        "Guarda de estado indicando sessão ativa de geração e transmissão de telemetria no emulador."
    ),
    ("uno_ecu_emulator", "boot_mode"): (
        "Internal", "boolean",
        "Guarda de modo da fase de boot e inicialização do microcontrolador ATmega328P."
    ),
    ("uno_ecu_emulator", "boot_trigger"): (
        "Input", "boolean",
        "Gatilho de energização ou reinicialização da placa Arduino Uno (ATmega328P)."
    ),
    ("uno_ecu_emulator", "can_bus_operational"): (
        "Output", "boolean",
        "Controlador MCP2515 em modo normal a 500 kbps e LED D4 aceso indicando barramento operacional."
    ),
    ("uno_ecu_emulator", "commanded_profile"): (
        "Input", "integer",
        "Perfil de condução comandado remotamente via frame CAN ID 0x010 byte 0 (1=Eco, 2=Normal, 3=Sport)."
    ),
    ("uno_ecu_emulator", "coolant_temp_c"): (
        "Output", "double",
        "Temperatura do líquido de arrefecimento do motor em graus Celsius simulada pelo modelo físico."
    ),
    ("uno_ecu_emulator", "dbc_frames_emitted"): (
        "Output", "boolean",
        "Emissão cíclica concluída dos frames CAN de telemetria proprietária DBC dentro da janela do Timer1."
    ),
    ("uno_ecu_emulator", "latency_jitter_std_ms"): (
        "Output", "double",
        "Desvio padrão do jitter de latência round-trip das consultas OBD-II em milissegundos (critério AC-03 < 3 ms)."
    ),
    ("uno_ecu_emulator", "load_pct"): (
        "Output", "double",
        "Carga calculada do motor em percentual (0 a 100%) gerada pela simulação senoidal."
    ),
    ("uno_ecu_emulator", "maf_g_s"): (
        "Output", "double",
        "Vazão de massa de ar admitida (MAF) em gramas por segundo calculada pelo modelo físico."
    ),
    ("uno_ecu_emulator", "mcp2515_hardware_up"): (
        "Input", "boolean",
        "Módulo MCP2515 e transceptor CAN energizados e respondendo à comunicação SPI."
    ),
    ("uno_ecu_emulator", "mcp2515_rx_polled"): (
        "Output", "boolean",
        "Varredura periódica por polling do registrador de recepção do MCP2515 concluída."
    ),
    ("uno_ecu_emulator", "mean_obd_latency_ms"): (
        "Output", "double",
        "Latência média round-trip de resposta às consultas diagnósticas OBD-II em ms (critério AC-02 < 10 ms)."
    ),
    ("uno_ecu_emulator", "obd_benchmark_running"): (
        "Input", "boolean",
        "Sinalizador indicando que a rotina formal de benchmark de desempenho OBD-II está em execução."
    ),
    ("uno_ecu_emulator", "obd_request_0x7df_received"): (
        "Input", "boolean",
        "Detecção de requisição OBD-II funcional broadcast no ID CAN 0x7DF."
    ),
    ("uno_ecu_emulator", "obd_response_0x7e8_sent"): (
        "Output", "boolean",
        "Resposta diagnóstica OBD-II transmitida no ID CAN 0x7E8 dentro do deadline de 2 ms."
    ),
    ("uno_ecu_emulator", "physics_model_updated"): (
        "Output", "boolean",
        "Atualização cíclica das variáveis físicas moduladas pela curva senoidal em PROGMEM concluída."
    ),
    ("uno_ecu_emulator", "profile_cmd_0x010_received"): (
        "Input", "boolean",
        "Recepção de novo comando CAN com ID 0x010 para troca de perfil de condução."
    ),
    ("uno_ecu_emulator", "rpm"): (
        "Output", "double",
        "Rotação instantânea do motor a combustão (RPM) gerada pelo modelo de física."
    ),
    ("uno_ecu_emulator", "speed_kmh"): (
        "Output", "double",
        "Velocidade calculada do veículo em quilômetros por hora (km/h)."
    ),
    ("uno_ecu_emulator", "throttle_pct"): (
        "Output", "double",
        "Posição relativa da borboleta de aceleração em percentual (0 a 100%)."
    ),
    ("uno_ecu_emulator", "timer1_50ms_tick"): (
        "Input", "boolean",
        "Interrupção periódica do Timer1 de hardware a cada 50 ms (20 Hz) para disparo dos frames DBC."
    ),
    ("uno_ecu_emulator", "timer2_1ms_tick"): (
        "Input", "boolean",
        "Interrupção periódica do Timer2 de hardware a cada 1 ms (1 kHz) para polling do controlador CAN."
    ),

    # ================= ESP32-S3 COLLECTOR (84 vars) =================
    ("esp32s3_collector", "active_session"): (
        "Internal", "boolean",
        "Guarda de modo indicando que a sessão de captura e gravação de telemetria está ativa."
    ),
    ("esp32s3_collector", "backlog_fifo_drained"): (
        "Output", "boolean",
        "Esvaziamento completo da fila de backlog em RAM e envio dos dados pendentes via MQTT concluído."
    ),
    ("esp32s3_collector", "benchmark_completion"): (
        "Input", "boolean",
        "Evento de conclusão da rodada formal de testes e benchmark de desempenho da telemetria."
    ),
    ("esp32s3_collector", "binary_batch_full"): (
        "Input", "boolean",
        "Sinalizador indicando lote de telemetria acumulado em memória atingindo o limite de 150 registros."
    ),
    ("esp32s3_collector", "boot_mode"): (
        "Internal", "boolean",
        "Guarda de modo da fase de boot, energização e configuração dos periféricos MCAL no ESP32-S3."
    ),
    ("esp32s3_collector", "boot_trigger"): (
        "Input", "boolean",
        "Gatilho de inicialização ou reinício a frio da placa ESP32-S3."
    ),
    ("esp32s3_collector", "buffer_occupancy"): (
        "Internal", "integer",
        "Volume instantâneo em bytes ocupado no buffer circular de gravação em MicroSD (capacidade 4096 B)."
    ),
    ("esp32s3_collector", "bus_off_clear_signaled"): (
        "Output", "boolean",
        "Sinal emitido liberando as tarefas do barramento CAN após recuperação bem-sucedida de Bus-Off."
    ),
    ("esp32s3_collector", "bus_off_error_detected"): (
        "Input", "boolean",
        "Interrupção de hardware do periférico TWAI sinalizando ocorrência de condição de erro Bus-Off."
    ),
    ("esp32s3_collector", "bus_off_flag_active"): (
        "Input", "boolean",
        "Estado em que a flag de sinalização de Bus-Off permanece ativa no sistema."
    ),
    ("esp32s3_collector", "bus_off_flag_set"): (
        "Output", "boolean",
        "Flag interna sinalizada pelo driver TWAI registrando a falha de Bus-Off para recuperação."
    ),
    ("esp32s3_collector", "bus_off_pause_signaled"): (
        "Output", "boolean",
        "Sinal de pausa cooperativa emitido para as tarefas task_can_rx e task_obd_poller suspenderem o TWAI."
    ),
    ("esp32s3_collector", "bus_off_pause_started"): (
        "Input", "boolean",
        "Evento indicando início da janela compulsória de espera e observação de bits recessivos."
    ),
    ("esp32s3_collector", "can_bus_healthy"): (
        "Input", "boolean",
        "Integridade elétrica e lógica do barramento CAN atestada sem presença de condição de Bus-Off."
    ),
    ("esp32s3_collector", "can_cmd_interleaved"): (
        "Output", "boolean",
        "Comando CAN de bancada intercalado com sucesso entre janelas de polling OBD sem violar a periodicidade."
    ),
    ("esp32s3_collector", "can_cmd_received_in_queue"): (
        "Input", "boolean",
        "Comando de atuação CAN recebido na fila interna de transmissão prioritária."
    ),
    ("esp32s3_collector", "can_frame_arrived"): (
        "Input", "boolean",
        "Interrupção de hardware do TWAI indicando novo frame CAN disponível no FIFO de recepção."
    ),
    ("esp32s3_collector", "chunk_preemption_yielded"): (
        "Output", "boolean",
        "Preempção voluntária da CPU liberada pela tarefa de escrita do SD entre blocos consecutivos de 512 B."
    ),
    ("esp32s3_collector", "command_executed_ack"): (
        "Output", "boolean",
        "Mensagem de confirmação (ACK) de execução de comando remoto publicada no tópico MQTT."
    ),
    ("esp32s3_collector", "connected_mode"): (
        "Internal", "boolean",
        "Guarda de modo indicando interface Wi-Fi autenticada e conexão com broker MQTT operacional."
    ),
    ("esp32s3_collector", "csv_line_available"): (
        "Input", "boolean",
        "Linha de telemetria em formato CSV pronta na memória para enfileiramento de gravação."
    ),
    ("esp32s3_collector", "csv_line_formatted"): (
        "Output", "boolean",
        "Frame de telemetria convertido e formatado como registro de texto CSV delimitado por vírgula."
    ),
    ("esp32s3_collector", "data_packets_dispatche"): (
        "Output", "boolean",
        "Pacotes de telemetria arbitrados e despachados no canal MQTT pela camada bsw_com em até 2,5 ms de CPU."
    ),
    ("esp32s3_collector", "dataset_recording"): (
        "Input", "boolean",
        "Sessão experimental de gravação de dataset de telemetria em execução ativa."
    ),
    ("esp32s3_collector", "dispatch_cycle_50ms"): (
        "Input", "boolean",
        "Sinalizador do ciclo de 50 ms da tarefa periódica de despacho de comunicação bsw_com."
    ),
    ("esp32s3_collector", "engineering_values_scaled"): (
        "Output", "boolean",
        "Conversão de grandezas físicas do frame CAN concluída com aplicação de escala e offset da base DBC."
    ),
    ("esp32s3_collector", "fat32_filesystem_mounted"): (
        "Output", "boolean",
        "Sistema de arquivos FAT32 do cartão MicroSD montado e validado operacional via interface SPI2."
    ),
    ("esp32s3_collector", "flush_signal_emitted"): (
        "Output", "boolean",
        "Sinal de descarga (flush) emitido para forçar gravação física no MicroSD ao atingir 3072 B no buffer."
    ),
    ("esp32s3_collector", "flush_timer_2s_expired"): (
        "Input", "boolean",
        "Expiração do temporizador periódico de 2 segundos para sincronização forçada do buffer de gravação do SD."
    ),
    ("esp32s3_collector", "frame_loss_percentage"): (
        "Output", "double",
        "Taxa percentual de perda de frames CAN medida durante o ensaio experimental (critério AC-01 <= 1%)."
    ),
    ("esp32s3_collector", "frame_timestamp_captured"): (
        "Output", "boolean",
        "Captura e vinculação de timestamp de 64 bits em microssegundos ao frame CAN recebido na interrupção."
    ),
    ("esp32s3_collector", "header_and_boot_lines_written"): (
        "Output", "boolean",
        "Cabeçalho formal CSV e evento inicial de boot persistidos atomicamente no novo arquivo de sessão do SD."
    ),
    ("esp32s3_collector", "heartbeat_diag_logged"): (
        "Output", "boolean",
        "Registro de telemetria interna (heartbeat, saúde das tarefas e uptime) persistido no SD."
    ),
    ("esp32s3_collector", "heartbeat_timer_60s"): (
        "Input", "boolean",
        "Expiração do temporizador periódico de 60 segundos para emissão de diagnóstico heartbeat."
    ),
    ("esp32s3_collector", "hw_watchdog_fed"): (
        "Output", "boolean",
        "Pulso de rearme periódico emitido para o Watchdog de Hardware em intervalo inferior a 2 s."
    ),
    ("esp32s3_collector", "ip_dhcp_assigned"): (
        "Output", "boolean",
        "Atribuição bem-sucedida de endereço IP local via DHCP pela pilha TCP/IP do Wi-Fi."
    ),
    ("esp32s3_collector", "logger_stall_30s_detected"): (
        "Input", "boolean",
        "Detecção de ausência de processamento de novos frames de telemetria pelo logger por 30 segundos."
    ),
    ("esp32s3_collector", "mcal_spi_sd_up"): (
        "Input", "boolean",
        "Barramento periférico SPI2 e transceptor do cartão MicroSD inicializados e respondendo."
    ),
    ("esp32s3_collector", "mcal_twai_up"): (
        "Input", "boolean",
        "Periférico TWAI de comunicação CAN configurado e inicializado operacional a 500 kbps."
    ),
    ("esp32s3_collector", "memory_supervision_active"): (
        "Input", "boolean",
        "Rotina de contabilidade e supervisão de consumo de memória SRAM em execução ativa."
    ),
    ("esp32s3_collector", "mqtt_batch_published"): (
        "Output", "boolean",
        "Lote binário de telemetria compacta despachado com sucesso para o broker MQTT no tópico de dados."
    ),
    ("esp32s3_collector", "mqtt_broker_connected"): (
        "Output", "boolean",
        "Sessão TCP e handshake de protocolo estabelecidos com sucesso com o broker MQTT remoto."
    ),
    ("esp32s3_collector", "mqtt_credentials_valid"): (
        "Input", "boolean",
        "Parâmetros de endereço, porta e credenciais de autenticação do broker MQTT validados no sistema."
    ),
    ("esp32s3_collector", "new_file_opened"): (
        "Input", "boolean",
        "Novo arquivo de sessão criado e aberto para escrita no sistema de arquivos do cartão MicroSD."
    ),
    ("esp32s3_collector", "null_mandatory_fields"): (
        "Output", "integer",
        "Contagem de campos obrigatórios nulos detectados nas linhas do dataset (invariante: deve ser 0)."
    ),
    ("esp32s3_collector", "obd_request_0x7df_transmitted"): (
        "Output", "boolean",
        "Requisição padrão funcional OBD-II transmitida no barramento CAN com identificador 0x7DF."
    ),
    ("esp32s3_collector", "obd_timeout_50ms_elapsed"): (
        "Input", "boolean",
        "Expiração da janela limite de 50 ms sem recepção de resposta OBD-II 0x7E8 da ECU."
    ),
    ("esp32s3_collector", "obd_timeout_recorded"): (
        "Output", "boolean",
        "Ocorrência de timeout de resposta OBD-II contabilizada nas métricas de integridade e diagnóstico."
    ),
    ("esp32s3_collector", "obd_timer_100ms_expired"): (
        "Input", "boolean",
        "Expiração do temporizador cíclico de 100 ms (10 Hz) para disparo de nova consulta OBD-II."
    ),
    ("esp32s3_collector", "obd_tx_cycle_completed"): (
        "Input", "boolean",
        "Ciclo de transmissão de requisição e recepção de resposta OBD-II finalizado com sucesso."
    ),
    ("esp32s3_collector", "offline_mode"): (
        "Internal", "boolean",
        "Guarda de modo indicando desconexão de rede Wi-Fi e comutação para armazenamento local no SD."
    ),
    ("esp32s3_collector", "overflow_logged_and_dropped"): (
        "Output", "boolean",
        "Ocorrência de estouro de canal RTE estático registrada em diagnóstico com descarte controlado de frame."
    ),
    ("esp32s3_collector", "pending_bytes_flushed"): (
        "Output", "boolean",
        "Gravação física de todos os bytes pendentes no buffer circular do SD e sincronização do arquivo concluída."
    ),
    ("esp32s3_collector", "pid_index_incremented"): (
        "Output", "boolean",
        "Índice sequencial do próximo parâmetro PID (velocidade, rotação, carga, etc.) incrementado no poller."
    ),
    ("esp32s3_collector", "ram_backlog_retained"): (
        "Output", "boolean",
        "Frames de telemetria retidos no buffer FIFO em RAM durante indisponibilidade transiente do MicroSD."
    ),
    ("esp32s3_collector", "raw_can_frame_ready"): (
        "Input", "boolean",
        "Frame CAN bruto disponível na fila de recepção para decodificação de sinais."
    ),
    ("esp32s3_collector", "recovery_window_elapsed"): (
        "Output", "boolean",
        "Janela normativa de espera de 128 ms decorrida autorizando o reinício do controlador TWAI."
    ),
    ("esp32s3_collector", "remote_command_received"): (
        "Input", "boolean",
        "Comando de controle ou configuração recebido através de subscrição no tópico de comando MQTT."
    ),
    ("esp32s3_collector", "replay_command_triggered"): (
        "Input", "boolean",
        "Comando de retransmissão (replay) de dados históricos do cartão SD validado e disparado."
    ),
    ("esp32s3_collector", "replay_streaming_active"): (
        "Output", "boolean",
        "Sessão de streaming de registros históricos do MicroSD ativa para envio contínuo via MQTT."
    ),
    ("esp32s3_collector", "rte_channel_overflow"): (
        "Input", "boolean",
        "Detecção de fila cheia ou tentativa de inserção em canal estático RTE com capacidade máxima esgotada."
    ),
    ("esp32s3_collector", "rte_channel_pushed"): (
        "Output", "boolean",
        "Frame de telemetria decodificado inserido com sucesso no canal estático RTE de comunicação inter-tarefas."
    ),
    ("esp32s3_collector", "sd_buffer_pushed"): (
        "Output", "boolean",
        "Linha de telemetria inserida com sucesso no buffer circular em RAM da tarefa de gravação do SD."
    ),
    ("esp32s3_collector", "sd_card_inserted"): (
        "Input", "boolean",
        "Presença física do cartão de memória MicroSD detectada e comunicação SPI operacional."
    ),
    ("esp32s3_collector", "sd_fallback_active"): (
        "Output", "boolean",
        "Modo de contingência com armazenamento exclusivo no cartão MicroSD ativado após queda de rede Wi-Fi."
    ),
    ("esp32s3_collector", "sd_write_failed"): (
        "Input", "boolean",
        "Detecção de erro de I/O na tentativa de escrita de bloco físico no cartão MicroSD."
    ),
    ("esp32s3_collector", "sector_write_chunk"): (
        "Input", "boolean",
        "Bloco de dados alinhado ao setor de 512 bytes pronto para gravação em lote no MicroSD."
    ),
    ("esp32s3_collector", "session_duration_reached"): (
        "Input", "boolean",
        "Duração limite configurada para a sessão de ensaio de telemetria atingida."
    ),
    ("esp32s3_collector", "session_file_rotated_atomically"): (
        "Output", "boolean",
        "Arquivo de log anterior sincronizado e fechado com abertura atômica de novo arquivo sequencial no SD."
    ),
    ("esp32s3_collector", "session_rotate_command"): (
        "Input", "boolean",
        "Comando explícito recebido requisitando a rotação imediata do arquivo de telemetria em gravação."
    ),
    ("esp32s3_collector", "session_stopped_and_flushed"): (
        "Output", "boolean",
        "Sessão finalizada com descarregamento de todos os buffers e fechamento seguro do arquivo no SD."
    ),
    ("esp32s3_collector", "sram_usage_kb"): (
        "Output", "double",
        "Consumo total de memória RAM estática e dinâmica medido em kilobytes (critério AC-05 < 200 KB)."
    ),
    ("esp32s3_collector", "stall_diag_logged"): (
        "Output", "boolean",
        "Alerta diagnóstico de travamento da tarefa logger persistido no log de eventos do SD."
    ),
    ("esp32s3_collector", "status_json_published"): (
        "Output", "boolean",
        "Mensagem JSON contendo status operacional, telemetria interna e uptime publicada no tópico MQTT."
    ),
    ("esp32s3_collector", "status_timer_5s"): (
        "Input", "boolean",
        "Expiração do temporizador periódico de 5 segundos para publicação de status operacional."
    ),
    ("esp32s3_collector", "system_tasks_healthy"): (
        "Input", "boolean",
        "Sinalizador confirmando que todas as tarefas essenciais do sistema estão ativas e sem travamento."
    ),
    ("esp32s3_collector", "telemetry_frame_parsed"): (
        "Input", "boolean",
        "Frame CAN bruto decodificado e validado pronto para ser despachado no canal RTE correspondente."
    ),
    ("esp32s3_collector", "telemetry_frame_received"): (
        "Input", "boolean",
        "Estrutura de dados de telemetria recebida na fila de processamento da tarefa de gravação."
    ),
    ("esp32s3_collector", "total_dataset_samples"): (
        "Output", "integer",
        "Número acumulado total de amostras de telemetria registradas no ensaio (critério AC-08 >= 72000)."
    ),
    ("esp32s3_collector", "twai_async_enabled"): (
        "Output", "boolean",
        "Modo assíncrono do controlador TWAI configurado com interrupções e filas habilitadas."
    ),
    ("esp32s3_collector", "twai_reset_mode_cleared"): (
        "Output", "boolean",
        "Flag de reset no registrador de controle físico do TWAI desmarcada via PAC sem reboot da CPU."
    ),
    ("esp32s3_collector", "wifi_credentials_configured"): (
        "Input", "boolean",
        "Parâmetros de rede Wi-Fi (SSID e senha) configurados e disponíveis na memória do sistema."
    ),
    ("esp32s3_collector", "wifi_disconnected"): (
        "Input", "boolean",
        "Evento sinalizando perda de conexão ou desconexão física da interface de rede Wi-Fi."
    ),
    ("esp32s3_collector", "wifi_reconnected"): (
        "Input", "boolean",
        "Restabelecimento bem-sucedido da conexão Wi-Fi com obtenção de endereço IP."
    )
}
