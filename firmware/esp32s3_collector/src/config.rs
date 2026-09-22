#![allow(dead_code)]

//! Constantes de configuração do Edge Telemetry Layer.
//!
//! Todos os parâmetros de operação do firmware são definidos aqui como `const`.
//! Nenhuma alocação dinâmica — tudo resolvido em compile-time.

// =============================================================================
// Barramento CAN
// =============================================================================

/// Baudrate do barramento CAN em Kbps.
pub const CAN_BAUDRATE_KBPS: u32 = 500;

/// CAN IDs reconhecidos pela DBC mínima do projeto.
pub const CAN_ID_SPEED_RPM: u32 = 0x100;
pub const CAN_ID_THROTTLE_LOAD: u32 = 0x200;
pub const CAN_ID_COOLANT_TEMP: u32 = 0x300;

/// CAN IDs do protocolo OBD-II (ISO 15765-4).
pub const OBD_REQUEST_ID: u32 = 0x7DF;   // Solicitação funcional broadcast
pub const OBD_RESPONSE_ID: u32 = 0x7E8;  // Resposta da ECU (endereço físico)

/// CAN ID para comando de perfil de condução (ESP32 → UNO R3).
pub const PROFILE_CMD_ID: u32 = 0x010;

// =============================================================================
// Polling OBD-II
// =============================================================================

/// Intervalo entre cada poll OBD-II em milissegundos.
pub const OBD_POLL_INTERVAL_MS: u64 = 100;

/// Timeout para aguardar resposta OBD-II (0x7E8) em milissegundos.
pub const OBD_TIMEOUT_MS: u64 = 50;

/// PIDs OBD-II consultados em round-robin.
/// Cada PID é consultado uma vez a cada 600 ms (6 PIDs × 100 ms).
///
/// | PID  | Sinal             | Fórmula                              |
/// |------|--------------------|--------------------------------------|
/// | 0x0D | Vehicle Speed      | A                                    |
/// | 0x0C | Engine RPM         | ((A << 8) | B) / 4                   |
/// | 0x11 | Throttle Position  | A * 100 / 255                        |
/// | 0x04 | Engine Load        | A * 100 / 255                        |
/// | 0x10 | MAF Air Flow       | ((A << 8) | B) / 100.0               |
/// | 0x05 | Coolant Temp       | A - 40                               |
pub const OBD_PIDS: [u8; 6] = [0x0D, 0x0C, 0x11, 0x04, 0x10, 0x05];

/// Modo OBD-II utilizado nas solicitações (Mode 01 — Show Current Data).
pub const OBD_MODE_REQUEST: u8 = 0x01;

/// Modo OBD-II esperado nas respostas (0x41 = Mode 01 + 0x40).
pub const OBD_MODE_RESPONSE: u8 = 0x41;

// =============================================================================
// Channel RTE (Runtime Environment)
// =============================================================================

/// Capacidade do channel de telemetria (bounded, sem heap).
/// A 20 frames/s do UNO R3, o buffer de 32 suporta 1,6 s de atraso.
pub const CHANNEL_CAPACITY: usize = 32;

// =============================================================================
// Hardware — Pinos GPIO do ESP32-S3 DevKit
// =============================================================================

// TWAI CAN: TX=GPIO4, RX=GPIO5
// SPI SD Card: CS=GPIO10, MOSI=GPIO11, CLK=GPIO12, MISO=GPIO13
pub const SPI_SD_CS: u8 = 10;
pub const SPI_SD_MOSI: u8 = 11;
pub const SPI_SD_CLK: u8 = 12;
pub const SPI_SD_MISO: u8 = 13;

// =============================================================================
// Conectividade Wi-Fi e Broker MQTT (Semana 3)
// =============================================================================

pub const MQTT_BROKER_PORT: u16 = 1883;
pub const MQTT_TOPIC_RAW: &str = "/telemetry/raw";
pub const MQTT_TOPIC_STATUS: &str = "/system/status";
pub const MQTT_TOPIC_REPLAY: &str = "/telemetry/replay";

