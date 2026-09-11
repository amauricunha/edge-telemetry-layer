//! Entry point do firmware Edge Telemetry Layer — ESP32-S3.
//!
//! Inicializa os periféricos TWAI e SPI2 e spawna as tasks Embassy sob o runtime do `esp-rtos`:
//! - `task_can_rx`: recepção de TODOS os frames CAN (DBC passivos + respostas OBD-II)
//! - `task_obd_poller`: envio ativo de solicitações OBD-II (apenas TX)
//! - `task_logger`: persistência CSV no SD Card e publicação MQTT com fallback FSM
//! - `task_sd_writer`: flush assíncrono do buffer circular de 4 KB para o SD Card físico
//! - `task_wifi`: gerenciamento de conexão Wi-Fi e MQTT com reconexão automática
//! - `task_status_publisher`: publicação periódica de diagnóstico em `/system/status`
//! - `task_watchdog`: watchdog de software — Bus-Off monitor (AC-07) + Logger stall detector
//!
//! ## Arquitetura AUTOSAR (Semanas 3–4)
//! ```text
//! APP  → task_can_rx, task_obd_poller, task_logger, csv_writer
//! RTE  → TELEMETRY_CHANNEL (Channel<TelemetryFrame, 32>)
//! BSW  → bsw_mem (SD_BUFFER 4 KB), bsw_com (CONN_STATE, task_wifi, task_status_publisher)
//! MCAL → mcal::twai (TWAI async), mcal::spi_sd (SPI2 + embedded-sdmmc FAT32)
//! ```
//!
//! ## Ownership do TWAI
//! O periférico TWAI é dividido via `split()` após conversão para `.into_async()`:
//! - `TwaiRx` → `task_can_rx` (recebe DBC e OBD respostas)
//! - `TwaiTx` → `task_obd_poller` (envia solicitações OBD)

#![no_std]
#![no_main]

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

mod app;
mod bsw;
mod config;
mod config_local;
mod mcal;
mod types;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};

use embassy_net::{Config as NetConfig, StackResources};
use esp_radio::wifi::{ControllerConfig, Interface, WifiController};
use esp_hal::rng::Rng;

use esp_hal::twai::{self, BaudRate, TwaiConfiguration, TwaiMode};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::interrupt::software::SoftwareInterruptControl;

// Necessário para o panic handler e logging via UART
use esp_backtrace as _;
use esp_println as _;

use crate::mcal::twai as mcal_twai;
use crate::types::TelemetryFrame;

// =============================================================================
// RTE — Channel estático de telemetria (D-02 do plan.md)
// =============================================================================

/// Channel bounded compartilhado entre tasks.
/// Capacidade 32: ao ritmo de 20 frames/s do UNO R3, suporta 1,6 s de atraso.
static TELEMETRY_CHANNEL: Channel<CriticalSectionRawMutex, TelemetryFrame, { config::CHANNEL_CAPACITY }> =
    Channel::new();

// =============================================================================
// RTE — Channel de Comandos CAN (Ex: Alterar Modo no Uno)
// =============================================================================
pub static CAN_CMD_CHANNEL: Channel<CriticalSectionRawMutex, crate::types::CanFrame, 5> = Channel::new();

// =============================================================================
// Timestamp de último envio OBD (para cálculo de latência)
// =============================================================================

/// Timestamp do último envio OBD-II (em ticks do sistema).
/// Compartilhado entre task_obd_poller (write) e task_can_rx (read).
pub(crate) static LAST_OBD_SEND_TICKS: portable_atomic::AtomicU64 =
    portable_atomic::AtomicU64::new(0);

/// PID da última solicitação OBD-II enviada.
pub(crate) static LAST_OBD_PID: portable_atomic::AtomicU8 =
    portable_atomic::AtomicU8::new(0);

/// Flag indicando se a resposta para o último envio OBD-II foi recebida.
pub(crate) static LAST_OBD_RESPONSE_RECEIVED: portable_atomic::AtomicBool =
    portable_atomic::AtomicBool::new(false);

// =============================================================================
// RTE — Signals de Sincronização de Bus-Off (AC-07)
// =============================================================================

/// Sinal para comandar a pausa cooperativa das tasks CAN.
pub static BUS_OFF_SIGNAL: embassy_sync::signal::Signal<CriticalSectionRawMutex, ()> =
    embassy_sync::signal::Signal::new();

/// Sinal para comandar a retomada cooperativa das tasks CAN após o recovery.
pub static BUS_OFF_CLEAR: embassy_sync::signal::Signal<CriticalSectionRawMutex, ()> =
    embassy_sync::signal::Signal::new();

