# Guia Completo de Instalação e Execução do NASA FRET no WSL2 (Ubuntu 24.04)

Este documento descreve detalhadamente todas as configurações, correções, instalações de ferramentas e passos executados no **WSL2 (Windows Subsystem for Linux)** para compilar e rodar a ferramenta **NASA FRET (Formal Requirements Elicitation Tool)** com interface gráfica nativa no Windows.

---

## 1. Visão Geral da Arquitetura

O NASA FRET é uma aplicação de desktop desenvolvida em **Electron**, com frontend em **React / Material-UI** e backend local em **Node.js**, integrado a motores de verificação formal em C/C++ e SMT Solvers (**Z3**, **LTLSIM / NuSMV**).

- **Sistema Operacional Hospedeiro:** Windows 10/11 (sem necessidade de privilégios de Administrador).
- **Ambiente de Execução:** WSL2 com distribuição **Ubuntu 24.04 LTS**.
- **Interface Gráfica (GUI):** Renderizada nativamente na área de trabalho do Windows através do **WSLg** (Wayland / XWayland).
- **Acesso ao Código:** Diretamente no sistema de arquivos do Windows em `/mnt/c/workspace/fret`.

---

## 2. Dependências de Sistema e Solvers (Ubuntu)

Para suportar o Electron e os motores formais, as seguintes bibliotecas e utilitários foram configurados no Ubuntu:

### 2.1. Bibliotecas Gráficas do Electron (Ubuntu 24.04)
O Ubuntu 24.04 usa versões empacotadas com sufixo `t64` para compatibilidade temporal de 64 bits:
```bash
sudo apt update
sudo apt install -y \
  libgtk-3-0t64 \
  libdrm2 \
  libgbm1 \
  libnss3 \
  libx11-xcb1 \
  libasound2t64
```

### 2.2. Ferramentas de Compilação
Necessárias para compilar o simulador de LTL (`ltlsim`) e módulos nativos do Node (`node-gyp`):
```bash
sudo apt install -y build-essential gcc g++ make git python3
```

### 2.3. SMT Solver Z3
Instalado via repositório de pacotes para dar suporte aos procedimentos formais de realizabilidade:
```bash
sudo apt install -y z3
```

### 2.4. Model Checker Kind 2 (v2.2.0 - Recomendado pela NASA)
O **Kind 2** é um provador de teoremas e model checker multi-engine baseado em SMT para a linguagem Lustre.
> **Nota Oficial do FRET:** A versão suportada é estritamente a **v2.2.0** (a versão v2.3.0+ quebra a compatibilidade da CLI com o FRET).

Instalação do binário pré-compilado para Linux x86_64:
```bash
curl -sL https://github.com/kind2-mc/kind2/releases/download/v2.2.0/kind2-v2.2.0-linux-x86_64.tar.gz -o /tmp/kind2.tar.gz
tar -xzf /tmp/kind2.tar.gz -C /tmp
sudo cp /tmp/kind2 /usr/local/bin/kind2
sudo chmod +x /usr/local/bin/kind2

# Validação da versão
kind2 --version  # Retorna: kind2 v2.2.0
```

### 2.5. Java Runtime Environment (OpenJDK 21)
Necessário para a execução do motor formal **JKind** e de seu wrapper de realizabilidade (`jrealizability`):
```bash
sudo apt install -y default-jre-headless unzip

# Validação do Java
java -version  # Retorna: openjdk version "21.0.x"
```

### 2.6. Suite JKind & JRealizability (v4.5.2)
O **JKind** é o motor de verificação formal alternativo utilizado pelo FRET para provas indutivas e checagem composicional de realizabilidade:
```bash
curl -sL https://github.com/loonwerks/jkind/releases/download/v4.5.2/jkind-4.5.2.zip -o /tmp/jkind.zip
sudo mkdir -p /opt/jkind
sudo unzip -o /tmp/jkind.zip -d /opt/
sudo chmod +x /opt/jkind/jkind /opt/jkind/jrealizability /opt/jkind/jlustre2kind /opt/jkind/jlustre2excel

# Criação de links simbólicos globais no PATH
sudo ln -sf /opt/jkind/jkind /usr/local/bin/jkind
sudo ln -sf /opt/jkind/jrealizability /usr/local/bin/jrealizability
sudo ln -sf /opt/jkind/jlustre2kind /usr/local/bin/jlustre2kind
sudo ln -sf /opt/jkind/jlustre2excel /usr/local/bin/jlustre2excel

# Validação das ferramentas
jkind -help
jrealizability -help
```

