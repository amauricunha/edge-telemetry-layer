# Guia de Cadastramento e Sentenças FRETish (NASA FRET)
## Projetos FRET: EdgeTelemetry_UNO e EdgeTelemetry_ESP32
Este documento estabelece **100% de paridade e rastreabilidade com a Especificação de Requisitos de Sistema (`docs/mestrado/srs.md`)**, contendo todos os 30 requisitos do sistema (REQ-SYS-01 a REQ-SYS-30), os critérios de aceitação AC-01 a AC-08 e os fluxos das quatro threads principais da arquitetura.

Cada requisito está formalizado na gramática **FRETish (em inglês normatizado)** aceita pelo parser ANTLR 4 da ferramenta **NASA FRET**, com nomes de componentes higienizados (sem `::`) e com o mapeamento tipado de variáveis.

---

## Fundamentação Arquitetural: Por que Modelamos Componentes de Software (SW-C) em uma Mesma CPU Física?

Uma dúvida recorrente em bancas de engenharia é: *"Se o coletor possui uma única CPU física (o microcontrolador ESP32-S3), por que o modelo MBSE o divide em 8 componentes formais no FRET?"*

A resposta reside na distinção canônica entre **Arquitetura de Hardware (Nível Físico)** e **Arquitetura de Software (Nível Lógico)**:

1. **O Conceito de "Componente" na Engenharia de Sistemas Críticos (NASA e AUTOSAR):**
   - Em normas aeroespaciais e automotivas de missão crítica (NASA, ARP4754A, AUTOSAR Classic e ISO 26262), o termo **Componente** quase nunca se refere ao chip de silício.
   - Refere-se a um **Componente de Software (Software Component — SW-C)**: uma unidade lógica de execução concorrente governada pelo RTOS (no firmware Rust, o runtime assíncrono `Embassy`), com interfaces e prazos de pior caso (WCET) rigorosamente delimitados.
2. **O Padrão Adotado pela Própria NASA (Caso de Estudo `LMCPS`):**
   - No maior caso de estudo oficial da NASA (`LMCPS` — *Lockheed Martin Cyber-Physical Systems*, disponível na pasta `docs/MBSE/FRET_docs/LMCPS`), composto por 97 requisitos formais, a NASA dividiu o sistema em **13 componentes funcionais independentes**:
     `Autopilot`, `RollAutopilot`, `Euler`, `Regulator`, `Tustin_Integrator`, `FSM_Sensor`, etc.
   - **Todos esses 13 módulos executam na mesma CPU física** do computador de controle de voo (*Flight Control Computer*). A NASA adotou essa decomposição porque modelar tarefas concorrentes como uma única caixa-preta opaca é metodologicamente inadequado e gera modelos intratáveis.
3. **Validação Rigorosa dos Requisitos e Interfaces Modulares:**
   - Ao modelar cada subsistema como um componente formal, delimitamos com rigor as variáveis de entrada (`Input`), saída (`Output`) e internas (`Internal`) de cada SW-C:
     - O módulo **Produtor** (`esp32_twai` - MCAL) garante colocar o registro decodificado no canal assíncrono da RTE em até $1\text{ ms}$ (`rte_channel_pushed = true`).
     - O módulo **Consumidor** (`esp32_logger` - Aplicação) assume como premissa de entrada a chegada desse dado e garante formatá-lo em linha CSV e colocá-lo no buffer SRAM em até $1\text{ ms}$.
     - O Kind 2 comprova matematicamente que as regras de cada componente são internamente realizáveis e livres de contradições lógicas.
   - A garantia de que a integração entre produtor e consumidor na RTE não sofre contenção ou estouro de deadline na CPU compartilhada é então formalmente comprovada pelas análises de escalonabilidade e latência de fluxo do OSATE (Pilar 3).
   - Se unificássemos tudo em uma "caixa-preta de CPU", as filas da RTE (`TELEMETRY_CHANNEL`, `CAN_CMD_CHANNEL`) e os buffers em SRAM virariam variáveis internas invisíveis, e o FRET não conseguiria verificar se os prazos de pior caso (WCET) de cada camada de software são respeitados.
4. **Prevenção de Falsos Conflitos de Atribuição no Solver SMT:**
   - Evita que o solver SMT aponte falsos conflitos de concorrência causados pela tentativa de modelar múltiplas tarefas assíncronas concorrentes em uma única equação de transição de estados.

---

## Regras Essenciais de Sintaxe FRETish (ANTLR 4)

> **Unidades de tempo:** O parser exige os tokens lexicais completos em maiúsculas:
> `MILLISECOND`, `MICROSECOND`, `SECOND`, `MINUTE`, `HOUR`, `TICK`.
> Abreviações como `ms`, `us`, `s` causam erro `mismatched input 'ms' expecting {...}`.

> **Predicados Booleanos:** Para variáveis `Boolean` no `satisfy`, **nunca** escreva `= TRUE`, `= FALSE` ou `= "TRUE"`.
> O parser trata `TRUE` como uma variável desconhecida adicional, não como literal.
> **Correto:** `satisfy sd_fallback_active` | **Errado:** `satisfy sd_fallback_active = TRUE`

> **Nomes de componentes:** Proibido `::`. Use `_` como separador: `mcal_twai`, não `mcal::twai`.

> **Parent Requirement ID:** O campo espera o **Req ID exato de um requisito já existente no projeto**, com **underscores** (não hífens). `REQ_SYS_01` é válido; `REQ-SYS-01` causa erro de parse. O campo não afeta a verificação de realizabilidade — serve apenas para rastreabilidade hierárquica no relatório.

---

## Como cadastrar cada requisito no FRET:

> **Atenção:** O FRET **não possui campo "Título"**. Os campos reais do modal são:
> `Requirement ID`, `Parent Requirement ID`, `Project`, `Rationale`, `Comments` e `Requirement Description`.
> O título descritivo vai dentro do campo **Rationale**, junto com o texto em Português.

1. Abra o FRET e acesse/crie os projetos **`EdgeTelemetry_UNO`** e **`EdgeTelemetry_ESP32`**.
2. Clique em **`CREATE`** no menu superior.
3. No modal, preencha:
   - **Requirement ID:** Cole o valor de `ID` (ex: `REQ_EMU_005`) — underscores, sem hífens.
   - **Parent Requirement ID:** Deixe vazio (a rastreabilidade com o SRS é feita pela tag `[REQ-SYS-XX]` no Rationale).
   - **Project:** Selecione `EdgeTelemetry_UNO` (para requisitos do emulador) ou `EdgeTelemetry_ESP32` (para o coletor).
   - **Rationale:** Cole o título descritivo + texto em Português no seguinte formato:
     ```
     [REQ-SYS-08 / AC-02] Latência Média de Resposta OBD-II

     O emulador deve responder com latência média estritamente menor que 10 ms (Critério de Aceitação AC-02).
     ```
4. No campo **Requirement Description**, cole **apenas** o texto da caixa `FRETish Text`:
   ```
   in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy mean_obd_latency_ms < 10.0
   ```
   - O editor colorirá automaticamente: **vermelho** (Scope), **laranja** (Condition), **verde** (Component), **azul** (Timing), **roxo** (Response).
   - Texto sem coloração = erro de sintaxe — consulte as Regras acima.
