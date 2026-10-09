//! Serviço de Comunicação BSW (Wi-Fi + MQTT via rust-mqtt).
//!
//! Gerencia o estado de rede (D-05), publicação MQTT assíncrona com autenticação e
//! relatório detalhado de erros de conexão e status do sistema.

use embassy_net::{tcp::TcpSocket, IpAddress, IpEndpoint, Ipv4Address, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{sta::StationConfig, Config as WifiConf, WifiController};
use portable_atomic::{AtomicBool, AtomicU8, Ordering};
use rust_mqtt::client::{client::MqttClient, client_config::ClientConfig};
use rust_mqtt::packet::v5::publish_packet::QualityOfService;

use crate::config_local;

/// Wrapper para TcpSocket 0.7 implementar os traits 0.6 para o rust-mqtt.
pub struct SocketWrapper<'a>(pub TcpSocket<'a>);

#[derive(Debug)]
pub struct SocketWrapperError;

impl embedded_io_async_06::Error for SocketWrapperError {
    fn kind(&self) -> embedded_io_async_06::ErrorKind {
        embedded_io_async_06::ErrorKind::Other
    }
}

impl<'a> embedded_io_async_06::ErrorType for SocketWrapper<'a> {
    type Error = SocketWrapperError;
}

impl<'a> embedded_io_async_06::Read for SocketWrapper<'a> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        // TcpSocket implements embedded_io_async 0.7 Read
        match embassy_net::tcp::TcpSocket::read(&mut self.0, buf).await {
            Ok(n) => Ok(n),
            Err(_) => Err(SocketWrapperError),
        }
    }
}

impl<'a> embedded_io_async_06::Write for SocketWrapper<'a> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        match embassy_net::tcp::TcpSocket::write(&mut self.0, buf).await {
            Ok(n) => Ok(n),
            Err(_) => Err(SocketWrapperError),
        }
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        match embassy_net::tcp::TcpSocket::flush(&mut self.0).await {
            Ok(_) => Ok(()),
            Err(_) => Err(SocketWrapperError),
        }
    }
}

impl<'a> embedded_io_async_06::ReadReady for SocketWrapper<'a> {
    fn read_ready(&mut self) -> Result<bool, <Self as embedded_io_async_06::ErrorType>::Error> {
        Ok(self.0.can_recv())
    }
}

/// Estado de conectividade da rede: 0=Disconnected, 1=Connected, 2=Reconnecting.
pub static CONN_STATE: AtomicU8 = AtomicU8::new(0);

use crate::types::BinaryFrame;

/// Status operacional da interface Wi-Fi.
pub static WIFI_OK: AtomicBool = AtomicBool::new(false);

/// Canal para envio de BinaryFrames MQTT (capacidade 500 = 9 KB em SRAM).
pub static MQTT_TX_CHANNEL: Channel<CriticalSectionRawMutex, BinaryFrame, 500> = Channel::new();

pub fn mqtt_publish_binary(frame: BinaryFrame) -> Result<(), ()> {
    if CONN_STATE.load(Ordering::Relaxed) != 1 {
        return Err(());
    }
    match MQTT_TX_CHANNEL.try_send(frame) {
        Ok(_) => Ok(()),
        Err(_) => {
            log::warn!("MQTT TX channel full, frame dropped");
            Err(())
        }
    }
}

/// Task Embassy do Core Network Stack (processamento de pacotes TCP/IP)
#[embassy_executor::task]
pub async fn task_net_stack(mut runner: embassy_net::Runner<'static, esp_radio::wifi::Interface>) {
    runner.run().await
}

