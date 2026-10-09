//! Logger de Telemetria (App) — Orquestrador de Persistência e Fallback.
//!
//! Consome `TelemetryFrame` do channel RTE, serializa em CSV, gerencia a FSM de
//! conectividade e direciona os dados para SD/MQTT sem perdas (logger_spec.md).

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;
use embassy_sync::mutex::Mutex;
use heapless::{String, Vec};
use portable_atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};

use crate::app::csv_writer;
use crate::bsw::bsw_com::{self, CONN_STATE};
use crate::bsw::bsw_mem;
use crate::config;
use crate::types::{BinaryFrame, SessionLabel, TelemetryFrame};

/// Rastreia o último estado de conexão notificado via linha DIAG (evita duplicatas).
/// 255 = estado inicial desconhecido; 0 = Disconnected; 1 = Connected.
static LAST_CONN_NOTIFIED: AtomicU8 = AtomicU8::new(255);

/// Contadores atômicos de diagnóstico e métricas do sistema.
#[allow(dead_code)]
pub struct LoggerStats {
    pub frames_received: AtomicU32,
    pub frames_to_sd: AtomicU32,
    pub frames_to_mqtt: AtomicU32,
    pub frames_lost: AtomicU32,
    pub overflow_count: AtomicU32,
    pub last_frame_ts_ms: AtomicU64,
}

impl LoggerStats {
    pub const fn new() -> Self {
        Self {
            frames_received: AtomicU32::new(0),
            frames_to_sd: AtomicU32::new(0),
            frames_to_mqtt: AtomicU32::new(0),
            frames_lost: AtomicU32::new(0),
            overflow_count: AtomicU32::new(0),
            last_frame_ts_ms: AtomicU64::new(0),
        }
    }
}

/// Instância estática global dos contadores observáveis.
pub static LOGGER_STATS: LoggerStats = LoggerStats::new();

/// Buffer de backlog de fallback na memória (16 KB = 50 frames × 320 B) (D-04).
/// Mantido em dimensão compatível com a SRAM interna para preservar a pilha de execução.
static BACKLOG_PSRAM: Mutex<CriticalSectionRawMutex, Vec<String<320>, 50>> =
    Mutex::new(Vec::new());

pub fn get_backlog_psram_bytes() -> usize {
    BACKLOG_PSRAM.try_lock().map(|g| g.len() * 320).unwrap_or(0)
}