### 2.7. Model Checker NuSMV (v2.6.0 - Motor de Test Case Generation e LTLSIM)
O **NuSMV** é o model checker simbólico baseado em BDD/SAT utilizado pelo FRET tanto para a simulação interativa de LTL (`LTLSIM`) quanto para a geração de casos de teste em componentes booleanos (`TEST CASE GENERATION`):
```bash
curl -sL https://nusmv.fbk.eu/distrib/NuSMV-2.6.0-linux64.tar.gz -o /tmp/NuSMV.tar.gz
tar -xzf /tmp/NuSMV.tar.gz -C /tmp
sudo cp /tmp/NuSMV-2.6.0-Linux/bin/NuSMV /usr/local/bin/NuSMV
sudo ln -sf /usr/local/bin/NuSMV /usr/local/bin/nusmv
sudo chmod +x /usr/local/bin/NuSMV

# Validação do NuSMV
NuSMV -h
```

---

## 3. Configuração do Node.js (via NVM)

O FRET necessita de uma versão do Node.js compatível com o `node-sass 7.0.3` e bibliotecas internas de AST (`v16.16.x` a `v20.x`). O Node v24 que estava ativo causava quebra de compatibilidade.

### 3.1. Correção do Arquivo `~/.bashrc` (Fim de Linha CRLF -> LF)
Identificamos que o `~/.bashrc` continha quebras de linha Windows (`CRLF`), o que fazia o bash procurar por `nvm.sh\r` e falhar ao carregar o NVM. O arquivo foi convertido para padrão Unix (`LF`):
```bash
sed -i 's/\r$//' ~/.bashrc
```

### 3.2. Instalação e Ativação do Node v20
Foi configurada a versão estável Node v20 via NVM:
```bash
# Inicialização do NVM no ~/.bashrc
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# Instalação do Node 20
nvm install 20
nvm alias default 20
nvm use 20
```
- **Versão validada:** Node `v20.20.2` com npm `10.8.2`.

---

## 4. Estrutura de Diretórios Obrigatória

O FRET utiliza o **PouchDB / LevelDB** e assume como premissa a existência da pasta `~/Documents` no diretório home do usuário para armazenar seus bancos de dados locais:
- `~/Documents/fret-db` (Projetos e requisitos)
- `~/Documents/model-db` (Variáveis e modelos de análise)

Criação do diretório no WSL2:
```bash
mkdir -p ~/Documents
```

---

## 5. Compilação do Simulador C (`LTLSIM`)

O FRET inclui um simulador de lógica temporal em C localizado em `tools/LTLSIM/ltlsim-core/simulator`. Ele foi compilado com o compilador nativo `gcc`:

```bash
cd /mnt/c/workspace/fret/tools/LTLSIM/ltlsim-core
make -C simulator
npm install
```
Isso gerou o binário executável `ltlsim`, que foi adicionado ao `PATH` do sistema.

---

## 6. Instalação e Build do FRET

Com o ambiente configurado, as dependências do projeto foram resolvidas e os pacotes de produção foram gerados:

```bash
cd /mnt/c/workspace/fret/fret-electron

# 1. Instalação de dependências e compilação das DLLs Webpack
npm install

# 2. Build dos bundles de produção (CLI, Processo Principal Electron e Interface React)
npm run build
```

Arquivos compilados com sucesso:
- `app/cli/fretCLI.main.js` (Interface de Linha de Comando)
- `app/main.prod.js` (Processo Principal Electron)
- `app/dist/renderer.prod.js` (Interface Gráfica React)
- `app/dist/style.css` (Estilos da Aplicação)

---

## 7. Tratamento de GPU e Renderização no WSLg

### 7.1. Diagnóstico da Mensagem de GPU
Ao iniciar o Electron no WSL2, a seguinte mensagem aparecia:
```text
[ERROR:viz_main_impl.cc(196)] Exiting GPU process due to errors during initialization
```
**Causa:** O Electron tenta inicializar aceleração gráfica por hardware na GPU do Windows através do driver D3D12 do WSLg. Quando ocorre uma incompatibilidade de driver virtual, o Chromium fecha o processo de GPU e recorre à renderização por software.

