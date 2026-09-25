# Especificação de Requisitos de Sistema e Software (SRS)
## Sistema: Edge Telemetry Layer (ESP32-S3 no_std Embassy & Emulador de ECU Arduino UNO R3)

---

## 1. Introdução e Visão Geral do Sistema

### 1.1 Propósito e Escopo
O **Edge Telemetry Layer** é um sistema embarcado de tempo real crítico para aquisição, processamento, diagnóstico e transmissão de telemetria veicular. O sistema opera em bancada Hardware-in-the-Loop (HIL) composta por dois nós de processamento interligados via barramento Controller Area Network (CAN 2.0B) a 500 kbps:

1. **Nó Coletor e Gateway de Borda (ESP32-S3):** Implementado em linguagem Rust sem biblioteca padrão (`no_std`) sobre o runtime assíncrono `Embassy` e arquitetura em camadas inspirada no padrão automotivo AUTOSAR (MCAL, BSW, RTE e Aplicação). É responsável por capturar passivamente o tráfego CAN, interrogar ativamente a ECU via protocolo OBD-II (ISO 15765-4), serializar dados em formato tabular CSV estático, gerenciar armazenamento local persistente em cartão MicroSD (FAT32 via SPI), transmitir pacotes binários compactos para nuvem via Wi-Fi/MQTT e supervisionar a integridade operacional (Watchdog e recuperação de Bus-Off).
2. **Emulador de ECU Automotiva (Arduino UNO R3):** Implementado em C++ bare-metal sobre microcontrolador ATmega328P acoplado a um controlador CAN MCP2515 e transceptor TJA1050 via barramento SPI. É responsável por simular o comportamento de uma unidade eletrônica de motor de combustão interna, emitindo ciclicamente frames de broadcast conformes a uma especificação DBC mínima e respondendo em tempo real às requisições de diagnóstico de seis Parâmetros Padronizados (PIDs) veiculares.

### 1.2 Arquitetura em Camadas do Coletor (ESP32-S3)
A arquitetura de software no coletor segue uma separação estrita de responsabilidades:
- **MCAL (Microcontroller Abstraction Layer):** Drivers de acesso direto aos periféricos de silício do ESP32-S3 (`mcal::twai` para o controlador CAN nativo e `mcal::spi_sd` para o barramento SPI2 com cartão SD).
- **BSW (Basic Software):** Serviços de infraestrutura para gerenciamento de memória e armazenamento (`bsw::bsw_mem`), serviços de rede e protocolo de transporte (`bsw::bsw_com`) e serviços de supervisão e recuperação de barramento (`bsw::bsw_diag`).
- **RTE (Runtime Environment):** Barramentos de comunicação inter-tarefas baseados em canais assíncronos estáticos sem alocação dinâmica (`TELEMETRY_CHANNEL` com capacidade para 32 mensagens e `CAN_CMD_CHANNEL` para comandos).
- **Camada de Aplicação:** Tarefas concorrentes do pipeline de processamento:
  - `task_can_rx`: recepção orientada a eventos de todo o tráfego do barramento TWAI.
  - `task_obd_poller`: coordenação cíclica das interrogações ativas de diagnóstico.
  - `task_logger`: consumo de amostras, serialização CSV e máquina de estados de fallback.
  - `task_sd_writer` / `task_tx_dispatch`: fatiamento e despacho físico para persistência ou rede.
  - `task_watchdog`: monitoramento de travamentos e recuperação elétrica de barramento.

---

## 2. Modos de Operação do Sistema

O sistema opera sob estados e modos mutuamente exclusivos:

1. **Modo Inicialização (Boot):** Configuração de clocks, periféricos TWAI, barramento SPI, montagem do sistema de arquivos FAT32, alocação estática de heap e spawn das tarefas no escalonador assíncrono.
2. **Modo Standby (Aguardando Sessão):** Barramento CAN em escuta, enlace Wi-Fi monitorado, sem gravação de telemetria no cartão SD ou publicação no tópico de telemetria bruta.
3. **Modo Sessão Ativa (Gravação e Coleta):** Todas as tarefas operam em regime nominal; frames DBC e OBD-II são estruturados, persistidos no cartão de memória e/ou transmitidos via MQTT. O arquivo de sessão ativo segue o padrão sequencial `S_XXXX.CSV`.
4. **Modo Fallback Offline (Wi-Fi Desconectado):** Em caso de perda do enlace de rede ou falha de conexão com o broker MQTT, o sistema desativa tentativas de transmissão em rede e redireciona compulsoriamente 100% dos dados para o buffer do cartão MicroSD, mantendo um backlog em RAM caso o SD falhe temporariamente.
5. **Modo Falha de Barramento (Bus-Off Recovery):** Estado ativado quando o controlador TWAI atinge nível crítico de erros elétricos no barramento físico. Todas as transmissões e leituras de barramento são pausadas cooperativamente por uma janela normatizada de 128 ms até que a rotina de reestabelecimento em baixo nível execute a recuperação física.
6. **Modo Replay sob Demanda:** Estado no qual a sessão ativa é suspensa e um arquivo histórico previamente gravado no cartão de memória é lido em blocos e transmitido via streaming MQTT para reprocessamento externo.