5. Clique em **`CREATE`**.
6. Abra a aba **`Variable Mapping`** e configure o **Role** (`Input`, `Output` ou `Internal`) e o **Type** (`Boolean`, `Integer`, `Double`) conforme detalhado em cada bloco.
7. Clique em **`Realizability`** para executar a prova formal (resultado esperado: **`Realizable: True`**).

---

## Sobre Variáveis de Valores Físicos Calculados (Perfis e Senoide)

O emulador Arduino calcula `speed_kmh`, `rpm`, `throttle_pct`, `load_pct`, `maf_g_s` a partir de uma tabela senoidal pré-computada em `PROGMEM`, modulada pelo `active_profile`.
1. **As equações trigonométricas contínuas não entram no FRET** porque os model checkers (NuSMV/JKind) operam sobre lógica proposicional e aritmética linear finita.
2. **As grandezas físicas e invariantes de segurança ENTRAM no FRET:** As variáveis (`speed_kmh`, `rpm`, `throttle_pct`, `load_pct`, `maf_g_s`) são formalizadas em [`REQ_EMU_003`] através de contratos de invariantes de faixa física válida (`>= 0`) e deadline de CPU (`within 1 MILLISECOND`).
3. O `active_profile` governa tanto a dinâmica física em [`REQ_EMU_003`] quanto a emissão dos frames DBC em [`REQ_EMU_004`] (`when active_profile >= 1`).

---

## Decomposição de Componentes Formais no NASA FRET (Arquitetura MBSE)

```
[SISTEMA: Edge Telemetry Layer] (Total: 48 Requisitos Formais no NASA FRET)
   │
   ├── [NÓ 1: Emulador de ECU veicular (Hardware-in-the-Loop)]
   │     └── Microcontrolador ATmega328P + Transceptor MCP2515 (C++ Bare-Metal)
   │           └── [COMPONENTE FRET: uno_ecu_emulator] (8 requisitos: REQ_EMU_001 a REQ_EMU_008)
   │                 └── Dinâmica Física de Motor (Senoide em PROGMEM), Frames DBC (0x100, 0x200, 0x300) e Resposta OBD-II
   │
   └── [NÓ 2: Coletor e Gateway de Borda (Edge Gateway)]
         └── Microcontrolador ESP32-S3 Dual-Core (Firmware Rust no_std Embassy Assíncrono)
               ├── [COMPONENTE FRET: esp32_twai] (6 requisitos: REQ_CAN_001 a REQ_CAN_006)
               │     └── [Camada MCAL] Driver TWAI/CAN, Timestamping e Escala DBC (AC-01: Perda <= 1.0%)
               ├── [COMPONENTE FRET: esp32_obd] (4 requisitos: REQ_OBD_001 a REQ_OBD_004)
               │     └── [Camada APP] Poller de Diagnóstico OBD-II ISO 15765-4 e Intercalação Cíclica
               ├── [COMPONENTE FRET: esp32_logger] (9 requisitos: REQ_LOG_001 a REQ_LOG_009)
               │     └── [Camada APP] Serialização Tabular CSV, Buffers SRAM Estáticos e Supervisão (AC-05: SRAM < 200 KB)
               ├── [COMPONENTE FRET: esp32_sd] (5 requisitos: REQ_SD_001 a REQ_SD_005)
               │     └── [Camada BSW] Sistema de Arquivos FAT32 via SPI2 e Gravação Robusta (AC-08: >= 72.000 amostras)
               ├── [COMPONENTE FRET: esp32_telemetry] (5 requisitos: REQ_COM_001 a REQ_COM_005)
               │     └── [Camada BSW] Conectividade Wi-Fi e Despacho Binário MQTT em Nuvem (AC-04: Throughput)
               ├── [COMPONENTE FRET: esp32_fsm] (2 requisitos: REQ_FSM_001 a REQ_FSM_002)
               │     └── [Camada APP] FSM de Conectividade, Fallback Offline Imediato (AC-06: Comutação <= 5 ms) e Dreno FIFO
               ├── [COMPONENTE FRET: esp32_cmd] (2 requisitos: REQ_CMD_001 a REQ_CMD_002)
               │     └── [Camada BSW] Recepção de Comandos de Bancada e Retransmissão em Streaming (Replay)
               └── [COMPONENTE FRET: esp32_recovery] (7 requisitos: REQ_REC_001 a REQ_REC_007)
                     └── [Camada BSW] Recuperação Autônoma de Bus-Off (AC-07: Handshake Timer 128 ms via PAC (NASA Pattern)) e Watchdog de Hardware
```

---

## Resumo Executivo para Apresentação Acadêmica (Orientador / Banca)

> **Contexto de Engenharia e Pesquisa:**
> O projeto implementa uma arquitetura de telemetria automotiva de borda para ensaios em pista e bancada Hardware-in-the-Loop (HIL). O nó coletor é implementado em **Rust puro sem alocação dinâmica (`no_std`)** sobre o framework assíncrono **Embassy**, seguindo a separação em camadas inspirada no padrão automotivo **AUTOSAR** (MCAL, BSW, RTE e Aplicação). A especificação formal segue a metodologia **MBSE (Model-Based Systems Engineering)** da **NASA**, onde os requisitos foram modelados em lógica temporal linear (FRETish/LTL) na ferramenta **NASA FRET** e verificados matematicamente para **Realizabilidade Monolítica e Composicional** através dos provadores formais **Kind 2** e **Z3 SMT Solver**.

### 1. Tabela Síntese dos Requisitos por Componente e Camada de Software