### 7.2. GPU é necessária para os cálculos formais?
**Não.** Os solvers lógicos (**Z3**, **NuSMV**, **LTLSIM**) trabalham com algoritmos determinísticos e árvores de decisão que rodam **100% em CPU e memória RAM**. A GPU é usada no Electron unicamente para desenhar a interface na tela.

### 7.3. Correção Aplicada
Para eliminar a falha de GPU e garantir abertura rápida e estável, o FRET foi configurado para inicializar com as flags:
- `--no-sandbox`: Permite execução segura dentro de containers/WSL.
- `--disable-gpu`: Desabilita a tentativa de aceleração 3D, utilizando renderização fluida por software (SwiftShader).

---

## 8. Scripts de Inicialização Criados

Dois scripts foram desenvolvidos para facilitar a inicialização diária do FRET:

### 8.1. `run_fret_wsl.sh` (Script Shell para WSL2)
Localizado na raiz do projeto (`c:/workspace/fret/run_fret_wsl.sh`), este script prepara todo o ambiente Linux antes de chamar o Electron:

```bash
#!/usr/bin/env bash
# NASA FRET WSL2 Launcher

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
nvm use 20 > /dev/null 2>&1 || true

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="$PATH:$SCRIPT_DIR/tools/LTLSIM/ltlsim-core/simulator"

export DISPLAY="${DISPLAY:-:0}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"

mkdir -p "$HOME/Documents"

echo "Iniciando o NASA FRET no WSL2 (modo gráfico)..."
echo "Aguarde alguns segundos enquanto a janela é carregada na tela."
cd "$SCRIPT_DIR/fret-electron"
./node_modules/.bin/electron --no-sandbox --disable-gpu ./app/ "$@"
```

### 8.2. `run_fret.bat` (Script Windows com 1 Clique)
Localizado na raiz do projeto (`c:/workspace/fret/run_fret.bat`), permite abrir o FRET com um duplo clique pelo Windows Explorer sem precisar abrir o terminal:

```bat
@echo off
title NASA FRET (WSL2)
echo ========================================================
echo   Iniciando NASA FRET via WSL2 (Ubuntu-24.04)
echo ========================================================
echo A interface grafica esta sendo inicializada...
echo (Mantenha esta janela aberta enquanto utiliza o FRET)
echo.
wsl.exe -d Ubuntu-24.04 bash -lic "/mnt/c/workspace/fret/run_fret_wsl.sh"
echo.
echo O NASA FRET foi finalizado.
pause
```

---

## 9. Isolamento do Repositório Git Local

Para evitar envios acidentais para o repositório público da NASA (`https://github.com/NASA-SW-VnV/fret`), o controle de versão foi isolado:

```bash
# Remoção do remote público
git remote remove origin

# Verificação
git remote -v  # (Retorna vazio, garantindo segurança)
```

Se você desejar vincular este projeto ao seu repositório privado no futuro:
```bash
git remote add origin <URL_DO_SEU_REPOSITORIO_PRIVADO>
git push -u origin local
```

---

## 10. Como Usar no Dia a Dia