---

## 3. Requisitos Funcionais do Sistema

### 3.1 Emulação de ECU e Tráfego CAN Passivo (Arduino UNO R3)

#### [REQ-SYS-01] Emissão Cíclica de Grandezas Dinâmicas do Motor (DBC)
- **Descrição do Comportamento:** O emulador de ECU deve atualizar periodicamente as variáveis físicas simuladas do motor (velocidade do veículo, rotação do motor, posição do acelerador, carga calculada, temperatura do líquido de arrefecimento e fluxo de massa de ar admitido) e transmiti-las no barramento CAN em conformidade com o dicionário de sinais DBC.
- **Condição Disparadora:** Base de tempo periódica gerada por interrupção de hardware (Timer1 a cada 50 ms / 20 Hz).
- **Grandezas e Periodicidades Emitidas:**
  - **ID `0x200`:** Posição do acelerador (0 a 100%, 1 byte) e carga do motor (0 a 100%, 1 byte). Período: a cada **50 ms**.
  - **ID `0x100`:** Velocidade linear (0 a 250 km/h, 2 bytes inteiros big-endian) e rotação do motor (0 a 8000 rpm, 2 bytes inteiros big-endian). Período: a cada **100 ms** (a cada 2 ciclos do Timer1).
  - **ID `0x300`:** Temperatura do líquido de arrefecimento (-40 a 215 °C codificada com offset de +40, 1 byte). Período: a cada **1000 ms** (a cada 20 ciclos do Timer1).
- **Perfis de Condução Simulados:** A dinâmica das variáveis deve ser calculada a partir de tabelas senoidais pré-computadas em memória de programa (`PROGMEM`), variando entre três perfis operacionais:
  - *Econômico (Perfil 1):* Velocidade 40–80 km/h, Rotação 1200–2500 rpm, Acelerador 10–30%.
  - *Normal (Perfil 2):* Velocidade 60–120 km/h, Rotação 2000–4000 rpm, Acelerador 20–60%.
  - *Esportivo (Perfil 3):* Velocidade 80–160 km/h, Rotação 3500–6500 rpm, Acelerador 50–100%.
- **Comutação de Perfil de Condução:** Ao receber um frame CAN com identificador `0x010` contendo no byte 0 o código do perfil (`0x01`, `0x02` ou `0x03`), o emulador deve atualizar o perfil ativo de simulação no próximo ciclo do Timer1.

#### [REQ-SYS-02] Processamento de Interrupção de Recepção no Emulador
- **Descrição do Comportamento:** O emulador deve verificar a presença de mensagens de entrada no controlador MCP2515 através de rotina periódica de alta frequência governada pelo Timer2 (base de tempo de 1 ms). Ao detectar uma mensagem com ID `0x7DF` (requisição OBD-II) ou ID `0x010` (troca de perfil), o emulador deve transferir os dados da mensagem para variáveis de troca atômica e sinalizar uma flag pendente para processamento fora da rotina de interrupção.
- **Restrição de Execução:** A rotina de interrupção de 1 ms não deve bloquear a CPU por mais de 50 µs em transferências SPI.

---

### 3.2 Aquisição e Recepção no Nó Coletor (ESP32-S3)

#### [REQ-SYS-03] Recepção Contínua e Assíncrona via TWAI
- **Descrição do Comportamento:** O nó coletor deve manter o periférico TWAI operando em modo assíncrono com barramento em 500 kbps, capturando todos os quadros trafegados (DBC passivo e respostas de diagnóstico) sem perda de sincronismo de clock.
- **Condição Disparadora:** Chegada de pacote CAN no buffer interno do hardware.
- **Comportamento e Encaminhamento:**
  - Registrar imediatamente o timestamp do sistema em milissegundos.
  - Diferenciar a origem da mensagem: se o identificador for `0x7E8`, tratar como resposta ativa de diagnóstico; se o identificador for `0x100`, `0x200` ou `0x300`, tratar como telemetria passiva DBC.
  - Decodificar os bytes brutos aplicando as respectivas escalas físicas.
  - Inserir o registro decodificado (`TelemetryFrame`) no canal assíncrono da RTE (`TELEMETRY_CHANNEL`).
- **Tratamento de Saturação:** Se o canal da RTE atingir sua capacidade máxima (32 elementos), a tarefa de recepção não deve travar; deve descartar o pacote excedente, incrementar o contador global de overflow e continuar esvaziando a FIFO do controlador TWAI.

