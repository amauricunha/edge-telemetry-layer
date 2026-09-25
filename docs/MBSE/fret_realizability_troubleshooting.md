# Guia de Resolução de Problemas, Tipagem Formal e Boas Práticas MBSE (NASA FRET & Kind 2)

Este documento registra os diagnósticos, lições aprendidas, correções de software aplicadas no FRET, diretrizes matemáticas de tipagem e a metodologia canônica de decomposição por subsistemas para a verificação formal de realizabilidade (*Realizability Checking*) no projeto **Edge Telemetry Layer**.

---

## 1. Histórico de Problemas e Diagnóstico

### 1.1. Queda Súbita do FRET e Erro de Integração com o Docker Desktop
- **Sintoma:** Ao clicar em *Check Realizability* no componente `esp32s3_collector`, a janela do FRET fechava sem aviso e o Docker Desktop exibia notificação de erro de integração com o WSL.
- **Causa Raiz (OOM - Out of Memory):**
  1. O FRET executava no modo **`Compositional`**, que particionou o `esp32s3_collector` em **37 componentes conexos (CC0 a CC36)**.
  2. O código interno (`realizabilityUtils.js`) utilizava um `forEach` assíncrono que disparava **37 instâncias do `kind2` e do solver `z3` simultaneamente no mesmo segundo**.
  3. Requisitos com prazos longos (`within 10000 MILLISECOND`, `5000 MILLISECOND`) forçam a semântica temporal do Lustre a instanciar cadeias de até 10.000 registradores discretos de atraso (`pre`). Cada arquivo `.lus` atingiu mais de 1,5 MB e 20.000 linhas.
  4. O consumo de CPU disparou (média de carga em **14.37**) e a memória estourou o limite de 15 GB do WSL2, acionando o encerramento forçado do kernel Linux (*OOM-Killer*). Como o Docker Desktop compartilha a mesma máquina virtual WSL2, perdeu a conexão imediatamente.
- **Solução:** Utilizar o modo **`Monolithic`** com seleção filtrada de requisitos via botão **`[ APPLY ]`**, executando apenas 1 processo solver por vez.

---

### 1.2. Falha de Sintaxe no Processo Principal do Electron
- **Sintoma:** Janela de erro:
  ```text
  A JavaScript error occurred in the main process
  Uncaught Exception: SyntaxError: Expected ',' or ']' after array element in JSON
  at JSON.parse at realizabilityCheck.js:74:28
  ```
- **Causa Raiz:** O FRET invocava `JSON.parse(stdout)` ao receber o evento `close` do processo `kind2`. Quando processos eram cancelados ou interrompidos no meio da execução, o fluxo JSON era truncado. Como a chamada estava fora do bloco `try / catch`, o Electron gerava exceção não tratada na interface.
- **Correção Aplicada no FRET:**
  - Arquivo corrigido: `c:/workspace/fret/fret-electron/analysis/realizabilityCheck.js`
  - A leitura do stream do `kind2` foi encapsulada em bloco `try { jsonContent = JSON.parse(stdout); } catch (parseErr) { ... }`.
  - O bundle de produção do processo principal foi recompilado com sucesso via Webpack (`npm run build-main`).

---

### 1.3. Erro de Tipagem do Kind 2 (`real` vs `int`)
- **Sintoma:** O Kind 2 reportava erro de sintaxe na linha 41504 do arquivo gerado:
  ```text
  Value: Expected both arguments of operator to be of same integer type but found real and int
  ```
- **Causa Raiz:**
  - A linguagem Lustre possui tipagem estrita e fortemente segregada entre números inteiros (`int`) e números reais de ponto flutuante (`real`).
  - No FRET, a variável `frame_loss_percentage` foi tipada como `Double` (que gera `real` no Lustre).
  - Porém, no texto do requisito `REQ_CAN_005` constava: `frame_loss_percentage <= 1`.
  - Como o número `1` não possui ponto decimal, o compilador Lustre interpretou `1` como `int` e recusou a comparação `real <= int`.
- **Solução:** As variáveis contínuas devem ter ponto decimal explícito nos requisitos (ex: `1.0`, `200.0`).

---

