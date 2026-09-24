Modelação Arquitetural, Verificação de Requisitos e Análise de Evolução em Sistemas Embarcados de Tempo Real
1. Âmbito e Objetivo Geral
O objetivo deste trabalho prático é realizar o ciclo completo de Engenharia de Sistemas Baseada em Modelos (MBSE) para um sistema embarcado de tempo real crítico, integrando a formalização e verificação de requisitos, a modelação arquitetural estrutural e comportamental em AADL (Architecture Analysis and Design Language), a análise estática e temporal no OSATE e a avaliação de variabilidade/substituição de componentes através do método CAvA (Component/Architecture Variability and Evolution approach).

O objeto de estudo é a engenharia reversa da camada Edge Telemetry Layer, contemplando exclusivamente a aquisição e diagnóstico automóvel (barramento CAN a 500 kbps e OBD-II ISO 15765-4), estruturação de dados em formato CSV, armazenamento local resiliente (cartão SD em FAT32) e comunicação remota (Wi-Fi / MQTT).
2. Estrutura dos Entregáveis (Quatro Pilares Obrigatórios)
+--------------------------------------------------------------------------------------------------+
|                                    ESTRUTURA DA ENTREGA                                          |
|                                                                                                  |
| [ 1. Requisitos Formais ] -> [ 2. Modelação AADL ] -> [ 3. Análises Temporais ] -> [ 4. CAvA ]  |
|      (FRET ou OSATE)             (Hardware + SW)         (Escalonabilidade + Latência)   (Evolução)   |
+--------------------------------------------------------------------------------------------------+


Pilar 1: Especificação, Formalização e Verificação de Requisitos
O grupo deve formalizar a especificação de requisitos do sistema de telemetria, escolhendo uma das abordagens facultadas:

Opção A (FRET / NASA):
Estruturar os requisitos funcionais e temporais na linguagem controlada FRETish, decompondo cada regra em scope, condition, component, timing e response.
Configurar a tabela de variáveis (Variable Mapping), tipando-as como Input, Output ou Internal.
Executar a Verificação de Realizabilidade (Checking Realizability) monolítica ou composicional para comprovar a ausência de conflitos de estados na FSM e deadlocks lógicos. Caso surjam conflitos, documentar o contraexemplo e a correção textual efetuada.

Opção B (Diretamente no OSATE):
Especificar os requisitos no ecossistema do OSATE, associando propriedades e anexos formais diretamente aos componentes correspondentes da arquitetura.

Entregáveis do Pilar 1:

Lista descritiva dos requisitos de engenharia:
Taxa de receção de tráfego CAN [AC-01].
Polling ativo cíclico e latência/jitter de resposta OBD-II [AC-02, AC-03].
Integridade da serialização CSV e bufferização [AC-04].
Fallback offline automático no cartão SD sob falha de Wi-Fi [AC-06].
Deteção e auto-recuperação do estado elétrico de Bus-Off com espera de 128 ms [AC-07].

Sentenças estruturadas formalizadas (no caso do FRET, incluir capturas do editor de requisitos, mapeamento de variáveis e prova de realizabilidade sem contradições lógicas).

Pilar 2: Modelação Arquitetural em AADL (OSATE)
Construção do modelo textual .aadl da camada de telemetria e respetiva instanciação funcional:

2.1. Plataforma de Execução (Hardware):
Declaração do processador (processor) com protocolo preemptivo por prioridades: Scheduling_Protocol => (POSIX_1003_HIGHEST_PRIORITY_FIRST_PROTOCOL);.
Declaração dos barramentos físicos (bus): barramento diferencial CAN (500 kbps) e barramento local SPI (para interface com o cartão SD).
Declaração dos dispositivos (device): transceptor CAN e leitor de cartão SD / módulo de comunicação. (Reutilizar componentes da pasta Library > devices se aplicável, ou criar os ficheiros .aadl necessários).

2.2. Camada de Software (Processos e Tarefas):
Declaração de data para os tipos de dados manipulados (frames CAN, pacotes de diagnóstico, payloads CSV).
Declaração das tarefas periódicas (thread) com os atributos temporais:
Dispatch_Protocol => Periodic;
Period
Compute_Execution_Time (faixa BCET .. WCET)
Deadline
Priority