#### [REQ-SYS-04] Cumprimento da Taxa de Recepção de Quadros (AC-01)
- **Descrição do Comportamento:** O sistema deve assegurar que, em regime nominal de operação contínua a 500 kbps, a taxa de perda total de quadros CAN permaneça estritamente inferior a 1%, garantindo taxa de recepção e entrega com sucesso igual ou superior a 99% do volume nominal gerado pela bancada.

---

### 3.3 Diagnóstico Ativo OBD-II (ISO 15765-4)

#### [REQ-SYS-05] Polling Cíclico de Diagnóstico a 10 Hz
- **Descrição do Comportamento:** Durante qualquer sessão de telemetria ativa, o nó coletor deve interrogar ciclicamente o barramento veicular através de requisições funcionais OBD-II Modo 01 direcionadas ao identificador CAN `0x7DF`.
- **Frequência de Interrogação:** O disparo de requisições deve ocorrer a cada **100 ms** (frequência de 10 Hz), com tolerância de temporização de ±2 ms.
- **Formato da Requisição (8 bytes):**
  - Byte 0: `0x02` (comprimento do payload de diagnóstico).
  - Byte 1: `0x01` (Modo 01 — dados em tempo real).
  - Byte 2: Código do PID solicitado.
  - Bytes 3–7: `0x00` (padding de preenchimento obrigatório).

#### [REQ-SYS-06] Escalonamento Round-Robin dos Parâmetros de Diagnóstico
- **Descrição do Comportamento:** O coletor deve alternar circularmente o parâmetro requisitado a cada ciclo de 100 ms sobre a lista fechada de seis PIDs suportados:
  1. `0x0D`: Velocidade do Veículo (`speed_kmh`, km/h)
  2. `0x0C`: Rotação do Motor (`rpm`, rotações por minuto)
  3. `0x11`: Posição da Válvula de Aceleração (`throttle_pct`, porcentagem)
  4. `0x04`: Carga Calculada do Motor (`engine_load_pct`, porcentagem)
  5. `0x10`: Fluxo de Massa de Ar admitido (`maf_g_s`, gramas por segundo)
  6. `0x05`: Temperatura do Líquido de Arrefecimento (`coolant_temp_c`, graus Celsius)
- **Período de Retorno:** Cada PID individual deve ser consultado exatamente uma vez a cada **600 ms** (6 PIDs × 100 ms).

#### [REQ-SYS-07] Codificação e Emissão de Resposta pela ECU Emulada
- **Descrição do Comportamento:** Ao receber a requisição funcional `0x7DF`, o emulador Arduino UNO R3 deve validar se o modo é `0x01`, codificar a grandeza simulada correspondente na mensagem de resposta física `0x7E8` e transmiti-la no barramento CAN.
- **Fórmulas Padronizadas de Codificação da Resposta:**
  - `0x04` (Engine Load): `Byte3 = (load_pct * 255) / 100` (1 byte de dado).
  - `0x05` (Coolant Temp): `Byte3 = temp_celsius + 40` (1 byte de dado).
  - `0x0C` (Engine RPM): `Byte3 = (rpm * 4) >> 8`, `Byte4 = (rpm * 4) & 0xFF` (2 bytes de dado).
  - `0x0D` (Vehicle Speed): `Byte3 = speed_kmh` (1 byte de dado).
  - `0x10` (MAF Air Flow): `Byte3 = (maf * 100) >> 8`, `Byte4 = (maf * 100) & 0xFF` (2 bytes de dado).
  - `0x11` (Throttle Pos): `Byte3 = (throttle_pct * 255) / 100` (1 byte de dado).
- **Formato da Resposta:** Byte 0 com comprimento de bytes úteis (`0x03` ou `0x04`), Byte 1 com modo espelhado com offset `0x40` (`0x41`), Byte 2 com o PID espelhado, Bytes seguintes com os dados codificados e bytes restantes completados com `0x00`.

#### [REQ-SYS-08] Limite de Latência Média de Resposta de Diagnóstico (AC-02)
- **Descrição do Comportamento:** O emulador de ECU deve processar e colocar no barramento a resposta física de diagnóstico em tempo hábil, de forma que a latência média de ida e volta (medida pelo coletor entre o instante de transmissão de `0x7DF` e a chegada do primeiro bit de `0x7E8`) seja estritamente inferior a **10 ms**.

#### [REQ-SYS-09] Limite de Dispersão e Jitter de Resposta de Diagnóstico (AC-03)
- **Descrição do Comportamento:** O emulador de ECU deve manter a regularidade temporal de suas respostas sob concorrência de timers, garantindo que o desvio padrão da latência de resposta OBD-II (`obd_latency_ms`) ao longo de uma sessão de ensaio não ultrapasse o limiar de **3 ms**.