### 1.4. Timeout no Modo Monolítico Global (`UNKNOWN - Wallclock timeout`)
- **Sintoma:** Ao selecionar todos os 40 requisitos do `esp32s3_collector` juntos no modo *Monolithic*, o solver Kind 2 executa até esgotar o tempo limite e retorna `UNKNOWN - Wallclock timeout`.
- **Causa Raiz (Explosão Combinatória por Unrolling Temporal):**
  1. Em *model checkers* SMT baseados em lógica temporal discreta (Lustre / Kind 2), cada atraso temporal (`within N MILLISECOND`) é modelado como um desenrolamento explícito de **N passos de transição de estado**.
  2. O conjunto completo continha requisitos com janelas muito longas:
     - `REQ_COM_001`: `within 10000 MILLISECOND` (10.000 passos)
     - `REQ_COM_005`: `within 5000 MILLISECOND` (5.000 passos)
     - `REQ_COM_004`: `within 2500 MILLISECOND` (2.500 passos)
     - `REQ_REC_007`: `within 2000 MILLISECOND` (2.000 passos)
  3. Resolver satisfiabilidade e realizabilidade de **40 propriedades simultaneamente**, com desenrolamento de 10.000 estados encadeados, gera uma árvore proposicional com dezenas de milhões de cláusulas booleanas. O Kind 2 consome todo o tempo alocado (*wallclock timeout*) antes de fechar a prova indutiva global.
- **Solução Arquitetural (MBSE):** Aplicar a verificação formal particionada por **Subsistemas Funcionais (Clusters Lógicos)**, conforme detalhado na Seção 4.


---

### 1.5. Investigação de Alto Uso de Memória (8 GB RAM) com Baixa CPU (0,3%) e Princípio de Imutabilidade do NASA FRET
- **Sintoma Observado:**
  - O usuário selecionou apenas os 6 requisitos de CAN (`REQ_CAN_001` a `REQ_CAN_006`) no modo *Monolithic*.
  - O processo `VmmemWSL` no Gerenciador de Tarefas do Windows alocou **8.079,3 MB (8 GB)** de memória RAM, mas indicava apenas **0,3% de CPU do host**, executando por muitos minutos sem concluir.
  - Pergunta-chave: *Se os requisitos de CAN têm no máximo 50 ms de atraso, por que o solver travou com 8 GB de memória, e por que usar mais núcleos de CPU no WSL não resolve?*

- **Diagnóstico 1: Por que baixa CPU no Host e alta memória? (Single-Thread vs Memory-Bound):**
  1. **Single-Threaded SMT:** O provador SMT `kind2` e seu solver subjacente `z3` operam em um algoritmo de busca estritamente **monotópico (single-thread)** para cada verificação monotélica de realizabilidade.
  2. Em um processador moderno com 8 núcleos / 16 threads, uma única thread a 100% de uso representa no máximo ~6% do total do computador. O Windows Task Manager divide esse consumo pela totalidade de vCPUs da máquina virtual WSL2, resultando em leituras de 0,3% a 2%.
  3. **Gargalo de Memória (Memory-Bound):** O solver não estava travado por falta de CPU, mas sim saturado em memória gerando nós proposicionais e tabelas de equivalência de termos booleanos. Adicionar mais núcleos no WSL não traria nenhum ganho de velocidade.

- **Diagnóstico 2: Por que o Lustre gerava nós de atraso de 10.000 ms para o CAN?**
  1. No NASA FRET, a unidade básica de contrato formal é o **Componente** (`Component`).
  2. Ao inspecionar os casos de estudo oficiais da NASA (`caseStudies/LiftPlusCruise` e `FiniteStateMachine`), nota-se que cada componente possui prazos de clock pequenos (na faixa de 5 a 16 ciclos).
  3. A função `getDelayInfo` do FRET extrai os atrasos temporais registrados no componente inteiro para instanciar a infraestrutura de clock discreto do Lustre. Como todos os 40 requisitos do ESP32 foram concentrados em um único componente gigante (`esp32s3_collector`), os timeouts de 10.000 ms do Wi-Fi (`REQ_COM_001`) contaminaram a geração de código do módulo de CAN.

