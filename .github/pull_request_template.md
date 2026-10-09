## Description
Provide a concise explanation of the changes made and the motivation behind them.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation / MBSE artifact update
- [ ] Refactoring / Performance improvement

## Subsystems Affected
- [ ] ESP32-S3 Firmware (Rust `no_std`, Embassy)
- [ ] Arduino UNO ECU Emulator (C++, PlatformIO)
- [ ] Model-Based Systems Engineering (NASA FRET / AADL / CAvA)
- [ ] DevOps & Cloud Ingestion (Docker, Telegraf, InfluxDB, Grafana)
- [ ] Analysis & Validation Scripts (`tools/`)

## Verification & Testing
Describe the tests you ran to verify your changes:
- [ ] Hardware-in-the-Loop (HIL) bench run
- [ ] `python tools/validate_csv.py <dataset.csv>` passed
- [ ] `python tools/compute_metrics.py <datasets>` passed
- [ ] `cargo fmt --check` passed
- [ ] `pio run -d firmware/uno_ecu_emulator` passed

## Security Checklist
- [ ] No credentials, Wi-Fi keys, or local tokens are included in this PR.
- [ ] Any sample configuration remains in `.example` or `.env_exemplo` files.