/// Task Embassy principal de logging e gerenciamento de fallback.
#[embassy_executor::task]
pub async fn task_logger(
    receiver: Receiver<'static, CriticalSectionRawMutex, TelemetryFrame, { config::CHANNEL_CAPACITY }>,
) {
    log::info!("APP Logger: task_logger iniciada (Processamento CSV, SD e MQTT)");

    // AC-L-01: Gravar o header CSV na abertura da sessão
    let header = csv_writer::csv_header();
    if bsw_mem::push_line(header).is_ok() {
        log::info!("APP Logger: Header CSV gravado na sessão");
    } else {
        log::warn!("APP Logger: Não foi possível gravar o header CSV (SD indisponível)");
    }
    
    // Injetar linha BOOT com metadados do firmware e hardware (Vector ASC style)
    let boot_line = csv_writer::serialize_boot(0, &SessionLabel::Normal);
    let _ = bsw_mem::push_line(&boot_line);

    loop {
        let frame = receiver.receive().await;
        LOGGER_STATS.frames_received.fetch_add(1, Ordering::Relaxed);
        LOGGER_STATS.last_frame_ts_ms.store(frame.timestamp_ms, Ordering::Relaxed);

        // 0. Monitorar Expiração de Sessão Temporizada
        if crate::types::SESSION_TIMER_ACTIVE.load(Ordering::Relaxed) {
            let start_uptime = crate::types::SESSION_START_MS.load(Ordering::Relaxed);
            let duration_ms = crate::types::SESSION_DURATION_MS.load(Ordering::Relaxed);
            let current_uptime = embassy_time::Instant::now().as_millis();
            if duration_ms > 0 && current_uptime.saturating_sub(start_uptime) >= duration_ms {
                crate::types::SESSION_TIMER_ACTIVE.store(false, Ordering::Relaxed);
                crate::types::SESSION_ACTIVE.store(false, Ordering::Relaxed);
                let diag = csv_writer::serialize_diag(
                    frame.timestamp_ms,
                    &frame.session_label,
                    "SESSION_COMPLETE_DURATION_REACHED",
                );
                let _ = bsw_mem::push_line(&diag);
                bsw_mem::flush_sync();
                log::warn!(
                    "APP Logger: Temporizador de sessão expirou ({} ms = {} min). Gravação finalizada e pausada.",
                    duration_ms, duration_ms / 60_000
                );
            }
        }

        // Se a sessão expirou ou foi parada via STOP, não registrar nem transmitir
        if !crate::types::SESSION_ACTIVE.load(Ordering::Relaxed) {
            continue;
        }

        // 1. Serializar frame em linha CSV
        let csv_line = csv_writer::serialize(&frame);

        // 2. FSM de Conectividade (0=Disconnected, 1=Connected, 2=Reconnecting)
        let state = CONN_STATE.load(Ordering::Relaxed);
        let last_notified = LAST_CONN_NOTIFIED.load(Ordering::Relaxed);
        let mut sd_success = false;
        let mut mqtt_success = false;

        // AC-L-03: Emitir linha DIAG quando Wi-Fi desconectar
        if state == 0 && last_notified != 0 {
            LAST_CONN_NOTIFIED.store(0, Ordering::Relaxed);
            let diag = csv_writer::serialize_diag(
                frame.timestamp_ms,
                &SessionLabel::Normal,
                "WIFI_DISCONNECTED",
            );
            let _ = bsw_mem::push_line(&diag);
            log::warn!("APP Logger: WIFI_DISCONNECTED — Modo Fallback SD ativado");
        }

        // Emitir DIAG quando Wi-Fi reconectar
        if state == 1 && last_notified != 1 {
            LAST_CONN_NOTIFIED.store(1, Ordering::Relaxed);
            let backlog_count = BACKLOG_PSRAM.try_lock()
                .map(|g| g.len())
                .unwrap_or(0);
            let mut diag_msg: String<64> = String::new();
            let _ = core::fmt::write(
                &mut diag_msg,
                format_args!("WIFI_RECONNECTED backlog={}", backlog_count),
            );
            let diag = csv_writer::serialize_diag(
                frame.timestamp_ms,
                &SessionLabel::Normal,
                &diag_msg,
            );
            let _ = bsw_mem::push_line(&diag);
            log::info!("APP Logger: WIFI_RECONNECTED — Backlog={} frames para descarregar", backlog_count);
        }

        let _mqtt_success = false;
        let _sd_success = false;
        let bin_frame = BinaryFrame::from(&frame);

        match state {
            1 => {
                // Connected: enviar via MQTT (Batch Binário) e gravar no SD como backup permanente
                if bsw_com::mqtt_publish_binary(bin_frame).is_ok() {
                    mqtt_success = true;
                    LOGGER_STATS.frames_to_mqtt.fetch_add(1, Ordering::Relaxed);
                }

                if bsw_mem::push_line(&csv_line).is_ok() {
                    sd_success = true;
                    LOGGER_STATS.frames_to_sd.fetch_add(1, Ordering::Relaxed);
                }
            }
            0 => {
                // Disconnected: gravar apenas no SD Card
                if bsw_mem::push_line(&csv_line).is_ok() {
                    sd_success = true;
                    LOGGER_STATS.frames_to_sd.fetch_add(1, Ordering::Relaxed);
                } else {
                    // SD falhou no modo desconectado: acumular no backlog PSRAM
                    if let Ok(mut guard) = BACKLOG_PSRAM.try_lock() {
                        let _ = guard.push(csv_line.clone());
                    }
                    // AC-L-05: Emitir DIAG de falha de escrita no SD
                    let diag = csv_writer::serialize_diag(
                        frame.timestamp_ms,
                        &SessionLabel::Normal,
                        "SD_WRITE_ERROR",
                    );
                    log::debug!("APP Logger: SD_WRITE_ERROR — frame acumulado no backlog PSRAM");
                    // Tentar gravar o DIAG se o SD voltar no próximo ciclo
                    let _ = bsw_mem::push_line(&diag);
                }
            }
            2 => {
                // Reconnecting: descarregar backlog do SD em ordem FIFO (First-In, First-Out)
                if let Ok(mut guard) = BACKLOG_PSRAM.try_lock() {
                    while !guard.is_empty() {
                        let backlog_line = guard.remove(0);
                        if bsw_mem::push_line(&backlog_line).is_err() {
                            let _ = guard.insert(0, backlog_line);
                            break;
                        }
                    }
                }
                CONN_STATE.store(1, Ordering::Relaxed); // Transiciona para Connected
            }
            _ => {}
        }

        // 4. Verificação de Perda Total de Dados (SD Falhou E MQTT Falhou)
        if !sd_success && !mqtt_success {
            LOGGER_STATS.frames_lost.fetch_add(1, Ordering::Relaxed);
            log::debug!(
                "LOGGER ALERTA: Frame id={:#05x} perdido (SD e MQTT falharam)",
                frame.can_id
            );
        }
    }
}
