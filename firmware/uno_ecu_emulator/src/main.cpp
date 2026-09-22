#include <Arduino.h>
#include <SPI.h>
#include <mcp_can.h>
#include "can_profiles.h"
#include "obd_responses.h"

// ==============================================================================
// CONFIGURAÇÕES DE HARDWARE E PINOS
// ==============================================================================
const int LED_PIN = 4;        // LED de Status CAN no pino D4
const int CAN_CS_PIN = 10;    // Chip Select do MCP2515 no pino D10

MCP_CAN mcp2515(CAN_CS_PIN);

// ==============================================================================
// VARIÁVEIS VOLÁTEIS DE ESTADO (ISRs <-> loop)
// ==============================================================================
volatile uint8_t  perfil_atual       = 2;      // Padrão: 2 (NORMAL)
volatile uint16_t sine_index         = 0;      // Índice da tabela senoidal (0 a 63)
volatile float    v_speed_kmh        = 60.0f;
volatile uint16_t v_rpm              = 2000;
volatile uint8_t  v_throttle_pct     = 30;
volatile uint8_t  v_load_pct         = 25;
volatile float    v_maf_g_s          = 4.5f;
volatile int8_t   v_coolant_temp_c   = 87;

volatile bool     obd_pending        = false;
volatile uint8_t  obd_pid            = 0x00;

// ==============================================================================
// INICIALIZAÇÃO DOS TIMERS (ATmega328P)
// ==============================================================================

void initTimer1() {
    // Configura Timer1 em modo CTC (Modo 4), prescaler 64
    // Frequência de clock do timer: 16 MHz / 64 = 250 kHz
    // Período desejado: 50 ms (20 Hz)
    // Ticks para 50 ms: 250.000 Hz * 0.05 s = 12500 ticks (contando de 0 a 12499)
    // OCR1A = 12499
    noInterrupts();
    TCCR1A = 0;
    TCCR1B = 0;
    TCNT1  = 0;

    OCR1A = 12499;
    TCCR1B |= _BV(WGM12);  // Modo CTC (Clear Timer on Compare Match)
    TCCR1B |= _BV(CS11) | _BV(CS10); // Prescaler 64
    TIMSK1 |= _BV(OCIE1A); // Habilita interrupção por comparação A
    interrupts();
}

void initTimer2() {
    // Configura Timer2 em modo CTC (Modo 2), prescaler 64
    // Frequência de clock do timer: 16 MHz / 64 = 250 kHz
    // Período desejado: 1 ms (1000 Hz)
    // Ticks para 1 ms: 250.000 Hz * 0.001 s = 250 ticks (contando de 0 a 249)
    // OCR2A = 249
    noInterrupts();
    TCCR2A = 0;
    TCCR2B = 0;
    TCNT2  = 0;

    OCR2A = 249;
    TCCR2A |= _BV(WGM21); // Modo CTC
    TCCR2B |= _BV(CS22);  // Prescaler 64
    TIMSK2 |= _BV(OCIE2A); // Habilita interrupção por comparação A
    interrupts();
}

// ==============================================================================
// INTERRUPÇÕES (ISRs)
// ==============================================================================

// Timer2 ISR (1 ms): Polling CAN RX
ISR(TIMER2_COMPA_vect) {
    // O MCP2515 é acessado via SPI. Como o Timer2 ISR tem prioridade sobre o Timer1
    // e executa de forma síncrona com o AVR, não há conflito entre as ISRs.
    // O loop principal desabilita interrupções ao fazer transações SPI para evitar corrupção.
    if (mcp2515.checkReceive() == CAN_MSGAVAIL) {
        long unsigned int rxId;
        unsigned char len = 0;
        unsigned char rxBuf[8];
        mcp2515.readMsgBuf(&rxId, &len, rxBuf);

        if (rxId == 0x7DF) {
            // Requisição OBD funcional (Modo 01)
            if (len >= 3 && rxBuf[1] == 0x01) {
                obd_pid = rxBuf[2];
                obd_pending = true;
            }
        } else if (rxId == 0x010) {
            // Comando CAN de troca de perfil de condução
            if (len >= 1) {
                uint8_t novo_perfil = rxBuf[0];
                if (novo_perfil >= 0x01 && novo_perfil <= 0x03) {
                    perfil_atual = novo_perfil;
                }
            }
        }
    }
}

