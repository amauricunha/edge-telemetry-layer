# Tabela: Variáveis, Origens e Taxas de Coleta — Edge Telemetry Layer

Esta tabela resume os sinais físicos monitorados no projeto, indicando suas origens de barramento, taxas reais de amostragem na bancada HIL da **Fase 1**, e o planejamento de expansão para a validação veicular.

## Tabela de Variáveis

| Categoria | Variável (Sinal) | Nome no Dataset | Origem / ID / PID | Amostragem HIL (Fase 1) | Amostragem Veicular (Fase 5) | Importância para Modelos Preditivos (ML) | Status na Fase 1 |
| :--- | :--- | :--- | :--- | :---: | :---: | :--- | :--- |
| **Intenção** | Posição do Acelerador | `throttle_pct` | OBD-II (PID 0x11) | ~600 ms (Sequencial)* | ~600 ms (Sequencial)* | Indica a intenção primária de aceleração e aceleração sob demanda. | **Ativo (Emulado)** |
| **Intenção** | Estado/Pressão do Freio | `brake_pressure` | CAN Raw (ABS/ESC) | *Não disponível* | 10 a 20 ms (Broadcast)** | Crucial para identificar frenagens bruscas e comportamento agressivo. | **Inativo (Fase 5)** |
| **Intenção** | Ângulo do Volante | `steering_angle` | CAN Raw (EPS/ESP) | *Não disponível* | 10 a 20 ms (Broadcast)** | Permite monitorar ziguezagues, curvas bruscas e estabilidade direcional. | **Inativo (Fase 5)** |
| **Dinâmica** | Velocidade do Veículo | `speed_kmh` | OBD-II (PID 0x0D) | ~600 ms (Sequencial)* | ~600 ms (Sequencial)* | Velocidade absoluta da dinâmica longitudinal, usada para calcular aceleração. | **Ativo (Emulado)** |
| **Dinâmica** | Rotações do Motor | `rpm` | OBD-II (PID 0x0C) | ~600 ms (Sequencial)* | ~600 ms (Sequencial)* | Indica a rotação do motor, fundamental para analisar trocas de marcha e ineficiência mecânica. | **Ativo (Emulado)** |
| **Dinâmica** | Aceleração Lateral | `lateral_accel` | CAN Raw (ESP) | *Não disponível* | 20 a 50 ms (Broadcast)** | Identifica curvas em alta velocidade e limites de atrito lateral. | **Inativo (Fase 5)** |
| **Consumo** | Fluxo de Massa de Ar | `maf_g_s` | OBD-II (PID 0x10) | ~600 ms (Sequencial)* | ~600 ms (Sequencial)* | Taxa de fluxo de ar de admissão. Proxy direta utilizada para estimar consumo instântaneo. | **Ativo (Emulado)** |
| **Consumo** | Carga do Motor | `engine_load_pct` | OBD-II (PID 0x04) | ~600 ms (Sequencial)* | ~600 ms (Sequencial)* | Porcentagem do binário máximo disponível sob a rotação atual. | **Ativo (Emulado)** |
| **Contexto** | Temp. do Arrefecimento | `coolant_temp_c` | OBD-II (PID 0x05) | ~600 ms (Sequencial)* | ~600 ms (Sequencial)* | Temperatura do motor. Filtra contexto térmico (motor frio vs motor em temperatura de trabalho). | **Ativo (Emulado)** |

---

## Detalhamento das Taxas de Coleta e Comportamentos

- **\* Polling Ativo OBD-II (Fase 1 HIL):** As consultas de diagnóstico funcionam por ciclo de requisição e resposta sequencial (*round-robin*). O coletor ESP32-S3 envia solicitações individuais a cada 100 ms. Como existem 6 PIDs padrão habilitados para a coleta HIL, um dado específico (ex: `rpm`) é atualizado na rede a cada $6 \times 100\text{ ms} = 600\text{ ms}$.
- **\*\* Sniffing Passivo CAN Raw (Fase 5 Veículo):** O coletor atua estritamente como ouvinte passivo. Os módulos originais do veículo (como freio ABS e direção elétrica EPS) publicam de forma contínua dados críticos em alta frequência (de 50 Hz a 100 Hz). Na Fase 5 de validação real no veículo, o coletor lerá esses frames diferenciais diretamente do barramento físico a cada 10 a 50 milissegundos sem causar overhead à rede veicular.
- **Diferenciação de Escopo:** Para garantir que a Fase 1 (bancada) seja viável com hardware e simulação de baixo custo, as variáveis dependentes de engenharia reversa do protocolo do Renault Sandero (Freio, Volante e Aceleração Lateral) foram categorizadas como inativas nesta etapa. Elas serão incorporadas após a análise e sniff de frames diferenciais reais em veículo durante os testes de campo da Fase 5.