// =============================================================================
// Task: Network Stack Runner (embassy-net)
// =============================================================================

#[embassy_executor::task]
async fn task_net_stack(mut runner: embassy_net::Runner<'static, Interface>) {
    runner.run().await
}

// =============================================================================
// Task: Recepção de TODOS os frames CAN (DBC + OBD-II)
// =============================================================================

/// Task Embassy que lê frames CAN do barramento e publica no channel.
#[embassy_executor::task]
async fn task_can_rx(
    mut rx: esp_hal::twai::TwaiRx<'static, esp_hal::Async>,
) {
    let sender = TELEMETRY_CHANNEL.sender();
    let mut overflow_count: u32 = 0;
    let mut dbc_frame_count: u32 = 0;
    let mut obd_frame_count: u32 = 0;

    log::info!("CAN RX: task iniciada — aguardando frames no barramento");

    loop {
        match embassy_futures::select::select(mcal_twai::twai_recv(&mut rx), BUS_OFF_SIGNAL.wait()).await {
            embassy_futures::select::Either::First(Ok(can_frame)) => {
                let now_ticks = Instant::now().as_ticks();
                let timestamp_ms = crate::types::get_current_timestamp_ms();

                // ----- Resposta OBD-II (0x7E8) -----
                if can_frame.id == config::OBD_RESPONSE_ID {
                    let send_ticks = LAST_OBD_SEND_TICKS.load(portable_atomic::Ordering::Relaxed);
                    let latency_ms = if send_ticks > 0 {
                        let delta_ticks = now_ticks.saturating_sub(send_ticks);
                        // Embassy usa 1 MHz tick rate por default, então converter ticks para ms
                        delta_ticks as f32 / 1000.0
                    } else {
                        0.0
                    };

                    if let Some(tf) = app::can_decoder::parse_obd_response(&can_frame, timestamp_ms, latency_ms) {
                        obd_frame_count += 1;
                        LAST_OBD_RESPONSE_RECEIVED.store(true, portable_atomic::Ordering::Relaxed);

                        log::debug!(
                            "OBD RX: PID={:#04x} latency={:.1}ms",
                            can_frame.data[2],
                            latency_ms,
                        );

                        if sender.try_send(tf).is_err() {
                            overflow_count += 1;
                            log::warn!(
                                "Channel overflow #{} — OBD frame descartado",
                                overflow_count
                            );
                        }
                    }
                    continue;
                }

                // ----- Frames DBC passivos (0x100, 0x200, 0x300) -----
                if let Some(tf) = app::can_decoder::parse_dbc_frame(&can_frame, timestamp_ms) {
                    dbc_frame_count += 1;

                    if dbc_frame_count.is_multiple_of(100) {
                        log::info!(
                            "CAN RX: DBC={} OBD={} overflow={}",
                            dbc_frame_count,
                            obd_frame_count,
                            overflow_count,
                        );
                    }

                    log::debug!(
                        "CAN DBC: id={:#05x} speed={:?} rpm={:?} throttle={:?} load={:?} coolant={:?}",
                        can_frame.id,
                        tf.speed_kmh,
                        tf.rpm,
                        tf.throttle_pct,
                        tf.engine_load_pct,
                        tf.coolant_temp_c,
                    );

                    if sender.try_send(tf).is_err() {
                        overflow_count += 1;
                        log::warn!(
                            "Channel overflow #{} — DBC frame descartado (id={:#05x})",
                            overflow_count,
                            can_frame.id,
                        );
                    }
                }
            }
            embassy_futures::select::Either::First(Err(e)) => {
                log::error!("CAN RX: erro de recepção — {:?}", e);
                // Sinalizar possível Bus-Off para task_watchdog processar (AC-07)
                crate::bsw::bsw_diag::BUS_OFF_DETECTED.store(true, portable_atomic::Ordering::Relaxed);
                Timer::after(Duration::from_millis(10)).await;
            }
            embassy_futures::select::Either::Second(_) => {
                log::warn!("CAN RX pausado cooperativamente — aguardando Bus-Off recovery");
                BUS_OFF_CLEAR.wait().await;
                log::info!("CAN RX retomado após Bus-Off recovery");
            }
        }
    }
}