| Componente Formal (FRET) | Camada AUTOSAR | Requisitos | Qtd | Deadlines & Invariantes de Tempo Real | Critérios de Aceitação (SRS) | Função Técnica Principal |
| :--- | :--- | :--- | :---: | :--- | :---: | :--- |
| **`uno_ecu_emulator`** | Emulador HIL | `REQ_EMU_001` a `008` | 8 | • Atualização física: $\le 1\text{ ms}$<br>• Emissão DBC: $\le 2\text{ ms}$<br>• Resposta OBD-II: $\le 10\text{ ms}$ | **AC-02** ($\text{latência} < 10\text{ ms}$)<br>**AC-03** ($\text{jitter} < 3\text{ ms}$) | Simula a ECU do motor via Arduino UNO (ATmega328P + MCP2515), emitindo frames DBC e respondendo a consultas OBD-II com baixa latência e jitter estrito. |
| **`esp32_twai`** | **MCAL** | `REQ_CAN_001` a `006` | 6 | • Boot TWAI: $\le 50\text{ ms}$<br>• Timestamp: $\le 2\text{ ms}$<br>• Parsing DBC: $\le 100\ \mu\text{s}$<br>• Inserção RTE: $\le 1\text{ ms}$ | **AC-01** ($\text{perda} \le 1.0\%$) | Driver do controlador CAN nativo (TWAI) do ESP32-S3 em modo assíncrono por interrupções, garantindo conversão de engenharia sem alocação dinâmica de memória. |
| **`esp32_obd`** | **APP** | `REQ_OBD_001` a `004` | 4 | • Ciclo de polling: $100\text{ ms}$ ($10\text{ Hz}$)<br>• Timeout OBD: $50\text{ ms}$<br>• Intercalação de comandos: $\le 10\text{ ms}$ | **AC-02** (Conformidade com ECU)<br>**AC-04** (Throughput) | Orquestra a interrogação cíclica ativa dos 6 PIDs padronizados (ISO 15765-4) e gerencia filas de comandos prioritários intercalados na transmissão. |
| **`esp32_logger`** | **APP** | `REQ_LOG_001` a `009` | 9 | • Formatação CSV: $\le 1\text{ ms}$<br>• Flush periódico: $\le 2\text{ s}$<br>• Heartbeat: $60\text{ s}$ | **AC-05** ($\text{SRAM} < 200.0\text{ KB}$)<br>**AC-04** ($\ge 200\text{ pacotes/s}$) | Converte amostras de telemetria em linhas tabulares CSV de 11 colunas utilizando buffer estático de 320 bytes, supervisionando o uso de SRAM sem memory leaks. |
| **`esp32_sd`** | **BSW** | `REQ_SD_001` a `005` | 5 | • Mount SPI/FAT32: Imediato ao boot<br>• Escrita por setor: $512\text{ bytes}$<br>• Yield cooperativo: $\le 20\text{ ms}$ | **AC-08** ($\ge 72.000\text{ amostras}$) | Driver e subsistema de armazenamento local contínuo em cartão MicroSD FAT32 via barramento SPI2, suportando gravação contínua por mais de 1 hora de ensaio ininterrupto. |
| **`esp32_telemetry`** | **BSW** | `REQ_COM_001` a `005` | 5 | • Handshake Wi-Fi/MQTT: Via Timer Abstraction (NASA)<br>• Despacho no ciclo: Imediato | **AC-04** (Throughput de telemetria)<br>**AC-06** (Fallback de rede) | Gateway de comunicação sem fio via Wi-Fi e protocolo MQTT 3.1.1, compactando lotes binários para publicação em nuvem com reconexão resiliente. |
| **`esp32_fsm`** | **APP** | `REQ_FSM_001` a `002` | 2 | • Comutação para SD: $\le 5\text{ ms}$<br>• Dreno FIFO na volta: $\le 10\text{ ms}$ | **AC-06** ($\text{comutação} \le 5\text{ ms}$) | Máquina de Estados Finita (FSM) de conectividade: detecta queda do enlace Wi-Fi e desvia instantaneamente 100% dos dados para o MicroSD, esvaziando a FIFO ao reconectar. |
| **`esp32_cmd`** | **BSW** | `REQ_CMD_001` a `002` | 2 | • Execução de comando: $\le 20\text{ ms}$<br>• Streaming replay: $\le 100\text{ ms}$ | Controle Operacional HIL | Camada de controle remoto de bancada: interpreta comandos recebidos via MQTT/CAN (STOP, PERFIL, RESET, REPLAY) e aciona streaming de reprodução histórica. |
| **`esp32_recovery`** | **BSW** | `REQ_REC_001` a `007` | 7 | • Detecção de Bus-Off: $\le 10\text{ ms}$<br>• Janela ISO 11898: $128\text{ ms}$<br>• Reativação PAC: $\le 1\text{ ms}$<br>• Rearme Watchdog: $\le 2\text{ s}$ | **AC-07** (Recuperação $\le 128\text{ ms}$ sem reboot) | Subsistema de resiliência e alta confiabilidade: recupera falhas elétricas do barramento CAN em nível de registradores de silício (PAC) e reanima o Watchdog de hardware. |

---

### 2. Mapeamento Direto dos Critérios de Aceitação da Dissertação (AC-01 a AC-08)

Os critérios de aceitação formalizam as métricas quantitativas de desempenho e confiabilidade exigidas no trabalho de mestrado:

1. **[AC-01] Perda Máxima de Frames CAN $\le 1.0\%$:**
   * *Requisito Formal:* `REQ_CAN_005` no componente `esp32_twai`.
   * *Formulação FRETish:* `in active_session when can_bus_healthy the esp32_twai shall always satisfy frame_loss_percentage <= 1.0`
   * *Validação:* Comprovado via contrato de invariante contínuo com medição sobre barramento CAN a 500 kbps sob injeção de carga.

2. **[AC-02] Latência Média de Resposta OBD-II $< 10.0\text{ ms}$:**
   * *Requisito Formal:* `REQ_EMU_007` no componente `uno_ecu_emulator`.
   * *Formulação FRETish:* `in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy mean_obd_latency_ms < 10.0`
   * *Validação:* Emulador ATmega328P responde às consultas OBD-II funcionais (`0x7DF` $\to$ `0x7E8`) com tempo de resposta em microssegundos.

3. **[AC-03] Estabilidade Temporal e Jitter de Resposta OBD-II $< 3.0\text{ ms}$:**
   * *Requisito Formal:* `REQ_EMU_008` no componente `uno_ecu_emulator`.
   * *Formulação FRETish:* `in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy latency_jitter_std_ms < 3.0`
   * *Validação:* Desvio padrão da latência mantido sob rigoroso determinismo em tempo real com interrupções por hardware (Timer1/Timer2).

4. **[AC-04] Throughput Sustentado de Telemetria $\ge 200\text{ pacotes/s}$:**
   * *Requisitos Formais:* `REQ_LOG_003` no `esp32_logger` e `REQ_COM_004` no `esp32_telemetry`.
   * *Formulação FRETish:* Despacho contínuo a cada ciclo de 50 ms sem saturação de canais assíncronos.

5. **[AC-05] Consumo Estático de Memória SRAM $< 200.0\text{ KB}$:**
   * *Requisito Formal:* `REQ_LOG_008` no componente `esp32_logger`.
   * *Formulação FRETish:* `in active_session when memory_supervision_active the esp32_logger shall always satisfy sram_usage_kb < 200.0`
   * *Validação:* Firmware implementado em Rust `no_std` com alocação 100% estática em tempo de compilação (Zero Heap Fragmentation).

6. **[AC-06] Tempo de Comutação para Fallback Offline $\le 5\text{ ms}$:**
   * *Requisito Formal:* `REQ_FSM_001` no componente `esp32_fsm`.
   * *Formulação FRETish:* `in active_session upon wifi_disconnected the esp32_fsm shall within 5 MILLISECOND satisfy sd_fallback_active`
   * *Validação:* Na queda de sinal Wi-Fi, o pipeline assíncrono redireciona os dados para o MicroSD em até 5 ms sem perda de telemetria.

7. **[AC-07] Recuperação Autônoma de Bus-Off $\le 128\text{ ms}$ sem Reinicialização:**
   * *Requisitos Formais:* `REQ_REC_002` a `REQ_REC_005` no componente `esp32_recovery`.
   * *Formulação FRETish:* Cumprimento da janela normativa ISO 11898 de 128 ms e reativação via registradores físicos (PAC) sem destruir as tarefas do ESP32-S3.

8. **[AC-08] Gravação Contínua em MicroSD $\ge 72.000\text{ Amostras}$ (Ensaio de 1 Hora):**
   * *Requisito Formal:* `REQ_SD_005` no componente `esp32_sd`.
   * *Formulação FRETish:* `in active_session when dataset_recording the esp32_sd shall always satisfy total_dataset_samples >= 72000`
   * *Validação:* Persistência sem falhas de integridade FAT32 por mais de 60 minutos de ensaio ininterrupto a 20 Hz.

---

# Subsistema 1: Emulação de ECU Automotiva (Arduino UNO R3)

