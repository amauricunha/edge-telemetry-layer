# Relatório Gate F1 — Edge Telemetry Layer

**Data de geração:** 2026-09-11 18:05  
**Arquivos analisados:** S_0001.CSV, S_0003.CSV, S_0005.CSV  
**Fase:** F1 — Bancada CAN + OBD-II  

---

## Critérios de Aceitação — Gate F1 (SPEC_F1 §7)

| ID | Critério | Threshold | Resultado | Status |
|---|---|---|---|---|
| **AC-01** | Taxa de recepção frames CAN | ≥ 99% | 100.0% | ✅ |
| **AC-02** | Latência média OBD-II | < 10 ms | 2.315 ms | ✅ |
| **AC-03** | Jitter OBD-II (desvio padrão) | < 3 ms | 3.356 ms | ❌ |
| **AC-04** | Integridade CSV | 0 nulos | 0 nulos | ✅ |
| **AC-05** | Uso de SRAM (Automático) | < 200 KB | 46.9 KB | ✅ |
| **AC-06** | Fallback SD automático | Funcionando | *(verificação manual)* | 🔲 |
| **AC-07** | Bus-Off recovery | Sem reinício SoC | *(verificação manual)* | 🔲 |
| **AC-08** | Dataset mínimo Dual-Source | ≥ 72.000 amostras | 73,333 | ✅ |

---

## Detalhes por Critério

### AC-01 — Taxa de Recepção CAN
55,820 frames DBC em 1800s (esperado ≈53998)

### AC-02 / AC-03 — Latência e Jitter OBD-II
n=17513 | média=2.31ms | σ=3.36ms | min=0.8ms | max=34.2ms | p95=8.5ms

**Distribuição de latência (percentis):**
| P25 | P50 (mediana) | P75 | P95 |
|---|---|---|---|
| 1.25 ms | 1.54 ms | 1.81 ms | 8.53 ms |

### AC-04 — Integridade do CSV
0 nulos em colunas obrigatórias; 0 fontes inválidas

### AC-05 — Uso de SRAM e Performance (HEARTBEAT)
Heap médio: 46.9 KB | Mínimo livre: 8.3 KB | (Estático < 150 KB)

#### Tabela 7 – Consumo de recursos do ESP32-S3 durante operação
| Recurso | Disponível | Utilizado | Percentual |
|---|---|---|---|
| SRAM interna (Heap) | 58 KB | 46.9 KB | 80.9% |
| Flash (firmware) | 16 MB | 400 KB | N/A |
| PSRAM (buffer telemetria) | 8 MB | 0.0 KB | < 0.1% |
| Tarefas Embassy ativas | — | 9 | — |

*Nota: A SRAM total é 512KB, mas o log rastreia a fração controlável do Heap (58KB). O restante é estático para as tasks e driver Wi-Fi.*

### AC-08 — Volume do Dataset

| Arquivo | Amostras |
|---|---|
| S_0001.CSV | 24,447 |
| S_0003.CSV | 24,451 |
| S_0005.CSV | 24,435 |
| **Total** | **73,333** |

---

## Verificações Manuais Restantes (AC-06, AC-07)

### AC-06 — Fallback SD
1. Iniciar coleta com Wi-Fi ativo (logs: `BSW Com: Conectando...`)
2. Desligar roteador durante coleta
3. Verificar log: `APP Logger: WIFI_DISCONNECTED — Modo Fallback SD ativado`
4. Religar roteador e verificar: `APP Logger: WIFI_RECONNECTED`
5. Montar SD no PC e verificar que CSV contém dados do período offline

### AC-07 — Bus-Off Recovery
1. Iniciar coleta com barramento CAN ativo
2. Desconectar cabo CAN brevemente (2–3 s)
3. Verificar log: `BSW Diag: *** BUS-OFF DETECTADO (evento #1) ***`
4. Verificar log: `BSW Diag: Janela de recovery Bus-Off concluída`
5. Verificar que coleta retoma **sem** log de reinício do SoC (`Edge Telemetry Layer v0.1.0`)

---

*Gerado automaticamente por `tools/compute_metrics.py`*  
*SPEC referência: SPEC_F1_Telemetria_CAN_OBD.md §7*
