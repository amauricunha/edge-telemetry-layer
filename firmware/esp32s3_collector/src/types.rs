#![allow(dead_code)]

//! Tipos centrais do Edge Telemetry Layer.
//!
//! Define as structs e enums compartilhados entre todas as camadas do firmware
//! (MCAL, RTE, APP). Nenhum tipo aqui depende de hardware — apenas dados puros.

/// Fonte do dado de telemetria.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataSource {
    /// Frame recebido passivamente via DBC (0x100, 0x200, 0x300).
    CanDbc,
    /// Resposta ativa de polling OBD-II (0x7E8).
    ObdPid,
}

impl DataSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSource::CanDbc => "CAN_DBC",
            DataSource::ObdPid => "OBD_PID",
        }
    }
}

impl core::fmt::Display for DataSource {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

use portable_atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

/// Base Unix Epoch em milissegundos (ex: 1724498400000). 0 se não sincronizado.
pub static EPOCH_BASE_MS: AtomicU64 = AtomicU64::new(0);

/// Uptime do ESP32-S3 em ms no momento em que o epoch foi sincronizado.
pub static SYNC_UPTIME_MS: AtomicU64 = AtomicU64::new(0);

/// Uptime do início da sessão ativa em milissegundos.
pub static SESSION_START_MS: AtomicU64 = AtomicU64::new(0);

/// Duração programada da sessão em milissegundos (0 = tempo contínuo / infinito).
pub static SESSION_DURATION_MS: AtomicU64 = AtomicU64::new(0);

/// Flag indicando se há um temporizador de sessão ativo.
pub static SESSION_TIMER_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Flag indicando se a sessão está ativa para gravação/transmissão (false quando STOP ou expirou).
pub static SESSION_ACTIVE: AtomicBool = AtomicBool::new(true);

/// Sincroniza a base de tempo Epoch em milissegundos (ex: via comando MQTT).
pub fn sync_epoch_time(epoch_ms: u64) {
    let now_uptime = embassy_time::Instant::now().as_millis();
    EPOCH_BASE_MS.store(epoch_ms, Ordering::Relaxed);
    SYNC_UPTIME_MS.store(now_uptime, Ordering::Relaxed);
    log::info!("BSW Time: Epoch sincronizado em {} ms (uptime = {} ms)", epoch_ms, now_uptime);
}

/// Retorna o timestamp atual em milissegundos (absoluto se sincronizado ou relativo ao boot).
pub fn get_current_timestamp_ms() -> u64 {
    let now_uptime = embassy_time::Instant::now().as_millis();
    let epoch_base = EPOCH_BASE_MS.load(Ordering::Relaxed);
    if epoch_base > 0 {
        let sync_uptime = SYNC_UPTIME_MS.load(Ordering::Relaxed);
        let elapsed = now_uptime.saturating_sub(sync_uptime);
        epoch_base + elapsed
    } else {
        now_uptime
    }
}

/// Inicia uma sessão com temporização configurável em minutos (0 = contínuo/infinito).
pub fn start_session_timer(duration_minutes: u32, epoch_opt: Option<u64>) {
    if let Some(epoch) = epoch_opt {
        sync_epoch_time(epoch);
    }
    let now_uptime = embassy_time::Instant::now().as_millis();
    SESSION_START_MS.store(now_uptime, Ordering::Relaxed);
    SESSION_ACTIVE.store(true, Ordering::Relaxed);
    
    if duration_minutes > 0 {
        let dur_ms = (duration_minutes as u64) * 60_000;
        SESSION_DURATION_MS.store(dur_ms, Ordering::Relaxed);
        SESSION_TIMER_ACTIVE.store(true, Ordering::Relaxed);
        log::info!("BSW Session: Sessão iniciada com temporizador de {} min ({} ms)", duration_minutes, dur_ms);
    } else {
        SESSION_DURATION_MS.store(0, Ordering::Relaxed);
        SESSION_TIMER_ACTIVE.store(false, Ordering::Relaxed);
        log::info!("BSW Session: Sessão iniciada em modo contínuo (tempo infinito)");
    }
}

/// Rótulo do perfil de condução ativo.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum SessionLabel {
    Economico,
    #[default]
    Normal,
    Esportivo,
}

impl SessionLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionLabel::Economico => "Economico",
            SessionLabel::Normal => "Normal",
            SessionLabel::Esportivo => "Esportivo",
        }
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => SessionLabel::Economico,
            2 => SessionLabel::Normal,
            3 => SessionLabel::Esportivo,
            _ => SessionLabel::Normal,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            SessionLabel::Economico => 1,
            SessionLabel::Normal => 2,
            SessionLabel::Esportivo => 3,
        }
    }
}

impl core::fmt::Display for SessionLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Variável global para armazenar o perfil de condução ativo via comandos MQTT.
pub static ACTIVE_SESSION_LABEL: AtomicU8 = AtomicU8::new(2); // Normal by default (2)

