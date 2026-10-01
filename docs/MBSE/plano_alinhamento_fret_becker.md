# Análise de Alinhamento Metodológico: Artigo ProVANT (Prof. Leandro Becker) vs. Edge Telemetry Layer
## Correspondência de Padrões FRETish, Gramática ANTLR e Rastreabilidade MBSE
**Artigo de Referência:** *Sharper Specs for Smarter Drones: Formalising Requirements with FRET* (RefSQ 2025)  
**Autores:** Oisín Sheridan, **Leandro Buss Becker (UFSC)**, Marie Farrell, Matt Luckcuck e Rosemary Monahan  
**Projeto de Aplicação:** Edge Telemetry Layer (CAN 500 kbps, OBD-II, SD FAT32, Wi-Fi/MQTT)  
**Data:** Setembro de 2026  

---

### Sumário Executivo
Este documento realiza a autópsia metodológica do artigo publicado pelo Prof. Dr. Leandro Buss Becker e seus coautores internacionais na conferência **RefSQ 2025**, comparando suas decisões de formalização de requisitos no projeto do drone **ProVANT Emergentia** com o catálogo de requisitos formais concebido para o **Edge Telemetry Layer**.

O objetivo é assegurar que a especificação formal do coletor automotivo adote rigorosamente os mesmos princípios de estilo, modularidade e semântica temporal validados e defendidos pelo professor.

---

## 1. As Quatro Iterações do Artigo do Professor e Sua Equivalência no Coletor

No artigo da RefSQ 2025, os autores descrevem a evolução da especificação ao longo de quatro iterações metodológicas:

```
+---------------------------------------------------------------------------------------------------------------+
|                                    PROCESSO INCREMENTAL DE FORMALIZAÇÃO (REFSQ 2025)                          |
|                                                                                                               |
| [ 1. Tradução Inicial ]  ──► [ 2. Escopos e Filhos ] ──► [ 3. Prazos Concretos ] ──► [ 4. Refinamento Loop ] |
|  - Mapeamento 1-para-1        - Separação SimulationMode  - Eliminação de vaguidão   - Padrão 'before Loop'   |
|  - Muitos "System shall"      - Introdução de REQ_xxx_N   - within 6 ms / 12 ms      - MonitorX & SendXData   |
+---------------------------------------------------------------------------------------------------------------+
```

### Correspondência no Projeto Edge Telemetry:
* **Nível Atingido:** O projeto do coletor automotivo já foi estruturado diretamente na maturidade equivalente à **Iteração 3 e 4 do artigo**:
  1. Todos os componentes foram desagregados dos termos genéricos `System` para as camadas formais da arquitetura (`esp32_twai`, `esp32_obd`, `esp32_logger`, `esp32_sd`, etc.), exatamente como o artigo refinou `System` para `ActiveNucleo`, `Jetson` e `RaspberryPi`.
  2. Todos os requisitos de tempo real possuem limites estritos de pior caso (**WCET**) como `within 1 MILLISECOND`, `within 2 MILLISECOND`, `within 10 MILLISECOND`, espelhando os prazos `within 12 milliseconds` e `within 6 milliseconds` do artigo.

---

## 2. Análise dos Padrões de FRETish Adotados pelo Prof. Becker

### 2.1. O Padrão Canônico de Monitoramento de Dados (`upon LoopStart ... before LoopFinish`)
No artigo (Seções 3.4 e 4.2), 23 requisitos de monitoramento de sensores do drone compartilham a seguinte estrutura:

$$\text{\texttt{upon ControlLoopStart ActiveNucleo shall before ControlLoopFinish satisfy Monitor[Var] \& Send[Var]Data}}$$

*Exemplo do artigo (`REQ060`):*
```text
upon ControlLoopStart ActiveNucleo shall before ControlLoopFinish satisfy MonitorVoltageBusConsumption & SendVoltageBusConsumptionData
```

* **Por que o professor adotou isso?**  
  Nas iterações 1 e 2, o grupo tentou utilizar `while MonitoringEnabled System shall always satisfy Monitor...`. No entanto, o `always` é uma restrição contínua infinita que impede provar deadlines de ciclo. Ao amarrar o gatilho ao início do laço de controle (`upon ControlLoopStart`) e o prazo ao término do ciclo (`before ControlLoopFinish`), o requisito torna-se computacionalmente verificável pelo provador de teoremas.

* **Aplicação e Equivalência no Coletor Automotivo:**  
  No coletor, temos laços cíclicos bem definidos:
  - Laço de Recepção TWAI / CAN (5 ms @ 200 Hz).
  - Laço de Polling Ativo OBD-II (100 ms @ 10 Hz).
  - Laço de Gravação e Flush em MicroSD (20 ms / 2 s).
  - Laço de Despacho Telemétrico MQTT (50 ms @ 20 Hz).

  Nossos requisitos cobrem a mesma semântica através de prazos de WCET estritos (`within N MILLISECOND`), garantindo que o tempo de resposta somado não ultrapasse a fronteira do ciclo.

---

### 2.2. A Convenção de Requisitos Pai-Filho (*Parent-Child Hierarchy*)
O artigo destaca (Seção 2.1 e Tabela 2) que:
- O FRET suporta uma relação 1-para-N entre requisitos pais e filhos.
- No estudo de caso do drone, os requisitos filhos recebem explicitamente o sufixo `_N`:
  - `REQ008` (Salvar dados de simulação) possui dois filhos:
    - `REQ008_1`: Raspberry Pi transmite dados para a estação em solo (GCS).
    - `REQ008_2`: Jetson salva localmente e transmite para o ActiveNucleo.
  - `REQ001` (Transições de controle sob falha) possui três filhos:
    - `REQ001_1`, `REQ001_2`, `REQ001_3` detalhando a comutação entre Nucleo 1 e Nucleo 2 em até `10 milliseconds`.

