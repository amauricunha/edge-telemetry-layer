# Guia Prático de Execução no OSATE: Pilar 2 e Pilar 3
## Modelação Arquitetural, Instanciação e Análises Temporais de Tempo Real
**Projeto:** Edge Telemetry Layer  
**Arquivo AADL Principal:** [`docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl)  
**Ambiente:** OSATE 2 (Open Source AADL Tool Environment) v2.10+ / Eclipse Modeling Framework  

---

### Sumário
1. [Visão Geral dos Componentes e Correspondência com o Pilar 2](#1-visão-geral-dos-componentes-e-correspondência-com-o-pilar-2)
2. [Instalação e Importação no OSATE](#2-instalação-e-importação-no-osate)
3. [Passo a Passo de Instanciação do Sistema Raiz (.aaxl2)](#3-passo-a-passo-de-instanciação-do-sistema-raiz-aaxl2)
4. [Execução da Análise de Escalonabilidade (Pilar 3.1 - Check Schedulability)](#4-execução-da-análise-de-escalonabilidade-pilar-31---check-schedulability)
5. [Execução da Análise de Latência Ponta a Ponta (Pilar 3.2 - Check Flow Latency)](#5-execução-da-análise-de-latência-ponta-a-ponta-pilar-32---check-flow-latency)
6. [Quadro Comparativo: Semântica Immediate vs Delayed](#6-quadro-comparativo-semântica-immediate-vs-delayed)
7. [Preparação para o Pilar 4 (Avaliação de Evolução Arquitetural - Framework CAvA)](#7-preparação-para-o-pilar-4-avaliação-de-evolução-arquitetural---framework-cava)

---

## 1. Visão Geral dos Componentes e Correspondência com o Pilar 2

O arquivo [`EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl) foi estritamente projetado segundo os requisitos da seção 2 do documento [`trabalho.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/trabalho.md):

### 1.1. Plataforma de Execução (Hardware - Seção 2.1)
- **`ESP32S3_Processor`**: Processador de 240 MHz com política de escalonamento preemptiva baseada em prioridades fixas:
  ```aadl
  Scheduling_Protocol => (POSIX_1003_HIGHEST_PRIORITY_FIRST_PROTOCOL);
  Priority_Range => 1 .. 64;
  ```
- **`CAN_Bus`**: Barramento automotivo diferencial a 500 kbps (tempo de transmissão de 1 bit = 2 µs, 16 µs/byte).
- **`SPI_Bus`**: Barramento local síncrono a 20 MHz para interface com cartão MicroSD.
- **Dispositivos (`device`)**:
  - `CAN_Transceiver`: Interface física diferencial CAN acoplada ao barramento.
  - `MicroSD_Device`: Mídia de armazenamento persistente com sistema de arquivos FAT32.
  - `WiFi_Device`: Módulo de comunicação sem fio via soquete TCP/IP e protocolo MQTT.
  - `Uno_ECU_Emulator_Device`: Emulador da ECU automotiva atuando como gerador de tráfego HIL.

### 1.2. Camada de Software (Processos e Threads - Seção 2.2)
Toda a pilha concorrente foi modelada dentro do processo `Telemetry_Process` e decomposta em quatro tarefas periódicas (`thread`) estritamente parametrizadas:

| Tarefa AADL | Função no Firmware | Período ($T$) | BCET | WCET ($C$) | Prazo ($D$) | Prioridade ($P$) | Taxa $U_i$ |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **`Task_CAN_RX`** | Leitura TWAI e empacotamento assíncrono | $5\text{ ms}$ | $200\ \mu\text{s}$ | $800\ \mu\text{s}$ ($0.8\text{ ms}$) | $5\text{ ms}$ | $10$ (Máxima) | $16.0\%$ |
| **`Task_Logger`** | Serialização em CSV e buffer SRAM | $20\text{ ms}$ | $500\ \mu\text{s}$ | $1500\ \mu\text{s}$ ($1.5\text{ ms}$) | $20\text{ ms}$ | $8$ | $7.5\%$ |
| **`Task_TX_Dispatch`** | Despacho para SD e nuvem MQTT | $50\text{ ms}$ | $1.0\text{ ms}$ | $3.0\text{ ms}$ | $50\text{ ms}$ | $6$ | $6.0\%$ |
| **`Task_OBD_Poller`** | Polling cíclico de PIDs (AC-02) | $100\text{ ms}$ | $1.0\text{ ms}$ | $5.0\text{ ms}$ | $100\text{ ms}$ | $4$ (Mínima) | $5.0\%$ |
| **TOTAL** | — | — | — | — | — | — | **$U = 34.5\%$** |

---

## 2. Estrutura do Projeto e Importação no OSATE

O projeto AADL foi estruturado em dois formatos complementares:
- **Formato A — Projeto Modular por Componentes / Pacotes (Padrão Recomendado para `DevCompatibility`):**  
  Localizado em [`docs/MBSE/osate_project/`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/), organiza o sistema em pacotes AADL independentes (`packages/`) e uma biblioteca padronizada de periféricos candidatos (`Library/devices/`), atendendo aos requisitos da ferramenta **DevCompatibility** do grupo do Prof. Leandro Becker.
- **Formato B — Ficheiro Consolidado Único:**  
  Localizado em [`docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl), consolida todas as declarações em um único arquivo para inspeção rápida.

