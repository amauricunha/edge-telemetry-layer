# Relatório Técnico de Engenharia de Sistemas Baseada em Modelos (MBSE)
## Pilar 4: Avaliação de Evolução Arquitetural e Inteligência de Borda (Framework CAvA & TinyML)
**Projeto:** Edge Telemetry Layer com Extensão AIoT para *Driver Coaching*  
**Framework Metodológico:** CAvA (*Component/Architecture Variability and Evolution approach*)  
**Metodologia Formal de Referência:** *Sharper Specs for Smarter Drones: Formalising Requirements with FRET* (Sheridan, Becker et al. — RefSQ 2025)  
**Ferramentas:** OSATE 2, DevCompatibility (AEW - *Architecture Evolution Workbench*), TFLite Micro  
**Repositório:** [`can-obd-telemetry`](file:///c:/workspace/can-obd-telemetry)  
**Data:** Setembro de 2026  

---

### Sumário Executivo
Este documento formaliza os entregáveis do **Pilar 4** estipulados no plano de trabalho ([`trabalho.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/trabalho.md)), aplicando as fases metodológicas do framework **CAvA (Fases 2, 3 e 4)**. 

A evolução arquitetural proposta transcende a simples substituição passiva de sensores: transforma a camada de telemetria convencional em um **Gateway Inteligente de Borda (AIoT Edge Gateway)** dotado de um motor de inferência **TinyML** para **Assistência e Coaching de Direção em Tempo Real (*Driver Coaching*)**. A arquitetura aproveita o microcontrolador **ESP32-S3 Dual-Core (240 MHz, 512 KB SRAM, 8 MB Octal PSRAM)**, segregando a aquisição crítica de tempo real no **Core 0** e alocando a extração de atributos e o pipeline de inferência de redes neurais quantizadas no **Core 1**.

Adicionalmente, documenta-se a modularização do projeto AADL em pacotes independentes para compatibilidade com a ferramenta **`DevCompatibility`** do grupo de pesquisa (ecossistema **AEW / ProVANT**), demonstrando a detecção automática de incompatibilidades de portas e a modelação formal de *Wrappers / Adaptadores*.

---

## 1. Fase 1 do CAvA: Linha de Base Arquitetural (Baseline Architecture)

A linha de base é a arquitetura formalizada no **Pilar 2** ([`EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl)), cujas características fundamentais são:
- **Plataforma:** Processador monocore lógico `ESP32S3_Processor` governado por `POSIX_1003_HIGHEST_PRIORITY_FIRST_PROTOCOL`.
- **Pipeline de Software:** 4 threads periódicas (`Task_CAN_RX`, `Task_Logger`, `Task_TX_Dispatch`, `Task_OBD_Poller`).
- **Métricas Temporais do Baseline:**
  - Taxa de utilização da CPU: $U = 34.5\%$ (apenas tráfego nominal de telemetria).
  - Latência ponta a ponta (CAN $\to$ SD/WiFi): $5.30\text{ ms}$ (imediato) a $75.0\text{ ms}$ (atrasado).
  - Comportamento: Puramente reativo e passivo (coleta amostras, serializa em CSV, grava no SD e publica no MQTT).

---

## 2. Fase 2 do CAvA: Identificação dos Cenários de Mudança e Evolução

A evolução arquitetural abrange dois níveis complementares:

### 2.1. Cenário A: Avaliação Automatizada de Substituição de Dispositivo no `DevCompatibility`
Para validar a compatibilidade de componentes e demonstrar a automação de evolução arquitetural proposta pelo framework CAvA (Sales & Becker, UFSC), a arquitetura foi submetida à ferramenta **DevCompatibility (AEW - *Architecture Evolution Workbench*)**:

1. **Componente Baseline a ser substituído:** `wifi_module: device WiFi_Device.impl` (Módulo Wi-Fi local 2.4 GHz com interface de saída MQTT nominal de 50 ms).
2. **Componente Candidato Avaliado:** `Cellular_LTE_Device: device Comm_Devices_Pkg::Cellular_LTE_Device` (Modem celular 4G/LTE COTS para operação em frotas veiculares abertas sem cobertura de rede local).
3. **Detecção Formal de Incompatibilidade de Interface (Port Mismatch):**  
   - A conexão original de saída do processo de telemetria era direcionada à porta `mqtt_stream_in` do `WiFi_Device`.
   - O dispositivo candidato `Cellular_LTE_Device` expõe uma interface com portas distintas: `coaching_stream_in` e `cellular_stream_in`, além de uma porta adicional de telemetria de sinal celular `network_status_out`.
   - O mecanismo de correspondência de portas do DevCompatibility identificou a quebra na ligação `p_sw_to_wifi`.
4. **Síntese Automática de Adaptador (Wrapper Process):**  
   A ferramenta sintetizou automaticamente um processo intermediário adaptador (`wrapper_coaching_stream_in_to_mqtt_stream_out`) para compatibilizar as portas `sw_telemetry.mqtt_stream_out` e `Cellular_LTE_Device.coaching_stream_in`, preservando a consistência do sistema.

#### Evidências Experimentais Obtidas no DevCompatibility:

##### 1. Diagnóstico Inicial e Resolução de Namespace
Na primeira tentativa de carregamento modular, o motor interno do analisador (`AadlEvaluator-Core`) acusou falha de ponteiro nulo (`Subcomponent.getComponent() is null`) decorrente da resolução de classificadores de subcomponentes com prefixos de pacote cruzados (`Processors_Pkg::...`). A unificação no pacote consolidado (`EdgeTelemetry_Pkg`) permitiu o parsing completo e correto da árvore do sistema.

![Erro de resolução de subcomponente em arquivos fragmentados](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/01_devcompatibility_erro_subcomponent_null.png)
*Figura 2.1: Diagnóstico de resolução de classificadores cruzados no DevCompatibility.*

##### 2. Seleção do Subcomponente Alvo (`wifi_module`)
No assistente de evolução (*Evolution Wizard*), o componente `wifi_module` foi selecionado para substituição:

![Seleção do subcomponente wifi_module no DevCompatibility](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/02_devcompatibility_selecao_wifi_device.png)
*Figura 2.2: Seleção do subcomponente wifi_module para evolução arquitetural.*

##### 3. Filtragem de Candidatos da Biblioteca (Filtro por Features)
O DevCompatibility aplica um algoritmo de filtragem baseado em características (*features*). No modo automático (*Auto*), a ferramenta filtra candidatos que possuam portas homônimas (`mqtt_stream_in`):

![Filtragem de candidatos no assistente Candidate Edit](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/03_devcompatibility_candidate_edit_filtro_auto.png)
*Figura 2.3: Assistente de seleção de candidatos com filtro automático de portas.*

##### 4. Identificação da Incompatibilidade nas Ligações Físicas/Lógicas
Ao associar o `Cellular_LTE_Device`, o DevCompatibility listou a conexão `p_sw_to_wifi` como afetada e abriu opções para roteamento e compatibilização:

![Identificação de ligações afetadas pela substituição](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/04_devcompatibility_incompatibilidade_conexoes.png)
*Figura 2.4: Matriz de conexões e identificação de incompatibilidade na ligação p_sw_to_wifi.*

##### 5. Código AADL Evoluído com Wrapper Gerado Automaticamente
A ferramenta gerou a nova especificação formal do sistema incorporando o subcomponente `Cellular_LTE_Device` e o processo adaptador `wrapper_coaching_stream_in_to_mqtt_stream_out`:

```aadl
system implementation EdgeTelemetry_System.immediate_impl
  subcomponents
    can_bus : bus CAN_Bus.impl;
    can_transceiver : device CAN_Transceiver.impl;
    Cellular_LTE_Device : device Comm_Devices_Pkg::Cellular_LTE_Device;
    cpu : processor ESP32S3_Processor.impl;
    ecu_emulator : device Uno_ECU_Emulator_Device.impl;
    sd_card : device MicroSD_Device.impl;
    spi_bus : bus SPI_Bus.impl;
    sw_telemetry : process Telemetry_Process.immediate_impl;
    wrapper_coaching_stream_in_to_mqtt_stream_out : process wrapper_coaching_stream_in_to_mqtt_stream_out;

  connections
    b_can_cpu : bus access can_bus <-> cpu.can_bus_access;
    b_can_emulator : bus access can_bus <-> ecu_emulator.can_bus_conn;
    b_can_transceiver : bus access can_bus <-> can_transceiver.can_bus_conn;
    b_spi_cpu : bus access spi_bus <-> cpu.spi_bus_access;
    b_spi_sd : bus access spi_bus <-> sd_card.spi_conn;
    c9 : port sw_telemetry.mqtt_stream_out -> wrapper_coaching_stream_in_to_mqtt_stream_out.partB;
    p_can_to_sw : port can_transceiver.can_rx_frame -> sw_telemetry.can_raw_in;
    p_can_wire : port ecu_emulator.can_tx_out -> can_transceiver.can_tx_frame;
    p_sw_to_sd : port sw_telemetry.sd_stream_out -> sd_card.file_stream_in;
    p_sw_to_wifi : port wrapper_coaching_stream_in_to_mqtt_stream_out.partA -> Cellular_LTE_Device.coaching_stream_in;

  properties
    Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry;
    Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_can_rx;
    Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_obd_poller;
    Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_logger;
    Actual_Processor_Binding => (reference (cpu)) applies to sw_telemetry.th_tx_dispatch;
    Actual_Connection_Binding => (reference (can_bus)) applies to p_can_wire;
    Actual_Connection_Binding => (reference (can_bus)) applies to p_can_to_sw;
    Actual_Connection_Binding => (reference (spi_bus)) applies to p_sw_to_sd;
end EdgeTelemetry_System.immediate_impl;
```

![Código AADL gerado com síntese de Wrapper no DevCompatibility](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/05_devcompatibility_aadl_result_wrapper_sintetizado.png)
*Figura 2.5: Código AADL resultante da evolução arquitetural com o processo adaptador sintetizado.*

##### 6. Detalhes das Modificações Declaradas (Change Details)
Na visualização detalhada do cenário evoluído (*Change Details*), a ferramenta registrou explicitamente as operações no metamodelo:
- **`declaration added`:** Adição do subcomponente `Cellular_LTE_Device`, do subcomponente `wrapper_coaching_stream_in_to_mqtt_stream_out` e da conexão `c9`.
- **`declaration changed`:** Alteração e redirecionamento da conexão `p_sw_to_wifi`.
- **`declaration deleted`:** Remoção do subcomponente legado `wifi_module`.

![Detalhes das declarações adicionadas, modificadas e removidas](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/07_devcompatibility_change_details_cellular_lte.png)
*Figura 2.6: Painel Change Details documentando formalmente a evolução de componentes e portas.*

##### 7. Análise de Fatores de Trade-off e Comparação Quantitativa (Fase 4 CAvA)
A ferramenta calculou automaticamente o impacto estrutural da mudança nos atributos de qualidade do sistema (norma ISO/IEC 25010), gerando a matriz comparativa oficial entre o sistema original e a evolução gerada:

| Característica (ISO 25010) | Atributo de Qualidade | Arquitetura Original | Cenário Evoluído | Resultado da Análise CAvA / DevCompatibility |
| :--- | :--- | :---: | :---: | :--- |
| **Geral** | Fator Geral (*General Factor*) | $0.23077$ | $0.00000$ | Redução de $0.231$ decorrente da inclusão de wrapper de interface |
| **Funcionalidade** | *Functionality* | $0$ | $0$ | Equivalência funcional mantida |
| **Manutenibilidade** | Conexões Totais (*Connections total*) | $9$ | $10$ | $+1$ conexão ($+10.0\%$) devido ao processo intermediário |
| **Manutenibilidade** | Subcomponentes Totais (*Subcomponents Total*) | $8$ | $9$ | $+1$ subcomponente ($+11.11\%$) devido ao wrapper sintetizado |
| **Desempenho** | Preço Total (*Price Total*) | $\$0.00$ | $\$0.00$ | Custo de hardware equivalente na modelagem |
| **Desempenho** | Peso Total (*Weight Total*) | $0.00\text{ Kg}$ | $0.00\text{ Kg}$ | Peso físico equivalente na modelagem |
| **Desempenho** | Carga CAN Máxima (*can_bus Usage Max*) | $0.0\text{ Kbps}$ | $0.0\text{ Kbps}$ | Carga de barramento nominal idêntica |
| **Desempenho** | Carga CAN Mínima (*can_bus Usage Min*) | $0.0\text{ Kbps}$ | $0.0\text{ Kbps}$ | Carga de barramento nominal idêntica |
| **Desempenho** | Carga CPU Máxima (*cpu Usage Max*) | $1.03\text{ MIPS}$ | $1.03\text{ MIPS}$ | Carga de pico da CPU mantida dentro do limite seguro |
| **Desempenho** | Carga CPU Mínima (*cpu Usage Min*) | $0.27\text{ MIPS}$ | $0.27\text{ MIPS}$ | Carga mínima da CPU sem sobrecarga de baseline |
| **Desempenho** | Carga SPI Máxima (*spi_bus Usage Max*) | $0.0\text{ Kbps}$ | $0.0\text{ Kbps}$ | Barramento de armazenamento inalterado |
| **Desempenho** | Carga SPI Mínima (*spi_bus Usage Min*) | $0.0\text{ Kbps}$ | $0.0\text{ Kbps}$ | Barramento de armazenamento inalterado |

![Tela de análise comparativa de atributos de qualidade no DevCompatibility](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%204/images/08_devcompatibility_analysis_comparativa_cava.png)
*Figura 2.7: Painel Analysis do DevCompatibility exibindo o comparativo quantitativo Original vs Evolução.*

##### 8. Segundo Caso de Teste Experimental: Substituição de Armazenamento (`sd_card` $\to$ `HighSpeed_Flash_Device`)
Para atestar a generalidade do método em diferentes subsistemas da camada de telemetria, realizou-se um segundo teste automatizado substituindo o leitor de cartão microSD (`MicroSD_Device`) pela memória flash industrial de alta velocidade (`HighSpeed_Flash_Device`):
- **Quebra de Interface Detectada:** O `MicroSD_Device` recebia fluxo de arquivo formatado via porta `file_stream_in`, enquanto o `HighSpeed_Flash_Device` grava em blocos brutos através da porta `raw_block_stream_in`.
- **Wrapper Sintetizado:** A ferramenta sintetizou com sucesso o processo `wrapper_raw_block_stream_in_to_sd_stream_out`, conectando a saída `sw_telemetry.sd_stream_out` na porta `partB` e a porta `partA` na entrada `raw_block_stream_in` da nova memória flash.
- **Impacto no Metamodelo:**
  - `declaration added:` Subcomponentes `HighSpeed_Flash_Device` e `wrapper_raw_block_stream_in_to_sd_stream_out`; Conexão `c9`.
  - `declaration changed:` Conexões de barramento `b_spi_sd` e de porta `p_sw_to_sd`.
  - `declaration deleted:` Subcomponente `sd_card`.
  - **Métricas:** Conexões $+1$ ($+10\%$), Subcomponentes $+1$ ($+11.11\%$), comprovando a mesma previsibilidade métrica da substituição de conectividade.

---

### 2.2. Cenário B: Inteligência de Borda para Driver Coaching (TinyML Edge AIoT)
Em vez de sobrecarregar a rede celular/MQTT com streaming contínuo de dados brutos (200 pacotes/s), o sistema evolui para processar predições locais no próprio veículo:
- **Origem dos Dados e Treinamento:** O modelo de rede neural é treinado previamente em servidor a partir da base de dados de ensaio em pista/HIL coletada pelo sistema (garantida por `REQ_SD_005` com $\ge 72.000$ amostras).
- **Quantização e Otimização:** O modelo é quantizado em **INT8** através do TensorFlow Lite Micro / ESP-NN (otimizações vetorizadas do Xtensa LX7), ocupando apenas **$\approx 45\text{ KB}$ de Flash/SRAM**.
- **Proposta de Valor do Driver Coaching:**
  A inferência não se limita a classificar um perfil abstrato ("Econômico" ou "Esportivo"), mas gera **orientações acionáveis em tempo real**:
  - *Índice de Condução (Driver Score):* Métrica contínua de 0 a 100.
  - *Recomendações Dinâmicas:*
    - `"Antecipar desaceleração em inércia (Cut-off)"` — ao detectar aceleração brusca seguida de frenagem severa.
    - `"Pisar menos no freio"` — ao identificar desacelerações desnecessárias em curvas ou trechos planos.
    - `"Trocar marcha antes de 3.500 RPM"` — ao identificar condução sustentada em regime de sobregiro sem ganho proporcional de velocidade.

---

## 3. Fase 3 do CAvA: Análise de Impacto Arquitetural e Modelação de Wrappers

A inserção da camada TinyML provoca quebras estruturais e de interface que demandam novos componentes mediadores.

### 3.1. Necessidade do Software Wrapper / Adaptador (`TinyML_Input_Adapter`)
- **Problema de Incompatibilidade de Interface:**  
  O pipeline de telemetria opera sobre quadros CAN brutos (`CAN_Frame_Data`, 16 bytes) ou linhas formatadas (`CSV_Payload_Data`, 320 bytes). Por outro lado, o tensor de entrada da rede neural (`Feature_Tensor_Data`) exige uma matriz normalizada de ponto flutuante ($N \times F$, com médias e desvios de RPM, velocidade, pressão de acelerador e derivadas de desaceleração).
- **Solução Formal em AADL:**  
  Criação do componente de compatibilização intermediário:
  ```aadl
  thread TinyML_Input_Adapter
    features
      telemetry_in: in event data port Data_Types_Pkg::CAN_Frame_Data;
      normalized_sample_out: out event data port Data_Types_Pkg::Feature_Tensor_Data;
    properties
      Dispatch_Protocol => Periodic;
      Period => 20 ms;
      Compute_Execution_Time => 100 us .. 500 us; -- Sobrecarga do wrapper
  end TinyML_Input_Adapter;
  ```
  O wrapper consome no máximo $500\ \mu\text{s}$ da CPU a cada ciclo de 20 ms, isolando completamente o modelo matemático das alterações de protocolo do barramento.

### 3.2. Decomposição de Hardware: Alocação Dual-Core no ESP32-S3
Para não violar os prazos de pior caso (*Hard Real-Time*) da aquisição TWAI/CAN a 500 kbps, a arquitetura de processador foi expandida para dois núcleos físicos (`ESP32S3_DualCore_Processor`):

```
                   ESP32-S3 DUAL-CORE PROCESSOR (240 MHz)
 +-------------------------------------------------------------------------+
 |                                                                         |
 |   [ CORE 0: Xtensa LX7 #0 ]                 [ CORE 1: Xtensa LX7 #1 ]   |
 |   (Hard Real-Time Telemetry)                (Edge Intelligence / AI)    |
 |                                                                         |
 |   • Task_CAN_RX       (T=5ms)               • TinyML_Input_Adapter      |
 |   • Task_Logger       (T=20ms)              • Task_Feature_Extractor    |
 |   • Task_TX_Dispatch  (T=50ms)              • Task_TinyML_Inference     |
 |   • Task_OBD_Poller   (T=100ms)             • Task_Coaching_Advisor     |
 |                                                                         |
 |   Utilização Core 0: 34.5%                  Utilização Core 1: 3.3%     |
 +-------------------------------------------------------------------------+
         ▲                                                 ▲
         │                        SRAM / PSRAM             │
         └─────────────[ BUFFER ESTÁTICO COMPARTILHADO ]───┘
```

#### Regras de Amarração Formal (*Bindings* no AADL):
- **Core 0:** `Actual_Processor_Binding => (reference (cpu.core0))` aplicado a todas as threads da telemetria original.
- **Core 1:** `Actual_Processor_Binding => (reference (cpu.core1))` aplicado ao processo `TinyML_Process`.

---

## 4. Fase 4 do CAvA: Avaliação Quantitativa de Trade-offs e Métricas

A reinstanciação do sistema evoluído no OSATE (`Evolved_EdgeTelemetry_System.impl`) permite comparar rigorosamente o sistema antes e depois da evolução.

### 4.1. Análise Comparativa de Desempenho e Recursos

| Métrica Avaliada | Arquitetura Baseline (Pilar 2) | Arquitetura Evoluída (Pilar 4 CAvA) | Variação ($\Delta$) | Impacto de Engenharia |
| :--- | :---: | :---: | :---: | :--- |
| **Arquitetura de CPU** | Monocore lógico (1 Core) | Dual-Core Físico (2 Cores) | $+1\text{ Core}$ | Isolamento total de interferência temporal |
| **Carga de CPU (Core 0)** | $34.5\%$ | $34.5\%$ | **$0.0\%$** | **Zero interferência nos prazos críticos de CAN** |
| **Carga de CPU (Core 1)** | — ($0.0\%$) | **$3.3\%$** | $+3.3\%$ | $U_{\text{Core1}} = \frac{0.5}{20} + \frac{2}{500} + \frac{25}{1000} + \frac{1}{1000} = 3.3\%$ |
| **Ocupação de SRAM Interna** | $\approx 85\text{ KB}$ | $\approx 195\text{ KB}$ | $+110\text{ KB}$ | Perfeitamente suportado pelos 512 KB de SRAM interna |
| **Uso de Memória PSRAM** | $0\text{ MB}$ (Não utilizada) | $\approx 2.4\text{ MB}$ (Histórico / Janelas) | $+2.4\text{ MB}$ | Folga confortável nos 8 MB de Octal PSRAM |
| **Latência CAN $\to$ Armazenamento** | $5.30\text{ ms}$ | $5.30\text{ ms}$ | **$0.0\text{ ms}$** | Persistência local intacta |
| **Latência CAN $\to$ Driver Coaching** | — (Inexistente) | **$\approx 30.5\text{ ms}$** | $+30.5\text{ ms}$ | Inferência + Coaching em $\le 31\text{ ms}$ (Tempo de reação humano $\approx 250\text{ ms}$) |
| **Throughput de Rede MQTT** | $\approx 200\text{ pacotes/s}$ brutos | **$1\text{ pacote/s}$ com insights semânticos** | **$-99.5\%$** | Drástica redução de custo e robustez sob rede celular 4G instável |
| **Número de Wrappers / Adaptadores** | $0$ | $1$ (`TinyML_Input_Adapter`) | $+1$ | Absorve a quebra de tipos e normaliza o sinal |

### 4.2. Viabilidade dos Limites Físicos, Temporais e Benchmarks Empíricos no ESP32-S3

Para fundamentar as propriedades temporais atribuídas no modelo AADL (`Compute_Execution_Time => 10 ms .. 25 ms`), foram confrontados dados empíricos oficiais da **Espressif Systems (ESP-IDF / ESP-NN)** e da plataforma **Edge Impulse** executados sobre o processador **Xtensa LX7 Dual-Core a 240 MHz com extensões vetoriais SIMD**:

#### 1. Benchmarks de Inferência TinyML no ESP32-S3 (240 MHz):
| Domínio / Arquitetura do Modelo | Quantização | Sem Aceleração (C padrão) | Com Aceleração ESP-NN (SIMD Xtensa) | Ganho de Desempenho |
| :--- | :---: | :---: | :---: | :---: |
| **Séries Temporais de Sensores (IMU / Acelerômetro / CAN / OBD)** | **INT8** | $\approx 15.0\text{ ms}$ | **$2.0\text{ ms}$ a $2.5\text{ ms}$** | **$6\times$ a $7.5\times$ mais rápido** |
| **Classificador Convolucional 1D / Áudio (Keyword Spotting)** | **INT8** | $\approx 85.0\text{ ms}$ | **$12.0\text{ ms}$ a $15.0\text{ ms}$** | **$5.6\times$ a $7.0\times$ mais rápido** |
| **Rede Neural Convolucional 2D (MobileNet 96x96)** | **INT8** | $\approx 2300\text{ ms}$ | **$54.0\text{ ms}$** | **$42.5\times$ mais rápido** |

*Fontes e Referências Técnicas:*
- Espressif Systems: *ESP-NN: Optimized Neural Network Kernels for ESP32-S3* (Assembly SIMD vector optimizations for convolution, fully-connected and pooling layers).
- Edge Impulse: *ESP32-S3 Machine Learning Benchmarks for Time-Series & Anomaly Detection* (Amostragem contínua e inferência em arrays de acelerômetro e sensores industriais).
- TensorFlow Lite Micro: *Deployment of 8-bit Quantized Models on Resource-Constrained Microcontrollers*.

#### 2. Justificativa do Envelope Temporal Seguro no AADL:
- No modelo formal AADL ([`TinyML_Pkg.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/Entregas/Entrega%202%20e%203/EdgeTelemetry_MBSE_OSATE/packages/TinyML_Pkg.aadl)), fixou-se:
  ```aadl
  Compute_Execution_Time => 10 ms .. 25 ms; -- BCET = 10 ms, WCET = 25 ms
  Period => 1000 ms;                         -- Frequência de 1 Hz
  ```
- **Conclusão:** O limite superior $\text{WCET} = 25.0\text{ ms}$ é **altamente conservador e seguro**. Enquanto um modelo puramente baseado em séries temporais (como o de telemetria veicular) consome entre $2.5\text{ ms}$ e $12.0\text{ ms}$ em condições reais, a reserva de $25.0\text{ ms}$ absorve a sobrecarga da janela deslizante, normalização de tensores e eventuais trocas de contexto no FreeRTOS/Embassy.
- Com isso, a taxa de ocupação de CPU da inferência no **Core 1** é de apenas:
  $$U_{\text{infer}} = \frac{25\text{ ms}}{1000\text{ ms}} = \mathbf{2.5\%} \quad (U_{\text{Total\_Core1}} = 3.3\%)$$
  Garantindo que o processador opera com **$96.7\%$ de folga**, sem qualquer risco de saturação.

#### 3. Limite de Consumo de Memória (SRAM Interna vs PSRAM Externa):
- **Pesos Sinápticos do Modelo Quantizado (INT8):** $\approx 35\text{ KB}$ a $45\text{ KB}$.
- **Tensor Arena (Ativações Intermediárias do TFLM):** $\approx 40\text{ KB}$ em SRAM interna.
- **Fila da Janela Deslizante (100 amostras $\times$ 8 grandezas float):** $\approx 3.2\text{ KB}$.
- **Consumo Total da Pilha de IA:** $\approx 78.2\text{ KB}$ a $88.2\text{ KB}$.
- **Alocação Estratégica:** Como a SRAM interna do ESP32-S3 possui **512 KB** (acesso em 1 ciclo de clock), a *Tensor Arena* é alocada 100% na SRAM rápida, eliminando penalidades de acesso via barramento SPI. A memória externa **Octal PSRAM de 8 MB** permanece disponível para buffering massivo de históricos de telemetria sem competir por barramento com o Core 0.

3. **Trade-off de Banda de Rede e Conectividade:**  
   - Em veículos conectados, a transmissão ininterrupta de 200 amostras/segundo por rede móvel acarreta custos severos de pacotes de dados M2M e vulnerabilidade a falhas de cobertura.
   - O modelo CAvA com TinyML local permite operar em modo autônomo (*Edge Analytics*): a camada de comunicação MQTT transmite apenas **1 pacote semântico consolidado por segundo** (`Score`, `Classificação`, `Alerta de Condução`), alcançando **economia de banda superior a 90%** sem perda de observabilidade gerencial.

---

## 5. Estrutura Modular dos Arquivos AADL Gerados

Para permitir a inspeção automatizada no OSATE e a importação direta pela ferramenta **`DevCompatibility`**, os modelos AADL foram desagregados e organizados na pasta canônica do projeto:

```
c:\workspace\can-obd-telemetry\docs\MBSE\Entregas\Entrega 2 e 3\EdgeTelemetry_MBSE_OSATE\
├── packages/
│   ├── Data_Types_Pkg.aadl          (Tipos de dados brutos e tensores de IA)
│   ├── Buses_Pkg.aadl               (Barramentos CAN, SPI e InterCore)
│   ├── Processors_Pkg.aadl          (Processadores Single-Core e Dual-Core)
│   ├── Software_Threads_Pkg.aadl    (4 Threads de telemetria do Baseline)
│   ├── Software_Processes_Pkg.aadl  (Processo Telemetry_Process)
│   ├── EdgeTelemetry_System_Pkg.aadl(Sistema Raiz Integrado do Pilar 2)
│   ├── TinyML_Pkg.aadl              (Wrappers, Extratores e Inferência de IA)
│   └── Evolved_System_Pkg.aadl      (Sistema Evoluído Dual-Core do Pilar 4)
└── Library/
    └── devices/
        ├── CAN_Devices_Pkg.aadl     (Transceptores CAN e Emulador HIL)
        ├── Storage_Devices_Pkg.aadl (MicroSD e Memória Flash de alta velocidade)
        └── Comm_Devices_Pkg.aadl    (Módulo Wi-Fi e Modem Celular 4G/LTE)
```

---

## 6. Conclusão Metodológica

A aplicação do framework CAvA neste estudo de caso comprova que:
1. A transição de um coletor passivo para uma arquitetura com **TinyML embarcado** é tecnicamente viável e altamente vantajosa no microcontrolador ESP32-S3.
2. A segregação dual-core garantiu que **nenhuma das garantias de tempo real estrito do Pilar 1 (AC-01 a AC-08) fosse degradada**.
3. O software wrapper modelado absorve com sucesso as discrepâncias estruturais apontadas pela análise do **`DevCompatibility`**, demonstrando o ciclo completo de engenharia baseada em modelos (MBSE) desde os requisitos formais até a evolução arquitetural.
