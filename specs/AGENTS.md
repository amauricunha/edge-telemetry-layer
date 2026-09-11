# AGENTS.md — Edge Telemetry Layer (F1)
> Arquivo lido automaticamente pelo Antigravity IDE ao abrir o workspace.
> Define o comportamento do agente para este projeto.

## Identidade do projeto

Este é o repositório do projeto **Edge Telemetry Layer**: uma arquitetura embarcada de baixo custo para aquisição, estruturação e persistência resiliente de telemetria automotiva via CAN/OBD-II em bancada HIL.

## Regras de comportamento do agente

### Antes de qualquer geração de código
1. Ler `specs/CONSTITUTION.md` — os princípios P1 a P5 são não-negociáveis
2. Ler a `spec.md` da semana em andamento para entender o escopo atual
3. Confirmar qual `tasks.md` está sendo executado e qual task está sendo endereçada

### Ciclo SDD obrigatório para qualquer nova feature
```
/specify → /clarify → /plan → /tasks → /implement → /analyze
```
Nunca pular `/specify` e `/plan`. Nunca gerar código antes de `/plan` aprovado.

### Restrições de código (Rust / firmware)

- **PROIBIDO:** qualquer import de `std::alloc`, `Box<T>`, `Vec<T>` de std, `String` de std
- **OBRIGATÓRIO:** `heapless::Vec`, `heapless::String`, arrays com tamanho em const generics
- **PROIBIDO:** acesso direto a `esp_hal` em módulos `app/` ou `bsw/` — apenas via traits MCAL
- **OBRIGATÓRIO:** todo `unsafe` block tem comentário explicando o invariant de soundness
- **OBRIGATÓRIO:** `cargo clippy -- -D warnings` passa antes de qualquer commit

### Restrições de código (Arduino / C++)

- **PROIBIDO:** `delay()` em qualquer ISR
- **PROIBIDO:** `Serial.print()` em ISRs (bloqueia por tempo indeterminado)
- **PROIBIDO:** `sin()` em runtime — usar tabelas `PROGMEM`
- **OBRIGATÓRIO:** toda variável compartilhada entre ISR e `loop()` é `volatile`

### Sobre geração de testes

- Todo módulo `mcal/` tem um `MockDriver` correspondente
- Todo módulo `app/csv_writer.rs` tem testes unitários com `#[cfg(test)]`
- Critérios de aceitação (AC-*) são verificados por scripts Python em `tools/`, não manualmente

### Sobre commits e branches

- Cada semana tem sua própria branch: `feat/semana-N-<nome>`
- Commits seguem Conventional Commits: `feat:`, `fix:`, `test:`, `docs:`
- Nenhum commit direto em `main` — apenas via PR após gate aprovado
- Credenciais (SSID, senha Wi-Fi, IP do broker) nunca commitadas — usar `config_local.rs` no `.gitignore`

### Como o agente deve responder a pedidos de mudança de escopo

Se a mudança afeta critérios de aceitação ou a CONSTITUTION → atualizar a `spec.md` da semana antes de gerar código. Registrar em `docs/DECISIONS.md`.
Se a mudança é cosmética (renomear variável, ajustar formato de log) → pode implementar diretamente.

### Artefatos que o agente pode criar autonomamente

- Linhas de código dentro de módulos especificados
- Testes unitários para módulos da aplicação
- Documentação técnica e relatórios de métricas em `docs/`
- Scripts de validação e análise em `tools/`

### Artefatos que requerem aprovação humana antes de criar

- Novos arquivos não previstos no `plan.md` (especialmente em `mcal/` ou `bsw/`)
- Mudanças em `CONSTITUTION.md`
- Alterações nos critérios de aceitação (AC-*)
- Qualquer código que acesse periféricos de hardware fora da camada MCAL
