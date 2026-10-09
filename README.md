# Edge Telemetry Layer

[![CI](https://github.com/amauricunha/edge-telemetry-layer/actions/workflows/ci.yml/badge.svg)](https://github.com/amauricunha/edge-telemetry-layer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Data License: CC BY 4.0](https://img.shields.io/badge/Data%20License-CC%20BY%204.0-lightgrey.svg)](LICENSE-DATA)
[![Firmware: Rust no_std](https://img.shields.io/badge/Firmware-Rust_no__std_(Embassy)-orange.svg)](firmware/esp32s3_collector)
[![Emulator: C++ PlatformIO](https://img.shields.io/badge/Emulator-C%2B%2B_PlatformIO-blue.svg)](firmware/uno_ecu_emulator)
[![MBSE: SAE AS5506B AADL](https://img.shields.io/badge/MBSE-SAE_AS5506B_AADL-navy.svg)](docs/MBSE)
[![Formal Verification: NASA FRET](https://img.shields.io/badge/Requirements-NASA_FRET-red.svg)](docs/MBSE/Entregas/Entrega%201)

> **Language / Idioma:** English | [🇧🇷 Versão em Português](README.pt-BR.md)

---

## Abstract

**Edge Telemetry Layer** is an open-source, low-cost embedded edge telemetry and Hardware-in-the-Loop (HIL) diagnostics platform for automotive CAN bus (500 kbps) and OBD-II (ISO 15765-4). Developed within the **Federal University of Santa Catarina (UFSC)**, it combines:

1. **Memory-Safe Edge Collector (Rust `no_std`):** Runs on an **ESP32-S3** under the asynchronous **Embassy** runtime, performing cyclic OBD-II PID polling, deterministic CSV logging with zero runtime dynamic allocation, local FAT32 SD card persistence with automatic fallback, and MQTT publishing over Wi-Fi.
2. **HIL ECU Emulator (C++):** Runs on an **Arduino UNO R3** paired with an **MCP2515** CAN controller, emulating vehicle engine dynamics (Economic, Normal, and Sport profiles) and responding to diagnostic PID queries with sub-3 ms jitter.
3. **Model-Based Systems Engineering (MBSE) Verification:** Covers the full lifecycle across NASA's Formal Requirements Elicitation Tool (**NASA FRET** / Kind 2 SMT prover), **SAE AS5506B AADL** architectural modeling in **OSATE**, static scheduling and latency analyses, and architectural variability evaluation (**CAvA**) for dual-core and **TinyML** edge intelligence evolution.

---

## Architecture Topology

```text
  +--------------------------------------------------------------------------------+
  |                          HARDWARE-IN-THE-LOOP (HIL) BENCH                      |
  |                                                                                |
  |   +-----------------------+                    +---------------------------+   |
  |   |    Arduino UNO R3     |                    |         ESP32-S3          |   |
  |   |   (ECU Simulator)     |                    |   (Edge Telemetry Unit)   |   |
  |   |                       |                    |                           |   |
  |   |   - Profile Engine    |   CAN Bus 500kbps  |   - TWAI CAN Driver       |   |
  |   |   - OBD-II Responder  |<==================>|   - OBD-II Active Poller  |   |
  |   |   - MCP2515 SPI       |  (120 Ohm term.)   |   - CSV Structurer        |   |
  |   +-----------------------+                    |   - FAT32 SD Storage (SPI)|   |
  |                                                |   - Embassy Async RTOS    |   |
  |                                                +-------------+-------------+   |
  +--------------------------------------------------------------|-----------------+
                                                                 | Wi-Fi (MQTT)
                                                                 v
                                                   +---------------------------+
                                                   |       DEVOPS STACK        |
                                                   |   (Docker Compose)        |
                                                   |                           |
                                                   |   - Telegraf (MQTT ingest)|
                                                   |   - InfluxDB v2 (TSDB)    |
                                                   |   - Grafana (Real-Time)   |
                                                   +---------------------------+
```

---

## MBSE Engineering Pillars

The engineering specification follows four formal deliverables:

| Pillar | Engineering Domain | Formal Tool | Deliverables & Artifacts |
|---|---|---|---|
| **Pillar 1** | **Formal Requirements Elicitation** | [NASA FRET](https://github.com/NASA-SW-VnV/fret) + Kind 2 / Z3 | 48 FRETish temporal contracts across 9 software components; 100% formal realizability verified in 0.41 s. See [`docs/MBSE/Entregas/Entrega 1/`](docs/MBSE/Entregas/Entrega%201/). |
| **Pillar 2** | **Architectural Modeling** | [OSATE](https://osate.org/) (SAE AS5506B AADL) | Execution platform (`ESP32S3_Processor`, CAN/SPI buses, peripherals) and software processes/threads. See [`docs/MBSE/Entregas/Entrega 2 e 3/`](docs/MBSE/Entregas/Entrega%202%20e%203/). |
| **Pillar 3** | **Real-Time Timing Verification** | OSATE Plugins (Schedule & Latency) | Static priority-preemptive schedulability ($U = 34.5\% \le 75.68\%$) and end-to-end latency analysis (immediate vs. delayed connections: 62.1% latency reduction). |
| **Pillar 4** | **Variability & Architecture Evolution** | [CAvA](https://doi.org/10.1007/s10270-016-0563-7) + TinyML | Component variability analysis replacing cloud pipeline with on-device TinyML driver coaching; ISO/IEC 25010 quality trade-off analysis. See [`docs/MBSE/Entregas/Entrega 4/`](docs/MBSE/Entregas/Entrega%204/). |

---

## Empirical Acceptance Criteria (HIL Bench Results)

Validated across three reproducible driving sessions (`dataset/S_0001.CSV`, `dataset/S_0003.CSV`, `dataset/S_0005.CSV`) with `tools/compute_metrics.py`:

| Criterion | Target Metric | Required Value | Obtained Bench Value | Status |
|---|---|---|---|:---:|
| **[AC-01]** | CAN Traffic Reception & Loss Rate | Loss $\le 1.0\%$ @ 500 kbps | **0.00%** (0 loss across 30k+ frames) | **PASSED** |
| **[AC-02]** | OBD-II Cyclic Polling Latency | Mean Latency $< 10.0$ ms | **2.28 ms** (std: 0.14 ms) | **PASSED** |
| **[AC-03]** | OBD-II Response Jitter | Temporal Jitter $< 3.0$ ms | **0.18 ms** | **PASSED** |
| **[AC-04]** | CSV Serialization & Throughput | Throughput $\ge 200$ pkt/s | **214 pkt/s** (100% structural integrity) | **PASSED** |
| **[AC-05]** | Static SRAM Consumption | Memory $< 200.0$ KB | **58.0 KB** static heap / zero runtime leaks | **PASSED** |
| **[AC-06]** | Offline Fallback on SD | Fallback time $\le 5.0$ ms | **1.20 ms** upon Wi-Fi / MQTT drop | **PASSED** |
| **[AC-07]** | CAN Bus-Off Auto-Recovery | 128 ms wait (ISO 11898) | **128.4 ms** recovery handshake | **PASSED** |
| **[AC-08]** | Continuous SD Recording | Samples $\ge 72,000$ / 1h | **72,400+** samples successfully synced | **PASSED** |

---

## Repository Structure

```text
├── .github/
│   ├── workflows/ci.yml       # GitHub Actions: dataset checks, PlatformIO build, cargo fmt
│   ├── dependabot.yml         # Automated dependency updates for Cargo, Pip, Actions
│   └── ISSUE_TEMPLATE/        # Bug report and feature request templates
├── dataset/
│   ├── README.md              # Bench reproduction protocol and session schemas
│   ├── S_0001.CSV             # Bench dataset: Economic driving session
│   ├── S_0003.CSV             # Bench dataset: Normal driving session
│   └── S_0005.CSV             # Bench dataset: Sport driving session
├── DevOps/
│   ├── docker-compose.yml     # InfluxDB v2, Telegraf, and Grafana service stack
│   └── telegraf.conf          # Telegraf MQTT consumer and InfluxDB writer
├── docs/
│   ├── spec_arquitetura.md    # High-level architecture specification
│   ├── DBC_minimal.dbc        # CAN database definition (DBC format)
│   ├── OBD-PIDS.md            # Supported standard OBD-II PIDs
│   ├── PAYLOAD_SCHEMA.md      # Binary MQTT and CSV storage schema
│   ├── DECISIONS.md           # Architecture Decision Records (ADRs)
│   ├── variaveis.md           # Variable catalog and sampling frequencies
│   ├── pratica_bancada.md     # HIL bench wiring and pinout diagram
│   ├── relatorio_gate_f1.md   # Consolidated engineering gate report
│   └── MBSE/                  # Model-Based Systems Engineering artifacts
│       ├── Entregas/          # Deliverables: Pillar 1 (FRET), Pillars 2 & 3 (OSATE), Pillar 4 (CAvA)
│       └── overleaf/          # Publication-ready LaTeX report (Overleaf package)
├── firmware/
│   ├── esp32s3_collector/     # Rust (no_std, Embassy) edge collector for ESP32-S3
│   ├── uno_ecu_emulator/      # C++ (PlatformIO) ECU emulator for Arduino UNO R3
│   └── uno_ecu_emulator_ino/  # Arduino IDE mirror of the ECU emulator
├── tools/
│   ├── .env_exemplo           # Environment template for analysis scripts
│   ├── requirements.txt       # Python dependencies
│   ├── validate_csv.py        # CSV schema and checksum validator
│   ├── compute_metrics.py     # Automated calculation of criteria AC-01 to AC-08
│   ├── plot_session.py        # Telemetry visualization and charting utility
│   ├── send_command.py        # Remote MQTT session command dispatcher
│   └── analyze_influxdb.py    # InfluxDB data extractor and analysis tool
├── .gitattributes             # Line ending normalization and GitHub Linguist overrides
├── .gitignore                 # Build artifacts, secrets, and IDE exclusions
├── .zenodo.json               # Zenodo publication metadata
├── CITATION.cff               # Machine-readable academic citation metadata
├── CONTRIBUTING.md            # Guidelines for issues, code style, and pull requests
├── LICENSE                    # MIT License (Code, firmware, tools)
├── LICENSE-DATA               # CC BY 4.0 License (Datasets and MBSE documentation)
├── README.md                  # This document
├── README.pt-BR.md            # Brazilian Portuguese documentation
└── SECURITY.md                # Vulnerability disclosure policy
```

---

## Quickstart Guide

### 1. ECU Emulator (Arduino UNO R3)

Connect the Arduino UNO R3 to an MCP2515 CAN module via SPI. Build and flash the firmware using **PlatformIO**:

```bash
cd firmware/uno_ecu_emulator
pio run -t upload
```

*(Alternatively, open `firmware/uno_ecu_emulator_ino/uno_ecu_emulator_ino.ino` in the Arduino IDE).*

### 2. Edge Collector (ESP32-S3)

Prerequisites: Rust ESP toolchain (`espup`).

```bash
cd firmware/esp32s3_collector

# Copy the configuration template and fill in your Wi-Fi / MQTT broker settings:
cp src/config_local.rs.example src/config_local.rs

# Build and flash using cargo-espflash:
source ~/export-esp.sh
cargo +esp run --release
```

### 3. Cloud Ingestion & Dashboard (Docker Compose)

Create your local `.env` with strong passwords, then launch the telemetry ingestion stack:

```bash
cd DevOps

# Create your local environment file:
cat << 'EOF' > .env
INFLUX_USER=telemetry
INFLUX_PASSWORD=StrongAdminPassword123!
INFLUX_ORG=telemetry_org
INFLUX_BUCKET=telemetry
INFLUX_TOKEN=GeneratedSecretTokenString123!
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=StrongGrafanaPassword123!
MQTT_BROKER_HOST=host.docker.internal
EOF

# Start InfluxDB, Telegraf, and Grafana:
docker compose up -d
```

- **Grafana:** `http://localhost:3000` (User: `admin`)
- **InfluxDB v2:** `http://localhost:8897`

### 4. Dataset Validation & Metrics

Run automated validation over the published HIL datasets:

```bash
# Install Python dependencies:
pip install -r tools/requirements.txt

# Validate CSV format and checksum integrity:
python tools/validate_csv.py dataset/S_0001.CSV

# Compute AC-01 to AC-08 acceptance metrics:
python tools/compute_metrics.py dataset/S_0001.CSV dataset/S_0003.CSV dataset/S_0005.CSV

# Plot session telemetry:
python tools/plot_session.py dataset/S_0001.CSV
```

---

## Citation

If you use the **Edge Telemetry Layer** firmware, HIL emulator, datasets, or MBSE architectural models in academic research, please cite it using:

### BibTeX
```bibtex
@misc{cunha2026edgetelemetry,
  author       = {Cunha, Amauri},
  title        = {{Edge Telemetry Layer: An Open-Source Embedded Real-Time Telemetry and Hardware-in-the-Loop Architecture for CAN and OBD-II with Formal MBSE Verification}},
  year         = {2026},
  publisher    = {GitHub},
  journal      = {GitHub repository},
  howpublished = {\url{https://github.com/amauricunha/edge-telemetry-layer}},
  doi          = {10.5281/zenodo.placeholder}
}
```

Or click the **"Cite this repository"** button in GitHub's sidebar (powered by [`CITATION.cff`](CITATION.cff)).

---

## Academic Context & Acknowledgments

This project was developed within the **Department of Automation and Systems (DAS)** at the **Federal University of Santa Catarina (UFSC)**, under the academic mentorship of **Prof. Dr. Leandro Buss Becker**.

Special acknowledgment to the methodologies developed in:
- Sheridan, O., Becker, L. B., Farrell, M., Luckcuck, M., & Monahan, R. (2025). *Sharper Specs for Smarter Drones: Formalising Requirements with FRET*. RefSQ 2025, Lecture Notes in Computer Science, Springer.
- Sales, I., & Becker, L. B. (2018). *CAvA: Component and Architecture Variability and Evolution Approach for Critical Embedded Systems*. Software & Systems Modeling.

---

## Safety Disclaimer

> [!WARNING]
> This software and hardware architecture are designed for research, bench experimentation, and Hardware-in-the-Loop (HIL) environments. OBD-II requests are strictly limited to standard read-only functional diagnostic PIDs (Service 01). Connecting third-party diagnostic equipment to production motor vehicles while driving carries inherent physical and electrical risks. Testing on real vehicles is solely at the user's discretion and responsibility.

---

## License

- **Source Code, Firmware & Tooling:** [MIT License](LICENSE) &copy; 2026 Amauri Cunha.
- **Datasets & MBSE Specifications:** [Creative Commons Attribution 4.0 International (CC BY 4.0)](LICENSE-DATA).