### 2.1. Como Importar o Projeto Modular no OSATE:
1. Abra o **OSATE** no seu ambiente Eclipse.
2. Certifique-se de que o workspace contém as bibliotecas padrão AADL (`Plugin_Resources` contendo `Base_Types`, `Deployment_Properties`, `Timing_Properties` e `Communication_Properties`). Caso não existam, crie um novo projeto AADL:
   - `File` > `New` > `AADL Project` (Nome sugerido: `EdgeTelemetry_MBSE`).
3. Importe a pasta modular do projeto:
   - Clique com o botão direito no projeto criado > `Import...` > `General` > `File System`.
   - Aponte para a pasta `c:\workspace\can-obd-telemetry\docs\MBSE\osate_project`.
   - Marque as pastas **`packages/`** e **`Library/`** e clique em **`Finish`**.
4. **Verificação no AADL Navigator:** A árvore de arquivos ficará organizada da seguinte forma:
   ```
   EdgeTelemetry_MBSE/
   ├── packages/
   │   ├── Data_Types_Pkg.aadl
   │   ├── Buses_Pkg.aadl
   │   ├── Processors_Pkg.aadl
   │   ├── Software_Threads_Pkg.aadl
   │   ├── Software_Processes_Pkg.aadl
   │   ├── EdgeTelemetry_System_Pkg.aadl (Sistema Raiz do Baseline - Pilar 2/3)
   │   ├── TinyML_Pkg.aadl               (Extensão de IA de Borda - Pilar 4)
   │   └── Evolved_System_Pkg.aadl       (Sistema Evoluído Dual-Core - Pilar 4)
   └── Library/
       └── devices/
           ├── CAN_Devices_Pkg.aadl      (CAN Baseline + CAN-FD Candidato)
           ├── Storage_Devices_Pkg.aadl  (MicroSD + Flash candidatos)
           └── Comm_Devices_Pkg.aadl     (Wi-Fi + Modem LTE candidatos)
   ```
5. Verifique na aba **Problems** se não há erros de sintaxe (zero erros).

---

## 3. Passo a Passo de Instanciação do Sistema Raiz (.aaxl2)

Para executar qualquer análise estática ou temporal no OSATE, o modelo declarativo deve ser **instanciado** em um modelo concreto de objetos interconectados.

### No Formato Modular (Recomendado):
1. No painel **AADL Navigator**, abra a pasta `packages/` e selecione o arquivo **`EdgeTelemetry_System_Pkg.aadl`**.
2. Expanda o arquivo até localizar a implementação raiz:
   - `EdgeTelemetry_System.immediate_impl` (para análise de tempo real com semântica imediata)
   - `EdgeTelemetry_System.delayed_impl` (para análise com semântica atrasada)
3. Clique com o **botão direito** sobre `EdgeTelemetry_System.immediate_impl` e selecione:
   - **`Instantiate (Create System Instance)`**
4. O OSATE criará automaticamente a pasta `instances/` e o arquivo compilado:
   - `instances/EdgeTelemetry_System_immediate_impl_Instance.aaxl2`
5. Repita o procedimento para `EdgeTelemetry_System.delayed_impl`, gerando:
   - `instances/EdgeTelemetry_System_delayed_impl_Instance.aaxl2`

*(Nota: Caso utilize o ficheiro único consolidado `EdgeTelemetryLayer_Pilar2.aadl`, o procedimento de instanciação é idêntico).*

---

## 4. Execução da Análise de Escalonabilidade (Pilar 3.1 - Check Schedulability)

A análise comprova formalmente se o conjunto de tarefas periódicas é viável e executável em tempo hábil sob o protocolo preemptivo com prioridades estáticas POSIX.