#### [REQ-SYS-10] Monitoramento e Tratamento de Timeout de Diagnóstico
- **Descrição do Comportamento:** Ao emitir uma requisição `0x7DF`, o coletor deve armar uma janela de espera de **50 ms**. Caso a resposta física `0x7E8` não seja detectada dentro deste intervalo:
  - Declarar condição de timeout para a interrogação atual.
  - Manter a flag de resposta como não-recebida.
  - Registrar evento de diagnóstico de advertência no console de log (`OBD_TIMEOUT`).
  - Aguardar o tempo restante para completar os 100 ms nominais do ciclo sem realizar novas transmissões precipitadas, avançando normalmente para o próximo PID da lista circular.

---

### 3.4 Estruturação de Dados, Serialização e Bufferização em Memória

#### [REQ-SYS-11] Serialização Tabular Determinística em CSV sem Alocação Dinâmica
- **Descrição do Comportamento:** Todo dado capturado e validado deve ser serializado pela tarefa de registro em uma linha de texto no padrão CSV, contendo rigorosamente 11 colunas delimitadas por vírgulas, terminadas em quebra de linha `\n`.
- **Restrição de Alocação de Memória:** A formatação da string CSV deve utilizar exclusivamente buffers estáticos alocados em tempo de compilação (`heapless::String<320>`), sem invocar o alocador dinâmico de heap (`malloc`, `alloc` ou formatação da biblioteca padrão `std`), prevenindo fragmentação de memória durante execuções prolongadas.
- **Esquema Padronizado das 11 Colunas:**
  1. `timestamp_ms`: Inteiro positivo representando os milissegundos decorridos desde o início da sessão ou época Unix absoluta.
  2. `source`: Identificador de proveniência (`CAN_DBC`, `OBD_PID` ou `DIAG`).
  3. `can_id`: Identificador hexadecimal do frame (`0x100`, `0x200`, `0x300`, `0x7E8` ou `0x000`).
  4. `speed_kmh`: Ponto flutuante formatado com 1 casa decimal (ex: `60.5`), ou vazio se não aplicável ao frame.
  5. `rpm`: Ponto flutuante formatado como valor inteiro (ex: `2100`), ou vazio se não aplicável.
  6. `throttle_pct`: Ponto flutuante com 1 casa decimal (ex: `25.1`), ou vazio.
  7. `engine_load_pct`: Ponto flutuante com 1 casa decimal (ex: `28.3`), ou vazio.
  8. `maf_g_s`: Ponto flutuante com 2 casas decimais (ex: `12.45`), ou vazio.
  9. `coolant_temp_c`: Ponto flutuante com 1 casa decimal (ex: `87.0`), ou vazio.
  10. `session_label`: Rótulo textual do perfil ativo (`Economico`, `Normal` ou `Esportivo`).
  11. `obd_latency_ms`: Ponto flutuante com 3 casas decimais (ex: `4.215`) presente exclusivamente em linhas de origem `OBD_PID`, ou vazio em frames passivos DBC.

#### [REQ-SYS-12] Garantia de Integridade Estrutural do Dataset (AC-04)
- **Descrição do Comportamento:** O sistema deve garantir que 100% das linhas de dados do dataset gerado possuam todos os quatro campos mandatórios de rastreabilidade preenchidos (`timestamp_ms`, `source`, `can_id` e `session_label`), e que nenhum campo numérico possua caracteres corrompidos, deslocamentos de vírgula ou campos vazios indevidos que inviabilizem o parsing por ferramentas de análise de dados.

#### [REQ-SYS-13] Buffer Circular Intermediário em Memória SRAM
- **Descrição do Comportamento:** Para desacoplar a frequência de chegada de mensagens da latência de gravação física no cartão MicroSD, o serviço de memória BSW deve gerenciar um buffer estático em anel na memória SRAM interna com dimensão de exatamente **4096 bytes** (equivalente a 8 setores físicos FAT32 de 512 bytes).
- **Acesso Concorrente:** A escrita no buffer em anel deve ser protegida por seção crítica de rápida execução (tempo de retenção inferior a 50 µs).

#### [REQ-SYS-14] Políticas de Esvaziamento do Buffer (Flush Híbrido)
- **Descrição do Comportamento:** A transferência do conteúdo em RAM para o cartão de armazenamento físico deve ser acionada por dois gatilhos independentes:
  1. *Esvaziamento por Limiar de Ocupação:* No momento em que o volume de bytes acumulados atingir ou ultrapassar **3584 bytes** (85% da capacidade do buffer), emitir imediatamente um sinal assíncrono de despertar para a tarefa de gravação.
  2. *Esvaziamento por Temporizador:* Independentemente do volume acumulado, a cada **2 segundos** de inatividade de escrita, a tarefa de gravação deve acordar e persistir quaisquer bytes pendentes no arquivo ativo.

