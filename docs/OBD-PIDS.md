# Tabela OBD-II — Edge Telemetry Layer

Este arquivo resume a tabela de PIDs genéricos OBD-II utilizada pelo sistema **Edge Telemetry Layer** (coletor ESP32-S3 + emulador Arduino UNO R3) em bancada HIL.

## Como o coletor interpreta

O firmware no ESP32-S3 envia solicitações de diagnóstico periódicas para o identificador funcional `0x7DF` utilizando o Modo `0x01` (dados de diagnóstico em tempo real), requisitando um PID por vez. O emulador Arduino UNO R3 processa a solicitação e responde através do identificador físico `0x7E8`.

O frame de resposta OBD-II possui a seguinte estrutura:
- **Byte 0**: Quantidade de bytes de dados adicionais na resposta (ex: `0x03` ou `0x04`).
- **Byte 1**: Modo de resposta (Modo de requisição + `0x40`, resultando em `0x41` para o Modo 01).
- **Byte 2**: O PID correspondente à solicitação.
- **Bytes seguintes (3 em diante)**: Os bytes brutos que contêm os dados, a serem convertidos conforme a fórmula correspondente de cada PID.

---

## PIDs Suportados na Fase 1 (Bancada HIL)

A tabela abaixo descreve as seis variáveis de interesse principais selecionadas para o dataset e validadas nos experimentos de bancada do artigo:

| PID | Nome no Dataset | Unidade | Fórmula de Decodificação | Descrição do Sinal |
| :---: | :--- | :---: | :--- | :--- |
| **0x04** | `engine_load_pct` | % | `A * 100 / 255` | Carga calculada do motor |
| **0x05** | `coolant_temp_c` | °C | `A - 40` | Temperatura do líquido de arrefecimento |
| **0x0C** | `rpm` | rpm | `(A * 256 + B) / 4` | Rotações por minuto do motor |
| **0x0D** | `speed_kmh` | km/h | `A` | Velocidade linear do veículo |
| **0x10** | `maf_g_s` | g/s | `(A * 256 + B) / 100` | Fluxo de massa de ar admitido (proxy de consumo) |
| **0x11** | `throttle_pct` | % | `A * 100 / 255` | Posição do acelerador (intenção do condutor) |

---

## Estrutura de Amostra no Dataset (CSV)

As leituras OBD-II decodificadas são salvas na estrutura de arquivo CSV gerada pelo coletor com a seguinte assinatura:

```csv
timestamp_ms,source,can_id,speed_kmh,rpm,throttle_pct,engine_load_pct,maf_g_s,coolant_temp_c,session_label,obd_latency_ms
10245,OBD_PID,0x7E8,85.0,2450.0,30.0,42.5,12.45,87.0,Normal,4.2
```

- **timestamp_ms**: Tempo decorrido desde o início da sessão de coleta (em milissegundos).
- **source**: Identifica a origem do dado (`OBD_PID` para consultas ativas ou `CAN_DBC` para sniffing passivo).
- **can_id**: Identificador CAN de resposta (`0x7E8`).
- **session_label**: Rótulo da sessão correspondente ao perfil simulado (`Econômico`, `Normal` ou `Esportivo`).
- **obd_latency_ms**: Latência de ida e volta (intervalo temporal em milissegundos entre o envio de `0x7DF` e a chegada do frame `0x7E8`).

---

## Observações Importantes sobre o Sandero e Bancada HIL

- **Variáveis de CAN Raw Proprietárias (Fora de Escopo na Fase 1):** Sinais como *pressão de freio*, *ângulo do volante* e *aceleração lateral* dependem de frames de broadcast proprietários na rede CAN do Renault Sandero. Esses sinais não estão disponíveis via OBD-II genérico do Modo 01. Por necessitarem de engenharia reversa do arquivo DBC do fabricante com acesso físico ao veículo, essas variáveis estão fora de escopo para a bancada HIL da Fase 1 e serão introduzidas em etapas futuras (Fase 5).
- **Consumo Estimado:** O PID `0x10` (fluxo MAF) é utilizado como proxy para estimativa de consumo de combustível nas sessões simuladas.
- **Outros PIDs Genéricos:** PIDs como tensão do módulo de controle (`0x42`) ou nível de combustível (`0x2F`) são opcionais e não fazem parte do conjunto central de validação do dataset acadêmico.