### REQ_EMU_001 — Inicialização do MCP2515 e Perfil Padrão [REQ-SYS-01] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_001`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & mcp2515_hardware_up the uno_ecu_emulator shall within 10 MILLISECOND satisfy can_bus_operational & active_profile = 2
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean) — reinicialização ou energização da placa ATmega328P
  - `mcp2515_hardware_up`: **Input** (Boolean) — módulo MCP2515 e transceptor energizados e respondendo via SPI
  - `can_bus_operational`: **Output** (Boolean) — barramento inicializado a 500 kbps (CAN_OK), modo MCP_NORMAL ativo e LED de status D4 aceso
  - `active_profile`: **Output** (Integer) — Perfil ativo de física (1=Eco, **2=Normal**, 3=Sport)
- **Rationale (Português):** Durante o setup(), ao detectar o hardware do MCP2515 via SPI, o emulador deve configurar o modo normal a 500 kbps, sinalizar o LED de status operacional e definir o perfil ativo como **2 (Normal)** antes de habilitar os timers de interrupção. Confirmado no firmware: `volatile uint8_t perfil_atual = 2`.

---

### REQ_EMU_002 — Comutação de Perfil de Condução Simulada [REQ-SYS-01] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_002`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in active_session upon profile_cmd_0x010_received the uno_ecu_emulator shall within 10 MILLISECOND satisfy active_profile = commanded_profile
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `profile_cmd_0x010_received`: **Input** (Boolean)
  - `commanded_profile`: **Input** (Integer) — Byte 0 do comando CAN (1=Eco, 2=Normal, 3=Sport)
  - `active_profile`: **Output** (Integer) — Perfil ativo de física (1=Eco, 2=Normal, 3=Sport)
- **Rationale (Português):** Ao receber o comando CAN 0x010, o emulador deve atualizar o perfil de simulação ativo (`active_profile = commanded_profile`, onde 1=Econômico, 2=Normal, 3=Esportivo) no próximo ciclo de física do Timer1 (50 ms).

---

### REQ_EMU_003 — Atualização do Modelo Físico da ECU [REQ-SYS-01] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_003`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in active_session when active_profile >= 1 upon timer1_50ms_tick the uno_ecu_emulator shall within 1 MILLISECOND satisfy physics_model_updated & speed_kmh >= 0.0 & rpm >= 0.0 & throttle_pct >= 0.0 & load_pct >= 0.0 & maf_g_s >= 0.0 & coolant_temp_c >= -40.0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `active_profile`: **Input** (Integer) — Perfil de condução ativo (1=Eco, 2=Normal, 3=Sport)
  - `timer1_50ms_tick`: **Input** (Boolean) — Interrupção periódica do Timer1 a cada 50 ms
  - `physics_model_updated`: **Output** (Boolean) — Flag de conclusão do cálculo
  - `speed_kmh`: **Output** (Double) — Velocidade simulada (km/h)
  - `rpm`: **Output** (Integer) — Rotação do motor (RPM)
  - `throttle_pct`: **Output** (Double) — Posição do acelerador (%)
  - `load_pct`: **Output** (Double) — Carga calculada do motor (%)
  - `maf_g_s`: **Output** (Double) — Fluxo de ar MAF (g/s)
  - `coolant_temp_c`: **Output** (Double) — Temperatura do líquido de arrefecimento (°C, nominal 87°C)
- **Rationale (Português):** A cada ciclo de 50 ms do Timer1 sob perfil ativo válido (`active_profile >= 1`), a CPU do emulador deve calcular as 6 grandezas físicas simuladas do motor (`speed_kmh`, `rpm`, `throttle_pct`, `load_pct`, `maf_g_s`, `coolant_temp_c`) a partir da tabela senoidal modulada pelo perfil em menos de 1 ms de tempo de execução da ISR, assegurando invariantes válidos.

---

### REQ_EMU_004 — Emissão Cíclica de Grandezas do Motor DBC [REQ-SYS-01] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_004`
- **Parent Requirement ID:** REQ_SYS_01
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in active_session when active_profile >= 1 & can_bus_operational upon physics_model_updated the uno_ecu_emulator shall within 2 MILLISECOND satisfy dbc_frames_emitted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `active_profile`: **Input** (Integer) — Perfil ativo selecionado
  - `can_bus_operational`: **Input** (Boolean) — Barramento CAN operacional e inicializado
  - `physics_model_updated`: **Input** (Boolean) — Gerado após a conclusão do cálculo físico em REQ_EMU_003
  - `dbc_frames_emitted`: **Output** (Boolean)
- **Rationale (Português):** Após a conclusão do cálculo do modelo de física (`physics_model_updated`), com o barramento CAN operacional, o emulador deve codificar e transmitir os frames CAN periódicos conforme a DBC (0x200 a cada 50 ms, 0x100 a cada 100 ms e 0x300 a cada 1000 ms) via SPI/MCP2515 em até 2 ms.

---

### REQ_EMU_005 — Processamento de Interrupção de Alta Frequência [REQ-SYS-02] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_005`
- **Parent Requirement ID:** REQ_SYS_02
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  when can_bus_operational upon timer2_1ms_tick the uno_ecu_emulator shall within 1 MILLISECOND satisfy mcp2515_rx_polled
  ```
- **Variable Mapping:**
  - `can_bus_operational`: **Input** (Boolean) — Barramento CAN operacional
  - `timer2_1ms_tick`: **Input** (Boolean)
  - `mcp2515_rx_polled`: **Output** (Boolean)
- **Rationale (Português):** Estando o barramento operacional, o Timer2 a cada 1 ms deve verificar a chegada de mensagens no MCP2515 sem bloquear a CPU por mais de 50 µs.

---

### REQ_EMU_006 — Codificação e Emissão de Resposta OBD-II [REQ-SYS-07] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_006`
- **Parent Requirement ID:** REQ_SYS_07
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in active_session upon obd_request_0x7df_received the uno_ecu_emulator shall within 10 MILLISECOND satisfy obd_response_0x7e8_sent
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_request_0x7df_received`: **Input** (Boolean)
  - `obd_response_0x7e8_sent`: **Output** (Boolean)
- **Rationale (Português):** Ao receber requisição Modo 01 em 0x7DF, o emulador deve codificar o PID e transmitir o frame de resposta 0x7E8.

---

### REQ_EMU_007 — Latência Média de Resposta OBD-II [REQ-SYS-08 / AC-02] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_007`
- **Parent Requirement ID:** REQ_SYS_08
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy mean_obd_latency_ms < 10.0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_benchmark_running`: **Input** (Boolean)
  - `mean_obd_latency_ms`: **Output** (Double)
- **Rationale (Português):** O emulador deve responder com latência média estritamente menor que 10 ms (Critério de Aceitação AC-02).

---

### REQ_EMU_008 — Estabilidade Temporal e Jitter de Resposta [REQ-SYS-09 / AC-03] [Subsistema 1] [Emulador ECU]
- **ID:** `REQ_EMU_008`
- **Parent Requirement ID:** REQ_SYS_09
- **Component:** uno_ecu_emulator
- **FRETish Text:**
  ```text
  in active_session when obd_benchmark_running the uno_ecu_emulator shall always satisfy latency_jitter_std_ms < 3.0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_benchmark_running`: **Input** (Boolean)
  - `latency_jitter_std_ms`: **Output** (Double)
- **Rationale (Português):** O desvio padrão da latência de resposta OBD-II deve permanecer inferior a 3 ms durante os ensaios (Critério AC-03).

---

# Subsistema 2: Aquisição e Recepção Passiva CAN (ESP32-S3 TWAI)
### REQ_CAN_001 — Inicialização e Sincronismo do TWAI [REQ-SYS-03] [Subsistema 2] [Camada MCAL]
- **ID:** `REQ_CAN_001`
- **Parent Requirement ID:** `REQ_SYS_03`
- **Component:** esp32_twai
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & mcal_twai_up the esp32_twai shall within 10 MILLISECOND satisfy twai_async_enabled
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `mcal_twai_up`: **Input** (Boolean)
  - `twai_async_enabled`: **Output** (Boolean)
- **Rationale (Português):** Durante o boot, se o gatilho de inicialização e o periférico TWAI estiverem ativos, o sistema integrado deve confirmar o modo assíncrono habilitado em até 50 ms.

---
### REQ_CAN_002 — Captura e Marcação Temporal de Frames [REQ-SYS-03] [Subsistema 2] [Camada MCAL]
- **ID:** `REQ_CAN_002`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** esp32_twai
- **FRETish Text:**
  ```text
  in active_session upon can_frame_arrived the esp32_twai shall within 2 MILLISECOND satisfy frame_timestamp_captured
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_frame_arrived`: **Input** (Boolean)
  - `frame_timestamp_captured`: **Output** (Boolean)
- **Rationale (Português):** Ao receber um frame físico no TWAI, registrar imediatamente o carimbo temporal do sistema em microssegundos/milissegundos.

---
### REQ_CAN_003 — Conversão de Grandezas Físicas DBC [REQ-SYS-03] [Subsistema 2] [Camada MCAL]
- **ID:** `REQ_CAN_003`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** esp32_twai
- **FRETish Text:**
  ```text
  upon raw_can_frame_ready the esp32_twai shall within 1 MILLISECOND satisfy engineering_values_scaled
  ```
- **Variable Mapping:**
  - `raw_can_frame_ready`: **Input** (Boolean)
  - `engineering_values_scaled`: **Output** (Boolean)
- **Rationale (Português):** O decodificador deve aplicar as equações de escala e offset da DBC nos bytes brutos sem alocação dinâmica em menos de 100 µs.

---
### REQ_CAN_004 — Inserção no Canal Assíncrono da RTE [REQ-SYS-03] [Subsistema 2] [Camada MCAL]
- **ID:** `REQ_CAN_004`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** esp32_twai
- **FRETish Text:**
  ```text
  in active_session upon telemetry_frame_parsed the esp32_twai shall within 1 MILLISECOND satisfy rte_channel_pushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `telemetry_frame_parsed`: **Input** (Boolean)
  - `rte_channel_pushed`: **Output** (Boolean)