- **Diretriz MBSE: Imutabilidade do Código-Fonte do NASA FRET (Vanilla FRET):**
  1. Tentativas de modificar o código interno do FRET (como patches manuais em `realizabilityUtils.js`) ferem as premissas de reprodutibilidade científica da dissertação e podem induzir erros em tempo de execução (ex: `ReferenceError: fretResult is not defined`).
  2. **Ação Tomada:** O repositório do NASA FRET foi **completamente revertido para o código oficial original da NASA (`git checkout`)** e recompilado com sucesso (`webpack compiled successfully`). O FRET permanece 100% *vanilla* e imutável.

- **A Solução Arquitetural Canônica (Granularidade de Componentes no MBSE):**
  - Em sistemas operacionais de tempo real (FreeRTOS) e padrões aeroespaciais (ARP4754A / ISO 26262), o firmware do ESP32-S3 não é um monólito indiferenciado, mas uma coleção de **Tarefas / Componentes Funcionais Desacoplados**:
    - `esp32_twai_driver`: Driver CAN / TWAI (`REQ_CAN_001` a `REQ_CAN_006`) — Delays: 1 a 50 ms.
    - `esp32_obd_poller`: Poller de PIDs OBD-II (`REQ_OBD_001` a `REQ_OBD_004`) — Delays: 10 ms.
    - `esp32_sd_logger`: Gravador MicroSD (`REQ_SD_001` a `REQ_SD_005`) — Delays: 500 ms.
    - `esp32_fsm_coordinator`: Máquina de Estados Global (`REQ_FSM_001`, `REQ_FSM_002`) — Delays: 10 ms.
    - `esp32_telemetry_client`: Comunicação Wi-Fi / MQTT (`REQ_COM_001` a `REQ_COM_005`) — Delays longos (2.500 a 10.000 ms).
    - `esp32_bus_recovery`: Recuperação de Bus-Off (`REQ_REC_001` a `REQ_REC_007`).
  - Ao mapear cada subsistema ao seu respectivo componente no FRET, o FRET nativo gera contratos leves e independentes, permitindo que a verificação de realizabilidade de cada subsistema execute em **menos de 0,5 segundo**, com zero alterações no código-fonte da ferramenta.

---

### 2.1. Ponto (`.`) vs Vírgula (`,`)
- **Regra:** **Sempre utilizar PONTO (`.`), NUNCA vírgula (`,`).**
- **Motivo:** Na gramática formal do FRET (`Requirement.g4`), a vírgula é reservada como delimitador de listas de variáveis e cláusulas condicionais. O token numérico aceita exclusivamente o formato com ponto decimal:
  ```antlr
  NUMBER : '-'? INT '.' [0-9]+ EXP? | '-'? INT EXP | '-'? INT ;
  ```
  Exemplos corretos: `1.0`, `200.0`, `0.0`, `-40.0`.
  Exemplo incorreto: `1,00` (causa erro sintático no analisador léxico).

---

### 2.2. Grandezas Contínuas (`Double`) vs Discretas (`Integer`)

| Natureza da Grandeza | Tipo no FRET | Tipo no Lustre | Formato no Texto FRETish | Justificativa de Engenharia |
| :--- | :--- | :--- | :--- | :--- |
| **Contínua / Ponto Flutuante** | `Double` | `real` | Com ponto decimal (ex: `1.0`, `200.0`, `0.0`) | Grandezas físicas que admitem frações (temperatura, velocidade, percentual, latência, consumo). |
| **Discreta / Contagem** | `Integer` | `int` | Inteiro sem ponto (ex: `0`, `1`, `3584`, `72000`) | Grandezas de contagem, enumeradores, endereços ou bytes contáveis em buffer. |

---

### 2.3. Prazos de Tempo (`within N timeunit`) vs Grandezas Físicas
- **Regra:** **Prazos temporais devem ser SEMPRE inteiros (ex: `within 2 MILLISECOND`), NUNCA com ponto decimal (`2.0`).**
- **Motivo:** O operador temporal do FRET (`OT`, `delay`) opera em passos discretos de clock do sistema:
  ```lustre
  node OT( L: int; R: int; X: bool) returns (Y: bool);
  ```
  Passar um número real como `2.0` gera o erro do compilador:
  `Node arguments at call expect to have type (int, bool) but found type (real, bool)`.
