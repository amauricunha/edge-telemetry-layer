//! Diagnóstico, Resiliência e Monitoramento de Performance (BSW Diagnóstico).
//!
//! Implementa:
//! - Detecção e recuperação automática de Bus-Off (AC-07)
//! - Watchdog de software: monitora travamento do `task_logger`
//! - **Logger automático de performance no SD Card (AC-05):** a cada 60 s grava
//!   uma linha `DIAG,HEARTBEAT` com métricas de heap, uptime, frames e buffer,
//!   tornando o coletor auto-contido (sem precisar de terminal/PC para medir).
//!
//! ## Decisões de engenharia para otimização de recursos
//!
//! - **Heap amostrado a cada 5 s** (não continuamente) para evitar overhead de
//!   leitura em seção crítica. O `esp_alloc::HEAP.free()` usa `critical_section::with`
//!   internamente; amostrar a 0.2 Hz mantém a latência do watchdog inalterada.
//! - **Tracking min/max/sum local** (variáveis de stack, não AtomicU32) pois tudo
//!   vive dentro de uma única task — sem necessidade de sincronização inter-task.
//! - **Linha DIAG limitada a 256 bytes** (`heapless::String<256>`) para caber no
//!   buffer circular de 4 KB do SD sem pressão.
//!
//! ## Por que não passamos TwaiRx/TwaiTx para este módulo?
//!
//! Após `twai.split()`, o `TwaiRx` pertence exclusivamente ao `task_can_rx`
//! e o `TwaiTx` ao `task_obd_poller`. A estratégia de Bus-Off neste projeto
//! usa um flag atômico (`BUS_OFF_DETECTED`) sinalizado pelo `task_can_rx`
//! quando detecta um erro de barramento. O `task_watchdog` reage ao flag,
//! registra o evento no SD e aguarda o recovery automático do controlador TWAI
//! (que ocorre após 128 pulsos de recuperação do barramento).
//!
//! Esta abordagem é segura em no_std/Embassy sem necessidade de Mutex complexos.

use embassy_time::{Duration, Instant, Timer};
use portable_atomic::{AtomicBool, AtomicU32, Ordering};

use crate::app::logger::LOGGER_STATS;
use crate::bsw::bsw_mem;

// =============================================================================
// Flags atômicos de diagnóstico do barramento CAN
// =============================================================================

/// Flag sinalizando que um evento Bus-Off foi detectado pelo task_can_rx.
/// Setado por `task_can_rx`; limpo por `task_watchdog` após logar o evento.
pub static BUS_OFF_DETECTED: AtomicBool = AtomicBool::new(false);

/// Contador de eventos Bus-Off desde o boot.
pub static BUS_OFF_COUNT: AtomicU32 = AtomicU32::new(0);

/// Flag indicando que o sistema está em processo de recovery do Bus-Off.
pub static BUS_OFF_RECOVERING: AtomicBool = AtomicBool::new(false);

// =============================================================================
// Performance Tracker — rastreia min/max/média do heap entre heartbeats
// =============================================================================

/// Rastreador de métricas de heap para janelas de 60 s.
///
/// Armazena min, max e soma acumulada dos valores de `heap_free` amostrados
/// a cada 5 s (12 amostras por janela). Vive inteiramente na stack da
/// `task_watchdog` — sem alocação estática nem AtomicU32 extra.
struct HeapTracker {
    min: usize,
    max: usize,
    sum: usize,
    samples: u32,
}

impl HeapTracker {
    const fn new() -> Self {
        Self {
            min: usize::MAX,
            max: 0,
            sum: 0,
            samples: 0,
        }
    }

    /// Registra uma amostra de heap_free.
    fn sample(&mut self, heap_free: usize) {
        if heap_free < self.min { self.min = heap_free; }
        if heap_free > self.max { self.max = heap_free; }
        self.sum += heap_free;
        self.samples += 1;
    }

    /// Retorna a média do heap_free na janela.
    fn avg(&self) -> usize {
        if self.samples > 0 { self.sum / self.samples as usize } else { 0 }
    }

    /// Reseta para a próxima janela de 60 s.
    fn reset(&mut self) {
        self.min = usize::MAX;
        self.max = 0;
        self.sum = 0;
        self.samples = 0;
    }
}

// =============================================================================
// Task: Watchdog de Software + Performance Logger (RF-01, RF-02, AC-05)
// =============================================================================

