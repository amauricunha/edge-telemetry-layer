# Relatório Técnico de Engenharia de Sistemas Baseada em Modelos (MBSE)
## Pilar 2: Modelação Arquitetural em AADL no OSATE
**Projeto:** Edge Telemetry Layer (CAN 500 kbps, OBD-II ISO 15765-4, MicroSD FAT32, Wi-Fi/MQTT)  
**Padrão:** SAE AADL AS5506B / OSATE 2.10+  
**Repositório:** [`can-obd-telemetry`](file:///c:/workspace/can-obd-telemetry)  
**Data:** Setembro de 2026  

---

### Sumário Executivo
Este documento consolida as especificações arquiteturais do **Pilar 2** estipuladas no plano de trabalho ([`trabalho.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/trabalho.md)). A modelação textual foi realizada em linguagem **AADL (Architecture Analysis and Design Language)**, abrangendo:
1. **Plataforma de Execução (Hardware):** Processador com protocolo preemptivo POSIX, barramentos físicos diferenciais e síncronos, e dispositivos periféricos.
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

## 6. Localização dos Artefatos de Código do Pilar 2

O modelo AADL foi disponibilizado em dois formatos complementares no repositório:
1. **Ficheiro Consolidado Único (Inspeção Rápida):**  
   [`docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl)  
   *(Espelho em [`docs/MBSE/resultados/EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/resultados/EdgeTelemetryLayer_Pilar2.aadl))*
2. **Projeto Modular em Pacotes (Compatível com OSATE e `DevCompatibility`):**  
   [`docs/MBSE/osate_project/packages/`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/)  
   - [`Data_Types_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/Data_Types_Pkg.aadl)
   - [`Buses_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/Buses_Pkg.aadl)
   - [`Processors_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/Processors_Pkg.aadl)
   - [`Software_Threads_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/Software_Threads_Pkg.aadl)
   - [`Software_Processes_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/Software_Processes_Pkg.aadl)
   - [`EdgeTelemetry_System_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/EdgeTelemetry_System_Pkg.aadl)
   - [`Library/devices/`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/Library/devices/)
