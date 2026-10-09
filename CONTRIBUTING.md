# Contributing to Edge Telemetry Layer

Thank you for your interest in contributing to the **Edge Telemetry Layer** project! We welcome bug reports, feature suggestions, documentation enhancements, and pull requests.

## Code of Conduct
Please ensure all interactions in discussions, issues, and pull requests remain respectful, constructive, and collaborative.

## Reporting Bugs
1. Check existing [Issues](https://github.com/amauricunha/edge-telemetry-layer/issues) to ensure the bug hasn't already been reported.
2. If reporting a new issue, use the **Bug Report** template.
3. Include your hardware setup (ESP32-S3 board, Arduino model, CAN transceivers, wiring details), firmware version, and reproduction steps.
4. If the bug relates to dataset processing, please provide sample CSV snippet or run `tools/validate_csv.py`.

## Suggesting Enhancements
1. Open an issue using the **Feature Request** template.
2. Clearly describe the problem you are solving, the proposed technical solution, and any architectural trade-offs.
3. For architectural changes touching Model-Based Systems Engineering (MBSE), specify any impact on FRETish requirements or AADL packages.

## Development Guidelines

### Firmware — Rust ESP32-S3 (`firmware/esp32s3_collector`)
- The firmware targets `no_std` environments using the **Embassy** asynchronous runtime.
- **Never introduce dynamic heap allocation** (`alloc`) in real-time execution paths (ISR / main loop) without static bounds.
- Maintain memory safety and static buffer size guarantees.
- Ensure the code passes `cargo fmt --check`.

### Firmware — Arduino UNO Emulator (`firmware/uno_ecu_emulator`)
- The emulator is maintained with **PlatformIO** (`platformio.ini`) and mirrored in `uno_ecu_emulator_ino`.
- Keep CAN DBC frame formatting aligned with `docs/DBC_minimal.dbc`.
- Validate that standard OBD-II PID responses conform to `docs/OBD-PIDS.md`.

### Python Analysis Tools (`tools/`)
- Adhere to PEP 8 standards.
- Run `python tools/validate_csv.py` and `python tools/compute_metrics.py` against sample sessions before submitting changes.

### Documentation & MBSE Artifacts
- Formal requirements live in `docs/MBSE/Entregas/Entrega 1/`.
- AADL models and OSATE packages live in `docs/MBSE/Entregas/Entrega 2 e 3/EdgeTelemetry_MBSE_OSATE/`.
- Preserve traceability between system requirements (SRS), FRET IDs, and AADL components.

## Submitting a Pull Request
1. Fork the repository and create your feature branch: `git checkout -b feature/my-new-feature`.
2. Commit your changes with clear, semantic commit messages (e.g., `feat:`, `fix:`, `docs:`).
3. Do **not** commit local credentials (`config_local.rs`, `.env`, passwords, tokens).
4. Push your branch to GitHub and open a Pull Request using the provided PR template.