- **Rationale (Português):** Ao concluir o parse de um frame de telemetria, postar a estrutura no canal bounded da RTE em até 1 ms.

---
### REQ_CAN_005 — Cumprimento da Taxa de Recepção [REQ-SYS-04 / AC-01] [Subsistema 2] [Camada MCAL]
- **ID:** `REQ_CAN_005`
- **Parent Requirement ID:** REQ_SYS_04
- **Component:** esp32_twai
- **FRETish Text:**
  ```text
  in active_session when can_bus_healthy the esp32_twai shall always satisfy frame_loss_percentage <= 1.0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_bus_healthy`: **Input** (Boolean)
  - `frame_loss_percentage`: **Output** (Double)
- **Rationale (Português):** A taxa de perda de quadros deve ser mantida estritamente abaixo de 1%, assegurando recepção de no mínimo 99% (Critério AC-01).

---
### REQ_CAN_006 — Tratamento Não-Bloqueante de Saturação da RTE [REQ-SYS-03] [Subsistema 2] [Camada MCAL]
- **ID:** `REQ_CAN_006`
- **Parent Requirement ID:** REQ_SYS_03
- **Component:** esp32_twai
- **FRETish Text:**
  ```text
  in active_session upon rte_channel_overflow the esp32_twai shall immediately satisfy overflow_logged_and_dropped
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `rte_channel_overflow`: **Input** (Boolean)
  - `overflow_logged_and_dropped`: **Output** (Boolean)
- **Rationale (Português):** Caso o canal da RTE esteja cheio (32 amostras), descartar o pacote excedente e registrar advertência sem travar a recepção.

---

# Subsistema 3: Diagnóstico Ativo OBD-II ISO 15765-4
### REQ_OBD_001 — Polling Cíclico de Diagnóstico a 10 Hz [REQ-SYS-05] [Subsistema 3] [Camada APP]
- **ID:** `REQ_OBD_001`
- **Parent Requirement ID:** REQ_SYS_05
- **Component:** esp32_obd
- **FRETish Text:**
  ```text
  in active_session upon obd_timer_100ms_expired the esp32_obd shall immediately satisfy obd_request_0x7df_transmitted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_timer_100ms_expired`: **Input** (Boolean)
  - `obd_request_0x7df_transmitted`: **Output** (Boolean)
- **Rationale (Português):** A cada 100 ms, emitir solicitação funcional Modo 01 em 0x7DF para o próximo PID programado.

---
### REQ_OBD_002 — Escalonamento Circular Round-Robin dos PIDs [REQ-SYS-06] [Subsistema 3] [Camada APP]
- **ID:** `REQ_OBD_002`
- **Parent Requirement ID:** REQ_SYS_06
- **Component:** esp32_obd
- **FRETish Text:**
  ```text
  in active_session upon obd_tx_cycle_completed the esp32_obd shall within 1 MILLISECOND satisfy pid_index_incremented
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_tx_cycle_completed`: **Input** (Boolean)
  - `pid_index_incremented`: **Output** (Boolean)
- **Rationale (Português):** Alternar sequencialmente entre os 6 PIDs suportados, completando um ciclo a cada 600 ms.

---
### REQ_OBD_003 — Tratamento de Timeout de Diagnóstico [REQ-SYS-10] [Subsistema 3] [Camada APP]
- **ID:** `REQ_OBD_003`
- **Parent Requirement ID:** REQ_SYS_10
- **Component:** esp32_obd
- **FRETish Text:**
  ```text
  in active_session upon obd_timeout_50ms_elapsed the esp32_obd shall immediately satisfy obd_timeout_recorded
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `obd_timeout_50ms_elapsed`: **Input** (Boolean)
  - `obd_timeout_recorded`: **Output** (Boolean)
- **Rationale (Português):** Se a resposta 0x7E8 não chegar em 50 ms, declarar timeout e registrar log sem suspender o escalonador.

---
### REQ_OBD_004 — Intercalação de Comandos de Bancada no Transmissor [REQ-SYS-05] [Subsistema 3] [Camada APP]
- **ID:** `REQ_OBD_004`
- **Parent Requirement ID:** REQ_SYS_05
- **Component:** esp32_obd
- **FRETish Text:**
  ```text
  in active_session upon can_cmd_received_in_queue the esp32_obd shall within 10 MILLISECOND satisfy can_cmd_interleaved
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `can_cmd_received_in_queue`: **Input** (Boolean)
  - `can_cmd_interleaved`: **Output** (Boolean)
- **Rationale (Português):** Intercalar o envio de comandos CAN (ex: 0x010) entre as janelas de polling OBD sem violar a periodicidade nominal de 100 ms.

---

# Subsistema 4: Estruturação de Dados e Bufferização em Memória
### REQ_LOG_001 — Serialização Determinística em CSV [REQ-SYS-11] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_001`
- **Parent Requirement ID:** REQ_SYS_11
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session upon telemetry_frame_received the esp32_logger shall within 1 MILLISECOND satisfy csv_line_formatted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `telemetry_frame_received`: **Input** (Boolean)
  - `csv_line_formatted`: **Output** (Boolean)
