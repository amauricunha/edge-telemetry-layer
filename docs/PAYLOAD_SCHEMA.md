# Formatos de Payload e Persistência (Edge Telemetry Layer)

Este documento descreve os formatos reais utilizados no projeto para a persistência local (SD Card) e a transmissão via MQTT, refletindo a arquitetura atualizada de alta performance (Fase 2).

> [!NOTE]
> O formato JSON original para telemetria de alta frequência foi descontinuado devido a gargalos de performance e alocação dinâmica (heap fragmentation). Em seu lugar, adotamos **CSV Estático no SD Card** e **Pacotes Binários sobre MQTT**.

---

## 1. Persistência Local no SD Card (CSV)

Todos os frames validados (DBC passivo e OBD-II ativo) são serializados sem alocação dinâmica em uma linha CSV pré-formatada. 

**Formato do Cabeçalho (11 colunas):**
`timestamp_ms,source,can_id,speed_kmh,rpm,throttle_pct,engine_load_pct,maf_g_s,coolant_temp_c,session_label,obd_latency_ms`

**Exemplo de Payload (DBC):**
`1500,CanDbc,0x100,50.5,,40.2,30.1,12.45,90.5,NORMAL,`

**Exemplo de Payload (OBD-II):**
`2000,ObdPid,0x7e8,,2500,,,,90.5,NORMAL,25.4`

### Eventos de Diagnóstico (DIAG)
Eventos do sistema (`LOGGER_STALL`, `BUS_OFF_EVENT_X`, `BUS_OFF_RECOVERED_...`) também são salvos no mesmo CSV, utilizando colunas nulas para manter a estrutura do schema:
`35000,DIAG,0x000,,,,,,,,NOR,BUS_OFF_EVENT_1`

---

## 2. Transmissão MQTT

A camada BSW gerencia a publicação MQTT em tópicos distintos, dependendo da natureza do dado.

### 2.1. Telemetria Bruta (Binário Compacto)

Para garantir throughput em redes intermitentes, a telemetria é agrupada em lotes binários. Cada registro utiliza a `struct BinaryFrame` (18 bytes com `#[repr(C, packed)]`).

- **Tópico:** `veicular/sc/florianopolis/pesquisa/sandeiro_can/3001/telemetria/S{id}/raw`
- **Formato:** *Binary Stream*
- **Estrutura C/Rust:**
  ```rust
  #[repr(C, packed)]
  pub struct BinaryFrame {
      pub timestamp_ms: u32,
      pub can_id: u16,
      pub source: u8, // 0 = CanDbc, 1 = ObdPid
      pub label: u8,
      pub val1: f32,
      pub val2: f32,
      pub obd_latency: u16,
  }
  ```

### 2.2. Status do Sistema (JSON)

Para monitoramento de saúde (*Health Check*), a placa envia um *heartbeat* a cada 30 segundos formatado em JSON.

- **Tópico:** `veicular/sc/florianopolis/pesquisa/sandeiro_can/3001/system/status`
- **Formato:** JSON (texto plano)
- **Exemplo de Payload:**
  ```json
  {
    "session_id": "S0001",
    "sd_ok": true,
    "wifi_ok": true,
    "frames_received": 1195,
    "lost": 0,
    "uptime_ms": 30000
  }
  ```
