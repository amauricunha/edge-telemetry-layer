//! Camada BSW — Basic Software Layer.
//!
//! Re-exporta os serviços de memória (SD Card), comunicação (Wi-Fi / MQTT)
//! e diagnóstico (Bus-Off recovery, watchdog de software).

pub mod bsw_com;
pub mod bsw_diag;
pub mod bsw_mem;