Tarefas do pipeline de software:
task_can_rx: leitura do controlador TWAI e inserção no canal assíncrono.
task_obd_poller: disparo de solicitações funcionais no ID 0x7DF e receção de PIDs em 0x7E8.
task_logger: serialização para linha CSV e gestão de buffers estáticos.
task_tx_dispatch: encaminhamento de pacotes para gravação em cartão SD ou publicação MQTT.

Declaração do processo (process) encapsulando as threads e estabelecendo conexões via portas (data port ou event data port).

2.3. Fluxos de Informação e Semântica de Portas:
Declaração dos caminhos de fluxo internos (flow path, flow source, flow sink).
Declaração do fluxo ponta a ponta (end to end flow): desde a chegada do frame no transceptor CAN, passando pela cadeia de software, até a saída no descritor de ficheiro/comunicação.
Configuração e comparação das políticas de conexão: avaliar a semântica immediate versus delayed.

2.4. Integração do Sistema e Alocação (Bindings):
Declaração da implementação raiz (system implementation) integrando processador, barramentos, dispositivos e processos.
Amarração formal de software no hardware: Actual_Processor_Binding e Actual_Connection_Binding.
Instanciação bem-sucedida do sistema raiz, gerando o ficheiro compilado de instância .aaxl2 na pasta instances.

Pilar 3: Análises Estáticas e Temporais no OSATE
Validação analítica do comportamento temporal sobre o modelo instanciado:

Análise de Escalonabilidade (Check Schedulability):
Execução da verificação de escalonamento para garantir a viabilidade das tarefas sob prioridades fixas preemptivas.
Apresentação da taxa de utilização global da CPU ($U \le 100\%$) e verificação de que o tempo de resposta no pior caso de cada tarefa não excede o respetivo prazo limite (deadline).

Análise de Latência Ponta a Ponta (Check Flow Latency):
Execução da ferramenta analítica de latência sobre o fluxo ponta a ponta (end to end flow).
Comparação matemática entre:
Modo delayed: impacto do atraso de amostragem na retenção de ciclo (delayed sampling).
Modo immediate: redução da latência global devido ao encadeamento síncrono das tarefas no mesmo período.

Pilar 4: Avaliação de Evolução Arquitetural (Framework CAvA)
Aplicação das fases metodológicas do framework CAvA (Fases 2, 3 e 4):

Cenário de Substituição: Propor a substituição de um dispositivo da arquitetura de telemetria (por exemplo, atualização do transceptor CAN ou alteração do módulo de comunicação/armazenamento) por um dispositivo candidato alternativo.

Análise de Impacto Arquitetural:
Identificar alterações de ligações, portas e quebras de compatibilidade provocadas pelo novo componente.
Modelação de componentes de compatibilização intermediários (wrappers ou adaptadores) necessários na camada de software para ajustar interfaces ou tipos de dados.

Avaliação de Trade-offs:
Reinstanciar o sistema evoluído no OSATE e recalcular as análises temporais.
Comparar métricas de desempenho (latência ponta a ponta, carga adicional de CPU decorrente dos novos wrappers e número de conexões modificadas), fundamentando a seleção da arquitetura consolidada.
3. Estrutura do Relatório Final
Introdução e Descrição do Sistema: Arquitetura do Edge Telemetry Layer, divisão em camadas (inspirada em AUTOSAR/BSW), barramento CAN a 500 kbps e diagnóstico OBD-II.
Especificação e Verificação de Requisitos: Catálogo de requisitos formalizados e evidências de validação (tabela de mapeamento e status de realizabilidade no FRET ou declarações formais no OSATE).
Modelação AADL: Código textual .aadl dos componentes de hardware, processos, threads, portas e fluxos de dados, acompanhado do diagrama arquitetural gerado pela ferramenta.
Resultados das Análises Temporais: Relatórios extraídos do OSATE para o Check Schedulability e detalhamento comparativo do Check Flow Latency (immediate vs. delayed).
Estudo de Evolução Arquitetural (CAvA): Descrição do componente substituído, especificação dos wrappers adicionados e análise quantitativa dos trade-offs de desempenho do sistema evoluído.
Conclusão: Síntese sobre a conformidade da arquitetura perante os requisitos e restrições de tempo real estabelecidos.