### 4.1. Como Executar no OSATE:
1. Abra a instância gerada: `EdgeTelemetry_System_immediate_impl_Instance.aaxl2`.
2. No menu superior do OSATE, acerte a perspectiva de análise ou clique no menu:
   - **`Analyses`** > **`Timing`** > **`Check Schedulability`**
3. O relatório detalhado de escalonabilidade será gerado no painel inferior ou na pasta `reports/schedulability/`.

### 4.2. Demonstração Matemática e Validação Teórica:
O OSATE utiliza o método de Análise do Tempo de Resposta no Pior Caso (WCRT - *Worst-Case Response Time*):

1. **Taxa de Utilização Global da CPU ($U$):**
   $$U = \sum_{i=1}^{n} \frac{C_i}{T_i} = \frac{0.8}{5} + \frac{1.5}{20} + \frac{3.0}{50} + \frac{5.0}{100} = 0.16 + 0.075 + 0.06 + 0.05 = \mathbf{34.5\%}$$
   - **Condição de Liu & Layland para 4 tarefas:**
     $$U_{LL}(4) = 4 \times (2^{1/4} - 1) \approx \mathbf{75.68\%}$$
   - Como $U = 34.5\% \le 75.68\%$, o sistema é **incondicionalmente escalonável**.

2. **Equação Recurrente de Tempo de Resposta no Pior Caso ($R_i$):**
   $$R_i^{(k+1)} = C_i + \sum_{j \in hp(i)} \left\lceil \frac{R_i^{(k)}}{T_j} \right\rceil C_j$$

   - **`Task_CAN_RX`** (Prioridade 10):
     $$R_1 = C_1 = 0.8\text{ ms} \le D_1 (5.0\text{ ms}) \quad \color{green}{\checkmark\ \text{SUCESSO}}$$
   - **`Task_Logger`** (Prioridade 8):
     $$R_2^{(0)} = C_2 = 1.5\text{ ms}$$
     $$R_2^{(1)} = 1.5 + \left\lceil \frac{1.5}{5.0} \right\rceil \times 0.8 = 1.5 + 1 \times 0.8 = 2.3\text{ ms} \le D_2 (20.0\text{ ms}) \quad \color{green}{\checkmark\ \text{SUCESSO}}$$
   - **`Task_TX_Dispatch`** (Prioridade 6):
     $$R_3^{(0)} = C_3 = 3.0\text{ ms}$$
     $$R_3^{(1)} = 3.0 + \left\lceil \frac{3.0}{5.0} \right\rceil \times 0.8 + \left\lceil \frac{3.0}{20.0} \right\rceil \times 1.5 = 3.0 + 0.8 + 1.5 = 5.3\text{ ms} \le D_3 (50.0\text{ ms}) \quad \color{green}{\checkmark\ \text{SUCESSO}}$$
   - **`Task_OBD_Poller`** (Prioridade 4):
     $$R_4^{(0)} = C_4 = 5.0\text{ ms}$$
     $$R_4^{(1)} = 5.0 + \left\lceil \frac{5.0}{5.0} \right\rceil \times 0.8 + \left\lceil \frac{5.0}{20.0} \right\rceil \times 1.5 + \left\lceil \frac{5.0}{50.0} \right\rceil \times 3.0 = 5.0 + 0.8 + 1.5 + 3.0 = 10.3\text{ ms} \le D_4 (100.0\text{ ms}) \quad \color{green}{\checkmark\ \text{SUCESSO}}$$

**Conclusão da Análise de Escalonabilidade:**  
Todas as quatro tarefas do pipeline crítico cumprem seus prazos com folga temporal expressiva ($\text{Slack} > 60\%$), garantindo ausência total de preempções infinitas ou estouros de deadline.

---

## 5. Execução da Análise de Latência Ponta a Ponta (Pilar 3.2 - Check Flow Latency)

A análise de fluxo ponta a ponta avalia o tempo total que um dado leva para viajar desde a sua chegada física no transceptor CAN, percorrer todas as camadas do firmware (leitura, serialização CSV e despacho) até atingir a persistência no MicroSD ou transmissão Wi-Fi.

### 5.1. Como Executar no OSATE:
1. Abra o arquivo de instância desejado (ex: `EdgeTelemetry_System_immediate_impl_Instance.aaxl2`).
2. No menu superior, clique em:
   - **`Analyses`** > **`Flows`** > **`Check Flow Latency`**
3. O OSATE inspecionará o fluxo declarado:
   - `end_to_end_can_to_sd`
   - `end_to_end_can_to_mqtt`
