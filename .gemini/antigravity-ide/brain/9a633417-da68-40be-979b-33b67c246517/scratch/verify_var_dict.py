import json

# Complete mapping dictionary for all 108 variables:
# (component_name, variable_name) -> (idType, dataType, description)

VAR_DICT = {
    # ================= UNO ECU EMULATOR (24 vars) =================
    ("uno_ecu_emulator", "active_profile"): (
        "Output", "integer",
        "Perfil ativo de dinamica veicular no emulador de ECU (1=Eco, 2=Normal, 3=Sport)."
    ),
    ("uno_ecu_emulator", "active_session"): (
        "Internal", "boolean",
        "Guarda de estado indicando sessao ativa de geracao e transmissao de telemetria no emulador."
    ),
    ("uno_ecu_emulator", "boot_mode"): (
        "Internal", "boolean",
        "Guarda de modo da fase de boot e inicializacao do microcontrolador ATmega328P."
    ),
    ("uno_ecu_emulator", "boot_trigger"): (
        "Input", "boolean",
        "Gatilho de energizacao ou reinicializacao da placa Arduino Uno (ATmega328P)."
    ),
    ("uno_ecu_emulator", "can_bus_operational"): (
        "Output", "boolean",
        "Controlador MCP2515 em modo normal a 500 kbps e LED D4 aceso indicando barramento operacional."
    ),
    ("uno_ecu_emulator", "commanded_profile"): (
        "Input", "integer",
        "Perfil de conducao comandado remotamente via frame CAN ID 0x010 byte 0 (1=Eco, 2=Normal, 3=Sport)."
    ),
    ("uno_ecu_emulator", "coolant_temp_c"): (
        "Output", "double",
        "Temperatura do liquido de arrefecimento do motor em graus Celsius simulada pelo modelo fisico."
    ),
    ("uno_ecu_emulator", "dbc_frames_emitted"): (
        "Output", "boolean",
        "Emissao ciclica concluida dos frames CAN de telemetria proprietaria DBC dentro da janela do Timer1."
    ),
    ("uno_ecu_emulator", "latency_jitter_std_ms"): (
        "Output", "double",
        "Desvio padrao do jitter de latencia round-trip das consultas OBD-II em milissegundos (criterio AC-03 < 3 ms)."
    ),
    ("uno_ecu_emulator", "load_pct"): (
        "Output", "double",
        "Carga calculada do motor em percentual (0 a 100%) gerada pela simulacao senoidal."
    ),
    ("uno_ecu_emulator", "maf_g_s"): (
        "Output", "double",
        "Vazao de massa de ar admitida (MAF) em gramas por segundo calculada pelo modelo fisico."
    ),
    ("uno_ecu_emulator", "mcp2515_hardware_up"): (
        "Input", "boolean",
        "Modulo MCP2515 e transceptor CAN energizados e respondendo a comunicacao SPI."
    ),
    ("uno_ecu_emulator", "mcp2515_rx_polled"): (
        "Output", "boolean",
        "Varredura periodica por polling do registrador de recepcao do MCP2515 concluida."
    ),
    ("uno_ecu_emulator", "mean_obd_latency_ms"): (
        "Output", "double",
        "Latencia media round-trip de resposta as consultas diagnosticas OBD-II em ms (criterio AC-02 < 10 ms)."
    ),
    ("uno_ecu_emulator", "obd_benchmark_running"): (
        "Input", "boolean",
        "Sinalizador indicando que a rotina formal de benchmark de desempenho OBD-II esta em execucao."
    ),
    ("uno_ecu_emulator", "obd_request_0x7df_received"): (
        "Input", "boolean",
        "Deteccao de requisicao OBD-II funcional broadcast no ID CAN 0x7DF."
    ),
    ("uno_ecu_emulator", "obd_response_0x7e8_sent"): (
        "Output", "boolean",
        "Resposta diagnostica OBD-II transmitida no ID CAN 0x7E8 dentro do deadline de 2 ms."
    ),
    ("uno_ecu_emulator", "physics_model_updated"): (
        "Output", "boolean",
        "Atualizacao ciclica das variaveis fisicas moduladas pela curva senoidal em PROGMEM concluida."
    ),
    ("uno_ecu_emulator", "profile_cmd_0x010_received"): (
        "Input", "boolean",
        "Recepcao de novo comando CAN com ID 0x010 para troca de perfil de conducao."
    ),
    ("uno_ecu_emulator", "rpm"): (
        "Output", "double",
        "Rotacao instantanea do motor a combustao (RPM) gerada pelo modelo de fisica."
    ),
    ("uno_ecu_emulator", "speed_kmh"): (
        "Output", "double",
        "Velocidade calculada do veiculo em quilometros por hora (km/h)."
    ),
    ("uno_ecu_emulator", "throttle_pct"): (
        "Output", "double",
        "Posicao relativa da borboleta de aceleracao em percentual (0 a 100%)."
    ),
    ("uno_ecu_emulator", "timer1_50ms_tick"): (
        "Input", "boolean",
        "Interrupcao periodica do Timer1 de hardware a cada 50 ms (20 Hz) para disparo dos frames DBC."
    ),
    ("uno_ecu_emulator", "timer2_1ms_tick"): (
        "Input", "boolean",
        "Interrupcao periodica do Timer2 de hardware a cada 1 ms (1 kHz) para polling do controlador CAN."
    ),

    # ================= ESP32-S3 COLLECTOR (84 vars) =================
    ("esp32s3_collector", "active_session"): (
        "Internal", "boolean",
        "Guarda de modo indicando que a sessao de captura e gravacao de telemetria esta ativa."
    ),
    ("esp32s3_collector", "backlog_fifo_drained"): (
        "Output", "boolean",
        "Esvaziamento completo da fila de backlog em RAM e envio dos dados pendentes via MQTT concluido."
    ),
    ("esp32s3_collector", "benchmark_completion"): (
        "Input", "boolean",
        "Evento de conclusao da rodada formal de testes e benchmark de desempenho da telemetria."
    ),
    ("esp32s3_collector", "binary_batch_full"): (
        "Input", "boolean",
        "Sinalizador indicando lote de telemetria acumulado em memoria atingindo o limite de 150 registros."
    ),
    ("esp32s3_collector", "boot_mode"): (
        "Internal", "boolean",
        "Guarda de modo da fase de boot, energizacao e configuracao dos perifericos MCAL no ESP32-S3."
    ),
    ("esp32s3_collector", "boot_trigger"): (
        "Input", "boolean",
        "Gatilho de inicializacao ou reinicio a frio da placa ESP32-S3."
    ),
    ("esp32s3_collector", "buffer_occupancy"): (
        "Internal", "integer",
        "Volume instantaneo em bytes ocupado no buffer circular de gravacao em MicroSD (capacidade 4096 B)."
    ),
    ("esp32s3_collector", "bus_off_clear_signaled"): (
        "Output", "boolean",
        "Sinal emitido liberando as tarefas do barramento CAN apos recuperacao bem-sucedida de Bus-Off."
    ),
    ("esp32s3_collector", "bus_off_error_detected"): (
        "Input", "boolean",
        "Interrupcao de hardware do periferico TWAI sinalizando ocorrencia de condicao de erro Bus-Off."
    ),
    ("esp32s3_collector", "bus_off_flag_active"): (
        "Input", "boolean",
        "Estado em que a flag de sinalizacao de Bus-Off permanece setada no sistema."
    ),
    ("esp32s3_collector", "bus_off_flag_set"): (
        "Output", "boolean",
        "Flag interna sinalizada pelo driver TWAI registrando a falha de Bus-Off para recuperacao."
    ),
    ("esp32s3_collector", "bus_off_pause_signaled"): (
        "Output", "boolean",
        "Sinal de pausa cooperativa emitido para as tarefas task_can_rx e task_obd_poller suspenderem o TWAI."
    ),
    ("esp32s3_collector", "bus_off_pause_started"): (
        "Input", "boolean",
        "Evento indicando inicio da janela compulsória de espera e observacao de bits recessivos."
    ),
    ("esp32s3_collector", "can_bus_healthy"): (
        "Input", "boolean",
        "Integridade eletrica e logica do barramento CAN atestada sem presenca de condicao de Bus-Off."
    ),
    ("esp32s3_collector", "can_cmd_interleaved"): (
        "Output", "boolean",
        "Comando CAN de bancada intercalado com sucesso entre janelas de polling OBD sem violar a periodicidade."
    ),
    ("esp32s3_collector", "can_cmd_received_in_queue"): (
        "Input", "boolean",
        "Comando de atuacao CAN recebido na fila interna de transmissao prioritaria."
    ),
    ("esp32s3_collector", "can_frame_arrived"): (
        "Input", "boolean",
        "Interrupcao de hardware do TWAI indicando novo frame CAN disponivel no FIFO de recepcao."
    ),
    ("esp32s3_collector", "chunk_preemption_yielded"): (
        "Output", "boolean",
        "Preempcao voluntaria da CPU liberada pela tarefa de escrita do SD entre blocos consecutivos de 512 B."
    ),
    ("esp32s3_collector", "command_executed_ack"): (
        "Output", "boolean",
        "Mensagem de confirmacao (ACK) de execucao de comando remoto publicada no topico MQTT."
    ),
    ("esp32s3_collector", "connected_mode"): (
        "Internal", "boolean",
        "Guarda de modo indicando interface Wi-Fi autenticada e conexao com broker MQTT operacional."
    ),
    ("esp32s3_collector", "csv_line_available"): (
        "Input", "boolean",
        "Linha de telemetria em formato CSV pronta na memoria para enfileiramento de gravacao."
    ),
    ("esp32s3_collector", "csv_line_formatted"): (
        "Output", "boolean",
        "Frame de telemetria convertido e formatado como registro de texto CSV delimitado por virgula."
    ),
    ("esp32s3_collector", "data_packets_dispatche"): (
        "Output", "boolean",
        "Pacotes de telemetria arbitrados e despachados no canal MQTT pela camada bsw_com em ate 2,5 ms de CPU."
    ),
    ("esp32s3_collector", "dataset_recording"): (
        "Input", "boolean",
        "Sessao experimental de gravacao de dataset de telemetria em execucao ativa."
    ),
    ("esp32s3_collector", "dispatch_cycle_50ms"): (
        "Input", "boolean",
        "Sinalizador do ciclo de 50 ms da tarefa periodica de despacho de comunicacao bsw_com."
    ),
    ("esp32s3_collector", "engineering_values_scaled"): (
        "Output", "boolean",
        "Conversao de grandezas fisicas do frame CAN concluida com aplicacao de escala e offset da base DBC."
    ),
    ("esp32s3_collector", "fat32_filesystem_mounted"): (
        "Output", "boolean",
        "Sistema de arquivos FAT32 do cartao MicroSD montado e validado operacional via interface SPI2."
    ),
    ("esp32s3_collector", "flush_signal_emitted"): (
        "Output", "boolean",
        "Sinal de descarga (flush) emitido para forcar gravacao fisica no MicroSD ao atingir 3072 B no buffer."
    ),
    ("esp32s3_collector", "flush_timer_2s_expired"): (
        "Input", "boolean",
        "Expiracao do temporizador periodico de 2 segundos para sincronizacao forcada do buffer de gravacao do SD."
    ),
    ("esp32s3_collector", "frame_loss_percentage"): (
        "Output", "double",
        "Taxa percentual de perda de frames CAN medida durante o ensaio experimental (criterio AC-01 <= 1%)."
    ),
    ("esp32s3_collector", "frame_timestamp_captured"): (
        "Output", "boolean",
        "Captura e vinculacao de timestamp de 64 bits em microssegundos ao frame CAN recebido na interrupcao."
    ),
    ("esp32s3_collector", "header_and_boot_lines_written"): (
        "Output", "boolean",
        "Cabecalho formal CSV e evento inicial de boot persistidos atomicamente no novo arquivo de sessao do SD."
    ),
    ("esp32s3_collector", "heartbeat_diag_logged"): (
        "Output", "boolean",
        "Registro de telemetria interna (heartbeat, saude das tarefas e uptime) persistido no SD."
    ),
    ("esp32s3_collector", "heartbeat_timer_60s"): (
        "Input", "boolean",
        "Expiracao do temporizador periodico de 60 segundos para emissao de diagnostico heartbeat."
    ),
    ("esp32s3_collector", "hw_watchdog_fed"): (
        "Output", "boolean",
        "Pulso de rearme periodico emitido para o Watchdog de Hardware em intervalo inferior a 2 s."
    ),
    ("esp32s3_collector", "ip_dhcp_assigned"): (
        "Output", "boolean",
        "Atribuicao bem-sucedida de endereco IP local via DHCP pela pilha TCP/IP do Wi-Fi."
    ),
    ("esp32s3_collector", "logger_stall_30s_detected"): (
        "Input", "boolean",
        "Deteccao de ausencia de processamento de novos frames de telemetria pelo logger por 30 segundos."
    ),
    ("esp32s3_collector", "mcal_spi_sd_up"): (
        "Input", "boolean",
        "Barramento periferico SPI2 e transceptor do cartao MicroSD inicializados e respondendo."
    ),
    ("esp32s3_collector", "mcal_twai_up"): (
        "Input", "boolean",
        "Periferico TWAI de comunicacao CAN configurado e inicializado operacional a 500 kbps."
    ),
    ("esp32s3_collector", "memory_supervision_active"): (
        "Input", "boolean",
        "Rotina de contabilidade e supervisao de consumo de memoria SRAM em execucao ativa."
    ),
    ("esp32s3_collector", "mqtt_batch_published"): (
        "Output", "boolean",
        "Lote binario de telemetria compacta despachado com sucesso para o broker MQTT no topico de dados."
    ),
    ("esp32s3_collector", "mqtt_broker_connected"): (
        "Output", "boolean",
        "Sessao TCP e handshake de protocolo estabelecidos com sucesso com o broker MQTT remoto."
    ),
    ("esp32s3_collector", "mqtt_credentials_valid"): (
        "Input", "boolean",
        "Parametros de endereco, porta e credenciais de autenticacao do broker MQTT validados no sistema."
    ),
    ("esp32s3_collector", "new_file_opened"): (
        "Input", "boolean",
        "Novo arquivo de sessao criado e aberto para escrita no sistema de arquivos do cartao MicroSD."
    ),
    ("esp32s3_collector", "null_mandatory_fields"): (
        "Output", "integer",
        "Contagem de campos obrigatorios nulos detectados nas linhas do dataset (invariante: deve ser 0)."
    ),
    ("esp32s3_collector", "obd_request_0x7df_transmitted"): (
        "Output", "boolean",
        "Requisicao padrao funcional OBD-II transmitida no barramento CAN com identificador 0x7DF."
    ),
    ("esp32s3_collector", "obd_timeout_50ms_elapsed"): (
        "Input", "boolean",
        "Expiracao da janela limite de 50 ms sem recepcao de resposta OBD-II 0x7E8 da ECU."
    ),
    ("esp32s3_collector", "obd_timeout_recorded"): (
        "Output", "boolean",
        "Ocorrencia de timeout de resposta OBD-II contabilizada nas metricas de integridade e diagnostico."
    ),
    ("esp32s3_collector", "obd_timer_100ms_expired"): (
        "Input", "boolean",
        "Expiracao do temporizador ciclico de 100 ms (10 Hz) para disparo de nova consulta OBD-II."
    ),
    ("esp32s3_collector", "obd_tx_cycle_completed"): (
        "Input", "boolean",
        "Ciclo de transmissao de requisicao e recepcao de resposta OBD-II finalizado com sucesso."
    ),
    ("esp32s3_collector", "offline_mode"): (
        "Internal", "boolean",
        "Guarda de modo indicando desconexao de rede Wi-Fi e comutacao para armazenamento local no SD."
    ),
    ("esp32s3_collector", "overflow_logged_and_dropped"): (
        "Output", "boolean",
        "Ocorrencia de estouro de canal RTE estatico registrada em diagnostico com descarte controlado de frame."
    ),
    ("esp32s3_collector", "pending_bytes_flushed"): (
        "Output", "boolean",
        "Gravacao fisica de todos os bytes pendentes no buffer circular do SD e sincronizacao do arquivo concluida."
    ),
    ("esp32s3_collector", "pid_index_incremented"): (
        "Output", "boolean",
        "Indice sequencial do proximo parametro PID (velocidade, rotacao, carga, etc.) incrementado no poller."
    ),
    ("esp32s3_collector", "ram_backlog_retained"): (
        "Output", "boolean",
        "Frames de telemetria retidos no buffer FIFO em RAM durante indisponibilidade transiente do MicroSD."
    ),
    ("esp32s3_collector", "raw_can_frame_ready"): (
        "Input", "boolean",
        "Frame CAN bruto disponivel na fila de recepcao para decodificacao de sinais."
    ),
    ("esp32s3_collector", "recovery_window_elapsed"): (
        "Output", "boolean",
        "Janela normativa de espera de 128 ms decorrida autorizando o reinicio do controlador TWAI."
    ),
    ("esp32s3_collector", "remote_command_received"): (
        "Input", "boolean",
        "Comando de controle ou configuracao recebido atraves de subscricao no topico de comando MQTT."
    ),
    ("esp32s3_collector", "replay_command_triggered"): (
        "Input", "boolean",
        "Comando de retransmissao (replay) de dados historicos do cartao SD validado e disparado."
    ),
    ("esp32s3_collector", "replay_streaming_active"): (
        "Output", "boolean",
        "Sessao de streaming de registros historicos do MicroSD ativa para envio continuo via MQTT."
    ),
    ("esp32s3_collector", "rte_channel_overflow"): (
        "Input", "boolean",
        "Deteccao de fila cheia ou tentativa de insercao em canal estatico RTE com capacidade maxima esgotada."
    ),
    ("esp32s3_collector", "rte_channel_pushed"): (
        "Output", "boolean",
        "Frame de telemetria decodificado inserido com sucesso no canal estatico RTE de comunicacao inter-tarefas."
    ),
    ("esp32s3_collector", "sd_buffer_pushed"): (
        "Output", "boolean",
        "Linha de telemetria inserida com sucesso no buffer circular em RAM da tarefa de gravacao do SD."
    ),
    ("esp32s3_collector", "sd_card_inserted"): (
        "Input", "boolean",
        "Presenca fisica do cartao de memoria MicroSD detectada e comunicacao SPI operacional."
    ),
    ("esp32s3_collector", "sd_fallback_active"): (
        "Output", "boolean",
        "Modo de contingencia com armazenamento exclusivo no cartao MicroSD ativado apos queda de rede Wi-Fi."
    ),
    ("esp32s3_collector", "sd_write_failed"): (
        "Input", "boolean",
        "Deteccao de erro de I/O na tentativa de escrita de bloco fisico no cartao MicroSD."
    ),
    ("esp32s3_collector", "sector_write_chunk"): (
        "Input", "boolean",
        "Bloco de dados alinhado ao setor de 512 bytes pronto para gravacao em lote no MicroSD."
    ),
    ("esp32s3_collector", "session_duration_reached"): (
        "Input", "boolean",
        "Duracao limite configurada para a sessao de ensaio de telemetria atingida."
    ),
    ("esp32s3_collector", "session_file_rotated_atomically"): (
        "Output", "boolean",
        "Arquivo de log anterior sincronizado e fechado com abertura atomica de novo arquivo sequencial no SD."
    ),
    ("esp32s3_collector", "session_rotate_command"): (
        "Input", "boolean",
        "Comando explicito recebido requisitando a rotacao imediata do arquivo de telemetria em gravacao."
    ),
    ("esp32s3_collector", "session_stopped_and_flushed"): (
        "Output", "boolean",
        "Sessao finalizada com descarregamento de todos os buffers e fechamento seguro do arquivo no SD."
    ),
    ("esp32s3_collector", "sram_usage_kb"): (
        "Output", "double",
        "Consumo total de memoria RAM estatica e dinamica medido em kilobytes (criterio AC-05 < 200 KB)."
    ),
    ("esp32s3_collector", "stall_diag_logged"): (
        "Output", "boolean",
        "Alerta diagnostico de travamento da tarefa logger persistido no log de eventos do SD."
    ),
    ("esp32s3_collector", "status_json_published"): (
        "Output", "boolean",
        "Mensagem JSON contendo status operacional, telemetria interna e uptime publicada no topico MQTT."
    ),
    ("esp32s3_collector", "status_timer_5s"): (
        "Input", "boolean",
        "Expiracao do temporizador periodico de 5 segundos para publicacao de status operacional."
    ),
    ("esp32s3_collector", "system_tasks_healthy"): (
        "Input", "boolean",
        "Sinalizador confirmando que todas as tarefas essenciais do sistema estao ativas e sem travamento."
    ),
    ("esp32s3_collector", "telemetry_frame_parsed"): (
        "Input", "boolean",
        "Frame CAN bruto decodificado e validado pronto para ser despachado no canal RTE correspondente."
    ),
    ("esp32s3_collector", "telemetry_frame_received"): (
        "Input", "boolean",
        "Estrutura de dados de telemetria recebida na fila de processamento da tarefa de gravacao."
    ),
    ("esp32s3_collector", "total_dataset_samples"): (
        "Output", "integer",
        "Numero acumulado total de amostras de telemetria registradas no ensaio (criterio AC-08 >= 72000)."
    ),
    ("esp32s3_collector", "twai_async_enabled"): (
        "Output", "boolean",
        "Modo assincrono do controlador TWAI configurado com interrupcoes e filas habilitadas."
    ),
    ("esp32s3_collector", "twai_reset_mode_cleared"): (
        "Output", "boolean",
        "Flag de reset no registrador de controle fisico do TWAI desmarcada via PAC sem reboot da CPU."
    ),
    ("esp32s3_collector", "wifi_credentials_configured"): (
        "Input", "boolean",
        "Parametros de rede Wi-Fi (SSID e senha) configurados e disponiveis na memoria do sistema."
    ),
    ("esp32s3_collector", "wifi_disconnected"): (
        "Input", "boolean",
        "Evento sinalizando perda de conexao ou desconexao fisica da interface de rede Wi-Fi."
    ),
    ("esp32s3_collector", "wifi_reconnected"): (
        "Input", "boolean",
        "Restabelecimento bem-sucedido da conexao Wi-Fi com obtencao de endereco IP."
    )
}

print(f"Total entries mapped in dictionary: {len(VAR_DICT)}")

# Verify all variables in EdgeTelemetryLayer_req_var.json exist in VAR_DICT
with open('docs/MBSE/EdgeTelemetryLayer_req_var.json', encoding='utf-8') as f:
    j = json.load(f)

missing = []
for v in j['variables']:
    comp = v['component_name']
    name = v['variable_name']
    if (comp, name) not in VAR_DICT:
        missing.append((comp, name))

print(f"Missing from dictionary: {missing}")
