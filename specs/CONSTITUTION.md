# Project Constitution — Edge Telemetry Layer (F1)
version: 1.0.0
project: Edge Telemetry Layer / Fase 1
last-updated: 2025-06-30

## Princípios Não-Negociáveis

### P1 — Separação de camadas obrigatória
Todo código embarcado segue separação MCAL / BSW / RTE / APP.
Nenhuma tarefa de aplicação (APP) acessa diretamente registradores de hardware.
Toda comunicação cross-layer usa channels assíncronos do Embassy (`embassy::channel`).

### P2 — Zero alocação dinâmica em runtime
Proibido: `alloc`, `Box`, `Vec` do std, `String` do std em código de firmware.
Permitido: `heapless::Vec<T, N>`, `heapless::String<N>`, arrays estáticos com tamanho em compile-time.
Justificativa: determinismo de memória para conformidade ASIL-A (Fase futura).

### P3 — Testabilidade por camada
Cada módulo deve ser testável com mock do nível inferior.
`mcal/twai.rs` expõe trait `CanDriver` que pode ser substituído por `MockCanDriver` em testes.
Critérios de aceitação (AC-01 a AC-08) são verificados por scripts automatizados, não manualmente.

### P4 — Documentação viva
Qualquer decisão de arquitetura que afaste da spec original deve ser registrada em `DECISIONS.md`.
O campo `obd_latency_ms` no CSV deve sempre ser populado para linhas `OBD_PID`; nunca omitido.

### P5 — Segurança dos dados do SD
Flush do buffer no SD deve ocorrer a cada 10 s no máximo.
Em falha de gravação no SD, o sistema continua coletando em memória (buffer circular) e loga o erro.
Nenhum frame CAN pode ser descartado silenciosamente; toda perda deve ser contabilizada em contador persistente.