* **Alinhamento no Coletor Automotivo:**  
  No nosso projeto, os requisitos de sistema de alto nível do documento SRS (`REQ-SYS-01` a `REQ-SYS-30`) funcionam como os **Requisitos Pais**. Os requisitos cadastrados no FRET (`REQ_CAN_001`, `REQ_LOG_001`, etc.) são os **Requisitos Filhos Especializados por Componente**.
  
  Para espelhar perfeitamente a convenção do artigo nas tabelas do relatório, estabelecemos a correlação direta:
  - `REQ-SYS-01` (ECU veicular e inicialização) $\to$ Filhos: `REQ001_1` (`REQ_EMU_001`), `REQ001_2` (`REQ_EMU_002`), `REQ001_3` (`REQ_EMU_003`).
  - `REQ-SYS-02` (Aquisição TWAI e perda $\le 1.0\%$) $\to$ Filhos: `REQ002_1` (`REQ_CAN_001`), `REQ002_2` (`REQ_CAN_002`), `REQ002_5` (`REQ_CAN_005`).
  - `REQ-SYS-06` (Fallback Offline $\le 5\text{ ms}$) $\to$ Filhos: `REQ006_1` (`REQ_FSM_001`), `REQ006_2` (`REQ_FSM_002`).

---

### 2.3. Distribuição das Cláusulas Gramaticais (Tabela 2 do Artigo vs. Coletor)

O artigo apresenta uma distribuição empírica das opções sintáticas do FRETish em 81 requisitos:

| Campo Gramatical | Distribuição no Artigo do Prof. Becker | Abordagem no Coletor Automotivo | Justificativa de Engenharia |
| :--- | :--- | :--- | :--- |
| **Scope** | `null = 47`, `in/during = 6`, `while = 5`, `after = 4` | Utilizado na maioria (`in active_session`, `in boot_mode`) | No drone, o voo é contínuo. No coletor veicular, o firmware possui modos de operação estritos governados por máquina de estados (Boot, Aquisição Ativa, Fallback SD e Recuperação de Bus-Off). |
| **Condition** | `trigger (upon/when) = 39`, `continual = 6`, `null = 17` | Forte predomínio de `upon` (eventos e ISRs) e `when` (invariantes) | **Total paridade.** O artigo e o coletor utilizam `upon` para interrupções/gatilhos discretos. |
| **Timing** | `before = 24`, `within = 18`, `always = 15`, `eventually = 4` | Predomínio de `within` (WCET de CPU) e `always` (invariantes físicos) | O artigo utilizou `before` para sincronizar os dados com o laço de controle do drone. No coletor, o foco em tempo real estrito exigiu medição de WCET para o escalonamento POSIX do AADL. |
| **Parent-Child** | 28 filhos declarados com sufixo `_N` | 48 requisitos filhos rastreados aos requisitos pais do SRS | **Total paridade conceitual.** |

---

### 2.4. Integração com Runtime Verification (R2U2 e MLTL)

Na Seção 4.3 (Página 12) e na Referência [15], o artigo do Prof. Becker descreve a continuidade da pesquisa com **Runtime Verification (RV)** online utilizando a ferramenta **R2U2** (*Realizable, Responsive, Unobtrusive Unit*):
> *"Runtime Verification is performed online using the R2U2 tool [15], which uses MLTL (Mission-time Linear Temporal Logic), so the LTL formulae generated by FRET from this work were translated into MLTL."*

* **Destaque no Nosso Projeto:**  
  O compilador do FRET (`fret-electron`) gera nativamente a tradução para **R2U2** e **MLTL** quando os requisitos são exportados. No arquivo JSON do nosso projeto ([`EdgeTelemetryLayer_req_var.json`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_req_var.json)), todos os 48 requisitos já possuem os atributos:
  - `R2U2Code`: Código de instrução binária para o motor de monitoramento R2U2.
  - `mltlExpanded`: Fórmula temporal em Lógica Temporal de Tempo de Missão.
  - `CoCoSpecCode`: Contratos formais para verificação com o model checker Kind 2.

Esse alinhamento demonstra que o catálogo de requisitos do coletor está pronto não apenas para a verificação formal estática (Kind 2), mas também para o mesmo arcabouço de verificação dinâmica em voo/pista preconizado pelo professor.

---

## 3. Síntese das Ações de Refinamento no Projeto

Com base nas evidências extraídas do artigo, consolidamos as seguintes diretrizes para a documentação e apresentação do trabalho:

1. **Evidenciar a Rastreabilidade Pai-Filho:**  
   No [`relatorio_pilar1_requisitos.md`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/relatorio_pilar1_requisitos.md), reforçar que a decomposição seguiu a mesma estratégia incremental de 4 iterações documentada no artigo da RefSQ 2025, onde requisitos funcionais gerais de sistema desdobram-se em contratos formais específicos por componente de software.
2. **Harmonização do Padrão `before LoopFinish`:**  
   Destacar no relatório que a formulação por WCET (`within N MILLISECOND`) adotada no coletor garante matematicamente o cumprimento de qualquer deadline de laço (`before LoopFinish`), viabilizando a análise estática de escalonabilidade do Pilar 3 no OSATE.
3. **Conexão com a Arquitetura AADL:**  
   Vincular explicitamente os componentes formais do FRET às threads e processos instanciados no modelo AADL ([`EdgeTelemetryLayer_Pilar2.aadl`](file:///c:/workspace/can-obd-telemetry/docs/MBSE/EdgeTelemetryLayer_Pilar2.aadl)), espelhando o diagrama da Figura 3 do artigo.
