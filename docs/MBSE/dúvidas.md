# Dúvidas e Resoluções Formais (FRET / MBSE)

## 1. Precisamos declarar variáveis, como perfil de direção?
- **Dúvida:** `dbc_frames_emitted` teria que ter o valor de cada frame calculado pela função de cálculo da onda lá? O cálculo da onda está no sistema e precisa?
- **Resolução:**
  - **Sim, foram decompostos formalmente em dois requisitos encadeados:**
    1. [`REQ_EMU_008`](file:///c:/workspace/can-obd-telemetry/docs/mestrado/fret.md#L68-L81): Modela o **cálculo da dinâmica física na CPU** (`physics_model_updated within 1 MILLISECOND`). A computação matemática contínua (senoide em PROGMEM) é interna da CPU e não é modelada com fórmulas trigonométricas no FRET (incompatível com NuSMV/JKind), mas seu **consumo de CPU / deadline** é formalizado com precisão.
    2. [`REQ_EMU_001`](file:///c:/workspace/can-obd-telemetry/docs/mestrado/fret.md#L83-L98): Modela a **emissão física dos frames no barramento** (`dbc_frames_emitted within 2 MILLISECOND`) acionada logo após `physics_model_updated`.
  - A variável `active_profile` foi declarada e mapeada como `Output` (Integer: 1=Eco, 2=Normal, 3=Sport) em [`REQ_EMU_002`](file:///c:/workspace/can-obd-telemetry/docs/mestrado/fret.md#L100-L115) e [`REQ_EMU_007`](file:///c:/workspace/can-obd-telemetry/docs/mestrado/fret.md#L118-L131).

---

## 2. Declarar variáveis e cálculos / requisitos do perfil padrão e sintético
- **Dúvida:** O sistema tem que calcular a senoide de perfil para gerar dados sintéticos interligado com o perfil recebido, ou se não recebeu, acho que o padrão é econômico? Como declaramos? Já está declarado ou é novo requisito?
- **Resolução:**
  - **Identificado gap no SRS e adicionado no FRET como [`REQ_EMU_007`](file:///c:/workspace/can-obd-telemetry/docs/mestrado/fret.md#L118-L131):**
    - No firmware real do Arduino (`firmware/uno_ecu_emulator/src/main.cpp`), o perfil padrão é **Normal (2)** (`volatile uint8_t perfil_atual = 2`), e não econômico.
    - O requisito formal criado foi:
      `in boot_mode upon boot_complete the uno_ecu_emulator shall immediately satisfy active_profile = 2`
    - Comutação dinâmica via comando CAN 0x010 é coberta por [`REQ_EMU_002`](file:///c:/workspace/can-obd-telemetry/docs/mestrado/fret.md#L100-L115) (`active_profile = commanded_profile within 50 MILLISECOND`).


## emulador e coletor tem que estar em projetos separados? eles se integram de alguma forma ou precisam estar no mesmo projeto só componentes separados?