#### [REQ-SYS-15] Escrita Fatiada em Chunks e Preempção Cooperativa
- **Descrição do Comportamento:** Durante a operação física de escrita no cartão SD através da interface SPI, o bloco de dados a ser descarregado não deve ser transmitido de forma contínua em rajada única.
- **Fatiamento em 256 Bytes:** O serviço BSW deve fracionar a gravação em pedaços de no máximo **256 bytes**.
- **Cooperação:** Após a transmissão de cada fatia de 256 bytes, a tarefa de gravação deve compulsoriamente ceder o controle da CPU ao escalonador através de preempção cooperativa (`yield_now()`), assegurando que o tempo de bloqueio contínuo do barramento não ultrapasse **0,76 ms**, impedindo o transbordo da FIFO do controlador TWAI.

---

### 3.5 Persistência em Cartão MicroSD e Gerenciamento de Arquivos

#### [REQ-SYS-16] Inicialização e Montagem FAT32
- **Descrição do Comportamento:** O coletor deve inicializar o leitor de cartão MicroSD via barramento SPI dedicado (pinos CS no GPIO 10, MOSI no GPIO 11, CLK no GPIO 12 e MISO no GPIO 13), detectar a presença física do cartão e montar o volume formatado em FAT32 na raiz `/`.
- **Detecção de Falha:** Caso o cartão não seja detectado no slot ou a montagem falhe, o sistema deve registrar advertência crítica no console, sinalizar o status de erro na flag global `SD_OK = false` e operar em modo de contingência em rede.

#### [REQ-SYS-17] Ciclo de Vida e Transição Atômica de Sessões
- **Descrição do Comportamento:** Ao receber comando para abertura de nova viagem ou rotação de arquivo, o sistema deve executar a troca de arquivos de maneira atômica e segura contra perdas:
  1. Forçar o esvaziamento síncrono imediato (`flush_sync`) de qualquer byte remanescente no buffer em RAM para o arquivo CSV atual.
  2. Fechar o descritor do arquivo anterior.
  3. Abrir o próximo arquivo sequencialmente indexado no formato de 4 dígitos decimais: `S_0001.CSV`, `S_0002.CSV`, e assim sucessivamente.
  4. Gravar obrigatoriamente a linha de cabeçalho na **linha 1** do novo arquivo.
  5. Gravar obrigatoriamente a linha de inicialização de sistema (`BOOT`) na **linha 2** do novo arquivo, registrando a data/hora base e os metadados de configuração.
  6. Liberar a escrita das amostras de telemetria apenas a partir da **linha 3**.

#### [REQ-SYS-18] Temporização de Sessão Automática
- **Descrição do Comportamento:** Caso um comando de início de sessão contenha parâmetro de duração temporizada (ex: 10 minutos), o sistema deve registrar a marca temporal inicial e monitorar o tempo decorrido. Ao atingir a duração estipulada, o sistema deve:
  - Injetar uma linha de diagnóstico indicando `SESSION_COMPLETE_DURATION_REACHED`.
  - Executar o flush síncrono de fechamento no cartão SD.
  - Suspender a admissão de novos frames no buffer de persistência.
  - Notificar o término da sessão no tópico MQTT de status.

---

### 3.6 Conectividade em Nuvem e Telemetria Remota (Wi-Fi e MQTT)

#### [REQ-SYS-19] Conexão Wi-Fi e Pilha TCP/IP no_std
- **Descrição do Comportamento:** O nó coletor deve gerenciar a conexão da interface de rádio em modo Station (STA) a um ponto de acesso sem fio pré-configurado, obter parâmetros de rede via protocolo DHCP e manter uma pilha TCP/IP assíncrona (`embassy-net`) em execução contínua sem depender do runtime padrão C do sistema operacional.

#### [REQ-SYS-20] Cliente MQTT e Transmissão em Lotes Binários Compactos
- **Descrição do Comportamento:** O sistema deve estabelecer conexão com broker MQTT utilizando autenticação por usuário e senha, mantendo soquete de transmissão assíncrono de baixo consumo de recursos (`rust-mqtt`).
- **Empacotamento em Lotes Binários:** Para minimizar a sobrecarga de cabeçalhos de rede e evitar gargalos de alocação de texto, os frames de telemetria transmitidos em tempo real via MQTT devem ser convertidos na estrutura binária compacta de 18 bytes (`BinaryFrame` com alinhamento packed de hardware).
- **Agrupamento Dinâmico:** A tarefa de comunicação deve acumular até **150 frames binários por pacote** e despachá-los no tópico dinâmico `/telemetry/S{id}/raw` com Nível de Garantia de Entrega QoS 0, reduzindo o tráfego de pacotes IP individuais.

#### [REQ-SYS-21] Publicação Periódica de Status e Keepalive do Sistema
- **Descrição do Comportamento:** A cada **5 segundos**, o coletor deve montar e publicar uma mensagem em formato JSON no tópico `/system/status` contendo:
  - Identificador da sessão ativa (ex: `S0001`).
  - Estado operacional (`RECORDING` ou `STANDBY`).
  - Status de integridade do cartão SD (`sd_ok`: verdadeiro/falso).
  - Status do enlace Wi-Fi (`wifi_ok`: verdadeiro/falso).
  - Total de frames capturados desde o boot.
  - Total de frames perdidos por falha simultânea de meios.
  - Tempo de atividade do sistema em milissegundos (`uptime_ms`).