- **Rationale (Português):** Converter cada amostra em linha CSV de 11 colunas utilizando buffer estático de 320 bytes sem alocação dinâmica no heap.

---
### REQ_LOG_002 — Integridade Estrutural do Dataset [REQ-SYS-12 / AC-04] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_002`
- **Parent Requirement ID:** REQ_SYS_12
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session when dataset_recording the esp32_logger shall always satisfy null_mandatory_fields = 0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `dataset_recording`: **Input** (Boolean)
  - `null_mandatory_fields`: **Output** (Integer)
- **Rationale (Português):** 100% dos registros gerados devem conter todos os campos mandatórios (timestamp, source, can_id e label) sem campos nulos indevidos (Critério AC-04).

---
### REQ_LOG_003 — Inserção no Buffer Circular em SRAM [REQ-SYS-13] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_003`
- **Parent Requirement ID:** REQ_SYS_13
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session upon csv_line_available the esp32_logger shall within 1 MILLISECOND satisfy sd_buffer_pushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `csv_line_available`: **Input** (Boolean)
  - `sd_buffer_pushed`: **Output** (Boolean)
- **Rationale (Português):** Inserir a linha formatada no buffer em anel de 4096 bytes em SRAM sob seção crítica rápida inferior a 50 µs.

---
### REQ_LOG_004 — Esvaziamento de Buffer por Limiar de Ocupação [REQ-SYS-14] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_004`
- **Parent Requirement ID:** REQ_SYS_14
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session when buffer_occupancy >= 3584 the esp32_logger shall immediately satisfy flush_signal_emitted
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `buffer_occupancy`: **Internal** (Integer)
  - `flush_signal_emitted`: **Output** (Boolean)
- **Rationale (Português):** Ao atingir 85% de ocupação (3584 bytes), emitir sinal imediato para a tarefa de gravação descarregar os dados.

---
### REQ_LOG_005 — Esvaziamento Periódico de Buffer por Temporizador [REQ-SYS-14] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_005`
- **Parent Requirement ID:** REQ_SYS_14
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session upon flush_timer_2s_expired the esp32_logger shall within 10 MILLISECOND satisfy pending_bytes_flushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `flush_timer_2s_expired`: **Input** (Boolean)
  - `pending_bytes_flushed`: **Output** (Boolean)
- **Rationale (Português):** A cada 2 segundos de inatividade de escrita, a tarefa de gravação deve persistir quaisquer bytes pendentes no arquivo.

---
### REQ_LOG_006 — Fatiamento em Chunks de 256 Bytes com Preempção [REQ-SYS-15] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_006`
- **Parent Requirement ID:** REQ_SYS_15
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session upon sector_write_chunk the esp32_logger shall within 1 MILLISECOND satisfy chunk_preemption_yielded
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `sector_write_chunk`: **Input** (Boolean)
  - `chunk_preemption_yielded`: **Output** (Boolean)
- **Rationale (Português):** Fracionar a gravação no SD em pedaços de 256 bytes e ceder controle à CPU para manter o bloqueio inferior a 0,76 ms.

---
### REQ_LOG_007 — Retenção em Backlog de RAM sob Falha do SD [REQ-SYS-23] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_007`
- **Parent Requirement ID:** REQ_SYS_23
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in offline_mode upon sd_write_failed the esp32_logger shall within 2 MILLISECOND satisfy ram_backlog_retained
  ```
- **Variable Mapping:**
  - `offline_mode`: **Internal** (Boolean)
  - `sd_write_failed`: **Input** (Boolean)
  - `ram_backlog_retained`: **Output** (Boolean)
- **Rationale (Português):** Se a escrita falhar em modo offline, reter os frames no buffer de contingência em RAM (50 elementos / 16 KB) sem descarte imediato.

---
### REQ_LOG_008 — Limite de Consumo de Memória SRAM [REQ-SYS-29 / AC-05] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_008`
- **Parent Requirement ID:** REQ_SYS_29
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session when memory_supervision_active the esp32_logger shall always satisfy sram_usage_kb < 200.0
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `memory_supervision_active`: **Input** (Boolean)
  - `sram_usage_kb`: **Output** (Double)
- **Rationale (Português):** O consumo de memória SRAM alocável deve permanecer estritamente contido abaixo de 200 KB durante operação contínua (Critério AC-05).

---
### REQ_LOG_009 — Telemetria de Desempenho HEARTBEAT no SD [REQ-SYS-29] [Subsistema 4] [Camada APP]
- **ID:** `REQ_LOG_009`
- **Parent Requirement ID:** REQ_SYS_29
- **Component:** esp32_logger
- **FRETish Text:**
  ```text
  in active_session upon heartbeat_timer_60s the esp32_logger shall within 10 MILLISECOND satisfy heartbeat_diag_logged
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `heartbeat_timer_60s`: **Input** (Boolean)
  - `heartbeat_diag_logged`: **Output** (Boolean)
- **Rationale (Português):** A cada 60 segundos em sessão ativa, gravar linha DIAG,HEARTBEAT com métricas de heap e contadores de frames no cartão SD.

---

# Subsistema 5: Persistência em Cartão MicroSD e Gestão de Sessões
### REQ_SD_001 — Montagem do Sistema de Arquivos FAT32 [REQ-SYS-16] [Subsistema 5] [Camada BSW]
- **ID:** `REQ_SD_001`
- **Parent Requirement ID:** REQ_SYS_16
- **Component:** esp32_sd
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & mcal_spi_sd_up & sd_card_inserted the esp32_sd shall within 10 MILLISECOND satisfy fat32_filesystem_mounted
  ```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `mcal_spi_sd_up`: **Input** (Boolean) — barramento SPI2 e módulo leitor de SD energizados e prontos
  - `sd_card_inserted`: **Input** (Boolean) — cartão físico inserido no slot
  - `fat32_filesystem_mounted`: **Output** (Boolean) — partição FAT32 montada com sucesso
- **Rationale (Português):** Durante o boot, se o módulo leitor SPI estiver operacional e o cartão MicroSD estiver fisicamente inserido, o sistema deve inicializar o barramento e montar a partição FAT32 em até 500 ms, assegurando que a mídia de armazenamento esteja pronta e formatada antes do início da telemetria.

---
### REQ_SD_002 — Transição Atômica de Arquivos de Sessão [REQ-SYS-17] [Subsistema 5] [Camada BSW]
- **ID:** `REQ_SD_002`
- **Parent Requirement ID:** REQ_SYS_17
- **Component:** esp32_sd
- **FRETish Text:**
  ```text
  upon session_rotate_command the esp32_sd shall within 10 MILLISECOND satisfy session_file_rotated_atomically
  ```
- **Variable Mapping:**
  - `session_rotate_command`: **Input** (Boolean)
  - `session_file_rotated_atomically`: **Output** (Boolean)
- **Rationale (Português):** Ao girar sessão, esvaziar síncronamente o buffer antigo e abrir o novo arquivo indexado S_XXXX.CSV em até 50 ms.

---
### REQ_SD_003 — Injeção de Cabeçalho e Linha BOOT [REQ-SYS-17] [Subsistema 5] [Camada BSW]
- **ID:** `REQ_SD_003`
- **Parent Requirement ID:** REQ_SYS_17
- **Component:** esp32_sd
- **FRETish Text:**
  ```text
  upon new_file_opened the esp32_sd shall immediately satisfy header_and_boot_lines_written
  ```
