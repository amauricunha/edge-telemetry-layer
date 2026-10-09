use crate::config;
use crate::types::{CanFrame, DataSource, TelemetryFrame};

// =============================================================================
// Parser DBC — decodifica frames CAN em TelemetryFrame
// =============================================================================

/// Decodifica um frame CAN bruto conforme a DBC mínima do projeto.
///
/// # Frames reconhecidos
/// - `0x100`: `speed_kmh` (bytes 0-1, big-endian u16) e `rpm` (bytes 2-3, big-endian u16)
/// - `0x200`: `throttle_pct` (byte 0, 0–100%) e `engine_load_pct` (byte 1, 0–100%)
/// - `0x300`: `coolant_temp_c` (byte 0, valor direto em °C)
///
/// # Retorno
/// `Some(TelemetryFrame)` se o ID for reconhecido, `None` para IDs desconhecidos.
pub fn parse_dbc_frame(frame: &CanFrame, timestamp_ms: u64) -> Option<TelemetryFrame> {
    match frame.id {
        config::CAN_ID_SPEED_RPM => {
            let speed = ((frame.data[0] as u16) << 8 | frame.data[1] as u16) as f32;
            let rpm = ((frame.data[2] as u16) << 8 | frame.data[3] as u16) as f32;
            let mut tf = TelemetryFrame::new(timestamp_ms, DataSource::CanDbc, frame.id);
            tf.speed_kmh = Some(speed);
            tf.rpm = Some(rpm);
            Some(tf)
        }
        config::CAN_ID_THROTTLE_LOAD => {
            let throttle = frame.data[0] as f32 * 100.0 / 255.0;
            let load = frame.data[1] as f32 * 100.0 / 255.0;
            let mut tf = TelemetryFrame::new(timestamp_ms, DataSource::CanDbc, frame.id);
            tf.throttle_pct = Some(throttle);
            tf.engine_load_pct = Some(load);
            Some(tf)
        }
        config::CAN_ID_COOLANT_TEMP => {
            let temp = frame.data[0] as f32 - 40.0;
            let mut tf = TelemetryFrame::new(timestamp_ms, DataSource::CanDbc, frame.id);
            tf.coolant_temp_c = Some(temp);
            Some(tf)
        }
        _ => None,
    }
}

/// Decodifica uma resposta OBD-II (0x7E8) em `TelemetryFrame`.
///
/// Formato ISO 15765-4: `[num_bytes, 0x41, PID, A, B, ...]`
pub fn parse_obd_response(
    frame: &CanFrame,
    timestamp_ms: u64,
    latency_ms: f32,
) -> Option<TelemetryFrame> {
    if frame.dlc < 4 || frame.data[1] != config::OBD_MODE_RESPONSE {
        return None;
    }

    let pid = frame.data[2];
    let a = frame.data[3];
    let b = if frame.dlc > 4 { frame.data[4] } else { 0 };

    let virtual_id = match pid {
        0x0D => 0x701,  // speed
        0x0C => 0x702,  // rpm
        0x11 => 0x703,  // throttle
        0x04 => 0x704,  // engine load
        0x10 => 0x705,  // maf
        0x05 => 0x706,  // coolant
        _    => return None,
    };

    let mut tf = TelemetryFrame::new(timestamp_ms, DataSource::ObdPid, virtual_id);
    tf.obd_latency_ms = Some(latency_ms);

    match pid {
        0x0D => tf.speed_kmh = Some(a as f32),
        0x0C => tf.rpm = Some(((a as u16) << 8 | b as u16) as f32 / 4.0),
        0x11 => tf.throttle_pct = Some(a as f32 * 100.0 / 255.0),
        0x04 => tf.engine_load_pct = Some(a as f32 * 100.0 / 255.0),
        0x10 => tf.maf_g_s = Some(((a as u16) << 8 | b as u16) as f32 / 100.0),
        0x05 => tf.coolant_temp_c = Some(a as f32 - 40.0),
        _ => {}
    }

    Some(tf)
}
