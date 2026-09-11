# Edge Telemetry Layer

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Firmware: Rust](https://img.shields.io/badge/Firmware-Rust_no__std-orange.svg)](firmware/esp32s3_collector)
[![Emulator: C++](https://img.shields.io/badge/Emulator-C%2B%2B_PlatformIO-blue.svg)](firmware/uno_ecu_emulator)
[![Status](https://img.shields.io/badge/Status-Open_Source-green.svg)]()

## Descrição do Projeto
Este repositório contém o código-fonte, arquitetura e documentação para a **Edge Telemetry Layer**: uma arquitetura embarcada de baixo custo projetada para aquisição, estruturação e persistência resiliente de dados de telemetria automotiva provenientes do barramento CAN e da interface OBD-II em ambiente de bancada Hardware-in-the-Loop (HIL).

A arquitetura adota uma organização em camadas inspirada no padrão AUTOSAR Classic, polling ativo de diagnóstico via protocolo ISO 15765-4 (OBD-II), persistência local resiliente em cartão SD (formato FAT32) com fallback automático e publicação dos dados estruturados via MQTT em tempo real.

## Principais Destaques
- **Coletor de Borda em Rust:** Firmware assíncrono para o ESP32-S3 desenvolvido em Rust com o runtime Embassy, garantindo segurança de memória e concorrência sem alocação dinâmica em runtime (`no_std`).
- **Emulador de ECU:** Firmware C++ para Arduino UNO R3 integrado ao controlador CAN MCP2515, gerando dados de condução sintéticos e respondendo a solicitações de diagnóstico OBD-II.
- **Estruturação de Dados e Diagnóstico In-situ:** O módulo `crate::logger` centraliza a estruturação dos dados adquiridos no formato CSV e insere linhas de diagnóstico (`source = DIAG`) diretamente no dataset. Essas linhas registram falhas de conectividade (Wi-Fi/MQTT), problemas no barramento (Bus-Off) e, através do evento de `HEARTBEAT`, monitoram autonomamente o consumo de memória (SRAM) e a saúde dos buffers. Os scripts de análise desconsideram essas linhas do fluxo de dados principal, evitando contaminação.
- **Controle Remoto de Sessão (MQTT):** Suporte à rotulagem de dados e limpeza de memória remotamente. Através da assinatura do tópico `/coach/command`, a placa pode formatar o cartão SD (comando `RESET`) para garantir testes reprodutíveis, ou rotacionar os arquivos de log sem perda de dados (comandos `ECO`, `SPT`, `NOR`), mantendo a segregação estrita das sessões de direção.
- **Conformidade de Engenharia:** Abstração de hardware (MCAL/BSW/APP), modularização e auto-recuperação cooperativa de falhas elétricas (Bus-Off no barramento CAN).

## Estrutura do Repositório
```text
├── firmware/
│   ├── esp32s3_collector/     # Coletor de Borda em Rust (no_std, Embassy) para ESP32-S3
│   ├── uno_ecu_emulator/      # Emulador de ECU em C++ (PlatformIO) para Arduino UNO R3
│   └── uno_ecu_emulator_ino/  # Emulador de ECU (Arduino IDE) para Arduino UNO R3
├── DevOps/
│   ├── docker-compose.yml     # InfluxDB v2, Telegraf e Grafana para telemetria
│   └── telegraf.conf          # Configuração de ingestão MQTT binária e CSV para InfluxDB
├── dataset/
│   ├── README.md              # Guia de reprodução e estrutura das sessões de coleta
│   ├── S_0001.CSV             # Dataset de bancada - Sessão Econômica
│   ├── S_0003.CSV             # Dataset de bancada - Sessão Normal
│   └── S_0005.CSV             # Dataset de bancada - Sessão Esportiva
├── docs/
│   ├── spec_arquitetura.md    # Especificação detalhada da arquitetura do sistema
│   ├── DBC_minimal.dbc        # Definição das mensagens e sinais CAN DBC
│   ├── OBD-PIDS.md            # Tabela de PIDs OBD-II utilizados no projeto
│   ├── PAYLOAD_SCHEMA.md      # Estrutura Binária (MQTT) e CSV (SD Card)
│   ├── DECISIONS.md           # Registros de Decisões de Arquitetura (ADR)
│   ├── variaveis.md           # Dicionário de variáveis e taxas de amostragem
│   ├── pratica_bancada.md     # Guia de bancada HIL e fiação física
│   └── relatorio_gate_f1.md   # Relatório consolidado dos critérios de aceitação (AC-01 a AC-08)
├── tools/
│   ├── .env_exemplo           # Modelo de variáveis de ambiente
│   ├── validate_csv.py        # Validação de integridade e critérios dos datasets CSV
│   ├── compute_metrics.py     # Cálculo automatizado dos critérios AC-01 a AC-08
│   ├── plot_session.py        # Utilitário de plotagem e visualização de telemetria
│   ├── send_command.py        # Envio de comandos remotos via MQTT para o ESP32-S3
│   └── analyze_influxdb.py    # Análise e extração de telemetria do InfluxDB
├── LICENSE                    # Licença MIT
└── README.md                  # Este arquivo
```

## Como utilizar

### Pré-requisitos

1. **Hardware:**
   - Microcontrolador ESP32-S3 DevKit.
   - Placa Arduino UNO R3.
   - Controlador CAN MCP2515 com transceptor TJA1050 (para o emulador).
   - Transceptor CAN compatível com 3.3V (ex: SN65HVD230) para o ESP32-S3.
   - Cartão MicroSD (formatado em FAT32) e leitor/módulo SPI para o ESP32-S3.
   - Resistores de terminação de 120 Ω e jumpers de conexão.

2. **Software/Ambiente:**
   - Toolchain Rust para ESP32 (`espup`, `cargo-espflash`).
   - PlatformIO ou Arduino IDE para o firmware do emulador Arduino UNO.
   - Broker MQTT (Mosquitto ou container Docker) acessível na rede.

### Configuração e Execução

1. **Emulador de ECU (Arduino UNO):**
   - Compile e grave o firmware a partir de `firmware/uno_ecu_emulator/` (PlatformIO) ou `firmware/uno_ecu_emulator_ino/` (Arduino IDE).

2. **Coletor de Borda (ESP32-S3):**
   - Navegue até `firmware/esp32s3_collector/`.
   - Copie o arquivo de exemplo de configuração local:
     ```bash
     cp src/config_local.rs.example src/config_local.rs
     ```
   - Preencha o `src/config_local.rs` com suas credenciais de Wi-Fi e IP do Broker MQTT.
   - Compile e grave o firmware:
     ```bash
     source ~/export-esp.sh
     cargo +esp run --release
     ```

3. **Montagem e Operação em Bancada:**
   - Siga o guia de fiação em [`docs/pratica_bancada.md`](docs/pratica_bancada.md).
   - Envie comandos remotos de início de sessão via MQTT utilizando `tools/send_command.py`.

4. **Validação de Métricas:**
   - Valide os arquivos CSV gravados no SD Card com os scripts de análise:
     ```bash
     python tools/validate_csv.py dataset/S_0001.CSV
     python tools/compute_metrics.py dataset/S_0001.CSV dataset/S_0003.CSV dataset/S_0005.CSV
     ```

## Metodologia de Validação

O projeto é validado em ambiente Hardware-in-the-Loop (HIL) em bancada. O emulador Arduino UNO R3 injeta frames sintéticos baseados no perfil de condução selecionado e responde às solicitações OBD-II geradas ativamente a cada 100 ms pelo coletor ESP32-S3. Os datasets gerados no cartão SD e publicados via MQTT são inspecionados para validação formal de taxas de recebimento, jitter de resposta, integridade estrutural, uso de memória SRAM e auto-recuperação sob falhas (queda de Wi-Fi e Bus-Off elétrico no CAN).

## Licença

Este projeto está licenciado sob a Licença MIT - consulte o arquivo [LICENSE](LICENSE) para mais detalhes.