---

### 3.7 Máquina de Estados de Conectividade e Fallback Offline

#### [REQ-SYS-22] Máquina de Estados Finitos de Rede (FSM)
- **Descrição do Comportamento:** O coletor deve manter uma máquina de estados com três condições mutuamente exclusivas governando o direcionamento dos dados:
  - **Estado 0 (Disconnected):** Ausência de enlace Wi-Fi ou perda de soquete TCP com o broker.
  - **Estado 1 (Connected):** Conexão Wi-Fi ativa, IP atribuído e sessão MQTT operacional.
  - **Estado 2 (Reconnecting):** Transição de restabelecimento de conexão em andamento, executando procedimentos de descarregamento de pendências.

#### [REQ-SYS-23] Fallback Automático e Transparente para Cartão SD (AC-06)
- **Descrição do Comportamento:** Na ocorrência de desconexão forçada ou espontânea da rede sem fio (transição do Estado 1 para o Estado 0):
  - Inserir no arquivo CSV local uma linha de diagnóstico estruturada contendo o evento `WIFI_DISCONNECTED`.
  - Comutar imediatamente o fluxo de telemetria, direcionando 100% das amostras capturadas exclusivamente para o buffer do cartão MicroSD.
  - Não provocar qualquer descarte ou perda de amostras de telemetria em decorrência da indisponibilidade da rede.

#### [REQ-SYS-24] Procedimento de Reconexão e Descarregamento de Backlog
- **Descrição do Comportamento:** Ao detectar o restabelecimento da conectividade com o broker MQTT (transição para o Estado 2 e em seguida Estado 1):
  - Injetar no arquivo CSV local uma linha de diagnóstico registrando o evento `WIFI_RECONNECTED` acompanhada do número de registros retidos no backlog.
  - Caso existam amostras retidas no buffer de contingência em memória RAM (`BACKLOG_PSRAM`), descarregá-las integralmente para o cartão SD na estrita ordem cronológica de chegada (FIFO).
  - Retomar a transmissão dos pacotes de telemetria em tempo real no canal MQTT.

---

### 3.8 Controle Remoto, Comandos e Replay de Dados

#### [REQ-SYS-25] Processamento de Comandos Remotos de Bancada
- **Descrição do Comportamento:** O coletor deve manter subscrição permanente no tópico MQTT `/coach/command` e interpretar os seguintes comandos textuais:
  - `STOP`: Interromper imediatamente a coleta ativa, forçar flush de fechamento do arquivo no cartão SD e publicar a confirmação `SESSION_STOPPED`.
  - `ECO,minutos,[epoch]` / `NOR,minutos,[epoch]` / `SPT,minutos,[epoch]`: Racionar nova sessão com o rótulo correspondente (Econômico, Normal ou Esportivo), configurar o temporizador de término, atualizar a referência de tempo Unix absoluto e enviar o comando CAN `0x010` correspondente para a ECU emulada no barramento.
  - `RESET`: Formatar a tabela de sessões existentes, limpar buffers de memória e reinicializar a contagem a partir do arquivo `S_0001.CSV`.
  - `TIME,epoch`: Sincronizar o relógio base do firmware com a época Unix (milissegundos) fornecida pela estação de bancada.
  - `LIST_SESSIONS`: Varrer o diretório raiz do cartão SD, enumerar todos os arquivos `S_XXXX.CSV` existentes e publicar a lista completa em formato JSON no tópico `/system/status`.

#### [REQ-SYS-26] Replay de Dados Históricos sob Demanda
- **Descrição do Comportamento:** Ao receber a instrução `REPLAY,N` no tópico de comandos, o sistema deve:
  - Pausar e fechar com segurança a sessão ativa caso esteja gravando.
  - Abrir o arquivo de histórico correspondente (`S_{N:04}.CSV`) em modo de leitura continuada.
  - Fatiar o conteúdo em blocos de até 1536 bytes alinhados à quebra de linha `\n`.
  - Transmitir os blocos sequencialmente no tópico `/telemetry/replay` com pausas cooperativas de 5 ms entre transmissões.
  - Publicar o evento `REPLAY_COMPLETE` ao atingir o final do arquivo (EOF).

---

### 3.9 Supervisão, Diagnóstico In-Situ e Tolerância a Falhas