// =============================================================================
// Entry point
// =============================================================================

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // 58KB é o valor de equilíbrio perfeito: dá 16KB de folga ao esp-radio sem estourar o linker de release
    esp_alloc::heap_allocator!(size: 58 * 1024);
    
    // Inicializar periféricos do ESP32-S3
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Inicializar logging via UART/JTAG com nível INFO garantido
    esp_println::logger::init_logger(log::LevelFilter::Info);

    log::info!("====================================================");
    log::info!("  Edge Telemetry Layer v0.2.0 — ESP32-S3");
    log::info!("  Semana 3: CAN + OBD + CSV + SD Card + MQTT");
    log::info!("====================================================");

    // 1. Inicializar o escalonador do RTOS
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    // -------------------------------------------------------------------------
    // Configurar periférico TWAI (CAN) — Camada MCAL
    // -------------------------------------------------------------------------
    let twai_rx_pin = peripherals.GPIO5;
    let twai_tx_pin = peripherals.GPIO4;

    log::info!("TWAI: configurando a 500 Kbps (TX=GPIO4, RX=GPIO5)");

    let mut twai_config = TwaiConfiguration::new(
        peripherals.TWAI0,
        twai_rx_pin,
        twai_tx_pin,
        BaudRate::B500K,
        TwaiMode::Normal,
    );

    // Aceitar todos os IDs — filtragem feita em software
    twai_config.set_filter(
        const { twai::filter::SingleStandardFilter::new(
            b"xxxxxxxxxxx",
            b"x",
            [b"xxxxxxxx", b"xxxxxxxx"],
        ) },
    );

    // Converter para modo assíncrono e iniciar
    let twai = twai_config.into_async().start();
    
    // Dividir em TX e RX para ownership segura em tasks separadas
    let (twai_rx, twai_tx) = twai.split();

    log::info!("TWAI: periférico iniciado em modo Assíncrono");

    // -------------------------------------------------------------------------
    // Configurar SD Card via barramento SPI2 — Camada MCAL
    // Pinos: CS=GPIO10, MOSI=GPIO11, CLK=GPIO12, MISO=GPIO13
    // init_and_probe chama bsw_mem::set_sd_present(true/false) internamente
    // -------------------------------------------------------------------------
    log::info!("SPI SD: inicializando barramento SPI2 (CS=GPIO10, MOSI=GPIO11, CLK=GPIO12, MISO=GPIO13)...");
    mcal::spi_sd::init_and_probe(
        peripherals.SPI2,
        peripherals.GPIO10,
        peripherals.GPIO11,
        peripherals.GPIO12,
        peripherals.GPIO13,
    );

    // -------------------------------------------------------------------------
    // Configurar Wi-Fi e TCP/IP Stack (esp-radio + embassy-net)
    // -------------------------------------------------------------------------
    log::info!("Inicializando driver Wi-Fi (esp-radio)...");
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);

    let wifi_controller = WifiController::new(peripherals.WIFI, ControllerConfig::default())
        .expect("Falha ao inicializar WifiController");
    
    let wifi_interface = Interface::station();
    let net_config = NetConfig::dhcpv4(Default::default());

    static STACK_RESOURCES: static_cell::StaticCell<StackResources<3>> = static_cell::StaticCell::new();
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        STACK_RESOURCES.init(StackResources::<3>::new()),
        seed,
    );

    // -------------------------------------------------------------------------
    // Spawnar tasks Embassy
    // -------------------------------------------------------------------------

    spawner.spawn(task_net_stack(runner).unwrap());
    log::info!("Spawned: task_net_stack");

    // Task de recepção CAN (DBC + OBD respostas)
    spawner.spawn(task_can_rx(twai_rx).unwrap());
    log::info!("Spawned: task_can_rx");

    // Task de polling OBD-II ativo (apenas TX)
    spawner.spawn(app::obd_poller::task_obd_poller(twai_tx).unwrap());
    log::info!("Spawned: task_obd_poller");

    // Task de gravação e descarregamento assíncrono do SD Card (BSW Memória)
    spawner.spawn(bsw::bsw_mem::task_sd_writer().unwrap());
    log::info!("Spawned: task_sd_writer");

    // Task de conectividade e reconexão Wi-Fi (BSW Comunicação)
    spawner.spawn(bsw::bsw_com::task_wifi(wifi_controller, stack).unwrap());
    log::info!("Spawned: task_wifi");

    // Task de publicação periódica de status do sistema MQTT
    spawner.spawn(bsw::bsw_com::task_status_publisher().unwrap());
    log::info!("Spawned: task_status_publisher");

    // Task Logger principal de persistência e fallback (APP)
    spawner.spawn(app::logger::task_logger(TELEMETRY_CHANNEL.receiver()).unwrap());
    log::info!("Spawned: task_logger");

    // Task Watchdog de software — Bus-Off monitor + Logger stall detector (Semana 4)
    spawner.spawn(bsw::bsw_diag::task_watchdog().unwrap());
    log::info!("Spawned: task_watchdog (Bus-Off monitor + Logger stall)");

    log::info!("Sistema iniciado — persistência SD + MQTT ativa");
    log::info!("====================================================");
}
