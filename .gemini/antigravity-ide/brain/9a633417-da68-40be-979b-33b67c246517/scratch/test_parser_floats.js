const parser = require('/mnt/c/workspace/fret/fret-electron/app/parser/FretParser');

const testCases = [
  "in active_session when can_bus_healthy the esp32s3_collector shall always satisfy frame_loss_percentage <= 1.0",
  "in active_session when memory_supervision_active the esp32s3_collector shall always satisfy sram_usage_kb < 200.0",
  "in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy mean_obd_latency_ms < 10.0",
  "in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy latency_jitter_std_ms < 3.0",
  "in active_session when active_profile >= 1 upon timer1_50ms_tick the uno_ecu_emulator shall within 1 MILLISECOND satisfy physics_model_updated & speed_kmh >= 0.0 & rpm >= 0.0 & throttle_pct >= 0.0 & load_pct >= 0.0 & maf_g_s >= 0.0 & coolant_temp_c >= -40.0"
];

for (const tc of testCases) {
  try {
    const res = parser.parse(tc);
    console.log("PASS:", tc.substring(0, 50) + "...");
  } catch (err) {
    console.error("FAIL:", tc, err.message);
  }
}
