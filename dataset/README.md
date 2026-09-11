# Dataset — Edge Telemetry Layer Fase 1

Este diretório armazena os datasets gerados durante as sessões de coleta da Fase 1 (bancada CAN + OBD-II).

> **Nota:** Os arquivos CSV reais **não são commitados** no repositório (ver `.gitignore`).
> Este README documenta como reproduzir a coleta e onde encontrar os dados.

---

## Estrutura esperada após coleta

```
dataset/
├── README.md               ← este arquivo
├── S_0001.CSV              ← Cenário C1: Econômico (10 min)
├── S_0002.CSV              ← Cenário C2: Normal (10 min)
├── S_0003.CSV              ← Cenário C3: Esportivo (10 min)
└── relatorio_gate_f1.md    ← Gerado por tools/compute_metrics.py
```

---

## Como gerar o dataset

### Pré-requisitos
- Hardware montado conforme `docs/pratica_bancada.md`
- SD Card formatado FAT32 inserido no módulo microSD
- ESP32-S3 conectado via USB
- Arduino UNO R3 com firmware `uno_ecu_emulator` carregado
- Broker MQTT acessível (ver `DevOps/docker-compose.yml`)

### Passo 1 — Flash do firmware ESP32-S3
```bash
cd firmware/esp32s3_collector
source ~/export-esp.sh
cargo +esp run --release
```

### Passo 2 — Coletar Cenário C1 (Econômico, 10 min)
1. Publicar no MQTT para definir o perfil:
   ```bash
   mosquitto_pub -h <IP_BROKER> -u <USUARIO> -P <SENHA> \
       -t /coach/session/label -m "Economico"
   ```
2. No Arduino, enviar comando CAN 0x010 byte 0 = 0x01 (Econômico)
3. Aguardar 10 minutos de coleta
4. O arquivo `S_0001.CSV` é gravado automaticamente no SD Card

### Passo 3 — Repetir para C2 (Normal) e C3 (Esportivo)
- C2: perfil 0x02, label "Normal" → `S_0002.CSV`
- C3: perfil 0x03, label "Esportivo" → `S_0003.CSV`

### Passo 4 — Copiar do SD Card
Montar o SD Card no PC e copiar os arquivos para este diretório:
```bash
cp /Volumes/SD_CARD/*.CSV dataset/
```

---

## Validação do dataset

```bash
cd <raiz do projeto>
pip install -r tools/requirements.txt

# Validar cada sessão individualmente
python tools/validate_csv.py dataset/S_0001.CSV
python tools/validate_csv.py dataset/S_0002.CSV
python tools/validate_csv.py dataset/S_0003.CSV

# Gerar relatório Gate F1 completo
python tools/compute_metrics.py dataset/S_0001.CSV dataset/S_0002.CSV dataset/S_0003.CSV

# Gerar figuras para o artigo
python tools/plot_session.py dataset/S_0001.CSV dataset/S_0002.CSV dataset/S_0003.CSV
```

---

## Schema das colunas

| Coluna | Tipo | Unidade | Descrição |
|---|---|---|---|
| `timestamp_ms` | uint64 | ms | Milissegundos desde boot do ESP32-S3 |
| `source` | string | — | `CAN_DBC`, `OBD_PID` ou `DIAG` |
| `can_id` | string | hex | Arbitration ID do frame CAN |
| `speed_kmh` | float | km/h | Velocidade (ID 0x100 / PID 0x0D) |
| `rpm` | float | rpm | Rotação do motor (ID 0x100 / PID 0x0C) |
| `throttle_pct` | float | % | Posição do acelerador (ID 0x200 / PID 0x11) |
| `engine_load_pct` | float | % | Carga do motor (ID 0x200 / PID 0x04) |
| `maf_g_s` | float | g/s | Fluxo de massa de ar (PID 0x10) |
| `coolant_temp_c` | float | °C | Temperatura do arrefecimento (ID 0x300 / PID 0x05) |
| `session_label` | string | — | Perfil ativo: `ECO`, `NOR`, `SPT` |
| `obd_latency_ms` | float\|vazio | ms | Latência OBD-II (apenas linhas `OBD_PID`) |

Referência completa: `SPEC_F1_Telemetria_CAN_OBD.md` §6.2