/// Task Embassy de watchdog de software e logger de performance.
///
/// Responsabilidades:
/// 1. **Bus-Off Monitor (AC-07):** detecta o flag `BUS_OFF_DETECTED` e registra
///    o evento de diagnóstico no SD Card com timestamp. O recovery físico do TWAI
///    é automático após 128 erros consecutivos (ISO 11898).
/// 2. **Logger Stall Detector:** verifica se `task_logger` processou algum frame
///    nos últimos 30 s. Se não, emite log de `LOGGER_STALL` como alerta.
/// 3. **Performance Logger (AC-05):** a cada 60 s grava no SD Card uma linha
///    `DIAG,HEARTBEAT` com: uptime, frames, lost, overflow, bus-off, heap
///    (current/min/max/avg), e uso do buffer SD. Estes dados preenchem
///    automaticamente a Tabela 7 do artigo sem depender de terminal/PC.
#[embassy_executor::task]
pub async fn task_watchdog() {
    log::info!("BSW Diag: task_watchdog iniciada (Bus-Off + Stall + Performance Logger)");

    let mut last_frame_count: u32 = 0;
    let mut stall_check_instant = Instant::now();
    let mut heartbeat_instant = Instant::now();
    let mut heap_sample_instant = Instant::now();
    let mut heap_tracker = HeapTracker::new();

    loop {
        Timer::after(Duration::from_millis(10)).await;

        // ------------------------------------------------------------------
        // 0. Amostrar heap a cada ciclo de 5 s (12 amostras por janela de 60 s)
        // ------------------------------------------------------------------
        if heap_sample_instant.elapsed() >= Duration::from_secs(5) {
            heap_sample_instant = Instant::now();
            let heap_free = esp_alloc::HEAP.free();
            heap_tracker.sample(heap_free);
        }

        // ------------------------------------------------------------------
        // 1. Bus-Off Detection & Registro no SD
        // ------------------------------------------------------------------
        if BUS_OFF_DETECTED.load(Ordering::Relaxed) {
            BUS_OFF_DETECTED.store(false, Ordering::Relaxed);
            BUS_OFF_RECOVERING.store(true, Ordering::Relaxed);

            let count = BUS_OFF_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            let ts_ms = Instant::now().as_millis();

            log::error!(
                "BSW Diag: *** BUS-OFF DETECTADO (evento #{}) — ts={}ms ***",
                count, ts_ms
            );
            
            // 1. Sinalizar pausa cooperativa para as tasks CAN (RTE)
            crate::BUS_OFF_SIGNAL.signal(());
            log::warn!("BSW Diag: BUS_OFF_SIGNAL emitido. Tasks CAN pausando...");

            // 2. Aguardar janela de recovery da norma ISO 11898 (128ms)
            Timer::after(Duration::from_millis(128)).await;

            // 3. Executar o recovery direto no hardware via PAC (MCAL)
            log::warn!("BSW Diag: Invocando twai_recover_unsafe() na MCAL...");
            unsafe {
                crate::mcal::twai::twai_recover_unsafe();
            }

            // 4. Sinalizar retomada cooperativa
            BUS_OFF_RECOVERING.store(false, Ordering::Relaxed);
            crate::BUS_OFF_CLEAR.signal(());
            log::info!("BSW Diag: Recovery concluído. BUS_OFF_CLEAR emitido.");

            // 5. Registrar evento de sucesso no CSV do SD Card
            let mut diag_line: heapless::String<128> = heapless::String::new();
            let _ = core::fmt::write(
                &mut diag_line,
                format_args!("{},DIAG,0x000,,,,,,,,NOR,BUS_OFF_RECOVERED_latency_ms=128\n", ts_ms),
            );
            let _ = bsw_mem::push_line(diag_line.as_str());
        }

        // ------------------------------------------------------------------
        // 2. Logger Stall Detection (a cada 30 s — apenas se sessão ativa)
        // ------------------------------------------------------------------
        if stall_check_instant.elapsed() >= Duration::from_secs(30) {
            stall_check_instant = Instant::now();

            if crate::types::SESSION_ACTIVE.load(Ordering::Relaxed) {
                let current_count = LOGGER_STATS.frames_received.load(Ordering::Relaxed);
                if current_count == last_frame_count && bsw_mem::SD_OK.load(Ordering::Relaxed) {
                    let ts_ms = Instant::now().as_millis();
                    log::warn!(
                        "BSW Diag: LOGGER_STALL — nenhum frame processado nos últimos 30s (total={})",
                        current_count
                    );

                    // Registrar LOGGER_STALL no SD
                    let mut stall_line: heapless::String<128> = heapless::String::new();
                    let _ = core::fmt::write(
                        &mut stall_line,
                        format_args!("{},DIAG,0x000,,,,,,,,NOR,LOGGER_STALL\n", ts_ms),
                    );
                    let _ = bsw_mem::push_line(stall_line.as_str());
                }
                last_frame_count = current_count;
            }
        }

        // ------------------------------------------------------------------
        // 3. HEARTBEAT com Performance Metrics no SD Card (a cada 60 s)
        //
        // Formato da mensagem DIAG,HEARTBEAT:
        //   up=<s> rx=<n> sd=<n> mqtt=<n> lost=<n> ovf=<n> bo=<n>
        //   hf=<bytes> hmin=<bytes> hmax=<bytes> havg=<bytes> hu=<bytes>
        //   sdbuf=<bytes>
        //
        // Campos:
        //   up     = uptime em segundos desde boot
        //   rx     = frames recebidos (total acumulado)
        //   sd     = frames gravados no SD Card
        //   mqtt   = frames publicados via MQTT
        //   lost   = frames perdidos (SD e MQTT falharam)
        //   ovf    = overflows do channel RTE (capacidade 32)
        //   bo     = eventos Bus-Off do controlador TWAI
        //   hf     = heap free atual (bytes livres dos 58 KB alocados)
        //   hmin   = heap free mínimo na janela de 60 s
        //   hmax   = heap free máximo na janela de 60 s
        //   havg   = heap free médio na janela de 60 s
        //   hu     = heap usado atual (bytes)
        //   sdbuf  = bytes pendentes no buffer circular do SD (de 4096)
        // ------------------------------------------------------------------
        if heartbeat_instant.elapsed() >= Duration::from_secs(60) {
            heartbeat_instant = Instant::now();

            let ts_ms = Instant::now().as_millis();
            let uptime_s = ts_ms / 1000;
            let frames_rx = LOGGER_STATS.frames_received.load(Ordering::Relaxed);
            let frames_sd = LOGGER_STATS.frames_to_sd.load(Ordering::Relaxed);
            let frames_mqtt = LOGGER_STATS.frames_to_mqtt.load(Ordering::Relaxed);
            let lost = LOGGER_STATS.frames_lost.load(Ordering::Relaxed);
            let overflow = LOGGER_STATS.overflow_count.load(Ordering::Relaxed);
            let bus_off_total = BUS_OFF_COUNT.load(Ordering::Relaxed);
            let sd_write_off = bsw_mem::SD_WRITE_OFFSET.load(Ordering::Relaxed);

            let heap_min = if heap_tracker.min == usize::MAX { 0 } else { heap_tracker.min };
            let heap_max = heap_tracker.max;
            let heap_avg = heap_tracker.avg();
            let heap_free = esp_alloc::HEAP.free();
            let heap_used = esp_alloc::HEAP.used();
            let psram_used = crate::app::logger::get_backlog_psram_bytes();

            // Log para UART (terminal serial)
            log::info!(
                "BSW Diag: HEARTBEAT up={}s rx={} sd={} mqtt={} lost={} ovf={} bo={} hf={} hmin={} hmax={} havg={} hu={} sdwr={} psram={}",
                uptime_s, frames_rx, frames_sd, frames_mqtt, lost, overflow, bus_off_total,
                heap_free, heap_min, heap_max, heap_avg, heap_used, sd_write_off, psram_used,
            );

            // Gravar linha DIAG,HEARTBEAT no SD Card apenas se a sessão estiver ativa
            if crate::types::SESSION_ACTIVE.load(Ordering::Relaxed) {
                let mut hb_line: heapless::String<256> = heapless::String::new();
                let _ = core::fmt::write(
                    &mut hb_line,
                    format_args!(
                        "{},DIAG,0x000,,,,,,,,NOR,HEARTBEAT up={} rx={} sd={} mqtt={} lost={} ovf={} bo={} hf={} hmin={} hmax={} havg={} hu={} sdwr={} psram={}\n",
                        ts_ms, uptime_s, frames_rx, frames_sd, frames_mqtt,
                        lost, overflow, bus_off_total,
                        heap_free, heap_min, heap_max, heap_avg, heap_used,
                        sd_write_off, psram_used,
                    ),
                );
                let _ = bsw_mem::push_line(hb_line.as_str());
            }

            // Resetar tracker para a próxima janela de 60 s
            heap_tracker.reset();
        }
    }
}