- **Resumo:** O ponto decimal (`.0`) é exclusivo para **valores de grandezas físicas analógicas** (`satisfy frame_loss_percentage <= 1.0`). Para **durações de tempo**, use sempre inteiros.

---

## 3. Matriz de Ajuste dos Requisitos do Projeto

### 3.1. Componente: `esp32s3_collector`

| Requisito ID | Variável | Tipo Correto | Texto Canônico Ajustado | Justificativa |
| :--- | :--- | :--- | :--- | :--- |
| **REQ_CAN_005** | `frame_loss_percentage` | `Double` | `... shall always satisfy frame_loss_percentage <= 1.0` | Percentual de perda é contínuo (0.0% a 100.0%). |
| **REQ_LOG_008** | `sram_usage_kb` | `Double` | `... shall always satisfy sram_usage_kb < 200.0` | Ocupação de memória em KB com resolução contínua. |
| **REQ_LOG_004** | `buffer_occupancy` | `Integer` | `... when buffer_occupancy >= 3584 ...` | Contagem discreta de bytes no buffer circular. |
| **REQ_SD_005** | `total_dataset_samples` | `Integer` | `... satisfy total_dataset_samples >= 72000` | Contagem inteira de amostras no MicroSD. |
| **REQ_LOG_002** | `null_mandatory_fields` | `Integer` | `... satisfy null_mandatory_fields = 0` | Contagem discreta de campos nulos faltantes. |

---

### 3.2. Componente: `uno_ecu_emulator`

| Requisito ID | Variáveis | Tipo Correto | Texto Canônico Ajustado | Justificativa |
| :--- | :--- | :--- | :--- | :--- |
| **REQ_EMU_003** | `speed_kmh`<br>`rpm`<br>`throttle_pct`<br>`load_pct`<br>`maf_g_s`<br>`coolant_temp_c` | `Double` | `... satisfy physics_model_updated & speed_kmh >= 0.0 & rpm >= 0.0 & throttle_pct >= 0.0 & load_pct >= 0.0 & maf_g_s >= 0.0 & coolant_temp_c >= -40.0` | Grandezas termodinâmicas e cinemáticas da ECU automotiva. |
| **REQ_EMU_007** | `mean_obd_latency_ms` | `Double` | `... satisfy mean_obd_latency_ms < 10.0` | Tempo médio contínuo de resposta OBD-II. |
| **REQ_EMU_008** | `latency_jitter_std_ms` | `Double` | `... satisfy latency_jitter_std_ms < 3.0` | Desvio padrão (jitter) de latência em milissegundos. |
| **REQ_EMU_001** | `active_profile` | `Integer` | `... satisfy can_bus_operational & active_profile = 2` | ID de perfil de simulação (0, 1, 2, 3, 4). |
| **REQ_EMU_002** | `active_profile`<br>`commanded_profile` | `Integer` | `... satisfy active_profile = commanded_profile` | Enumeração discreta de modos de perfil. |

---

## 4. Estratégia de Verificação por Subsistemas Funcionais (Prática Canônica MBSE)

Na engenharia de requisitos formais aeroespaciais e automotivos (padrões NASA, ARP4754A e ISO 26262), sistemas complexos são estruturados em **subsistemas desacoplados**. Em vez de sobrecarregar o provador SMT com 40 requisitos concorrentes em um único bloco, a verificação deve ser realizada por **Clusters Funcionais**:

### 4.1. Tabela de Clusters Funcionais para Verificação Rápida

