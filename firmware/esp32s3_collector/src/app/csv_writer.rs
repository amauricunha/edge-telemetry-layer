//! Serialização de pacotes de telemetria em formato CSV.
//!
//! Converte `TelemetryFrame` em linhas de texto formatadas sem alocação dinâmica no heap (D-01).

use core::fmt::Write;
use heapless::String;

use crate::types::{SessionLabel, TelemetryFrame};

/// Retorna a linha de cabeçalho das 11 colunas CSV do schema do projeto.
pub const fn csv_header() -> &'static str {
    "timestamp_ms,source,can_id,speed_kmh,rpm,throttle_pct,engine_load_pct,maf_g_s,coolant_temp_c,session_label,obd_latency_ms\n"
}

/// Serializa um `TelemetryFrame` em uma linha CSV formatada de até 320 bytes.
pub fn serialize(frame: &TelemetryFrame) -> String<320> {
    let mut s: String<320> = String::new();

    // Campos fixos: timestamp, source, can_id
    let _ = write!(
        s,
        "{},{},{:#05x},",
        frame.timestamp_ms,
        frame.source.as_str(),
        frame.can_id
    );

    // Campos opcionais numéricos (Option<f32>)
    write_opt_f32(&mut s, frame.speed_kmh, 1);
    s.push(',').unwrap();
    write_opt_f32(&mut s, frame.rpm, 0);
    s.push(',').unwrap();
    write_opt_f32(&mut s, frame.throttle_pct, 1);
    s.push(',').unwrap();
    write_opt_f32(&mut s, frame.engine_load_pct, 1);
    s.push(',').unwrap();
    write_opt_f32(&mut s, frame.maf_g_s, 2);
    s.push(',').unwrap();
    write_opt_f32(&mut s, frame.coolant_temp_c, 1);
    s.push(',').unwrap();

    // session_label e obd_latency_ms
    let _ = write!(s, "{},", frame.session_label.as_str());
    write_opt_f32(&mut s, frame.obd_latency_ms, 3);
    s.push('\n').unwrap();

    s
}

/// Serializa um evento de diagnóstico do sistema (DIAG) no formato CSV de 11 colunas.
pub fn serialize_diag(timestamp_ms: u64, label: &SessionLabel, _msg: &str) -> String<128> {
    let mut s: String<128> = String::new();
    let _ = write!(
        s,
        "{},DIAG,0x000,,,,,,{},\n",
        timestamp_ms,
        label.as_str()
    );
    s
}

/// Serializa a linha de BOOT contendo os metadados do firmware e hardware
pub fn serialize_boot(timestamp_ms: u64, label: &SessionLabel) -> String<128> {
    let mut s: String<128> = String::new();
    let _ = write!(
        s,
        "{},DIAG,0x000,,,,,,{},\n",
        timestamp_ms,
        label.as_str()
    );
    s
}

/// Escreve um `Option<f32>` formatado com número específico de casas decimais.
/// Se `None`, não escreve nada (campo fica vazio no CSV).
fn write_opt_f32(s: &mut String<320>, val: Option<f32>, decimals: u8) {
    if let Some(v) = val {
        match decimals {
            0 => { let _ = write!(s, "{:.0}", v); }
            1 => { let _ = write!(s, "{:.1}", v); }
            2 => { let _ = write!(s, "{:.2}", v); }
            _ => { let _ = write!(s, "{:.3}", v); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_header() {
        let header = csv_header();
        assert!(header.starts_with("timestamp_ms,source,can_id"));
        assert!(header.ends_with("\n"));
    }

    #[test]
    fn test_serialize_can_dbc() {
        let mut frame = TelemetryFrame::new(1500, DataSource::CanDbc, 0x100);
        frame.speed_kmh = Some(85.5);
        frame.rpm = Some(2500.0);

        let csv = serialize(&frame);
        assert!(csv.contains("1500,CAN_DBC,0x100,85.5,2500,,,,,,NOR,"));
        assert!(csv.len() < 300);
    }

    #[test]
    fn test_serialize_obd_maf() {
        let mut frame = TelemetryFrame::new(2000, DataSource::ObdPid, 0x7E8);
        frame.maf_g_s = Some(14.25);
        frame.obd_latency_ms = Some(1.234);

        let csv = serialize(&frame);
        assert!(csv.contains("2000,OBD_PID,0x7e8,,,,,14.25,,NOR,1.234"));
        assert!(csv.len() < 300);
    }

    #[test]
    fn test_serialize_diag() {
        let label = SessionLabel::Normal;
        let diag = serialize_diag(3000, &label, "WIFI_DISCONNECTED");
        assert_eq!(diag.as_str(), "3000,DIAG,0x000,,,,,,,,NOR,WIFI_DISCONNECTED\n");
    }
}
