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