4. Repita para a instância `EdgeTelemetry_System_delayed_impl_Instance.aaxl2`.

---

## 6. Quadro Comparativo: Semântica Immediate vs Delayed

A comparação entre as duas políticas de conexão exigida no Pilar 3 demonstra o impacto da amostragem em sistemas dirigidos por tempo real:

| Métrica Analítica | Modo `immediate` (Semântica Imediata) | Modo `delayed` (Semântica Atrasada) | Justificativa Teórica e Comportamento Temporal |
| :--- | :---: | :---: | :--- |
| **Mecanismo de Comunicação** | Sincronismo encadeado no mesmo ciclo de despacho | Retenção em registrador de saída até o final do período | No modo `immediate`, a tarefa receptora aguarda a conclusão da tarefa emissora para consumir o dado no mesmo frame temporal. No modo `delayed`, o dado é consumido somente na fronteira do próximo período. |
| **Latência Mínima CAN $\to$ SD** | **$\approx 1.70\text{ ms}$** | **$75.00\text{ ms}$** | `immediate`: soma dos BCETs ($0.2 + 0.5 + 1.0\text{ ms}$).<br>`delayed`: soma mínima dos períodos ($5 + 20 + 50\text{ ms}$). |
| **Latência Máxima CAN $\to$ SD** | **$\approx 5.30\text{ ms}$** | **$\approx 125.00\text{ ms}$** | `immediate`: soma dos WCETs e interferência de preempção ($0.8 + 1.5 + 3.0\text{ ms}$).<br>`delayed`: inclui atraso de amostragem de pior caso ($T_1 + T_2 + T_3 + \text{jitter}$). |
| **Jitter Ponta a Ponta** | Baixo ($3.6\text{ ms}$) | Alto ($50.0\text{ ms}$) | A semântica imediata minimiza o jitter por forçar a execução coordenada em cadeia. |
| **Aderência ao Firmware Real** | **Alta (Reflete canais RTE Embassy)** | Baixa (Apenas amostras assíncronas periódicas) | Os canais assíncronos em memória RAM do Rust Embassy notificam o consumidor imediatamente após o `send()`, aproximando-se da semântica `immediate`. |

---

## 7. Preparação para o Pilar 4: Integração com `DevCompatibility` e Framework CAvA

O **Pilar 4** aplica o framework **CAvA (Component/Architecture Variability and Evolution approach)** e utiliza a ferramenta **`DevCompatibility`** desenvolvida no grupo de pesquisa do Prof. Leandro Becker (UFSC).

### 7.1. Como Funciona a Análise no `DevCompatibility`:
A ferramenta `DevCompatibility` opera sobre o workspace modular (`docs/MBSE/osate_project/`) confrontando o sistema existente contra uma biblioteca de periféricos candidatos:
1. **Target Architecture (Arquitetura Alvo):** Seleciona o sistema baseline (`EdgeTelemetry_System_Pkg::EdgeTelemetry_System.immediate_impl`).
2. **Componente sob Avaliação:** Seleciona o dispositivo `can_transceiver` (baseado em `CAN_Devices_Pkg::CAN_Transceiver`).
3. **Candidate Library (Biblioteca de Candidatos):** A ferramenta faz a varredura automática na pasta `Library/devices/` e identifica os candidatos elegíveis em `CAN_Devices_Pkg.aadl`:
   - `CAN_FD_Transceiver_Candidate`: Dispositivo com maior taxa de amostragem ($2\text{ ms}$) e porta dedicada de diagnóstico elétrico (`bus_error_diag: out event port`).
4. **Detecção Automática de Incompatibilidades:**
   - **Incompatibilidade de Portas:** A porta `bus_error_diag` não possui correspondência no processo de telemetria existente.
   - **Incompatibilidade Temporal:** Aumento da taxa de injeção de frames de $200\text{ Hz}$ ($5\text{ ms}$) para $500\text{ Hz}$ ($2\text{ ms}$).
5. **Mitigação Formal via Wrappers e Extensão TinyML:**  
   O resultado do `DevCompatibility` direciona a criação de um adaptador de software intermediário (`TinyML_Input_Adapter`) e a evolução arquitetural para o **Sistema Dual-Core com TinyML** modelado em [`packages/Evolved_System_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/osate_project/packages/Evolved_System_Pkg.aadl) e detalhado no [Relatório Técnico do Pilar 4](file:///c:/workspace/can-obd-telemetry/docs/MBSE/relatorio_pilar4_cava_tinyml.md).
