# Guia Prático de Bancada HIL (Hardware-In-The-Loop) — Edge Telemetry Layer

Este documento descreve detalhadamente o passo a passo para configurar os ambientes de desenvolvimento, realizar a fiação elétrica de bancada e executar a coleta de dados de telemetria utilizando a arquitetura **Edge Telemetry Layer** (Coletor ESP32-S3 + Emulador Arduino UNO R3).

---

## 1. Configuração dos Ambientes de Desenvolvimento

### 1.1 Emulador de ECU (Arduino UNO R3)

O emulador simula o comportamento elétrico e de protocolos do veículo na bancada, enviando frames de dados sintéticos e respondendo a solicitações OBD-II.

1. **Instalar a Arduino IDE:** Baixe e instale a versão mais recente da Arduino IDE no seu sistema operacional.
2. **Instalar a biblioteca MCP2515:**
   - Acesse *Rascunho (Sketch)* -> *Incluir Biblioteca* -> *Gerenciar Bibliotecas*.
   - Procure por `MCP2515` (recomenda-se a biblioteca `coryjfowler/MCP_CAN_lib` na versão 1.5.0 ou superior).
   - Instale a biblioteca.
3. **Carregar o código:**
   - Abra o arquivo [simulador_sandero_r3.ino](file:///c:/workspace/can-obd-telemetry/src/cpp/arduino/simulador_sandero_r3/simulador_sandero_r3.ino) na Arduino IDE.
   - Conecte o Arduino UNO R3 ao computador via USB.
   - Selecione a placa `Arduino Uno` e a respectiva porta COM em *Ferramentas*.
   - Clique em *Carregar (Upload)* para gravar o firmware.

### 1.2 Coletor de Dados (ESP32-S3)

O coletor adquire os dados do barramento CAN e da interface OBD-II, estruturando-os e enviando-os para o cartão SD local e Broker MQTT.

1. **Instalar o Toolchain de Rust para ESP32:**
   - Instale o utilitário de instalação do ecossistema Espressif em Rust (`espup`):
     ```bash
     cargo install espup
     espup install
     ```
   - Siga as instruções do instalador para carregar as variáveis de ambiente em seu terminal.
2. **Instalar a ferramenta de gravação:**
   - Instale o utilitário `espflash` (se ainda não estiver instalado) para gravação direta via porta serial:
     ```bash
     cargo install espflash
     ```
3. **Configurar Credenciais e Compilar:**
   - Navegue até a pasta do coletor: `cd firmware/esp32s3_collector/`
   - Copie `src/config_local.rs.example` para `src/config_local.rs` e preencha suas credenciais de Wi-Fi e Broker MQTT (`WIFI_SSID`, `WIFI_PASSWORD`, `MQTT_BROKER_IP`, `MQTT_BROKER_OCTETS`, `MQTT_USER`, `MQTT_PASSWORD`).
   - Carregue as variáveis do compilador e grave na placa:
     ```bash
     source ~/export-esp.sh && cargo +esp run --release
     ```
   - *(Nota: O cargo-espflash tentará identificar a porta serial automaticamente. Sempre que alterar os dados em `src/config_local.rs`, é necessário rodar o comando acima novamente. As constantes em Rust são embutidas diretamente no binário durante a compilação).*

---

## 2. Diagrama de Conexão Elétrica (Fiação da Bancada HIL)

Para simular o ambiente elétrico automotivo, conectamos o emulador (Nó 1 - Arduino UNO R3 + MCP2515) ao coletor (Nó 2 - ESP32-S3 + Transceptor SN65HVD230) e o módulo SD Card.

### 2.1 Conexão do Emulador (Arduino UNO R3 ➔ MCP2515)

O módulo de controle MCP2515 se conecta ao microcontrolador ATmega328P do Arduino via barramento SPI padrão:

| Arduino UNO R3 | MCP2515 | Descrição |
| :---: | :---: | :--- |
| **5V** | VCC | Alimentação do módulo (5V) |
| **GND** | GND | Referência de terra |
| **D10** | CS | Chip Select (SPI) |
| **D11** | MOSI / SI | Master Out Slave In |
| **D12** | MISO / SO | Master In Slave Out |
| **D13** | SCK | Relógio de Barramento (SPI) |

### 2.2 Conexão do Coletor CAN (ESP32-S3 ➔ Transceptor CAN)

O transceptor CAN do coletor conecta-se ao periférico TWAI interno do ESP32-S3:

| ESP32-S3 | Transceptor CAN | Descrição |
| :---: | :---: | :--- |
| **3V3** | VCC | Alimentação do transceptor (3.3V) |
| **GND** | GND | Referência de terra |
| **GPIO 4** | TXD | Transmissão de dados CAN (TX) |
| **GPIO 5** | RXD | Recepção de dados CAN (RX) |

### 2.3 Conexão do Leitor SD Card (ESP32-S3 ➔ Módulo MicroSD SPI)

O leitor de cartão de memória conecta-se ao barramento SPI2 do ESP32-S3:

| ESP32-S3 | Módulo SD Card | Descrição |
| :---: | :---: | :--- |
| **5V0 / 5V** | +5V / VCC | Alimentação (5V recomendado para estabilidade) |
| **GND** | GND | Referência de terra |
| **GPIO 10** | CS | Chip Select (SPI) |
| **GPIO 11** | MOSI / DI | Master Out Slave In (Data In) |
| **GPIO 12** | SCK / CLK | Relógio do Barramento SPI |
| **GPIO 13** | MISO / DO | Master In Slave Out (Data Out) |

### 2.4 Conexão do Barramento CAN (Interconexão entre Nós)

Os nós se comunicam via par trançado diferencial de barramento físico CAN:

- 🟢 **CAN_H** do MCP2515 ➔ 🟢 **CAN_H** do Transceptor do ESP32-S3
- 🟡 **CAN_L** do MCP2515 ➔ 🟡 **CAN_L** do Transceptor do ESP32-S3
- ⚫ **GND** do Arduino ➔ ⚫ **GND** do ESP32-S3 (Terra comum obrigatório)
- 🔴 **Terminação:** Conectar um resistor de **120 Ω** em paralelo entre CAN_H e CAN_L nas duas pontas extremas do barramento (a resistência equivalente medida entre CAN_H e CAN_L deve ser de aproximadamente **60 Ω** com a bancada desligada).

---

## 3. Roteiro Prático do Experimento de Coleta

A figura abaixo apresenta a arquitetura de fluxos de dados utilizada para validação da bancada:

```text
  +----------------------+             +---------------------+
  |     Emulador CAN     |  Sinal CAN  |      Coletor        |
  |   (Arduino UNO R3)   |===========> |     (ESP32-S3)      |
  |  MCP2515 C++ / TJA   |  500 Kbps   | Rust/Embassy ➔ SD   |
  +----------------------+             +----------+----------+
                                                  |
                                                  | MQTT (JSON)
                                                  v
                                       +----------+----------+
                                       |    Broker MQTT      |
                                       |   <IP_BROKER>       |
                                       +----------+----------+
                                                  |
                         +------------------------+------------------------+
                         |                                                 |
                         v                                                 v
             +-----------+-----------+                         +-----------+-----------+
             |    MQTT Explorer/     |                         |  Node-RED / Telegraf  |
             |       Mosquitto       |                         |      (Integração)     |
             | Envia comando RESET e |                         | InfluxDB (Opcional    |
             | rotulagem (/command)  |                         | para monitoramento)   |
             +-----------------------+                         +-----------------------+
```

1. **Ligar o Hardware:** Conecte o ESP32-S3 e o Arduino UNO R3 à fonte de alimentação USB de 5V. O coletor ESP32-S3 se conectará automaticamente à rede Wi-Fi configurada.
2. **Limpar a Memória e Iniciar Sessão com Temporização (Terminal 1 ou MQTT Explorer):** Antes de ligar o emulador de condução, envie um comando para o tópico `/coach/command`. Você pode passar o modo, tempo de duração em minutos (ex: `10` para os 10 min do artigo, ou `0` para tempo contínuo) e opcionalmente o timestamp Epoch em milissegundos para alinhamento no InfluxDB:
   ```bash
   # Formata o SD e inicia Sessão 1 (Econômico, 10 min, com Epoch opcional)
   mosquitto_pub -h <IP_BROKER> -t "/coach/command" -m "RESET,ECO,10,1724498400000"
   ```
3. **Controle de Sessões e Automação de Bancada:**
   - **Trocar de Modo / Nova Sessão com Temporizador:**
     ```bash
     mosquitto_pub -h <IP_BROKER> -t "/coach/command" -m "SPT,10"  # Esportivo por 10 min
     mosquitto_pub -h <IP_BROKER> -t "/coach/command" -m "NOR,10"  # Normal por 10 min
     ```
   - **Encerrar Sessão Ativa:**
     ```bash
     mosquitto_pub -h <IP_BROKER> -t "/coach/command" -m "STOP"
     ```
   - **Consultar Sessões Existentes no SD Card:**
     ```bash
     mosquitto_pub -h <IP_BROKER> -t "/coach/command" -m "LIST_SESSIONS"
     # O ESP32-S3 responderá no tópico /system/status com o JSON da lista de arquivos
     ```
   - **Solicitar Replay de Histórico sob Demanda:**
     ```bash
     mosquitto_pub -h <IP_BROKER> -t "/coach/command" -m "REPLAY,SESSION,1"
     # O histórico será transmitido de forma cooperativa no tópico /telemetry/replay
     ```
4. **Gerar Dataset do Experimento e Analisar:** Ao finalizar a sessão, remova o SD Card da placa e copie os arquivos `S_0001.CSV` (Econômico) e `S_0002.CSV` (Esportivo) para a máquina de análise. Na pasta do projeto, utilize os utilitários da pasta `tools/` para validar e gerar métricas de performance (memória SRAM, frames perdidos, latência, etc):
   ```bash
   python tools/compute_metrics.py S_0001.CSV
   ```
5. **Teste de Tolerância a Falhas (Bus-Off Recovery - AC-07):** Durante a coleta em andamento, utilize um jumper ou fio condutor para colocar os terminais CAN_H e CAN_L (ou CAN_H e GND) em curto-circuito físico proposital. O sistema parará de receber telemetria e sinalizará o erro de Bus-Off. Remova o curto. A arquitetura de detecção e os Signals recuperarão cooperativamente o barramento via acesso síncrono *unsafe* no firmware, sem reiniciar o ESP32-S3. Este evento ficará registrado com a label `DIAG` no arquivo CSV gerado (ex: `BUS_OFF_RECOVERED_latency_ms=128`).