/// Frame de telemetria estruturado — unidade de transporte no channel.
///
/// Cada frame representa uma leitura decodificada de um sinal do barramento CAN
/// (passiva via DBC ou ativa via polling OBD-II). Campos `Option<f32>` são `None`
/// quando o frame não carrega aquele sinal específico.
#[derive(Clone, Debug)]
pub struct TelemetryFrame {
    /// Timestamp em milissegundos desde o boot do ESP32-S3.
    pub timestamp_ms: u64,
    /// Origem do dado (DBC passivo ou OBD-II ativo).
    pub source: DataSource,
    /// CAN Arbitration ID do frame original.
    pub can_id: u32,
    /// Velocidade do veículo em km/h (ID 0x100, bytes 0-1).
    pub speed_kmh: Option<f32>,
    /// Rotação do motor em RPM (ID 0x100, bytes 2-3).
    pub rpm: Option<f32>,
    /// Posição do acelerador em % (ID 0x200, byte 0).
    pub throttle_pct: Option<f32>,
    /// Carga do motor em % (ID 0x200, byte 1).
    pub engine_load_pct: Option<f32>,
    /// Fluxo de ar do MAF em g/s — PID 0x10 (ID 0x7E8).
    pub maf_g_s: Option<f32>,
    /// Temperatura do líquido de arrefecimento em °C (ID 0x300, byte 0).
    pub coolant_temp_c: Option<f32>,
    /// Latência round-trip do polling OBD-II em ms.
    /// Populado apenas para frames com `source == ObdPid`.
    pub obd_latency_ms: Option<f32>,
    /// Rótulo da sessão de condução ativa.
    pub session_label: SessionLabel,
}

impl TelemetryFrame {
    /// Cria um frame vazio (todos os sinais `None`) com timestamp e source.
    pub fn new(timestamp_ms: u64, source: DataSource, can_id: u32) -> Self {
        Self {
            timestamp_ms,
            source,
            can_id,
            speed_kmh: None,
            rpm: None,
            throttle_pct: None,
            engine_load_pct: None,
            maf_g_s: None,
            coolant_temp_c: None,
            obd_latency_ms: None,
            session_label: SessionLabel::from_u8(ACTIVE_SESSION_LABEL.load(Ordering::Relaxed)),
        }
    }
}

/// Frame CAN bruto — wrapper mínimo para desacoplar da API do esp-hal.
#[derive(Clone, Debug)]
pub struct CanFrame {
    /// CAN Arbitration ID (11-bit standard).
    pub id: u32,
    /// Dados do payload (até 8 bytes).
    pub data: [u8; 8],
    /// Data Length Code.
    pub dlc: u8,
}

impl CanFrame {
    /// Cria um novo frame CAN com o ID, dados e DLC especificados.
    pub fn new(id: u32, data: &[u8]) -> Self {
        let mut frame_data = [0u8; 8];
        let len = data.len().min(8);
        frame_data[..len].copy_from_slice(&data[..len]);
        Self {
            id,
            data: frame_data,
            dlc: len as u8,
        }
    }
}

/// Erros possíveis na camada CAN.
#[derive(Clone, Debug)]
pub enum CanError {
    /// O controlador TWAI entrou em estado Bus-Off (TEC > 255).
    BusOff,
    /// Timeout ao aguardar resposta (ex: OBD-II sem resposta em 50 ms).
    Timeout,
    /// Falha na transmissão do frame.
    TxFailed,
    /// Erro genérico do periférico TWAI.
    HalError,
}

impl core::fmt::Display for CanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CanError::BusOff => write!(f, "CAN_BUS_OFF"),
            CanError::Timeout => write!(f, "CAN_TIMEOUT"),
            CanError::TxFailed => write!(f, "CAN_TX_FAILED"),
            CanError::HalError => write!(f, "CAN_HAL_ERROR"),
        }
    }
}

/// Estrutura compacta de 18 bytes para empacotamento binário de telemetria (Opção A - Batching 1Hz).
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct BinaryFrame {
    pub timestamp_ms: u32,
    pub can_id: u16,
    pub source: u8,
    pub label: u8,
    pub val1: f32,
    pub val2: f32,
    pub obd_latency: u16,
}

impl From<&TelemetryFrame> for BinaryFrame {
    fn from(frame: &TelemetryFrame) -> Self {
        let val1 = frame.speed_kmh
            .or(frame.throttle_pct)
            .or(frame.maf_g_s)
            .unwrap_or(0.0);
        let val2 = frame.rpm
            .or(frame.engine_load_pct)
            .or(frame.coolant_temp_c)
            .unwrap_or(0.0);
        let source = match frame.source {
            DataSource::CanDbc => 0,
            DataSource::ObdPid => 1,
        };
        let label = frame.session_label.as_u8();
        let obd_latency = frame.obd_latency_ms.unwrap_or(0.0) as u16;

        Self {
            timestamp_ms: (frame.timestamp_ms & 0xFFFF_FFFF) as u32,
            can_id: frame.can_id as u16,
            source,
            label,
            val1,
            val2,
            obd_latency,
        }
    }
}