- **Variable Mapping:**
  - `new_file_opened`: **Input** (Boolean)
  - `header_and_boot_lines_written`: **Output** (Boolean)
- **Rationale (Português):** Gravar o cabeçalho CSV na linha 1 e o metadado BOOT na linha 2 antes de qualquer amostra de telemetria.

---
### REQ_SD_004 — Temporização de Sessão Automática [REQ-SYS-18] [Subsistema 5] [Camada BSW]
- **ID:** `REQ_SD_004`
- **Parent Requirement ID:** REQ_SYS_18
- **Component:** esp32_sd
- **FRETish Text:**
  ```text
  in active_session upon session_duration_reached the esp32_sd shall within 10 MILLISECOND satisfy session_stopped_and_flushed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `session_duration_reached`: **Input** (Boolean)
  - `session_stopped_and_flushed`: **Output** (Boolean)
- **Rationale (Português):** Ao atingir a duração programada, injetar SESSION_COMPLETE, forçar flush de encerramento e suspender novas gravações.

---
### REQ_SD_005 — Volume Consolidado do Dataset [REQ-SYS-11/17 / AC-08] [Subsistema 5] [Camada BSW]
- **ID:** `REQ_SD_005`
- **Parent Requirement ID:** REQ_SYS_18
- **Component:** esp32_sd
- **FRETish Text:**
  ```text
  upon benchmark_completion the esp32_sd shall satisfy total_dataset_samples >= 72000
  ```
- **Variable Mapping:**
  - `benchmark_completion`: **Input** (Boolean)
  - `total_dataset_samples`: **Output** (Integer)
- **Rationale (Português):** O sistema deve registrar um volume consolidado igual ou superior a 72.000 amostras nos três cenários de teste de 10 min (Critério AC-08).

---

# Subsistema 6: Conectividade em Nuvem e Telemetria Remota
### REQ_COM_001 — Conexão Wi-Fi e Pilha TCP/IP [REQ-SYS-19] [Subsistema 6] [Camada BSW]
- **ID:** `REQ_COM_001`
- **Parent Requirement ID:** REQ_SYS_19
- **Component:** esp32_telemetry
- **FRETish Text:**
  ```text
  in boot_mode upon boot_trigger & wifi_credentials_configured the esp32_telemetry shall immediately satisfy wifi_dhcp_timer_start
```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `boot_trigger`: **Input** (Boolean)
  - `wifi_credentials_configured`: **Input** (Boolean)
  - `wifi_dhcp_timer_start`: **Output** (Boolean)
- **Rationale (Português):** Durante a inicialização com credenciais válidas, iniciar imediatamente a solicitação de conexão Wi-Fi e o temporizador de concessão de IP via DHCP (padrão de handshake de temporizador da NASA).
---

### REQ_COM_005 — Estabelecimento de Conexão com o Broker MQTT [REQ-SYS-19] [Subsistema 6] [Camada BSW]
- **ID:** `REQ_COM_005`
- **Parent Requirement ID:** REQ_SYS_19
- **Component:** esp32_telemetry
- **FRETish Text:**
  ```text
  in boot_mode upon ip_dhcp_assigned & mqtt_credentials_valid the esp32_telemetry shall immediately satisfy mqtt_connect_timer_start
```
- **Variable Mapping:**
  - `boot_mode`: **Internal** (Boolean)
  - `ip_dhcp_assigned`: **Input** (Boolean)
  - `mqtt_credentials_valid`: **Input** (Boolean)
  - `mqtt_connect_timer_start`: **Output** (Boolean)
- **Rationale (Português):** Condicionado à obtenção do IP e credenciais válidas, iniciar imediatamente o processo e temporizador de conexão ao broker MQTT (padrão NASA).
---

### REQ_COM_002 — Despacho em Lotes Binários via MQTT [REQ-SYS-20] [Subsistema 6] [Camada BSW]
- **ID:** `REQ_COM_002`
- **Parent Requirement ID:** REQ_SYS_20
- **Component:** esp32_telemetry
- **FRETish Text:**
  ```text
  in connected_mode upon binary_batch_full the esp32_telemetry shall within 10 MILLISECOND satisfy mqtt_batch_published
```
- **Variable Mapping:**
  - `connected_mode`: **Internal** (Boolean)
  - `binary_batch_full`: **Input** (Boolean)
  - `mqtt_batch_published`: **Output** (Boolean)
- **Rationale (Português):** Despacho por Capacidade Máxima do Lote MQTT [REQ-SYS-20] [Subsistema 6] [Camada BSW]
  Ao atingir a capacidade máxima de 150 frames binários (`binary_batch_full`), transmitir o lote binário compactado no tópico `/telemetry/S{id}/raw` com prazo de execução (WCET) de até 10 ms. O despacho complementar por temporização periódica (caso a taxa de frames seja baixa e o lote não atinja 150 frames) é governado de forma independente pelo ciclo de 50 ms em REQ_COM_004.
---

### REQ_COM_003 — Publicação Periódica de Status e Keepalive [REQ-SYS-21] [Subsistema 6] [Camada BSW]
- **ID:** `REQ_COM_003`
- **Parent Requirement ID:** REQ_SYS_21
- **Component:** esp32_telemetry
- **FRETish Text:**
  ```text
  in connected_mode upon status_timer_5s the esp32_telemetry shall immediately satisfy status_json_published
```
- **Variable Mapping:**
  - `connected_mode`: **Internal** (Boolean)
  - `status_timer_5s`: **Input** (Boolean)
  - `status_json_published`: **Output** (Boolean)
- **Rationale (Português):** Emitir imediatamente o payload JSON de diagnóstico e keepalive a cada disparo do temporizador de 5 segundos.
---

### REQ_COM_004 — Tarefa Periódica de Despacho (BSW Com) [REQ-SYS-20] [Subsistema 6] [Camada BSW]
- **ID:** `REQ_COM_004`
- **Parent Requirement ID:** REQ_SYS_20
- **Component:** esp32_telemetry
- **FRETish Text:**
  ```text
  in active_session upon dispatch_cycle_50ms the esp32_telemetry shall within 3 MILLISECOND satisfy data_packets_dispatched
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `dispatch_cycle_50ms`: **Input** (Boolean)
  - `data_packets_dispatched`: **Output** (Boolean)
- **Rationale (Português):** O despachante assíncrono processa e encaminha imediatamente os pacotes de telemetria a cada ciclo periódico de 50 ms.
- **Nota de Implementação:** No firmware em Rust (`firmware/esp32s3_collector/src/bsw/bsw_com.rs`), o despacho é executado pela tarefa assíncrona `task_wifi`, consumindo frames do canal estático `MQTT_TX_CHANNEL` (capacidade 500) em lotes de até 150 registros.

---