1. **Pelo Windows:**
   - Dê um duplo clique no arquivo [`run_fret.bat`](file:///c:/workspace/fret/run_fret.bat).
   - O prompt do Windows abrirá e, em poucos segundos, a interface do FRET surgirá na sua tela.
   - *Nota:* Mantenha o prompt minimizado enquanto trabalha no FRET. Ao fechar o FRET, o prompt encerra automaticamente.

2. **Pelo Terminal do WSL2 (Ubuntu):**
   ```bash
   cd /mnt/c/workspace/fret
   ./run_fret_wsl.sh
   ```

---

## 11. Análise de Realizabilidade (Realizability Checking)

A análise de realizabilidade (*Realizability Checking*) verifica matematicamente se um conjunto de requisitos formais pode ser implementado por algum sistema físico/computacional sem que ocorram contradições, inconsistências temporais ou impasses sob quaisquer entradas válidas do ambiente.

### 11.1. Por que o FRET não suporta Realizability nativo no Windows?
Conforme detalhado no manual oficial da NASA:
> *"Note: Realizability checking is not currently supported in native Microsoft Windows installations."*

O backend do FRET invoca os executáveis dos *solvers* (`kind2`, `jrealizability`, `z3`) através de chamadas de processos filhos com semântica POSIX/Linux. Por este motivo, a execução do FRET dentro do **WSL2** é indispensável para habilitar este recurso.

### 11.2. Como o FRET Detecta as Dependências Internamente
Ao carregar a aba **REALIZABILITY CHECKING**, o arquivo `model/realizabilitySupport/realizabilityUtils.js` executa a função `checkDependenciesExist()`, buscando pelos binários no `PATH`:
- Configuração 1 (Padrão): `['kind2', 'z3']`
- Configuração 2: `['jkind', 'z3']` (requer `jkind`, `jrealizability` e `z3`)

Se nenhum conjunto completo for encontrado, o FRET exibe o aviso:
```text
"Dependencies missing for realizability checking. Click 'HELP' for details."
```
e desabilita o botão **ACTIONS**. Com a instalação realizada nas seções 2.4, 2.5 e 2.6, ambos os conjuntos ficam 100% disponíveis (`Missing: []`).

### 11.3. Procedimento para Executar a Análise

1. No menu superior ou lateral do FRET, acesse o **Analysis Portal**.
2. Abra a aba **`VARIABLE MAPPING`** e certifique-se de que todas as variáveis do componente estão com:
   - **Role:** `Input`, `Output` ou `Internal`
   - **Type:** `Boolean`, `Integer` ou `Double`
   - **Completed:** Marcado como concluído
3. Vá para a aba **`REALIZABILITY CHECKING`**.
4. Selecione o **System Component** desejado:
   - `uno_ecu_emulator` (8 requisitos do emulador Arduino)
   - `esp32s3_collector` (40 requisitos do nó coletor ESP32-S3)
5. Escolha o tipo de verificação:
   - **Monolithic:** Analisa todos os requisitos do componente de uma única vez em um único bloco lógico.
   - **Compositional:** Decompõe automaticamente o sistema em componentes conexos (**CC0**, **CC1**, **CC2**...), analisando subconjuntos desacoplados de saídas, o que torna a verificação muito mais rápida.
6. Clique em **ACTIONS** → **Check Realizability**.
7. O FRET executará o solver e apresentará o resultado:
   - **Realizable: True (Verde):** A especificação é formalmente consistente e realizável.
   - **Unrealizable (Vermelho):** Há conflito entre requisitos. Clique em **ACTIONS** → **Diagnose Unrealizable Requirements** para abrir o diagrama de acordes (*Chord Diagram*) e simular o contraexemplo interativo no **LTLSim**.


Viewed SKILL.md:1-100

# Masterclass NASA FRET & MBSE: Do Requisito ao Código sem Tentativa e Erro

---

## 1. A Mudança de Mentalidade: O Fluxo em V do MBSE

No desenvolvimento tradicional de sistemas embarcados, é muito comum o fluxo por "tentativa e erro": compra-se a placa, escreve-se o código no microcontrolador, testa-se na bancada e, quando algo falha (um buffer estoura, um temporizador entra em conflito ou o barramento trava), tenta-se corrigir com `delay()` ou flags improvisadas.

Em sistemas críticos (aeroespacial, automotivo, médico e telemetria de missão crítica), esse processo é proibitivo. A metodologia **MBSE (Model-Based Systems Engineering)** inverte essa lógica através do **Ciclo em V**:

```
        1. REQUISITOS FORMAIS (NASA FRET)
           - Semântica LTL matematicamente comprovável
           - Prova de Realizabilidade preventiva (sem código!)
                \                                  /   4. VERIFICAÇÃO & TESTES
                 \                                /       - Test Case Generation
                  \                              /        - Rastreabilidade AC-01 a AC-08
            2. ARQUITETURA DE SISTEMA (OSATE / AADL)     /
               - Modelagem de barramentos, memórias     /
               - Orçamento temporal e taxas de threads /
                    \                                 /
                     \                               /
                      3. IMPLEMENTAÇÃO DE FIRMWARE
                         - Rust no_std (Embassy) / C++ Bare-Metal
                         - Zero alocação dinâmica, determinístico
```

### O que é o NASA FRET e por que a NASA o criou?
Requisitos em linguagem natural livre (como "o sistema deve ler o CAN rápido e salvar no SD") são ambíguos, incompletos e contraditórios. O **NASA FRET (Formal Requirements Elicitation Tool)** resolve isso:
1. Permite escrever em uma linguagem natural controlada chamada **FRETish** (legível por humanos).
2. O compilador interno do FRET (baseado em ANTLR 4) traduz essa frase automaticamente para **Lógica Temporal Linear (LTL)** e **Lustre/CoCoSpec**.
3. O FRET aciona provadores matemáticos de teoremas (**Kind 2**, **JKind**, **Z3**) para provar se os requisitos são **Realizáveis (compatíveis e livres de conflitos)** antes de escrever uma única linha de código C++ ou Rust.

---

## 2. Passo 1: Como Começar o Projeto do Nosso Coletor

Antes de abrir o FRET, o engenheiro de sistemas responde a duas perguntas fundamentais:
1. **Qual é a Fronteira do Sistema (System Boundary)?**
   - O que está *dentro* do nosso dispositivo e o que é o *ambiente externo*.
   - No nosso caso: o hardware do ESP32-S3 é a fronteira física; o barramento CAN externo, o cartão MicroSD e a rede Wi-Fi são o ambiente externo.
2. **Quais são os Componentes Lógicos?**
   - Como vimos, mesmo rodando na **mesma CPU**, o firmware do coletor não é um bloco monolítico. Ele segue a arquitetura em camadas (**MCAL**, **BSW**, **APP**):
     - `esp32_twai`: Driver de baixo nível do barramento CAN.
     - `esp32_obd`: Máquina de polling OBD-II.
     - `esp32_logger`: Serializador CSV e gerenciador do anel SRAM.
     - `esp32_sd`: Driver SPI e sistema FAT32 do cartão MicroSD.
     - `esp32_telemetry`: Despachante de rede Wi-Fi / MQTT.
     - `esp32_fsm`: Máquina de estados de fallback offline.
     - `esp32_recovery`: Guardião de Bus-Off e Watchdog.
     - `esp32_cmd`: Processador de comandos remotos.

---

## 3. A Anatomia de uma Sentença FRETish

Toda sentença formal no FRET segue uma gramática estrita composta por até 5 blocos:

$$\underbrace{\text{[SCOPE]}}_{\text{Modo Operacional}} \quad \underbrace{\text{[CONDITIONS]}}_{\text{Gatilho / Evento}} \quad \underbrace{\text{COMPONENT}}_{\text{Quem executa}} \quad \text{shall} \quad \underbrace{\text{[TIMING]}}_{\text{Prazo WCET}} \quad \underbrace{\text{RESPONSES}}_{\text{Garantia Formal}}$$

Vamos dissecar cada bloco com o caso real do nosso coletor:

### Bloco 1: SCOPE (`in <mode>`)
Define **em qual estado operacional** o requisito é válido.
* Em vez de o sistema ser obrigado a fazer tudo o tempo todo, ele só age se estiver no modo correto:
  - `in boot_mode`: Válido apenas durante a inicialização do chip.
  - `in active_session`: Válido apenas quando a viagem/gravação foi iniciada.
  - `in offline_mode`: Válido apenas quando o Wi-Fi caiu.

### Bloco 2: CONDITIONS (`upon <trigger>` vs `when <condition>`)
Essa é a distinção mais importante que muitos erram:
* **`upon <trigger>` (Borda de Subida / Evento Discreto):**  
  Usado para eventos instantâneos, interrupções ou sinais que acabaram de acontecer.
  - *Exemplo real:* `upon can_frame_arrived` (um pacote acabou de chegar no hardware).
* **`when <condition>` (Nível Lógico / Estado Contínuo):**  
  Usado quando uma condição permanece verdadeira ao longo do tempo.
  - *Exemplo real:* `when can_bus_healthy` (enquanto o barramento estiver sem erros elétricos).
  - *Exemplo real:* `when buffer_occupancy >= 3584` (enquanto a memória estiver com 85% cheia).

### Bloco 3: COMPONENT (`the <component> shall`)
O nome higienizado do componente formal (sem caracteres especiais como `::`).
- *Exemplo:* `the esp32_twai shall` ou `the esp32_logger shall`.

### Bloco 4: TIMING (O Prazo Temporal)
Como descobrimos no nosso projeto, o timing deve seguir as regras de engenharia de tempo real:
* **`within N MILLISECOND` ($1 \le N \le 10$):** Prazo estrito de execução da CPU (**WCET**).
* **`immediately`:** No próprio ciclo de amostragem/passo seguinte.
* **`always`:** Invariante formal (deve ser verdadeiro em todos os instantes de tempo).
* **Padrão NASA Timer Handshake:** Para tempos longos ($\ge 128\text{ ms}$, segundos), dispara-se `timer_start` e reage-se `upon timer_expired`.

### Bloco 5: RESPONSES (`satisfy <predicate>`)
O que o componente garante como verdade:
* **Para Booleanos:** Escreve-se apenas o nome da variável! **Nunca use `= TRUE`**.
  - *Correto:* `satisfy frame_timestamp_captured`
  - *Incorreto:* `satisfy frame_timestamp_captured = TRUE` (o parser achará que `TRUE` é uma variável externa não declarada).
* **Para Valores Numéricos:** Comparações aritméticas formais.
  - *Exemplo:* `satisfy frame_loss_percentage <= 1.00`
  - *Exemplo:* `satisfy active_profile = 2`

---

## 4. Estudo de Caso Prático: Construindo um Requisito do Coletor do Zero

Vamos pegar um requisito crítico do nosso sistema e construí-lo juntos: **A captura do frame CAN pela camada MCAL (`REQ_CAN_002`)**.

### O Problema Físico na Bancada:
O Arduino emulador de motor transmite um frame CAN na velocidade de 500 kbps. O transceptor do ESP32-S3 recebe os pulsos elétricos nos pinos de silício. A CPU precisa registrar a marca temporal em microssegundos com deadline de tempo real para não acumular jitter.

### Passo A: Redigir a sentença em FRETish
Na janela do FRET, em **Requirement Description**, escrevemos:

```text
in active_session upon can_frame_arrived the esp32_twai shall within 2 MILLISECOND satisfy frame_timestamp_captured
```

Ao digitar, o editor do FRET colore instantaneamente cada termo:
- <span style="color:red">**in active_session**</span> (Vermelho: Escopo)
- <span style="color:orange">**upon can_frame_arrived**</span> (Laranja: Condição de disparo)
- <span style="color:green">**the esp32_twai shall**</span> (Verde: Componente responsável)
- <span style="color:blue">**within 2 MILLISECOND**</span> (Azul: Prazo temporal WCET)
- <span style="color:purple">**satisfy frame_timestamp_captured**</span> (Roxo: Resposta formal)

Se alguma palavra não ficar colorida, há um erro de sintaxe léxica (ex: escrever `ms` em vez de `MILLISECOND`).

---

## 5. Passo B: O Mapeamento de Variáveis (Variable Mapping)

Depois de criar o requisito, abrimos a aba **VARIABLE MAPPING**. Esta aba é o "esqueleto de tipos" do provador matemático.

Para cada variável que apareceu na sentença, devemos configurar duas propriedades cruciais:

```
┌──────────────────────────┬───────────┬───────────┬──────────────────────────────────────────────┐
│ Variável                 │ Role      │ Data Type │ Significado Físico no Firmware               │
├──────────────────────────┼───────────┼───────────┼──────────────────────────────────────────────┤
│ active_session           │ Internal  │ Boolean   │ Estado interno gerido pela máquina de modos  │
│ can_frame_arrived        │ Input     │ Boolean   │ Sinal/Interrupção física vinda do hardware   │
│ frame_timestamp_captured │ Output    │ Boolean   │ Sinal/Dado produzido e garantido pelo driver │
└──────────────────────────┴───────────┴───────────┴──────────────────────────────────────────────┘
```

### Por que o "Role" (Papel) é crucial?
* **Input (Entrada):** O solver considera que o ambiente externo tem controle total sobre essa variável. Ela pode oscilar a qualquer momento.
* **Output (Saída):** O solver verifica se o componente consegue forçar essa variável a assumir o valor correto sob todas as variações possíveis dos Inputs.
* **Internal (Interna):** Variável de estado mantida na memória do próprio componente.

---

## 6. Passo C: O Portal de Análise (Analysis Portal)

Aqui está o poder do FRET que substitui a tentativa e erro por rigor matemático. O Portal de Análise possui 4 funções principais:

### 1. Realizability Checking (Checagem de Realizabilidade)
* **O que faz:** Aciona o solver **Kind 2** ou **JKind** sobre a lógica Lustre gerada.
* **A Pergunta que o Solver Responde:** *"Considerando todos os inputs possíveis do ambiente (mesmo os piores cenários de ruído e concorrência), existe um algoritmo determinístico capaz de cumprir todas as obrigações sem violar nenhuma regra?"*
* **Resultado:**
  - **Círculo Verde com Checkmark (`Realizable: True`):** O conjunto de requisitos é matematicamente viável! Você tem garantia matemática de que é possível escrever o firmware.
  - **Círculo Vermelho com `X` (`Unrealizable: False`):** Há uma contradição lógica. O solver descobriu que duas ou mais regras são impossíveis de serem satisfeitas juntas.

### 2. Unrealizability Diagnosis (Diagnóstico de Conflitos)
Se der vermelho, você não precisa adivinhar onde errou:
* Clica-se em **`ACTIONS` → `Diagnose Unrealizable Requirements`**.
* O FRET exibe o **Grafo de Acordes (Chord Diagram)** e identifica o **Núcleo Insatisfatível Mínimo (Minimal Unsatisfiable Core)**:
  - Ele aponta exatamente: *"O requisito REQ-01 exige que o motor ligue em 5 ms sob sinal X, mas o requisito REQ-04 exige que o motor permaneça desligado enquanto o botão Y estiver pressionado. Existe um cenário onde X e Y ocorrem juntos e o controlador não tem como obedecer a ambos simultaneamente."*

### 3. Interactive Simulation (LTLSIM) — O que você viu na tela!
* O simulador gráfico do FRET permite rodar uma bancada de testes virtual antes do código:
  - Você pode forçar um valor no passo de tempo $t=0$, $t=1$, $t=2$ (por exemplo, simular a chegada de um pacote CAN).
  - O LTLSIM propaga a lógica e mostra com **pontos verdes** exatamente em qual passo temporal a saída é gerada e se o contrato foi satisfeito.

### 4. Test Case Generation (Geração Automática de Casos de Teste)
* O FRET decompõe os requisitos em **metas formais de cobertura (MC/DC)**.
* Utiliza o **NuSMV** (para lógica booleana) ou **Kind 2** (para lógica numérica) e sintetiza uma suíte de vetores de teste (tabelas de entradas e saídas esperadas) que você pode exportar e usar para validar o código Rust/C++ ou injetar no Simulink/CoCoSim.

---

## 7. Como Isso se Conecta com o OSATE (AADL)?

A ordem de um projeto MBSE de ponta a ponta segue este encadeamento:

1. **No NASA FRET (Camada de Contratos de Software):**  
   Você define as sentenças temporais, os limites de WCET e prova que os requisitos são consistentes (`Realizable`).
2. **No OSATE / AADL (Camada de Arquitetura de Sistema):**  
   Você modela os componentes físicos e lógicos:
   - Modela o barramento CAN como um `bus CAN_Bus { Bandwidth => 500 KBytesps; }`.
   - Modela o chip como um `processor ESP32S3`.
   - Modela as tarefas do firmware como `thread Task_TWAI_RX` e `thread Task_Logger`.
   - Vincula as portas dos componentes AADL às variáveis de `Input` e `Output` que você cadastrou no FRET!
   - Roda a análise de escalonabilidade (Schedulability Analysis) para garantir que a soma dos tempos WCET (os prazos de 1 ms, 2 ms, 10 ms do FRET) cabe dentro dos ciclos da CPU.
3. **No Código Real (Rust / C++):**  
   Você implementa as tarefas sabendo exatamente qual é a assinatura da função, quais canais conectar e qual é o limite de tempo estrito que cada função pode gastar.

---

## Próximos Passos da Aula

Esta foi a **Parte 1: A Fundamentação, o Pensamento por Contratos e o Caso de Estudo Inicial**.

Para a **Parte 2**, podemos explorar:
1. **Passo a passo no FRET:** Como criar um projeto novo do zero na interface gráfica sem herdar dados antigos.
2. **Casos com Números e Invariantes:** Como modelar variáveis de telemetria contínuas (ex: `speed_kmh`, `rpm`, `sram_usage_kb < 200.0`).
3. **Como exportar do FRET para o OSATE/Simulink:** Os formatos de saída e como alimentar as ferramentas seguintes.

O que achou desse primeiro módulo? Gostaria de aprofundar em algum ponto específico da gramática ou passar para o próximo passo?

Edited trabalho.md
Viewed trabalho.md:5-42
