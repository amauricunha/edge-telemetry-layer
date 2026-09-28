# Dúvidas e Resoluções Formais (FRET / MBSE)

## 1. Precisamos declarar variáveis, como perfil de direção?
- **Dúvida:** `dbc_frames_emitted` teria que ter o valor de cada frame calculado pela função de cálculo da onda lá? O cálculo da onda está no sistema e precisa?
- **Resolução:**
  - **Sim, foram decompostos formalmente em dois requisitos encadeados:**
    1. [`REQ_EMU_003`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md#L94-L109): Modela o **cálculo da dinâmica física na CPU** (`physics_model_updated within 1 MILLISECOND`). A computação matemática contínua (senoide em PROGMEM) é interna da CPU e não é modelada com fórmulas trigonométricas no FRET (incompatível com NuSMV/JKind), mas seu **consumo de CPU / deadline** é formalizado com precisão.
    2. [`REQ_EMU_004`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md#L111-L126): Modela a **emissão física dos frames no barramento** (`dbc_frames_emitted within 2 MILLISECOND`) acionada logo após `physics_model_updated`.
  - A variável `active_profile` foi declarada e mapeada como `Output` (Integer: 1=Eco, 2=Normal, 3=Sport) em [`REQ_EMU_001`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md#L62-L76) e [`REQ_EMU_002`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md#L78-L92).

---

## 2. Declarar variáveis e cálculos / requisitos do perfil padrão e sintético
- **Dúvida:** O sistema tem que calcular a senoide de perfil para gerar dados sintéticos interligado com o perfil recebido, ou se não recebeu, acho que o padrão é econômico? Como declaramos? Já está declarado ou é novo requisito?
- **Resolução:**
  - **Identificado gap no SRS e adicionado no FRET como [`REQ_EMU_001`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md#L62-L76):**
    - No firmware real do Arduino (`firmware/uno_ecu_emulator/src/main.cpp`), o perfil padrão é **Normal (2)** (`volatile uint8_t perfil_atual = 2`), e não econômico.
    - O requisito formal criado foi:
      `in boot_mode upon boot_trigger & mcp2515_hardware_up the uno_ecu_emulator shall within 10 MILLISECOND satisfy can_bus_operational & active_profile = 2`
    - Comutação dinâmica via comando CAN 0x010 é coberta por [`REQ_EMU_002`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md#L78-L92) (`active_profile = commanded_profile within 10 MILLISECOND`).

---

## 3. Emulador e Coletor têm que estar em projetos separados? Eles se integram ou ficam no mesmo projeto em componentes separados?
- **Dúvida:** O emulador Arduino UNO e o coletor ESP32-S3 precisam estar em arquivos de projetos diferentes no FRET, ou no mesmo projeto apenas com componentes formais distintos?
- **Resolução Arquitetural (Ambas as Abordagens são Válidas e Equivalentes no Solver):**
  - **NÃO é obrigatório estarem no mesmo projeto!** Você pode perfeitamente mantê-los como **dois projetos separados no FRET**, ou como **um projeto unificado com componentes distintos**. A matemática e o resultado no solver Kind 2 são **100% idênticos**, pois o FRET analisa cada componente de forma estritamente isolada.
  - **Como eles se integram formalmente:**
    - Os dois microcontroladores são nós fisicamente desacoplados (chips diferentes, memórias diferentes e linguagens diferentes: C++ bare-metal vs Rust no_std).
    - O único meio de integração entre eles é o **barramento CAN 2.0B (500 kbps)**.
    - Formalmente no FRET, cada microcontrolador enxerga o outro como **Ambiente Externo (Inputs e Outputs de barramento)**:
      - O ESP32-S3 enxerga as respostas do Arduino como entradas de barramento (`can_frame_arrived`, `obd_response_0x7e8`).
      - O Arduino UNO enxerga os comandos do ESP32 como entradas de barramento (`obd_request_0x7df_received`, `profile_cmd_0x010_received`).
    - O provador formal Kind 2 não cruza componentes nem projetos na verificação monolítica; ele verifica se as saídas do componente satisfazem suas próprias propriedades dadas as suas entradas.
  - **Comparativo entre as Duas Abordagens:**
    1. **Abordagem de 2 Projetos Separados (Visão de Produto / Firmware Independente):**
       - **Projeto 1:** `esp32s3_collector` ([`esp32s3_collector_req_var.json`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/esp32s3_collector_req_var.json)) — 40 requisitos dos 8 subsistemas do coletor.
       - **Projeto 2:** `uno_ecu_emulator` ([`uno_ecu_emulator_req_var.json`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/uno_ecu_emulator_req_var.json)) — 8 requisitos do emulador de ECU.
       - **Vantagem:** Mapeamento direto de 1 para 1 com os repositórios/firmwares físicos. Se você evoluir a simulação do Arduino para novos PIDs, o projeto do coletor permanece 100% intacto.
    2. **Abordagem de 1 Projeto Unificado (Visão de Bancada HIL / Sistema de Sistemas):**
       - **Projeto:** `EdgeTelemetryLayer` ([`EdgeTelemetryLayer_req_var.json`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var.json)) — 48 requisitos agrupados pelos mesmos componentes.
       - **Vantagem:** Facilita a apresentação acadêmica de um único dashboard integrado para o professor, contendo a especificação formal completa da bancada Hardware-in-the-Loop em um só arquivo.
  - **Disponibilização no Repositório:** O repositório agora fornece **ambos os formatos** para que você possa importar no FRET da forma que for mais conveniente para a sua apresentação.

---

## 4. Por que nem tudo é `immediately`? Como funcionam os requisitos de tempo e capacidade (ex: REQ_COM_002)?
- **Dúvida:** No `REQ_COM_002`, dizemos que publica imediatamente ao atingir a capacidade máxima, ou há capacidade máxima ou tempo em milissegundos? Por que não usar `immediately` para tudo?
- **Resolução Arquitetural (A Descoberta dos Exemplos Oficiais da NASA em `FRET_docs`):**
  - **O que a NASA documenta formalmente:**
    A documentação oficial do FRET (`timing.md`) alerta expressamente: *"Time units are currently ignored: they are not checked, converted or reasoned about by FRET. To FRET, all the units mean 'time points' (or 'time steps')."*
    Para cada passo $N$ em `within N MILLISECOND`, o FRET instancia uma cadeia desenrolada de $N$ registradores de atraso no Lustre (`pre X`). Valores grandes ($N \ge 128$) causam explosão combinatória no solver Z3 e timeout de 15 minutos!
  - **Por que NÃO substituir tudo cegamente por `immediately`:**
    Substituir tudo por `immediately` eliminaria os prazos de execução (Worst-Case Execution Time - WCET) e violaria os critérios de aceite de tempo real (AC-01 a AC-08). A física de uma CPU e de uma pilha de rede TCP/IP exige tempo finito para executar.
  - **A Arquitetura em Três Camadas (Consonante com os Exemplos da NASA):**
    1. **Hard Real-Time WCET ($1 \le N \le 10\text{ ms}$):**
       Mantém `within N MILLISECOND`. Como $N \le 10$, gera apenas de 1 a 10 registradores de atraso, provando o WCET em `< 0,05s` no Kind 2.
    2. **Temporizadores Físicos Longos ($\ge 128\text{ ms}$, segundos, minutos):**
       Adota o **NASA Timer Handshake Pattern** (como no case study oficial `LiquidMixer` com timers de 60s e 120s):
       - Disparo do timer: `immediately satisfy timer_start` (Output).
       - Reação à interrupção: `upon timer_expired` (Input) $\rightarrow$ executa ação dentro do WCET ($1 \dots 10\text{ ms}$).
    3. **Gatilhos Duplos de Bufferização (Capacidade vs. Timeout Periódico):**
       Em gateways telemáticos e gravadores de borda, o esvaziamento de buffers NUNCA depende apenas de capacidade ou apenas de tempo; depende de **ambos**:
       - **Gatilho de Capacidade (Lote Cheio):**
         - `REQ_COM_002`: `in connected_mode upon binary_batch_full the esp32_telemetry shall within 10 MILLISECOND satisfy mqtt_batch_published` (Ao atingir 150 frames, despacha compulsoriamente em até 10 ms para não perder dados por overflow).
         - `REQ_LOG_004`: `in active_session when buffer_occupancy >= 3584 the esp32_logger shall immediately satisfy flush_signal_emitted` (Buffer do MicroSD com 85% de ocupação / 7 setores).
       - **Gatilho de Timeout / Dreno Periódico (Baixa Taxa de Dados):**
         - `REQ_COM_004`: `in active_session upon dispatch_cycle_50ms the esp32_telemetry shall within 3 MILLISECOND satisfy data_packets_dispatched` (A cada 50 ms, qualquer resíduo é transmitido em até 3 ms, impedindo que dados fiquem retidos se o lote não encher).
         - `REQ_LOG_005`: `in active_session upon flush_timer_2s_expired the esp32_logger shall within 10 MILLISECOND satisfy pending_bytes_flushed` (A cada 2 s de inatividade, descarrega resíduos para o MicroSD).