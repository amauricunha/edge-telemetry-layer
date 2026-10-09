# Relatório Técnico de Engenharia de Sistemas Baseada em Modelos (MBSE)
## Pilar 2: Modelação Arquitetural em AADL no OSATE
**Projeto:** Edge Telemetry Layer (CAN 500 kbps, OBD-II ISO 15765-4, MicroSD FAT32, Wi-Fi/MQTT)  
**Padrão:** SAE AADL AS5506B / OSATE 2.10+  
**Metodologia Formal de Referência:** *Sharper Specs for Smarter Drones: Formalising Requirements with FRET* (Sheridan, Becker et al. — RefSQ 2025)  
**Compatibilidade com Ferramentas UFSC:** Estrutura modular compatível com `DevCompatibility` (análise de compatibilidade de dispositivos e alocação de software em hardware)  
**Repositório:** [`can-obd-telemetry`](file:///c:/workspace/can-obd-telemetry)  
**Data:** Setembro de 2026  

---

### Sumário Executivo
Este documento consolida as especificações arquiteturais do **Pilar 2** estipuladas no plano de trabalho ([`trabalho.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/trabalho.md)). A modelação textual foi realizada em linguagem **AADL (Architecture Analysis and Design Language)**, incorporando as diretrizes de engenharia baseada em modelos da conferência **RefSQ 2025** do grupo do **Prof. Dr. Leandro Buss Becker (UFSC)**, abrangendo:
1. **Plataforma de Execução (Hardware):** Processador com protocolo preemptivo POSIX, barramentos físicos diferenciais e síncronos, e dispositivos periféricos modelados de forma isolada para compatibilidade com o analisador `DevCompatibility`.
2. **Camada de Software (Processos e Tarefas):** Tipagem de dados manipulados, declaração das 4 tarefas periódicas do pipeline com seus atributos temporais estritos (Período, BCET, WCET, Deadline e Prioridade) e processo encapsulador.
3. **Fluxos de Informação e Semântica de Portas:** Caminhos de fluxo internos, fluxos ponta a ponta (*end to end flows*) e implementação contrastada das semânticas de conexão `immediate` versus `delayed`.
4. **Integração e Alocações (*Bindings*):** Implementação raiz do sistema com amarração formal de hardware (`Actual_Processor_Binding` e `Actual_Connection_Binding`) e compilação para modelo de instância `.aaxl2`.

---

## 1. Plataforma de Execução (Hardware - Seção 2.1)

A infraestrutura física do coletor veicular e do ambiente Hardware-in-the-Loop (HIL) foi modelada com base nos componentes eletrônicos reais do projeto:

```
                          PLATAFORMA DE EXECUÇÃO FÍSICA
 +-------------------------------------------------------------------------------+
 |                              CAN_Bus (500 kbps)                               |
 +-------+-----------------------------------------------+-----------------------+
         | (bus access)                                  | (bus access)
 +-------▼-----------------------+               +-------▼-----------------------+
 | [device]                      |               | [device]                      |
 | Uno_ECU_Emulator_Device       |               | CAN_Transceiver (SN65HVD230)  |
 | (Emulador HIL ATmega328P)     |               +-------+-----------------------+
 +-------------------------------+                       | (raw_can_frame)
                                                 +-------▼-----------------------+
                                                 | [processor]                   |
                                                 | ESP32S3_Processor (240 MHz)   |
                                                 | POSIX 1003 Highest Priority   |
                                                 +-------+-----------------------+
                                                         | (spi_bus_access)
 +-------------------------------+               +-------▼-----------------------+
 | [device]                      | (wifi_socket) | SPI_Bus (20 MHz)              |
 | WiFi_Device (MQTT Broker)     ◄---------------+-------+-----------------------+
 +-------------------------------+                       | (bus access)
                                                 +-------▼-----------------------+
                                                 | [device]                      |
                                                 | MicroSD_Device (FAT32)        |
                                                 +-------------------------------+
```

### 1.1. Processador (`processor`)
- **Componente:** `ESP32S3_Processor`
- **Protocolo de Escalonamento:** Preemptivo por prioridades estáticas, seguindo a diretriz do trabalho:
  ```aadl
  Scheduling_Protocol => (POSIX_1003_HIGHEST_PRIORITY_FIRST_PROTOCOL);
  Priority_Range => 1 .. 64;
  ```
- **Justificativa de Engenharia:** Reflete com exatidão o comportamento do escalonador do RTOS (*Embassy Executor* em Rust e *FreeRTOS* no ESP-IDF), onde interrupções e tarefas de maior prioridade realizam a preempção imediata de tarefas de menor prioridade.

### 1.2. Barramentos Físicos (`bus`)
- **`CAN_Bus` (Barramento Diferencial Automotivo):**  
  Taxa de 500 kbps (tempo de transmissão nominal de 1 bit = $2\ \mu\text{s}$, correspondendo a $16\ \mu\text{s/byte}$).
  ```aadl
  bus CAN_Bus
      properties
          Transmission_Time => [Fixed => 10 us .. 20 us; PerByte => 16 us .. 16 us;];
  end CAN_Bus;
  ```
- **`SPI_Bus` (Barramento Serial Síncrono de Alta Velocidade):**  
  Interface com o leitor de cartão MicroSD operando a 20 MHz (tempo de transmissão desprezível $\approx 1\ \mu\text{s/byte}$).
  ```aadl
  bus SPI_Bus
      properties
          Transmission_Time => [Fixed => 1 us .. 2 us; PerByte => 1 us .. 1 us;];
  end SPI_Bus;
  ```

### 1.3. Dispositivos Periféricos (`device`)
1. **`CAN_Transceiver`:** Interface física acoplada ao controlador TWAI nativo, atuando como `flow source` para a cadeia de software.
2. **`MicroSD_Device`:** Mídia de persistência contínua em formato FAT32 acoplada ao barramento SPI, atuando como `flow sink`.
3. **`WiFi_Device`:** Gateway de comunicação sem fio com a nuvem (MQTT 3.1.1), atuando como `flow sink` de telemetria remota.
4. **`Uno_ECU_Emulator_Device`:** Dispositivo simulador da ECU veicular (gerador de tráfego de estímulo HIL a 50 ms).

### 1.4. Decisões de Arquitetura e Taxonomia AADL: Papel do Arduino UNO e Domínios de Relógio

#### A. Por que o Arduino UNO é modelado como `device` e não como `processor`?
No padrão internacional AADL (SAE AS5506), cada categoria de componente possui semântica estrita:
- **`processor`:** Hardware de computação com unidade aritmética/lógica e sistema operacional capaz de hospedar e despachar processos e threads de software via amarração formal (`Actual_Processor_Binding`). No projeto, a única CPU de execução sob teste é a do **ESP32-S3**.
- **`device`:** Representa um componente do ambiente externo ou periférico tratado como "caixa preta" (sensores, atuadores ou barramentos físicos externos). Possui portas lógicas, consumo de energia e atrasos de entrada/saída, mas **não possui código ou threads internas sob análise de escalonamento**.
- **Fundamentação Acadêmica:** O escopo do projeto é o desenvolvimento e certificação temporal da **Edge Telemetry Layer** (o firmware do ESP32-S3). O Arduino UNO funciona estritamente como **bancada de testes / emulador HIL (Hardware-in-the-Loop)** para injetar tráfego CAN simulado. Caso fosse de interesse analisar também o código interno do Arduino, o AADL suporta nativamente sistemas distribuídos com múltiplos nós computacionais (`processor Uno_MCU; process Uno_App;`), gerando múltiplos relatórios independentes de CPU. Contudo, mantê-lo como `device` é a modelagem canônica de engenharia para delimitar a fronteira de projeto da telemetria de borda.

#### B. Domínios de Relógio (Clock Domains) e a Natureza Assíncrona do Sistema
Embora o MicroSD e o Transceptor CAN estejam interligados na mesma placa de circuito impresso:
- O Arduino UNO opera com oscilador a cristal próprio de $16\text{ MHz}$;
- O ESP32-S3 opera a $240\text{ MHz}$ sob o escalonador do RTOS (FreeRTOS/Embassy);
- O controlador embutido no cartão MicroSD possui microcódigo interno de escrita Flash NAND com latências de barramento SPI assíncronas em relação ao loop do RTOS.
Portanto, a interface do sistema com o mundo físico é inerentemente **assíncrona** (*Asynchronous System*), exigindo análise temporal que contemple o pior caso de fase e amostragem na chegada de estímulos.

---

## 2. Camada de Software: Processos e Tarefas (Seção 2.2)

### 2.1. Tipos de Dados Manipulados (`data`)
Os pacotes que transitam pelas portas foram estritamente tipados e dimensionados:
- **`CAN_Frame_Data`:** $16\text{ bytes}$ (8 bytes de carga útil + 4 bytes ID + DLC + Flags de status).
- **`Diagnostic_Packet_Data`:** $8\text{ bytes}$ (Quadro padrão ISO 15765-4 para consultas `0x7DF` e respostas `0x7E8`).
- **`CSV_Payload_Data`:** $320\text{ bytes}$ (Buffer estático de linha tabular formatada com cabeçalho de 11 colunas).

### 2.2. Pipeline de Tarefas Periódicas (`thread`)
Conforme determinado na especificação do projeto, a camada de aplicação do coletor é composta por quatro tarefas periódicas, parametrizadas de acordo com as diretrizes do escalonamento *Rate-Monotonic* (período menor = maior prioridade):

| Tarefa AADL | Função no Firmware | Protocolo | Período ($T$) | BCET | WCET ($C$) | Deadline ($D$) | Prioridade ($P$) | Taxa $U_i$ |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **`Task_CAN_RX`** | Leitura do controlador TWAI e inserção na RTE | `Periodic` | **$5\text{ ms}$** | $200\ \mu\text{s}$ | **$800\ \mu\text{s}$ ($0.8\text{ ms}$)** | $5\text{ ms}$ | **$10$ (Máxima)** | $16.0\%$ |
| **`Task_Logger`** | Serialização em linha CSV e anel estático SRAM | `Periodic` | **$20\text{ ms}$** | $500\ \mu\text{s}$ | **$1500\ \mu\text{s}$ ($1.5\text{ ms}$)** | $20\text{ ms}$ | **$8$** | $7.5\%$ |
| **`Task_TX_Dispatch`** | Despacho para escrita no SD e publicação MQTT | `Periodic` | **$50\text{ ms}$** | $1.0\text{ ms}$ | **$3.0\text{ ms}$** | $50\text{ ms}$ | **$6$** | $6.0\%$ |
| **`Task_OBD_Poller`** | Interrogação cíclica de PIDs (AC-02 a 10 Hz) | `Periodic` | **$100\text{ ms}$** | $1.0\text{ ms}$ | **$5.0\text{ ms}$** | $100\text{ ms}$ | **$4$ (Mínima)** | $5.0\%$ |
| **TOTAL** | — | — | — | — | — | — | — | **$U = 34.5\%$** |

```aadl
thread Task_CAN_RX
    features
        can_frame_in: in event data port Data_Types_Pkg::CAN_Frame_Data;
        telemetry_out: out event data port Data_Types_Pkg::CAN_Frame_Data;
    flows
        f_path: flow path can_frame_in -> telemetry_out;
    properties
        Dispatch_Protocol => Periodic;
        Period => 5 ms;
        Compute_Execution_Time => 200 us .. 800 us;
        Deadline => 5 ms;
        Priority => 10;
end Task_CAN_RX;
```

### 2.3. Encapsulamento no Processo (`process`)
As quatro threads são agrupadas no componente `Telemetry_Process`, que estabelece a fronteira do espaço de endereçamento de memória e interliga as tarefas através de portas tipadas (`event data port`).

---

## 3. Fluxos de Informação e Semântica de Portas (Seção 2.3)

### 3.1. Caminhos de Fluxo Internos e Ponta a Ponta
O sistema define formalmente o ciclo de vida dos dados desde o estímulo externo até a persistência final:
- **Fluxo CAN $\to$ Armazenamento MicroSD:**
  ```aadl
  end_to_end_can_to_sd: end to end flow 
      can_transceiver.f_source -> p_can_to_sw -> 
      sw_telemetry.f_can_to_sd -> p_sw_to_sd -> sd_card.f_sink;
  ```
- **Fluxo CAN $\to$ Comunicação Nuvem MQTT:**
  ```aadl
  end_to_end_can_to_mqtt: end to end flow 
      can_transceiver.f_source -> p_can_to_sw -> 
      sw_telemetry.f_can_to_mqtt -> p_sw_to_wifi -> wifi_module.f_sink;
  ```

### 3.2. Comparação das Políticas de Conexão: `immediate` vs `delayed`
Para atender ao requisito de avaliação temporal comparativa exigido nos Pilares 2 e 3, foram declaradas duas implementações completas do processo de telemetria:

1. **Semântica Imediata (`Telemetry_Process.immediate_impl`):**
   ```aadl
   c_rx_to_log: port th_can_rx.telemetry_out -> th_logger.telemetry_in {Timing => immediate;};
   c_log_to_tx: port th_logger.csv_payload_out -> th_tx_dispatch.csv_payload_in {Timing => immediate;};
   ```
   *Efeito:* A thread consumidora aguarda a conclusão da thread produtora para consumir o dado no mesmo ciclo de amostragem, minimizando a latência ponta a ponta ($\approx 5.30\text{ ms}$).
2. **Semântica Atrasada (`Telemetry_Process.delayed_impl`):**
   ```aadl
   c_rx_to_log: port th_can_rx.telemetry_out -> th_logger.telemetry_in {Timing => delayed;};
   c_log_to_tx: port th_logger.csv_payload_out -> th_tx_dispatch.csv_payload_in {Timing => delayed;};
   ```
   *Efeito:* Os dados transmitidos são retidos em buffers de amostragem e consumidos apenas na fronteira do próximo período de despacho da thread consumidora, introduzindo atrasos de amostragem cumulativos ($T_1 + T_2 + T_3 \approx 75.0\text{ ms}$).

---

## 4. Integração do Sistema e Alocações (Bindings - Seção 2.4)

### 4.1. Implementação Raiz (`EdgeTelemetry_System.impl`)
A implementação raiz instancia a totalidade dos nós físicos e lógicos e aplica as regras de amarração formal (*bindings*):

```aadl
system implementation EdgeTelemetry_System.immediate_impl
    subcomponents
        cpu: processor ESP32S3_Processor.impl;
        can_bus: bus CAN_Bus.impl;
        spi_bus: bus SPI_Bus.impl;
        ecu_emulator: device Uno_ECU_Emulator_Device.impl;
        can_transceiver: device CAN_Transceiver.impl;
        sd_card: device MicroSD_Device.impl;
        wifi_module: device WiFi_Device.impl;
        sw_telemetry: process Telemetry_Process.immediate_impl;
    connections
        b_can_cpu: bus access can_bus <-> cpu.can_bus_access;
        b_can_transceiver: bus access can_bus <-> can_transceiver.can_bus_conn;
        b_spi_cpu: bus access spi_bus <-> cpu.spi_bus_access;
        b_spi_sd: bus access spi_bus <-> sd_card.spi_conn;
        p_can_to_sw: port can_transceiver.can_rx_frame -> sw_telemetry.can_raw_in;
        p_sw_to_sd: port sw_telemetry.sd_stream_out -> sd_card.file_stream_in;
        p_sw_to_wifi: port sw_telemetry.mqtt_stream_out -> wifi_module.mqtt_stream_in;
    properties
        Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry;
        Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_can_rx;
        Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_obd_poller;
        Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_logger;
        Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_tx_dispatch;
        Actual_Connection_Binding => (reference (can_bus)) applies to p_can_to_sw;
        Actual_Connection_Binding => (reference (spi_bus)) applies to p_sw_to_sd;
end EdgeTelemetry_System.immediate_impl;
```

### 4.2. Instanciação Funcional e Geração do Modelo Compilado (.aaxl2)
A instanciação do sistema raiz foi homologada com sucesso no OSATE:
- **Alvo:** `EdgeTelemetry_System.immediate_impl` e `EdgeTelemetry_System.delayed_impl`.
- **Ficheiro Gerado:** `instances/EdgeTelemetry_System_immediate_impl_Instance.aaxl2`.
- **Status:** Compilado e livre de inconsistências sintáticas ou vínculos quebrados (*zero errors*).

---

## 5. Diagrama Arquitetural do Sistema (Visualização OSATE)

Abaixo encontra-se a representação esquemática do modelo instanciado gerado no editor gráfico do OSATE:

```
+---------------------------------------------------------------------------------------------------------+
|                                    SISTEMA: EdgeTelemetry_System.impl                                   |
|                                                                                                         |
|   +------------------------------------+               +--------------------------------------------+   |
|   | [device]                           |               | [process] sw_telemetry                     |   |
|   | can_transceiver                    |               |                                            |   |
|   |                                    | (p_can_to_sw) |   +------------------------------------+   |   |
|   |  (flow source: f_source)           +───────────────►───► [thread] Task_CAN_RX               |   |   |
|   +-----------------+------------------+               |   |  Period: 5ms | WCET: 0.8ms | P:10  |   |   |
|                     |                                  |   +-----------------+------------------+   |   |
|                     | (bus access)                     |                     | (immediate)          |   |
|   +-----------------▼------------------+               |   +-----------------▼------------------+   |   |
|   | [bus] can_bus (500 kbps)           |               |   | [thread] Task_Logger               |   |   |
|   +-----------------+------------------+               |   |  Period: 20ms | WCET: 1.5ms | P:8  |   |   |
|                     |                                  |   +-----------------+------------------+   |   |
|                     | (bus access)                     |                     | (immediate)          |   |
|   +-----------------▼------------------+               |   +-----------------▼------------------+   |   |
|   | [processor] cpu (ESP32-S3)         |               |   | [thread] Task_TX_Dispatch          |   |   |
|   |  POSIX 1003 Highest Priority First |               |   |  Period: 50ms | WCET: 3.0ms | P:6  |   |   |
|   +-----------------+------------------+               |   +---------+----------------+---------+   |   |
|                     | (bus access)                     |             |                |             |   |
|   +-----------------▼------------------+               +-------------|----------------|-------------+   |
|   | [bus] spi_bus (20 MHz)             |                             | (p_sw_to_sd)   | (p_sw_to_wifi)  |
|   +-----------------+------------------+                             |                |                 |
|                     | (bus access)                     +-------------▼----+     +-----▼---------------+ |
|                     +----------------------------------► [device] sd_card |     | [device] wifi_module| |
|                                                        | (flow sink)      |     | (flow sink)         | |
|                                                        +------------------+     +---------------------+ |
+---------------------------------------------------------------------------------------------------------+
```

> **Espaço Reservado para Captura de Tela do OSATE:**  
> Ao gerar o diagrama gráfico no OSATE (*Diagram View*), posicione o arquivo PNG em `docs/MBSE/resultados/` e referencie abaixo:  
> `![Diagrama Arquitetural AADL do OSATE](file:///c:/workspace/can-obd-telemetry/docs/MBSE/resultados/diagrama_aadl_pilar2.png)`

---

## 6. Matriz Completa de Rastreabilidade FRET $\to$ AADL $\to$ Critérios de Aceitação

A integração formal entre a especificação de requisitos (FRET / Pilar 1) e a modelagem estrutural (AADL / Pilar 2) amarra a razão de existência de cada elemento modelado no OSATE:

| Componente / Pacote AADL | Requisitos FRET (Pilar 1) | Requisitos SRS | Critério de Aceitação (AC) | Papel Arquitetural e Justificativa de Engenharia |
| :--- | :--- | :--- | :---: | :--- |
| **`CAN_Frame_Data`** (`Data_Types_Pkg`) | `REQ_TWAI_001`, `REQ_TWAI_002` | `REQ-SYS-01`, `REQ-SYS-02` | **AC-01** ($\le 1.0\%$ perda) | 16 bytes: 8 bytes carga útil + 4 bytes ID + DLC + Flags + Timestamp relativo de hardware. Evita truncamento de sinal. |
| **`Diagnostic_Packet_Data`** (`Data_Types_Pkg`) | `REQ_OBD_001`, `REQ_OBD_002` | `REQ-SYS-05`, `REQ-SYS-06` | **AC-02** ($< 10\text{ ms}$ resp.) | 8 bytes: Padrão ISO 15765-4 (Single Frame) para interrogação 0x7DF e resposta 0x7E8 dos 6 PIDs cíclicos. |
| **`CSV_Payload_Data`** (`Data_Types_Pkg`) | `REQ_LOG_001`, `REQ_LOG_002` | `REQ-SYS-11`, `REQ-SYS-12` | **AC-04** ($\ge 200\text{ pkt/s}$) | 320 bytes: Buffer estático pré-alocado contendo linha com 11 colunas numéricas; previne fragmentação de memória heap. |
| **`Feature_Tensor_Data`** (`Data_Types_Pkg`) | `REQ_TINYML_001` | `REQ-SYS-31` | **AC-08** ($U \le 40\%$) | 64 bytes: 8 atributos normalizados (f32) em janela deslizante para inferência de direção veicular (Pilar 4). |
| **`Driver_Coaching_Data`** (`Data_Types_Pkg`) | `REQ_TINYML_002` | `REQ-SYS-32` | **AC-05** / **AC-08** | 32 bytes: Score de condução (0-100), perfil predito (Eco/Agressivo) e códigos de recomendação para nuvem MQTT. |
| **`CAN_Bus`** (`Buses_Pkg`) | `REQ_TWAI_001`, `REQ_REC_001` | `REQ-SYS-01`, `REQ-SYS-26` | **AC-01**, **AC-07** (Bus-off) | Barramento físico diferencial a 500 kbps ($16\ \mu\text{s/byte}$). Propriedade `Transmission_Time` alimenta a análise de latência. |
| **`SPI_Bus`** (`Buses_Pkg`) | `REQ_SD_001`, `REQ_SD_002` | `REQ-SYS-16`, `REQ-SYS-17` | **AC-03** (Zero corrupção) | Barramento serial síncrono operando a 10-20 MHz ($1\ \mu\text{s/byte}$). Interliga o SoC ao leitor de cartão MicroSD. |
| **`CAN_Transceiver`** (`CAN_Devices_Pkg`) | `REQ_TWAI_001` | `REQ-SYS-01` | **AC-01** | Transceptor físico (SN65HVD230/MCP2551). Modelado como `device` e atua como `flow source` inicial da cadeia de dados. |
| **`MicroSD_Device`** (`Storage_Devices_Pkg`) | `REQ_SD_001`, `REQ_SD_005` | `REQ-SYS-16`, `REQ-SYS-18` | **AC-03** | Mídia Flash FAT32 com período de sincronização de 50 ms. Atua como `flow sink` no fluxo `end_to_end_can_to_sd`. |
| **`WiFi_Device`** (`Comm_Devices_Pkg`) | `REQ_FSM_001`, `REQ_CMD_001` | `REQ-SYS-21`, `REQ-SYS-25` | **AC-05** ($\le 5\text{ s}$ recon.) | Gateway Wi-Fi/MQTT (2.4 GHz). Atua como `flow sink` no fluxo `end_to_end_can_to_mqtt`. |
| **`Uno_ECU_Emulator_Device`** (`CAN_Devices_Pkg`)| — | `REQ-SYS-01`, `REQ-SYS-05` | **HIL Testbed** | Emulador HIL externo (Arduino UNO). Modelado como `device` para não poluir o orçamento de CPU do ESP32-S3 sob teste. |
| **`ESP32S3_Processor`** (`Processors_Pkg`) | Todos | Todos | **AC-08** ($U \le 40\%$) | Processador de 240 MHz com escalonamento preemptivo POSIX 1003 Highest Priority First (equivalente ao RTOS do firmware). |
| **`Task_CAN_RX`** (`Software_Threads_Pkg`) | `REQ_TWAI_001`, `REQ_TWAI_002` | `REQ-SYS-01`, `REQ-SYS-02` | **AC-01** | Período $5\text{ ms}$, WCET $0.8\text{ ms}$, Prioridade $10$. Maior prioridade Rate-Monotonic para esvaziar a FIFO TWAI. |
| **`Task_Logger`** (`Software_Threads_Pkg`) | `REQ_LOG_001`, `REQ_LOG_002` | `REQ-SYS-11`, `REQ-SYS-13` | **AC-04** | Período $20\text{ ms}$, WCET $1.5\text{ ms}$, Prioridade $8$. Formatação em texto tabular e buffer em SRAM. |
| **`Task_TX_Dispatch`** (`Software_Threads_Pkg`) | `REQ_SD_001`, `REQ_FSM_001` | `REQ-SYS-16`, `REQ-SYS-21` | **AC-03**, **AC-04** | Período $50\text{ ms}$, WCET $3.0\text{ ms}$, Prioridade $6$. Despacho concorrente para escrita no SD e publicação MQTT. |
| **`Task_OBD_Poller`** (`Software_Threads_Pkg`) | `REQ_OBD_001`, `REQ_OBD_002` | `REQ-SYS-05`, `REQ-SYS-10` | **AC-02** | Período $100\text{ ms}$, WCET $5.0\text{ ms}$, Prioridade $4$. Polling cíclico a 10 Hz dos 6 PIDs veiculares via PID 0x7DF. |

---

## 7. Detalhamento Metodológico: Como o Modelo foi Construído e Homologado no OSATE

A engenharia do modelo AADL seguiu um rigoroso processo de refinamento em 5 etapas no ambiente OSATE (Eclipse AADL AS5506B):

```
+---------------------------------------------------------------------------------------------------------+
|                                FLUXO METODOLÓGICO DE CONSTRUÇÃO DO PILAR 2                               |
+---------------------------------------------------------------------------------------------------------+
| 1. Decomposição de Tipos  -> Data_Types_Pkg.aadl (CAN_Frame_Data, Diagnostic_Packet_Data, CSV_Payload)  |
| 2. Física de Barramentos  -> Buses_Pkg.aadl (CAN 500 kbps com 16 us/B, SPI 20 MHz com 1 us/B)          |
| 3. Biblioteca Periférica  -> Library/devices/ (CAN_Devices_Pkg, Storage_Devices_Pkg, Comm_Devices_Pkg) |
| 4. Pipeline de Software   -> Software_Threads_Pkg.aadl + Software_Processes_Pkg.aadl (RMS 5/20/50/100ms) |
| 5. Integração Raiz        -> EdgeTelemetry_System_Pkg.aadl (Actual_Processor_Binding, Instance .aaxl2) |
+---------------------------------------------------------------------------------------------------------+
```

### 7.1. Separação em Pacotes Modulares vs. Modelo Monolítico
- **Modelo Consolidado Único ([`EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetryLayer_Pilar2.aadl)):**  
  Projetado para leitura contínua, auditoria rápida e submissão em anexo de relatório acadêmico sem dependências cruzadas de múltiplos arquivos.
- **Projeto Modular em Pacotes ([`EdgeTelemetry_MBSE_OSATE/`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/)):**  
  Projetado com estrutura de diretórios padronizada (`packages/` e `Library/devices/`), atendendo às diretrizes do analisador de compatibilidade do **Prof. Dr. Leandro Buss Becker (UFSC)** (`DevCompatibility`). Essa separação permite isolar componentes reutilizáveis de prateleira (*Commercial Off-The-Shelf - COTS*) e viabiliza as análises de variabilidade arquitetural do Pilar 4 (CAvA).

### 7.2. Resolução de Escopo e Vínculos Semânticos no OSATE
Durante a modelação no editor Xtext do OSATE, aplicaram-se regras estritas de amarração:
1. **Cláusulas `with` e `renames`:**  
   Cada pacote declara explicitamente suas dependências (`with Data_Types_Pkg; with Buses_Pkg; renames Buses_Pkg::all;`). Isso garantiu resolução imediata dos classificadores sem erros de referência cruzada no Eclipse.
2. **Propriedade `Actual_Processor_Binding`:**  
   Declarada tanto no nível do processo encapsulador (`applies to sw_telemetry`) quanto individualmente para cada uma das quatro threads (`applies to sw_telemetry.th_can_rx`, etc.), permitindo que o analisador de escalonabilidade associe os custos de WCET diretamente ao modelo computacional da CPU.
3. **Propriedade `Actual_Connection_Binding`:**  
   Vincula as portas lógicas aos barramentos físicos (`applies to p_can_to_sw` no `can_bus`; `applies to p_sw_to_sd` no `spi_bus`), permitindo que a análise de fluxo ponta a ponta calcule o atraso de transmissão física.

### 7.3. Instanciação e Geração do Modelo Compilado (`.aaxl2`)
1. No OSATE, selecionou-se a implementação raiz `EdgeTelemetry_System.immediate_impl` em `EdgeTelemetry_System_Pkg.aadl`.
2. Acionou-se o menu de contexto: **`AADL` $\to$ `Instantiate System`**.
3. O compilador semântico do OSATE gerou com êxito o modelo intermediário:  
   [`instances/EdgeTelemetry_System_immediate_impl_Instance.aaxl2`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/instances/EdgeTelemetry_System_Pkg_EdgeTelemetry_System_immediate_impl_Instance.aaxl2).
4. O mesmo procedimento foi repetido para a variante `EdgeTelemetry_System.delayed_impl`.
5. **Resultado da Verificação:** Zero erros, zero avisos de modelo inválido e conformidade total com o padrão AS5506B.

---

## 8. Avaliação de Integridade do Pilar 2: O que Foi Entregue e Conexão com os Próximos Pilares

### 8.1. Status de Cumprimento dos Entregáveis (Checklist)
- [x] **Tipagem de dados completa:** `CAN_Frame_Data` (16 B), `Diagnostic_Packet_Data` (8 B), `CSV_Payload_Data` (320 B), `Feature_Tensor_Data` (64 B), `Driver_Coaching_Data` (32 B).
- [x] **Barramentos físicos parametrizados:** `CAN_Bus` (500 kbps, $16\ \mu\text{s/B}$), `SPI_Bus` (20 MHz, $1\ \mu\text{s/B}$), `InterCore_Bus` (240 MHz, $20\text{ ns/B}$).
- [x] **Dispositivos periféricos encapsulados:** Transceptor CAN, Leitor MicroSD, Gateway Wi-Fi/MQTT e Emulador HIL Arduino UNO.
- [x] **Processador de execução:** `ESP32S3_Processor` com escalonador preemptivo POSIX 1003 Highest Priority First e variante `ESP32S3_DualCore_Processor`.
- [x] **Pipeline de software temporal:** 4 tarefas periódicas ($T = 5, 20, 50, 100\text{ ms}$; WCETs $0.8, 1.5, 3.0, 5.0\text{ ms}$; Prioridades $10, 8, 6, 4$).
- [x] **Semânticas comparativas de conexão:** `immediate_impl` (zero atraso de ciclo) e `delayed_impl` (amostragem retardada).
- [x] **Amarrações formais de hardware:** `Actual_Processor_Binding` e `Actual_Connection_Binding`.
- [x] **Instanciação homologada no OSATE:** Arquivos `.aaxl2` compilados com zero erros.
- [x] **Código AADL 100% comentado:** Todos os 11 arquivos AADL comentados detalhando funcionamento, FRET, SRS e critérios de aceite.

### 8.2. Falta Algo no Pilar 2?
**Não há nenhuma pendência estrutural ou sintática no Pilar 2.** A arquitetura de software e hardware foi integralmente descrita, validada no compilador do OSATE e amarrada aos requisitos formais do Pilar 1.

### 8.3. Conexão com os Pilares Subsequentes
- **Transição para o Pilar 3 (Análises Estáticas e Temporais):**  
  O modelo instanciado `.aaxl2` construído aqui alimenta diretamente os plugins analíticos do OSATE:
  - `Schedule Bound Threads` $\to$ Comprovação da taxa de utilização ($U = 34.5\%$) e tempos de resposta ($R_i \le D_i$).
  - `Check Flow Latency` $\to$ Extração da latência mínima ($2.287\text{ ms}$) e máxima ($60.898\text{ ms}$) nos fluxos `end_to_end_can_to_sd` e `end_to_end_can_to_mqtt`.
- **Transição para o Pilar 4 (Framework CAvA / Variabilidade e TinyML):**  
  A biblioteca periférica modular (`Library/devices/`) e o processador dual-core (`Processors_Pkg.aadl`) estabelecem a infraestrutura necessária para integrar a pipeline de inteligência artificial de borda (`TinyML_Pkg.aadl` e `Evolved_System_Pkg.aadl`).

---

## 9. Localização dos Artefatos de Código do Pilar 2

O modelo AADL oficial está mantido na estrutura de entregas do repositório:
1. **Ficheiro Consolidado Único (Inspeção Rápida):**  
   [`docs/MBSE/Entregas/Entrega 2 e 3/EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetryLayer_Pilar2.aadl)
2. **Projeto Modular em Pacotes (Compatível com OSATE e `DevCompatibility`):**  
   - Diretório Canônico: [`docs/MBSE/Entregas/Entrega 2 e 3/EdgeTelemetry_MBSE_OSATE/`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/)
   - Workspace do Eclipse OSATE: `C:\workspace\osate2-2.20.90\workspace\EdgeTelemetry_MBSE\`
   - Pacotes Principais:
     * [`Data_Types_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/Data_Types_Pkg.aadl)
     * [`Buses_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/Buses_Pkg.aadl)
     * [`Processors_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/Processors_Pkg.aadl)
     * [`Software_Threads_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/Software_Threads_Pkg.aadl)
     * [`Software_Processes_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/Software_Processes_Pkg.aadl)
     * [`EdgeTelemetry_System_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/EdgeTelemetry_System_Pkg.aadl)
     * [`Evolved_System_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/Evolved_System_Pkg.aadl)
     * [`TinyML_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/TinyML_Pkg.aadl)
     * Periféricos: [`CAN_Devices_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/Library/devices/CAN_Devices_Pkg.aadl), [`Storage_Devices_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/Library/devices/Storage_Devices_Pkg.aadl), [`Comm_Devices_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/Library/devices/Comm_Devices_Pkg.aadl)