/// Task Embassy de gerenciamento, diagnóstico e reconexão Wi-Fi / MQTT.
#[embassy_executor::task]
pub async fn task_wifi(mut controller: WifiController<'static>, stack: Stack<'static>) {
    let wifi_config = WifiConf::Station(
        StationConfig::default()
            .with_ssid(config_local::WIFI_SSID)
            .with_password(config_local::WIFI_PASSWORD.into()),
    );

    if let Err(e) = controller.set_config(&wifi_config) {
        log::error!("Falha ao configurar Wi-Fi: {:?}", e);
        return;
    }

    loop {
        CONN_STATE.store(0, Ordering::Relaxed);
        WIFI_OK.store(false, Ordering::Relaxed);

        log::info!(
            "BSW Com: Conectando à rede Wi-Fi '{}'...",
            config_local::WIFI_SSID
        );

        match controller.connect_async().await {
            Ok(_) => log::info!("Wi-Fi conectado!"),
            Err(e) => {
                log::error!("Falha ao conectar no Wi-Fi: {:?}, tentando em 5s", e);
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
        }

        log::info!("Aguardando IP (DHCP)...");
        stack.wait_config_up().await;
        if let Some(config) = stack.config_v4() {
            log::info!("IP recebido: {}", config.address);
        }
        WIFI_OK.store(true, Ordering::Relaxed);

        log::info!(
            "BSW Com: Conectando ao Broker MQTT {}:{}",
            config_local::MQTT_BROKER_IP,
            config_local::MQTT_BROKER_PORT
        );

        // Movemos os buffers massivos para BSS estático usando lazy init pra não matar a Stack da Task!
        static mut RX_BUFFER: [u8; 4096] = [0; 4096];
        static mut TX_BUFFER: [u8; 4096] = [0; 4096];
        static mut CLIENT_RX: [u8; 4096] = [0; 4096];
        static mut CLIENT_TX: [u8; 4096] = [0; 4096];
        static mut BATCH_BUF: [u8; 3602] = [0; 3602]; // 2 bytes header + 200 * 18 bytes
        static mut REPLAY_BUF: [u8; 2048] = [0; 2048];

        // Criamos fatias mutáveis seguras usando addr_of_mut! para contornar a regra static_mut_refs do Rust 2024
        let rx_buf = unsafe { &mut *core::ptr::addr_of_mut!(RX_BUFFER) };
        let tx_buf = unsafe { &mut *core::ptr::addr_of_mut!(TX_BUFFER) };
        let socket = TcpSocket::new(stack, rx_buf, tx_buf);
        let mut socket = socket;
        socket.set_timeout(None);

        let broker_addr = Ipv4Address::new(
            config_local::MQTT_BROKER_OCTETS[0],
            config_local::MQTT_BROKER_OCTETS[1],
            config_local::MQTT_BROKER_OCTETS[2],
            config_local::MQTT_BROKER_OCTETS[3],
        );
        let broker_endpoint = IpEndpoint::new(IpAddress::Ipv4(broker_addr), config_local::MQTT_BROKER_PORT);

        if let Err(e) = socket.connect(broker_endpoint).await {
            log::error!("Falha ao conectar TCP: {:?}", e);
            Timer::after(Duration::from_secs(5)).await;
            continue;
        }

        let client_rx_ref = unsafe { &mut *core::ptr::addr_of_mut!(CLIENT_RX) };
        let client_tx_ref = unsafe { &mut *core::ptr::addr_of_mut!(CLIENT_TX) };

        let mut mqtt_config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            rust_mqtt::utils::rng_generator::CountingRng(20000),
        );
        mqtt_config.add_client_id("esp32s3_edge");
        mqtt_config.add_username(config_local::MQTT_USER);
        mqtt_config.add_password(config_local::MQTT_PASSWORD);
        mqtt_config.max_packet_size = 4096;

        let mut client = MqttClient::<_, 5, _>::new(
            SocketWrapper(socket),
            client_tx_ref,
            4096,
            client_rx_ref,
            4096,
            mqtt_config,
        );

        match client.connect_to_broker().await {
            Ok(_) => {
                log::info!("Conectado ao Broker MQTT com sucesso (Modo Batch Binário 5Hz)!");
                CONN_STATE.store(1, Ordering::Relaxed);

                // Assinar o tópico de comandos
                if let Err(e) = client.subscribe_to_topic(config_local::MQTT_TOPIC_COMMAND).await {
                    log::warn!("BSW Com: Erro ao assinar tópico de comando: {:?}", e);
                } else {
                    log::info!("BSW Com: Assinado tópico de comandos: {}", config_local::MQTT_TOPIC_COMMAND);
                }

                // Esvazia o canal de msgs velhas
                while let Ok(_) = MQTT_TX_CHANNEL.try_receive() {}

                let mut last_status_publish = embassy_time::Instant::now();
                let mut total_mqtt_frames: u32 = 0;

                loop {
                    // 1. Verificar se há comandos MQTT recebidos (não bloqueante e imune a cancelamento)
                    match client.receive_message_if_ready().await {
                        Ok(Some((_topic, payload))) => {
                            if let Ok(text) = core::str::from_utf8(payload) {
                                let mut s: heapless::String<128> = heapless::String::new();
                                let _ = s.push_str(text);
                                log::info!("BSW Com: Comando MQTT recebido: '{}'", s.as_str());

                                if s.starts_with("LIST_SESSIONS") || s.starts_with("SESSIONS") {
                                    let files = crate::mcal::spi_sd::list_session_files();
                                    let curr_fn_guard = crate::mcal::spi_sd::CURRENT_FILENAME.try_lock();
                                    let curr_fn = match &curr_fn_guard {
                                        Ok(g) if !g.is_empty() => g.as_str(),
                                        _ => "S_0001.CSV",
                                    };
                                    let mut json: heapless::String<512> = heapless::String::new();
                                    let _ = core::fmt::write(&mut json, format_args!("{{\"event\":\"SESSION_LIST\",\"total\":{},\"active\":\"{}\",\"files\":[", files.len(), curr_fn));
                                    for (i, f) in files.iter().enumerate() {
                                        if i > 0 { let _ = core::fmt::write(&mut json, format_args!(",")); }
                                        let _ = core::fmt::write(&mut json, format_args!("\"{}\"", f.as_str()));
                                    }
                                    let _ = core::fmt::write(&mut json, format_args!("]}}"));
                                    let _ = client.send_message(
                                        config_local::MQTT_TOPIC_STATUS,
                                        json.as_bytes(),
                                        QualityOfService::QoS0,
                                        false,
                                    ).await;
                                    log::info!("BSW Com: Lista de sessões enviada: {}", json.as_str());
                                } else if s.contains("REPLAY") {
                                    if crate::types::SESSION_ACTIVE.load(Ordering::Relaxed) {
                                        log::info!("BSW Com: Finalizando sessão ativa antes de iniciar o Replay...");
                                        crate::types::SESSION_TIMER_ACTIVE.store(false, Ordering::Relaxed);
                                        crate::types::SESSION_ACTIVE.store(false, Ordering::Relaxed);
                                        crate::bsw::bsw_mem::flush_sync();
                                        let session_id = crate::bsw::bsw_mem::SESSION_ID.load(Ordering::Relaxed);
                                        let mut status_json: heapless::String<128> = heapless::String::new();
                                        let _ = core::fmt::write(
                                            &mut status_json,
                                            format_args!("{{\"event\":\"SESSION_STOPPED\",\"session_id\":\"S{:04}\"}}", session_id)
                                        );
                                        let _ = client.send_message(
                                            config_local::MQTT_TOPIC_STATUS,
                                            status_json.as_bytes(),
                                            QualityOfService::QoS0,
                                            false,
                                        ).await;
                                    }

                                    let mut target_idx: u16 = 1;
                                    let parts: heapless::Vec<&str, 4> = s.split(',').collect();
                                    for p in parts.iter() {
                                        if let Ok(num) = p.trim().parse::<u16>() {
                                            target_idx = num;
                                        }
                                    }
                                    let mut target_fn: heapless::String<16> = heapless::String::new();
                                    let _ = core::fmt::write(&mut target_fn, format_args!("S_{:04}.CSV", target_idx));
                                    log::info!("BSW Com: Replay streaming iniciado para '{}'...", target_fn.as_str());

                                    let mut total_replayed: u32 = 0;
                                    let mut replay_ok = false;

                                    if let Some(mut reader) = crate::mcal::spi_sd::SessionStreamReader::open(target_fn.as_str()) {
                                        let chunk_buf = unsafe { &mut *core::ptr::addr_of_mut!(REPLAY_BUF) };
                                        log::info!("BSW Com: Iniciando loop de transmissão do Replay...");

                                        let mut buf_len: usize = 0;

                                        loop {
                                            let space = 1536 - buf_len;
                                            let to_read = core::cmp::min(space, 512);
                                            let n = match reader.read_chunk(&mut chunk_buf[buf_len..buf_len + to_read]) {
                                                Ok(bytes) => bytes,
                                                Err(_) => {
                                                    log::error!("BSW Com: Erro na leitura do Replay no SD");
                                                    break;
                                                }
                                            };

                                            buf_len += n;

                                            if buf_len == 0 {
                                                log::info!("BSW Com: Fim do arquivo de Replay atingido (EOF).");
                                                replay_ok = true;
                                                break;
                                            }

                                            // Localizar a última quebra de linha completa
                                            let send_len = if n == 0 {
                                                buf_len
                                            } else if let Some(last_nl) = chunk_buf[..buf_len].iter().rposition(|&b| b == b'\n') {
                                                last_nl + 1
                                            } else {
                                                continue;
                                            };

                                            if send_len > 0 {
                                                if let Err(e) = client.send_message(
                                                    config_local::MQTT_TOPIC_REPLAY,
                                                    &chunk_buf[..send_len],
                                                    QualityOfService::QoS0,
                                                    false,
                                                ).await {
                                                    log::error!("BSW Com: Erro ao enviar chunk MQTT: {:?}", e);
                                                    break;
                                                }

                                                total_replayed += send_len as u32;
                                                if total_replayed % 20000 < send_len as u32 {
                                                    log::info!("BSW Com: Replay progresso: {} bytes transmitidos...", total_replayed);
                                                }

                                                chunk_buf.copy_within(send_len..buf_len, 0);
                                                buf_len -= send_len;

                                                Timer::after(Duration::from_millis(5)).await;
                                            }

                                            if n == 0 && buf_len == 0 {
                                                log::info!("BSW Com: Fim do arquivo de Replay atingido (EOF).");
                                                replay_ok = true;
                                                break;
                                            }
                                        }

                                        reader.close();
                                    } else {
                                        log::error!("BSW Com: Falha ao abrir arquivo '{}' para Replay", target_fn.as_str());
                                    }

                                    if replay_ok {
                                        let mut replay_status: heapless::String<128> = heapless::String::new();
                                        let _ = core::fmt::write(
                                            &mut replay_status,
                                            format_args!("{{\"event\":\"REPLAY_COMPLETE\",\"session\":\"{}\",\"bytes_sent\":{}}}", target_fn.as_str(), total_replayed)
                                        );
                                        let _ = client.send_message(
                                            config_local::MQTT_TOPIC_STATUS,
                                            replay_status.as_bytes(),
                                            QualityOfService::QoS0,
                                            false,
                                        ).await;
                                        log::info!("BSW Com: Replay concluído com sucesso ({} bytes transmitidos)!", total_replayed);
                                    }
                                } else if s.starts_with("STOP") {
                                    crate::types::SESSION_TIMER_ACTIVE.store(false, Ordering::Relaxed);
                                    crate::types::SESSION_ACTIVE.store(false, Ordering::Relaxed);
                                    crate::bsw::bsw_mem::flush_sync();
                                    let session_id = crate::bsw::bsw_mem::SESSION_ID.load(Ordering::Relaxed);
                                    let mut status_json: heapless::String<128> = heapless::String::new();
                                    let _ = core::fmt::write(
                                        &mut status_json,
                                        format_args!("{{\"event\":\"SESSION_STOPPED\",\"session_id\":\"S{:04}\"}}", session_id)
                                    );
                                    let _ = client.send_message(
                                        config_local::MQTT_TOPIC_STATUS,
                                        status_json.as_bytes(),
                                        QualityOfService::QoS0,
                                        false,
                                    ).await;
                                    log::info!("BSW Com: Sessão parada via comando STOP");
                                } else if s.starts_with("TIME,") {
                                    if let Some(epoch_str) = s.strip_prefix("TIME,") {
                                        if let Ok(epoch_val) = epoch_str.trim().parse::<u64>() {
                                            crate::types::sync_epoch_time(epoch_val);
                                        }
                                    }
                                } else {
                                    if s.contains("RESET") {
                                        crate::mcal::spi_sd::wipe_all_sessions();
                                    }

                                    let mut duration_min: u32 = 0;
                                    let mut epoch_opt: Option<u64> = None;
                                    let parts: heapless::Vec<&str, 6> = s.split(',').collect();
                                    for p in parts.iter() {
                                        let p_trim = p.trim();
                                        if let Ok(val) = p_trim.parse::<u64>() {
                                            if val > 1_000_000_000_000 {
                                                epoch_opt = Some(val);
                                            } else if val > 0 && val <= 1440 {
                                                duration_min = val as u32;
                                            }
                                        }
                                    }

                                    let label_opt = if s.contains("ECO") {
                                        Some((crate::types::SessionLabel::Economico, 0x01))
                                    } else if s.contains("SPT") {
                                        Some((crate::types::SessionLabel::Esportivo, 0x03))
                                    } else if s.contains("NOR") {
                                        Some((crate::types::SessionLabel::Normal, 0x02))
                                    } else {
                                        None
                                    };

                                    if let Some((label, can_code)) = label_opt {
                                        if let Some(new_session_idx) = crate::bsw::bsw_mem::flush_and_rotate_session(label, duration_min, epoch_opt) {
                                            let _ = crate::CAN_CMD_CHANNEL.try_send(crate::types::CanFrame::new(0x010, &[can_code]));
                                            
                                            let epoch_now = crate::types::get_current_timestamp_ms();
                                            let mut start_json: heapless::String<200> = heapless::String::new();
                                            let _ = core::fmt::write(
                                                &mut start_json,
                                                format_args!(
                                                    "{{\"event\":\"SESSION_START\",\"session_id\":\"S{:04}\",\"mode\":\"{}\",\"duration_min\":{},\"start_epoch_ms\":{}}}",
                                                    new_session_idx, label.as_str(), duration_min, epoch_now
                                                )
                                            );
                                            let _ = client.send_message(
                                                config_local::MQTT_TOPIC_STATUS,
                                                start_json.as_bytes(),
                                                QualityOfService::QoS0,
                                                false,
                                            ).await;
                                            log::info!("BSW Com: Evento SESSION_START publicado: {}", start_json.as_str());
                                        }
                                    }
                                }
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            log::error!("BSW Com: Erro na conexão MQTT: {:?}. Reconectando...", e);
                            break;
                        }
                    }

                    // 2. Verificar se há frames de telemetria para envio
                    if let Ok(first_frame) = MQTT_TX_CHANNEL.try_receive() {
                        let batch_slice = unsafe { &mut *core::ptr::addr_of_mut!(BATCH_BUF) };
                        let mut count: u16 = 1;

                        // Copiar o primeiro frame para o lote
                        let frame_bytes: &[u8] = unsafe {
                            core::slice::from_raw_parts(
                                &first_frame as *const BinaryFrame as *const u8,
                                core::mem::size_of::<BinaryFrame>(),
                            )
                        };
                        batch_slice[2..2 + 18].copy_from_slice(frame_bytes);

                        // Drenar imediatamente tudo o que já estiver acumulado na fila (até 150 frames por lote)
                        while count < 150 {
                            if let Ok(next_frame) = MQTT_TX_CHANNEL.try_receive() {
                                let frame_bytes: &[u8] = unsafe {
                                    core::slice::from_raw_parts(
                                        &next_frame as *const BinaryFrame as *const u8,
                                        core::mem::size_of::<BinaryFrame>(),
                                    )
                                };
                                let offset = 2 + (count as usize) * 18;
                                batch_slice[offset..offset + 18].copy_from_slice(frame_bytes);
                                count += 1;
                            } else {
                                break;
                            }
                        }

                        let total_len = (count as usize) * 18;
                        let session_id = crate::bsw::bsw_mem::SESSION_ID.load(Ordering::Relaxed);
                        let mut dynamic_topic: heapless::String<64> = heapless::String::new();
                        let _ = core::fmt::write(&mut dynamic_topic, format_args!("/telemetry/S{:04}/raw", session_id));

                        // Publicar Lote Binário Dinâmico
                        if let Err(e) = client.send_message(
                            dynamic_topic.as_str(),
                            &batch_slice[2..2 + total_len],
                            QualityOfService::QoS0,
                            false,
                        ).await {
                            log::error!("BSW Com: Erro ao publicar Lote MQTT: {:?}. Reconectando...", e);
                            break;
                        }

                        total_mqtt_frames += count as u32;
                        if total_mqtt_frames % 200 < count as u32 {
                            log::info!(
                                "BSW Com: MQTT TX lote de {} frames em '{}' (total: {})",
                                count,
                                dynamic_topic.as_str(),
                                total_mqtt_frames
                            );
                        }
                    } else {
                        // Sem dados de telemetria no momento (ex: sessão pausada ou intervalo entre pacotes):
                        // Cede tempo para o RTOS e verifica comandos novamente a cada 10ms
                        Timer::after(Duration::from_millis(10)).await;
                    }

                    // 3. Publicar status de saúde / keepalive a cada 5s (mantém conexão MQTT sempre ativa)
                    if last_status_publish.elapsed() >= Duration::from_secs(5) {
                        let is_active = crate::types::SESSION_ACTIVE.load(Ordering::Relaxed);
                        let state_str = if is_active { "RECORDING" } else { "STANDBY" };
                        let sd_status = crate::bsw::bsw_mem::SD_OK.load(Ordering::Relaxed);
                        let wifi_status = WIFI_OK.load(Ordering::Relaxed);
                        let frames = crate::app::logger::LOGGER_STATS.frames_received.load(Ordering::Relaxed);
                        let lost = crate::app::logger::LOGGER_STATS.frames_lost.load(Ordering::Relaxed);

                        let session_id = crate::bsw::bsw_mem::SESSION_ID.load(Ordering::Relaxed);
                        let uptime_ms = embassy_time::Instant::now().as_millis();
                        let mut status_json: heapless::String<220> = heapless::String::new();
                        let _ = core::fmt::write(
                            &mut status_json,
                            format_args!(
                                "{{\"state\":\"{}\",\"session_id\":\"S{:04}\",\"recording\":{},\"sd_ok\":{},\"wifi_ok\":{},\"frames_received\":{},\"lost\":{},\"uptime_ms\":{}}}",
                                state_str, session_id, is_active, sd_status, wifi_status, frames, lost, uptime_ms
                            ),
                        );

                        if let Err(e) = client.send_message(
                            config_local::MQTT_TOPIC_STATUS,
                            status_json.as_bytes(),
                            QualityOfService::QoS0,
                            false,
                        ).await {
                            log::error!("BSW Com: Falha ao publicar status MQTT ({:?}). Forçando reconexão...", e);
                            break;
                        } else {
                            log::info!("BSW Status (MQTT): publicado em '{}' -> {}", config_local::MQTT_TOPIC_STATUS, status_json);
                        }
                        last_status_publish = embassy_time::Instant::now();
                    }
                }
            }
            Err(e) => {
                log::error!("Falha no connect_to_broker MQTT: {:?}", e);
            }
        }

        log::warn!("Conexão perdida. Desconectando Wi-Fi para recomeçar...");
        CONN_STATE.store(0, Ordering::Relaxed);
        let _ = controller.disconnect_async().await;
        Timer::after(Duration::from_secs(5)).await;
    }
}

/// Task Embassy de publicação de status do sistema a cada 30 s.
#[embassy_executor::task]
pub async fn task_status_publisher() {
    log::info!(
        "BSW Com: task_status_publisher iniciada (Publicação em '{}')",
        config_local::MQTT_TOPIC_STATUS
    );

    loop {
        Timer::after(Duration::from_secs(30)).await;

        if CONN_STATE.load(Ordering::Relaxed) == 1 {
            let wifi_status = WIFI_OK.load(Ordering::Relaxed);
            let sd_status = crate::bsw::bsw_mem::SD_OK.load(Ordering::Relaxed);
            let frames = crate::app::logger::LOGGER_STATS.frames_received.load(Ordering::Relaxed);
            let lost = crate::app::logger::LOGGER_STATS.frames_lost.load(Ordering::Relaxed);

            log::info!(
                "BSW Status (30s): sd_ok={}, wifi_ok={}, frames_received={}, lost={}",
                sd_status, wifi_status, frames, lost
            );
        }
    }
}