#### [REQ-SYS-27] Detecção e Auto-Recuperação de Bus-Off sem Reinício do SoC (AC-07)
- **Descrição do Comportamento:** Quando falhas físicas no cabeamento CAN ou perturbações eletromagnéticas severas levarem o controlador TWAI ao estado de **Bus-Off** (acumulação de erros ultrapassando o limiar de 255 da norma ISO 11898):
  - A tarefa de recepção `task_can_rx` deve identificar o retorno de erro crítico e sinalizar atomicamente a flag global `BUS_OFF_DETECTED`.
  - A tarefa de supervisão `task_watchdog` deve capturar a sinalização em até 10 ms e emitir o sinal `BUS_OFF_SIGNAL`, forçando as tarefas de recepção e envio de diagnóstico a pausarem cooperativamente seus acessos ao periférico.
  - O sistema deve compulsoriamente aguardar a **janela de contenção de 128 ms** (correspondente a 128 sequências de 11 bits recessivos contínuos exigidos para estabilização do meio físico).
  - Após os 128 ms, a camada MCAL deve rearmar os registradores de hardware diretamente através de escrita de baixo nível no controlador (via Peripheral Access Crate - PAC), limpando as flags de reset sem reinicializar o processador ESP32-S3 e sem destruir as tarefas ativas do RTOS.
  - Emitir o sinal `BUS_OFF_CLEAR` para liberar as tarefas de comunicação e injetar um registro de diagnóstico no arquivo CSV informando `BUS_OFF_RECOVERED_latency_ms=128`.

#### [REQ-SYS-28] Detecção de Congelamento de Processamento (Logger Stall)
- **Descrição do Comportamento:** A cada **30 segundos** de sessão ativa com o cartão de memória funcional, a tarefa de supervisão deve verificar se o contador acumulado de frames processados sofreu algum incremento. Caso nenhum frame tenha sido processado ao longo dos últimos 30 segundos:
  - Declarar condição de travamento (*Logger Stall*).
  - Registrar alerta crítico no console serial.
  - Injetar compulsoriamente uma linha de diagnóstico `DIAG,LOGGER_STALL` no arquivo do cartão SD para registro forense de anomalia.

#### [REQ-SYS-29] Telemetria de Desempenho e Memória In-Situ (HEARTBEAT / AC-05)
- **Descrição do Comportamento:** Para viabilizar a auditoria de consumo de recursos sem necessidade de instrumentos externos conectados, o sistema deve executar auto-monitoramento periódico:
  - Amostrar silenciosamente a memória livre do alocador global de heap (`esp_alloc::HEAP.free()`) a cada **5 segundos**, consolidando valores mínimo, máximo e médio em variáveis locais.
  - A cada **60 segundos** durante sessão ativa, formatar e injetar uma linha especial do tipo `DIAG,HEARTBEAT` no arquivo CSV do cartão SD, contendo:
    - Tempo de atividade (`uptime`) em segundos.
    - Contadores acumulados de frames recebidos, persistidos no SD, publicados no MQTT e perdidos.
    - Contagem de overflows de canais e eventos de Bus-Off ocorridos.
    - Leituras de memória heap livre atual, mínima na janela, máxima na janela, média na janela e memória utilizada.
    - Ocupação instantânea do buffer do cartão SD em bytes.
- **Transparência de Dados:** As linhas de diagnóstico marcadas como `DIAG` devem possuir a mesma estrutura de 11 colunas e são logicamente ignoradas na contagem estatística do dataset de telemetria veicular.

#### [REQ-SYS-30] Temporizador de Supervisão por Hardware (Watchdog de Hardware)
- **Descrição do Comportamento:** A tarefa supervisora deve rearmar periodicamente o circuito temporizador de watchdog de hardware do microcontrolador em intervalos não superiores a **2 segundos**. Caso uma falha catastrófica ou bloqueio permanente do escalonador impeça o rearme dentro da janela de **5 segundos**, o hardware do microcontrolador deve provocar o reinício imediato e controlado do SoC.

---

## 4. Requisitos Não-Funcionais e Restrições de Engenharia

### 4.1 Restrições de Memória e Recursos do Coletor (AC-05)
- **Consumo Máximo de Memória SRAM Interna:** O consumo total de memória volátil (SRAM do microcontrolador) alocada para heaps dinâmicos, pilhas de tarefas e canais estáticos deve permanecer estritamente contido **abaixo de 200 KB** durante operação contínua de longa duração (30 minutos ou mais).
- **Isolamento de Memória para Edge AI:** Da memória externa PSRAM de 8 MB existente no ESP32-S3, a telemetria do sistema de borda deve estar matematicamente limitada a utilizar no máximo **512 KB** para buffers de contingência. A partição restante de mais de **7,5 MB (93,75% da PSRAM)** deve ser mantida completamente livre e isolada para alocação futura de tensores de redes neurais (modelos TinyML / Mamba-2).

### 4.2 Restrições de Tempo Real e Orçamento Temporal
- **Latência de Ponta a Ponta no Coletor:** O tempo total decorrido desde a disponibilização de um frame CAN no transceptor até a sua inserção no buffer de persistência não deve exceder **30 ms** em condições nominais de escalonamento.
- **Capacidade do Channel da RTE:** O canal estático `TELEMETRY_CHANNEL` deve possuir capacidade para suportar até 32 elementos `TelemetryFrame`, provendo uma margem temporal de absorção de surtos de até 1,6 segundos na taxa de transmissão de bancada (20 frames/s).