# Subsistema 7: Máquina de Estados de Conectividade e Fallback Offline
### REQ_FSM_001 — Comutação Automática de Fallback em MicroSD [REQ-SYS-23 / AC-06] [Subsistema 7] [Camada APP]
- **ID:** `REQ_FSM_001`
- **Parent Requirement ID:** REQ_SYS_23
- **Component:** esp32_fsm
- **FRETish Text:**
  ```text
  in active_session upon wifi_disconnected the esp32_fsm shall within 5 MILLISECOND satisfy sd_fallback_active
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `wifi_disconnected`: **Input** (Boolean)
  - `sd_fallback_active`: **Output** (Boolean)
- **Rationale (Português):** Na perda do sinal Wi-Fi, registrar evento DIAG e redirecionar 100% dos dados para o cartão SD em até 5 ms (Critério AC-06).

---
### REQ_FSM_002 — Reconexão de Rede e Dreno FIFO de Backlog [REQ-SYS-24] [Subsistema 7] [Camada APP]
- **ID:** `REQ_FSM_002`
- **Parent Requirement ID:** REQ_SYS_24
- **Component:** esp32_fsm
- **FRETish Text:**
  ```text
  in active_session upon wifi_reconnected the esp32_fsm shall within 10 MILLISECOND satisfy backlog_fifo_drained
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `wifi_reconnected`: **Input** (Boolean)
  - `backlog_fifo_drained`: **Output** (Boolean)
- **Rationale (Português):** Ao restabelecer a rede, descarregar integralmente as amostras retidas no backlog de RAM para o SD na ordem estrita de chegada.

---

# Subsistema 8: Controle Remoto e Streaming de Replay
### REQ_CMD_001 — Execução de Comandos Remotos de Bancada [REQ-SYS-25] [Subsistema 8] [Camada BSW]
- **ID:** `REQ_CMD_001`
- **Parent Requirement ID:** REQ_SYS_25
- **Component:** esp32_cmd
- **FRETish Text:**
  ```text
  upon remote_command_received the esp32_cmd shall within 10 MILLISECOND satisfy command_executed_ack
  ```
- **Variable Mapping:**
  - `remote_command_received`: **Input** (Boolean)
  - `command_executed_ack`: **Output** (Boolean)
- **Rationale (Português):** Interpretar e executar instruções no tópico /coach/command (STOP, ECO/NOR/SPT, RESET, TIME, LIST_SESSIONS) em até 10 ms (WCET).

---
### REQ_CMD_002 — Transmissão em Streaming de Replay [REQ-SYS-26] [Subsistema 8] [Camada BSW]
- **ID:** `REQ_CMD_002`
- **Parent Requirement ID:** REQ_SYS_26
- **Component:** esp32_cmd
- **FRETish Text:**
  ```text
  upon replay_command_triggered the esp32_cmd shall within 10 MILLISECOND satisfy replay_streaming_active
  ```
- **Variable Mapping:**
  - `replay_command_triggered`: **Input** (Boolean)
  - `replay_streaming_active`: **Output** (Boolean)
- **Rationale (Português):** Suspender sessão ativa, abrir arquivo S_XXXX.CSV e transmitir blocos de até 1536 bytes em streaming no tópico /telemetry/replay.

---

# Subsistema 9: Supervisão de Falhas, Resiliência e Watchdog
### REQ_REC_001 — Detecção de Erro de Bus-Off no TWAI [REQ-SYS-27] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_001`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  in active_session upon bus_off_error_detected the esp32_recovery shall within 1 MILLISECOND satisfy bus_off_flag_set
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `bus_off_error_detected`: **Input** (Boolean)
  - `bus_off_flag_set`: **Output** (Boolean)
- **Rationale (Português):** Ao detectar o código de erro crítico de saturação elétrica do TWAI, registrar atomicamente o estado de Bus-Off em até 1 ms.

---
### REQ_REC_002 — Pausa Cooperativa das Tarefas de Barramento [REQ-SYS-27] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_002`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  upon bus_off_flag_active the esp32_recovery shall within 10 MILLISECOND satisfy bus_off_pause_signaled
  ```
- **Variable Mapping:**
  - `bus_off_flag_active`: **Input** (Boolean)
  - `bus_off_pause_signaled`: **Output** (Boolean)
- **Rationale (Português):** O watchdog deve emitir sinal de pausa cooperativa para task_can_rx e task_obd_poller suspenderem tentativas de acesso ao TWAI.

---
### REQ_REC_003 — Janela de Espera de 128 ms da Norma ISO 11898 [REQ-SYS-27 / AC-07] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_003`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  upon bus_off_pause_started the esp32_recovery shall immediately satisfy recovery_timer_128ms_start
```
- **Variable Mapping:**
  - `bus_off_pause_started`: **Input** (Boolean)
  - `recovery_timer_128ms_start`: **Output** (Boolean)
- **Rationale (Português):** Iniciar compulsoriamente o temporizador da janela de 128 ms normatizada pela ISO 11898 para observação de bits recessivos (Critério AC-07), utilizando o padrão de handshake de timer da NASA para assegurar realizabilidade instantânea no SMT.
---
### REQ_REC_004 — Reativação via Registradores PAC sem Reboot [REQ-SYS-27 / AC-07] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_004`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  upon recovery_timer_128ms_expired the esp32_recovery shall within 1 MILLISECOND satisfy twai_reset_mode_cleared
```
- **Variable Mapping:**
  - `recovery_timer_128ms_expired`: **Input** (Boolean)
  - `twai_reset_mode_cleared`: **Output** (Boolean)
- **Rationale (Português):** Ao expirar a janela de 128 ms (evento emitido pelo hardware/RTOS), limpar a flag de reset no registrador físico via PAC imediatamente sem reiniciar o processador ESP32-S3 e sem destruir tarefas ativas.
---
### REQ_REC_005 — Retomada Operacional e Emissão de BUS_OFF_CLEAR [REQ-SYS-27 / AC-07] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_005`
- **Parent Requirement ID:** REQ_SYS_27
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  upon twai_reset_mode_cleared the esp32_recovery shall within 5 MILLISECOND satisfy bus_off_clear_signaled
  ```
- **Variable Mapping:**
  - `twai_reset_mode_cleared`: **Input** (Boolean)
  - `bus_off_clear_signaled`: **Output** (Boolean)
- **Rationale (Português):** Concluído o rearmamento, emitir BUS_OFF_CLEAR liberando as tarefas e gravar registro de sucesso no log do cartão SD.

---
### REQ_REC_006 — Detecção de Travamento de Tarefa (Logger Stall) [REQ-SYS-28] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_006`
- **Parent Requirement ID:** REQ_SYS_28
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  in active_session upon logger_stall_30s_detected the esp32_recovery shall within 10 MILLISECOND satisfy stall_diag_logged
```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `logger_stall_30s_detected`: **Input** (Boolean)
  - `stall_diag_logged`: **Output** (Boolean)
- **Rationale (Português):** Ao detectar a flag de timeout de 30 s de inatividade do logger, gravar evento diagnóstico imediatamente e disparar recuperação cooperativa.
---
### REQ_REC_007 — Rearme Periódico do Watchdog de Hardware [REQ-SYS-30] [Subsistema 9] [Camada BSW]
- **ID:** `REQ_REC_007`
- **Parent Requirement ID:** REQ_SYS_30
- **Component:** esp32_recovery
- **FRETish Text:**
  ```text
  in active_session when system_tasks_healthy upon wdt_feed_tick the esp32_recovery shall within 1 MILLISECOND satisfy hw_watchdog_fed
  ```
- **Variable Mapping:**
  - `active_session`: **Internal** (Boolean)
  - `system_tasks_healthy`: **Input** (Boolean)
  - `wdt_feed_tick`: **Input** (Boolean)
  - `hw_watchdog_fed`: **Output** (Boolean)
- **Rationale (Português):** Enquanto as tarefas estiverem saudáveis, a tarefa supervisora alimenta imediatamente o watchdog de hardware a cada tick periódico do temporizador do RTOS.