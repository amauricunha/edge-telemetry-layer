# Política de Segurança

## Versões suportadas

Apenas a branch `main` recebe correções de segurança.

## Reportando uma vulnerabilidade

**Não abra uma *issue* pública** para problemas de segurança.

Use o recurso [Private vulnerability reporting](https://github.com/amauricunha/edge-telemetry-layer/security/advisories/new)
do GitHub ou envie um e-mail para **dev.cunha@outlook.com** com:

- descrição do problema e impacto;
- passos para reproduzir (firmware, versão, hardware);
- sugestão de correção, se houver.

Resposta inicial em até 7 dias.

## Boas práticas para quem usa este projeto

- **Nunca** versione credenciais. Use os modelos:
  - `firmware/esp32s3_collector/src/config_local.rs.example` → `config_local.rs`
  - `tools/.env_exemplo` → `tools/.env`
- Os valores `changeme*` em `DevOps/docker-compose.yml` são apenas *placeholders*;
  defina `INFLUX_PASSWORD`, `INFLUX_TOKEN`, `GRAFANA_ADMIN_PASSWORD`, `MQTT_USER`
  e `MQTT_PASSWORD` em um `.env` antes de subir a stack.
- Não exponha o broker MQTT (1883) nem o InfluxDB à Internet sem autenticação e TLS.
- O barramento CAN/OBD-II é acessado em modo de leitura/diagnóstico; testes em veículos
  reais são de responsabilidade do usuário.