// Timer1 ISR (50 ms): Atualização de física e emissão periódica DBC
ISR(TIMER1_COMPA_vect) {
    // Garante que o LED de status esteja aceso (caso tenha sido apagado por atividade OBD-II)
    digitalWrite(LED_PIN, HIGH);

    // 1. Atualizar o índice senoidal da simulação
    sine_index = (sine_index + 1) % 64;

    // 2. Obter configurações do perfil ativo
    uint8_t p = perfil_atual;
    if (p < 1 || p > 3) p = 2; // Fallback para NORMAL caso inválido

    const PerfilConfig& config = PERFIS[p];

    // 3. Atualizar parâmetros físicos com base no perfil e na tabela senoidal
    v_speed_kmh = interpolar_senoide(config.speed_min, config.speed_max, sine_index);
    v_rpm = (uint16_t)interpolar_senoide(config.rpm_min, config.rpm_max, sine_index);
    v_throttle_pct = (uint8_t)interpolar_senoide(config.throttle_min, config.throttle_max, sine_index);
    v_load_pct = (uint8_t)(v_throttle_pct * 0.85f); // Carga do motor simulada proporcional ao pedal
    v_maf_g_s = 2.0f + ((float)v_throttle_pct * (float)v_load_pct) / 100.0f * 0.8f; // MAF proporcional a RPM/Throttle

    // 4. Envio de mensagens CAN DBC periódicas
    static uint8_t tick_count = 0;
    tick_count++;

    // ID 0x200 (Throttle + Load) - Transmitido a cada 50 ms (todo ciclo)
    uint8_t data_0x200[8] = {0};
    data_0x200[0] = (uint8_t)(((uint16_t)v_throttle_pct * 255) / 100);
    data_0x200[1] = (uint8_t)(((uint16_t)v_load_pct * 255) / 100);
    mcp2515.sendMsgBuf(0x200, 0, 8, data_0x200);

    // ID 0x100 (Speed + RPM) - Transmitido a cada 100 ms (a cada 2 ciclos de 50 ms)
    if (tick_count % 2 == 0) {
        uint8_t data_0x100[8] = {0};
        uint16_t speed_raw = (uint16_t)v_speed_kmh;
        uint16_t rpm_raw = v_rpm;
        data_0x100[0] = (uint8_t)(speed_raw >> 8);
        data_0x100[1] = (uint8_t)(speed_raw & 0xFF);
        data_0x100[2] = (uint8_t)(rpm_raw >> 8);
        data_0x100[3] = (uint8_t)(rpm_raw & 0xFF);
        mcp2515.sendMsgBuf(0x100, 0, 8, data_0x100);
    }

    // ID 0x300 (Coolant Temp) - Transmitido a cada 1000 ms (a cada 20 ciclos de 50 ms)
    if (tick_count % 20 == 0) {
        uint8_t data_0x300[8] = {0};
        data_0x300[0] = (uint8_t)(v_coolant_temp_c + 40);
        mcp2515.sendMsgBuf(0x300, 0, 8, data_0x300);
        tick_count = 0; // Reseta para evitar overflow acumulativo
    }
}

// ==============================================================================
// SETUP E LOOP PRINCIPAIS
// ==============================================================================

void setup() {
    Serial.begin(115200);
    pinMode(LED_PIN, OUTPUT);
    digitalWrite(LED_PIN, LOW);

    Serial.println(F("============================================="));
    Serial.println(F("     EMULADOR DE ECU CAN/OBD-II (UNO R3)"));
    Serial.println(F("============================================="));

    // Inicialização do MCP2515 a 500Kbps com cristal de 8MHz
    // Usamos MCP_ANY para desabilitar máscaras e filtros e escutar todas as mensagens no barramento.
    if (mcp2515.begin(MCP_ANY, CAN_500KBPS, MCP_8MHZ) == CAN_OK) {
        Serial.println(F("[CAN] MCP2515 Inicializado com Sucesso (500Kbps)"));
        mcp2515.setMode(MCP_NORMAL);
        digitalWrite(LED_PIN, HIGH); // LED aceso indica barramento operacional
    } else {
        Serial.println(F("[CAN] Erro ao inicializar MCP2515!"));
        // Loop de erro: pisca rápido a cada 200 ms
        while (1) {
            digitalWrite(LED_PIN, HIGH);
            delay(200);
            digitalWrite(LED_PIN, LOW);
            delay(200);
        }
    }

    // Inicialização dos temporizadores
    initTimer2(); // Timer2 para polling CAN RX (1 ms)
    initTimer1(); // Timer1 para atualização de física e transmissão DBC (50 ms)

    Serial.println(F("[BOOT] Emulador pronto e transmitindo no barramento."));
}

void loop() {
    // Processamento de requisições OBD-II pendentes sinalizadas pela ISR
    if (obd_pending) {
        // Desativa interrupções temporariamente para ler e limpar a flag atomicamente
        noInterrupts();
        uint8_t pid = obd_pid;
        obd_pending = false;
        interrupts();

        uint8_t resposta[8] = {0};
        montar_resposta_obd(pid, resposta);

        if (resposta[0] > 0) {
            // CRITICAL SECTION: O envio via SPI deve ser protegido contra interrupções de timers
            // que também utilizam o barramento SPI compartilhado.
            noInterrupts();
            mcp2515.sendMsgBuf(0x7E8, 0, 8, resposta);
            interrupts();

            // Print serial para feedback visual no monitor do Uno R3
            Serial.print(F("[OBD-II] Respondido PID: 0x"));
            Serial.println(pid, HEX);

            // Apaga o LED de status. Ele será reacendido pelo Timer1 no próximo ciclo (em até 50 ms),
            // criando um efeito visual de piscada perceptível e totalmente não-bloqueante.
            digitalWrite(LED_PIN, LOW);
        }
    }
}