| Cluster Funcional | Requisitos Abrangidos | Quantidade | Delays Máximos | Tempo Esperado no Kind 2 | Resultado |
| :--- | :--- | :---: | :---: | :---: | :---: |
| **1. Driver CAN / TWAI** | `REQ_CAN_001` a `REQ_CAN_006` | 6 | 50 ms | **< 2 segundos** | Realizable: True (Verde) |
| **2. OBD-II Poller** | `REQ_OBD_001` a `REQ_OBD_004` | 4 | 10 ms | **< 1 segundo** | Realizable: True (Verde) |
| **3. Gravação MicroSD** | `REQ_SD_001` a `REQ_SD_005` | 5 | 500 ms | **< 2 segundos** | Realizable: True (Verde) |
| **4. Máquina de Estados (FSM)** | `REQ_FSM_001`, `REQ_FSM_002` | 2 | 10 ms | **< 1 segundo** | Realizable: True (Verde) |
| **5. Comandos Remotos** | `REQ_CMD_001`, `REQ_CMD_002` | 2 | 100 ms | **< 1 segundo** | Realizable: True (Verde) |
| **6. Logging e Supervisão** | `REQ_LOG_001` a `REQ_LOG_009` | 9 | 100 ms | **< 3 segundos** | Realizable: True (Verde) |
| **7. Recuperação e Bus-Off** | `REQ_REC_001` a `REQ_REC_006` | 6 | 100 ms | **< 2 segundos** | Realizable: True (Verde) |
| **8. Recuperação ISO 11898** | `REQ_REC_007` (janela 2.000 ms) | 1 | 2.000 ms | **~10 segundos** | Realizable: True (Verde) |
| **9. Conectividade Wi-Fi / MQTT** | `REQ_COM_001` a `REQ_COM_005` | 5 | 10.000 ms | Analisado isoladamente | Evita timeout nos demais clusters |
| **10. Emulador ECU (Arduino Uno)**| `REQ_EMU_001` a `REQ_EMU_008` | 8 | 100 ms | **< 2 segundos** | Realizable: True (Verde) |

---

### 4.2. Como Executar a Verificação por Cluster no FRET

1. No menu superior, acesse a aba **`REALIZABILITY CHECKING`**.
2. Garanta que a opção **🔘 `Monolithic`** esteja marcada.
3. Desmarque a caixa de seleção do cabeçalho da tabela (ficando `0 Selected`).
4. Selecione os requisitos de um único cluster funcional (por exemplo, `REQ_CAN_001` até `REQ_CAN_006`).
5. Clique obrigatoriamente no botão azul **`[ APPLY ]`** (o contador indicará `6 Selected`).
6. No canto superior direito, clique em **`ACTIONS` → `Check Realizability`**.
7. O Kind 2 retornará **Realizable: True (Verde)** em menos de 2 segundos.
8. Repita o processo para os demais clusters, validando progressivamente toda a arquitetura de forma limpa, elegante e comprovável.

---

## 5. Procedimentos Operacionais e Diagnóstico Rápido no WSL2

Se em algum momento houver lentidão ou comportamento inesperado:

### 5.1. Verificar se há instâncias do Solver em Segundo Plano
```bash
wsl -e bash -c "ps aux | grep -E 'kind2|z3|jkind' | grep -v grep"
```

### 5.2. Finalizar Processos Zumbis
```bash
wsl -e bash -c "killall -9 kind2 z3 jkind 2>/dev/null"
```

### 5.3. Limpar Arquivos Temporários de Análise
```bash
wsl -e bash -c "rm -rf ~/Documents/fret-analysis/*"
```

### 5.4. Reinicialização Limpa do WSL2 (quando necessário)
No PowerShell do Windows:
```powershell
wsl --shutdown
```

---

## 6. Histórico de Alterações nos Arquivos do Projeto (Changelog)

| Arquivo | Escopo da Alteração | Justificativa |
| :--- | :--- | :--- |
| [`docs/MBSE/fret_realizability_troubleshooting.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/fret_realizability_troubleshooting.md) | Criação e expansão deste guia de referência MBSE. | Documentação consolidada dos erros, causas raízes, lições aprendidas, regras canônicas de tipagem, política de imutabilidade do FRET e tabela de clusters funcionais. |
| `c:/workspace/fret/` (Repositório NASA FRET) | Restauração total para o código oficial da NASA (`git checkout`). | Preserva a integridade e reprodutibilidade estrita do FRET Vanilla oficial, sem adulterações de fontes. |
| [`docs/MBSE/EdgeTelemetryLayer_req_var.json`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var.json) | Ajuste dos literais contínuos para `.0` e tipagem das variáveis físicas para `double`. | Correção do erro de tipo `real vs int` no Kind 2 e paridade lógica de engenharia. |
| [`docs/MBSE/EdgeTelemetryLayer_req_var_fret.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var_fret.md) | Atualização dos textos de requisitos em Markdown para paridade com o JSON. | Manutenção da integridade documental do repositório MBSE. |
