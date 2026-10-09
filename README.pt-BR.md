# Edge Telemetry Layer

[![CI](https://github.com/amauricunha/edge-telemetry-layer/actions/workflows/ci.yml/badge.svg)](https://github.com/amauricunha/edge-telemetry-layer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Data License: CC BY 4.0](https://img.shields.io/badge/Data%20License-CC%20BY%204.0-lightgrey.svg)](LICENSE-DATA)
[![Firmware: Rust no_std](https://img.shields.io/badge/Firmware-Rust_no__std_(Embassy)-orange.svg)](firmware/esp32s3_collector)
[![Emulator: C++ PlatformIO](https://img.shields.io/badge/Emulator-C%2B%2B_PlatformIO-blue.svg)](firmware/uno_ecu_emulator)
[![MBSE: SAE AS5506B AADL](https://img.shields.io/badge/MBSE-SAE_AS5506B_AADL-navy.svg)](docs/MBSE)
[![Formal Verification: NASA FRET](https://img.shields.io/badge/Requirements-NASA_FRET-red.svg)](docs/MBSE/Entregas/Entrega%201)

> **Idioma / Language:** Português | [🌐 English Version](README.md)

---

## Resumo Executivo

O **Edge Telemetry Layer** é uma plataforma embarcada aberta e de baixo custo para telemetria de borda e diagnóstico automotivo via barramento CAN (500 kbps) e OBD-II (ISO 15765-4) em ambiente Hardware-in-the-Loop (HIL). Desenvolvido no âmbito da **Universidade Federal de Santa Catarina (UFSC)**, o projeto integra:

1. **Coletor de Borda em Rust Seguro (`no_std`):** Executa em um microcontrolador **ESP32-S3** sob o runtime assíncrono **Embassy**, realizando *polling* cíclico de PIDs OBD-II, serialização determinística em CSV sem alocação dinâmica em runtime, persistência local em cartão SD (FAT32) com *fallback* automático e publicação de telemetria via MQTT sobre Wi-Fi.
2. **Emulador de ECU para Bancada HIL (C++):** Executa em um **Arduino UNO R3** acoplado ao controlador CAN **MCP2515**, sintetizando dinâmicas de condução veicular (perfis Econômico, Normal e Esportivo) e respondendo a requisições de diagnóstico OBD-II com jitter inferior a 3 ms.
3. **Engenharia de Sistemas Baseada em Modelos (MBSE):** Cobre o ciclo completo formal com a ferramenta de elicitação da NASA (**NASA FRET** / provador SMT Kind 2), modelagem arquitetural em **AADL (SAE AS5506B)** no **OSATE**, análises estáticas de escalonabilidade e latência ponta a ponta, e avaliação de variabilidade e evolução arquitetural (**CAvA**) para suporte a **TinyML** na borda.

---

## Topologia da Arquitetura

```text
  +--------------------------------------------------------------------------------+
  |                   BANCADA HARDWARE-IN-THE-LOOP (HIL)                           |
  |                                                                                |
  |   +-----------------------+                    +---------------------------+   |
  |   |    Arduino UNO R3     |                    |         ESP32-S3          |   |
  |   |   (Emulador de ECU)   |                    |   (Coletor Telemetria)    |   |
  |   |                       |                    |                           |   |
  |   |   - Perfis de Condução|   Barramento CAN   |   - Driver TWAI CAN       |   |
  |   |   - Respondedor OBD-II|<==================>|   - Poller Ativo OBD-II   |   |
  |   |   - MCP2515 (SPI)     | 500 kbps (120 Ohm) |   - Serializador CSV      |   |
  |   +-----------------------+                    |   - Cartão MicroSD (FAT32)|   |
  |                                                |   - RTOS Assíncrono       |   |
  |                                                +-------------+-------------+   |
  +--------------------------------------------------------------|-----------------+
                                                                 | Wi-Fi (MQTT)
                                                                 v
                                                   +---------------------------+
                                                   |       STACK DEVOPS        |
                                                   |   (Docker Compose)        |
                                                   |                           |
                                                   |   - Telegraf (Consumidor) |
                                                   |   - InfluxDB v2 (TSDB)    |
                                                   |   - Grafana (Dashboards)  |
                                                   +---------------------------+
```

---

## Pilares Metodológicos de MBSE

A engenharia do sistema estrutura-se em quatro entregáveis formais:

| Pilar | Domínio de Engenharia | Ferramenta Formal | Artefatos e Entregáveis |
|---|---|---|---|
| **Pilar 1** | **Formalização de Requisitos** | [NASA FRET](https://github.com/NASA-SW-VnV/fret) + Kind 2 / Z3 | 48 contratos temporais em FRETish decompostos em 9 componentes de software; 100% de realizabilidade formal comprovada em 0,41 s. Consulte [`docs/MBSE/Entregas/Entrega 1/`](docs/MBSE/Entregas/Entrega%201/). |
| **Pilar 2** | **Modelagem Arquitetural** | [OSATE](https://osate.org/) (SAE AS5506B AADL) | Plataforma de execução física (`ESP32S3_Processor`, barramentos CAN/SPI, periféricos) e processos/tarefas lógicas. Consulte [`docs/MBSE/Entregas/Entrega 2 e 3/`](docs/MBSE/Entregas/Entrega%202%20e%203/). |
| **Pilar 3** | **Análise Temporal de Tempo Real** | Plugins OSATE (*Schedule* & *Latency*) | Escalonabilidade estática preemptiva por prioridades ($U = 34{,}5\% \le 75{,}68\%$) e análise de latência ponta a ponta (conexões imediatas vs. atrasadas: redução de 62,1% no pior caso). |
| **Pilar 4** | **Variabilidade e Evolução (CAvA)** | [CAvA](https://doi.org/10.1007/s10270-016-0563-7) + TinyML | Análise de variabilidade substituindo pipeline de nuvem por inferência local TinyML (*Driver Coaching*); matriz de *trade-offs* de qualidade ISO/IEC 25010. Consulte [`docs/MBSE/Entregas/Entrega 4/`](docs/MBSE/Entregas/Entrega%204/). |

---

## Critérios de Aceitação Empíricos (Resultados de Bancada)

A validação foi conduzida sobre três sessões reproduzíveis de bancada (`dataset/S_0001.CSV`, `dataset/S_0003.CSV`, `dataset/S_0005.CSV`) através do utilitário `tools/compute_metrics.py`:

| Critério | Métrica Alvo | Limite Exigido | Valor Obtido em Bancada | Status |
|---|---|---|---|:---:|
| **[AC-01]** | Taxa de Recepção e Perda CAN | Perda $\le 1{,}0\%$ a 500 kbps | **0,00%** (0 perdas em mais de 30 mil frames) | **APROVADO** |
| **[AC-02]** | Latência de Polling OBD-II | Latência média $< 10{,}0$ ms | **2,28 ms** (desvio: 0,14 ms) | **APROVADO** |
| **[AC-03]** | Jitter de Resposta OBD-II | Jitter temporal $< 3{,}0$ ms | **0,18 ms** | **APROVADO** |
| **[AC-04]** | Throughput e Serialização CSV | Vazão $\ge 200$ pkt/s | **214 pkt/s** (100% de integridade estrutural) | **APROVADO** |
| **[AC-05]** | Consumo Estático de SRAM | Memória $< 200{,}0$ KB | **58,0 KB** de heap estático / zero vazamento | **APROVADO** |
| **[AC-06]** | Fallback Offline no MicroSD | Tempo de comutação $\le 5{,}0$ ms | **1,20 ms** sob queda de Wi-Fi ou MQTT | **APROVADO** |
| **[AC-07]** | Auto-recuperação de Bus-Off | Espera de 128 ms (ISO 11898) | **128,4 ms** no *handshake* de recuperação | **APROVADO** |
| **[AC-08]** | Gravação Contínua no SD | Volume $\ge 72.000$ amostras / 1h | **72.400+** amostras gravadas com sucesso | **APROVADO** |

---

## Estrutura do Repositório

```text
├── .github/
│   ├── workflows/ci.yml       # Integração contínua (validação de datasets, PlatformIO, Rust)
│   ├── dependabot.yml         # Automação de atualização de dependências (Cargo, Pip, Actions)
│   └── ISSUE_TEMPLATE/        # Modelos padronizados para relatos de bugs e novas funcionalidades
├── dataset/
│   ├── README.md              # Roteiro de reprodução de bancada e dicionário dos dados
│   ├── S_0001.CSV             # Dataset de bancada: Sessão Econômica
│   ├── S_0003.CSV             # Dataset de bancada: Sessão Normal
│   └── S_0005.CSV             # Dataset de bancada: Sessão Esportiva
├── DevOps/
│   ├── docker-compose.yml     # Orquestração do InfluxDB v2, Telegraf e Grafana
│   └── telegraf.conf          # Configuração de ingestão de telemetria MQTT
├── docs/
│   ├── spec_arquitetura.md    # Especificação detalhada da arquitetura
│   ├── DBC_minimal.dbc        # Definição de mensagens CAN no formato DBC
│   ├── OBD-PIDS.md            # Tabela de PIDs OBD-II suportados
│   ├── PAYLOAD_SCHEMA.md      # Esquemas de carga binária MQTT e persistência CSV
│   ├── DECISIONS.md           # Registros de Decisão Arquitetural (ADRs)
│   ├── variaveis.md           # Catálogo de variáveis e frequências de amostragem
│   ├── pratica_bancada.md     # Pinout físico e fiação da bancada HIL
│   ├── relatorio_gate_f1.md   # Relatório consolidado de critérios de aceitação
│   └── MBSE/                  # Modelos e relatórios formais de engenharia baseada em modelos
│       ├── Entregas/          # Pilar 1 (FRET), Pilares 2 e 3 (OSATE), Pilar 4 (CAvA)
│       └── overleaf/          # Pacote LaTeX do relatório de publicação (artigo consolidado)
├── firmware/
│   ├── esp32s3_collector/     # Firmware em Rust (no_std, Embassy) para o ESP32-S3
│   ├── uno_ecu_emulator/      # Firmware C++ (PlatformIO) do emulador de ECU para Arduino UNO R3
│   └── uno_ecu_emulator_ino/  # Espelho compatível com a IDE padrão do Arduino
├── tools/
│   ├── .env_exemplo           # Modelo de variáveis de ambiente para scripts
│   ├── requirements.txt       # Dependências Python dos utilitários
│   ├── validate_csv.py        # Validador de conformidade e integridade dos arquivos CSV
│   ├── compute_metrics.py     # Cálculo automatizado dos critérios AC-01 a AC-08
│   ├── plot_session.py        # Utilitário de plotagem e gráficos de telemetria
│   ├── send_command.py        # Despachante de comandos remotos de sessão via MQTT
│   └── analyze_influxdb.py    # Extrator e analisador de dados do InfluxDB
├── .gitattributes             # Normalização de finais de linha e regras do GitHub Linguist
├── .gitignore                 # Exclusões de compilação, credenciais e arquivos de IDE
├── .zenodo.json               # Metadados para arquivamento com DOI no Zenodo
├── CITATION.cff               # Metadados acadêmicos de citação
├── CONTRIBUTING.md            # Guia de contribuição e convenções de código
├── LICENSE                    # Licença MIT (Firmware, software e ferramentas)
├── LICENSE-DATA               # Licença CC BY 4.0 (Datasets e documentação MBSE)
├── README.md                  # Documentação principal em Inglês
├── README.pt-BR.md            # Este documento
└── SECURITY.md                # Política de reporte seguro de vulnerabilidades
```

---

## Guia de Inicialização Rápida

### 1. Emulador de ECU (Arduino UNO R3)

Conecte o Arduino UNO R3 ao transceptor CAN MCP2515 via interface SPI. Compile e faça o upload através do **PlatformIO**:

```bash
cd firmware/uno_ecu_emulator
pio run -t upload
```

*(Ou abra `firmware/uno_ecu_emulator_ino/uno_ecu_emulator_ino.ino` na IDE do Arduino).*

### 2. Coletor de Borda (ESP32-S3)

Pré-requisito: Toolchain Rust para ESP32 (`espup`).

```bash
cd firmware/esp32s3_collector

# Copie o modelo de configuração e preencha suas credenciais de Wi-Fi e IP do Broker:
cp src/config_local.rs.example src/config_local.rs

# Compile e grave utilizando cargo-espflash:
source ~/export-esp.sh
cargo +esp run --release
```

### 3. Ingestão em Nuvem e Dashboards (Docker Compose)

Crie o arquivo local `.env` na pasta `DevOps` com senhas seguras e inicie a stack:

```bash
cd DevOps

cat << 'EOF' > .env
INFLUX_USER=telemetry
INFLUX_PASSWORD=SuaSenhaSegura123!
INFLUX_ORG=telemetry_org
INFLUX_BUCKET=telemetry
INFLUX_TOKEN=SeuTokenGerado123!
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=SuaSenhaGrafana123!
MQTT_BROKER_HOST=host.docker.internal
EOF

docker compose up -d
```

- **Grafana:** `http://localhost:3000` (Usuário: `admin`)
- **InfluxDB v2:** `http://localhost:8897`

### 4. Validação de Datasets e Métricas

Execute a verificação automática dos dados coletados na bancada:

```bash
# Instale as dependências:
pip install -r tools/requirements.txt

# Valide a formatação e integridade do CSV:
python tools/validate_csv.py dataset/S_0001.CSV

# Calcule os critérios de aceitação AC-01 a AC-08:
python tools/compute_metrics.py dataset/S_0001.CSV dataset/S_0003.CSV dataset/S_0005.CSV

# Visualize graficamente a sessão:
python tools/plot_session.py dataset/S_0001.CSV
```

---

## Citação Acadêmica

Se você utilizar o firmware, datasets ou modelos de arquitetura deste projeto em pesquisas ou publicações, cite através do formato:

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

Ou utilize o botão **"Cite this repository"** na barra lateral do GitHub (integrado via [`CITATION.cff`](CITATION.cff)).

---

## Contexto Acadêmico e Agradecimentos

Este projeto foi desenvolvido no âmbito do **Departamento de Automação e Sistemas (DAS)** da **Universidade Federal de Santa Catarina (UFSC)**, sob orientação acadêmica do **Prof. Dr. Leandro Buss Becker**.

Agradecimentos especiais pelas metodologias formalizadas nas seguintes publicações de referência:
- Sheridan, O., Becker, L. B., Farrell, M., Luckcuck, M., & Monahan, R. (2025). *Sharper Specs for Smarter Drones: Formalising Requirements with FRET*. RefSQ 2025, Lecture Notes in Computer Science, Springer.
- Sales, I., & Becker, L. B. (2018). *CAvA: Component and Architecture Variability and Evolution Approach for Critical Embedded Systems*. Software & Systems Modeling.

---

## Isenção de Responsabilidade e Segurança

> [!WARNING]
> Esta arquitetura de software e hardware foi concebida para fins de pesquisa científica, experimentação de bancada e simulação Hardware-in-the-Loop (HIL). As requisições OBD-II implementadas restringem-se estritamente ao modo de leitura e diagnóstico padronizado (Serviço 01). A conexão de equipamentos em veículos reais em movimento envolve riscos físicos e elétricos intrínsecos, sendo de total responsabilidade do usuário.

---

## Licença

- **Código-fonte, Firmware e Utilitários:** [Licença MIT](LICENSE) &copy; 2026 Amauri Cunha.
- **Datasets e Especificações MBSE:** [Creative Commons Atribuição 4.0 Internacional (CC BY 4.0)](LICENSE-DATA).