### 4.3 Requisitos de Volume de Amostragem do Dataset (AC-08)
- **Volume Mínimo Gerado:** Em ensaios completos de bancada compreendendo os três perfis de condução (Econômico, Normal e Esportivo) por períodos de 10 minutos cada, o sistema deve registrar um volume consolidado de amostras de telemetria veicular igual ou superior a **72.000 linhas de dados válidos**.

---

## 5. Dicionário de Formatos e Protocolos de Comunicação

### 5.1 Protocolo CAN DBC Mínimo

| Identificador CAN | Direção | Periodicidade | Conteúdo dos Bytes |
|---|---|---|---|
| `0x100` | UNO R3 $\rightarrow$ ESP32-S3 | 100 ms | Byte 0-1: `speed_kmh` (uint16_be)<br>Byte 2-3: `rpm` (uint16_be)<br>Bytes 4-7: padding zerado |
| `0x200` | UNO R3 $\rightarrow$ ESP32-S3 | 50 ms | Byte 0: `throttle_pct * 255 / 100`<br>Byte 1: `engine_load_pct * 255 / 100`<br>Bytes 2-7: padding zerado |
| `0x300` | UNO R3 $\rightarrow$ ESP32-S3 | 1000 ms | Byte 0: `coolant_temp_c + 40`<br>Bytes 1-7: padding zerado |
| `0x7DF` | ESP32-S3 $\rightarrow$ UNO R3 | 100 ms | Byte 0: `0x02` (len)<br>Byte 1: `0x01` (mode)<br>Byte 2: PID solicitado<br>Bytes 3-7: `0x00` |
| `0x7E8` | UNO R3 $\rightarrow$ ESP32-S3 | Sob demanda | Byte 0: Comprimento útil<br>Byte 1: `0x41`<br>Byte 2: PID respondido<br>Bytes 3-4: Dados do parâmetro |
| `0x010` | ESP32-S3 $\rightarrow$ UNO R3 | Assíncrono | Byte 0: Código do perfil (`1`=Eco, `2`=Normal, `3`=Sport)<br>Bytes 1-7: `0x00` |

### 5.2 Estrutura do Pacote Binário MQTT (`BinaryFrame`)
Cada amostra individual agregada no lote de transmissão de alta velocidade possui exatamente **18 bytes**:
- `timestamp_ms` (uint32, 4 bytes): Marca temporal em milissegundos.
- `can_id` (uint16, 2 bytes): Identificador CAN de origem.
- `source` (uint8, 1 byte): Origem do dado (`0` = CAN_DBC, `1` = OBD_PID).
- `label` (uint8, 1 byte): Código numérico do perfil de condução ativo.
- `val1` (float32, 4 bytes IEEE 754): Primeira grandeza física (Velocidade ou Acelerador).
- `val2` (float32, 4 bytes IEEE 754): Segunda grandeza física (RPM ou Carga do Motor).
- `obd_latency` (uint16, 2 bytes): Latência de ida e volta em décimos de milissegundo.

---

## 6. Matriz de Atendimento aos Critérios de Aceitação de Engenharia (Gate F1)

| Identificador | Critério de Aceitação | Limiar Normativo | Subsistema Responsável |
|:---:|---|---|---|
| **AC-01** | Taxa de recepção de frames no barramento | $\ge 99,0\%$ do volume nominal teórico emitido | `task_can_rx` / `mcal::twai` |
| **AC-02** | Latência média de resposta OBD-II | $< 10\text{ ms}$ entre requisição e resposta física | `uno_ecu_emulator` / `task_obd_poller` |
| **AC-03** | Jitter temporal de resposta OBD-II | Desvio padrão $\sigma < 3\text{ ms}$ na distribuição | `uno_ecu_emulator` |
| **AC-04** | Integridade dos dados estruturados | $100\%$ das linhas CSV íntegras, sem campos vazios indevidos | `app::csv_writer` / `task_logger` |
| **AC-05** | Consumo de memória volátil SRAM | $< 200\text{ KB}$ em execução contínua com heap monitorado | `bsw::bsw_mem` / `bsw::bsw_diag` |
| **AC-06** | Fallback offline em cartão MicroSD | Gravação transparente em SD sob perda de sinal Wi-Fi | `task_logger` / `bsw::bsw_com` |
| **AC-07** | Auto-recuperação do estado elétrico Bus-Off | Recuperação com espera de 128 ms sem reinício do SoC | `task_watchdog` / `mcal::twai` |
| **AC-08** | Volume mínimo de telemetria gerado | $\ge 72.000$ registros tabulares consolidados nos 3 cenários | Pipeline Integrado de Bancada |