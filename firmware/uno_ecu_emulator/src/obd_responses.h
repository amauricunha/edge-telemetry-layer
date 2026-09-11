#ifndef OBD_RESPONSES_H
#define OBD_RESPONSES_H

#ifdef NATIVE_TEST
#include <stdint.h>
#else
#include <Arduino.h>
#endif

// Declaração das variáveis globais compartilhadas
extern volatile float    v_speed_kmh;
extern volatile uint16_t v_rpm;
extern volatile uint8_t  v_throttle_pct;
extern volatile uint8_t  v_load_pct;
extern volatile float    v_maf_g_s;
extern volatile int8_t   v_coolant_temp_c;

// Helper para codificar o valor do PID no formato bruto OBD-II
inline uint32_t encode_pid(uint8_t pid, float value) {
    switch (pid) {
        case 0x04: // Engine Load (%)
        case 0x11: // Throttle Position (%)
            return (uint32_t)((value * 255.0f) / 100.0f + 0.5f);
        case 0x05: // Coolant Temp (°C)
            return (uint32_t)(value + 40.0f + 0.5f);
        case 0x0C: // RPM
            return (uint32_t)(value * 4.0f + 0.5f);
        case 0x0D: // Speed (km/h)
            return (uint32_t)(value + 0.5f);
        case 0x10: // MAF (g/s)
            return (uint32_t)(value * 100.0f + 0.5f);
        default:
            return 0;
    }
}

// Função para montar a resposta OBD-II de 8 bytes no padrão ISO 15765-4 (Modo 01)
inline void montar_resposta_obd(uint8_t pid, uint8_t* buf) {
    for (uint8_t i = 0; i < 8; i++) {
        buf[i] = 0;
    }
    buf[1] = 0x41; // Modo 01 Response (0x01 + 0x40)
    buf[2] = pid;  // PID espelhado

    switch (pid) {
        case 0x04: // Engine Load
            buf[0] = 3; // comprimento (Modo + PID + 1 byte de dados)
            buf[3] = (uint8_t)encode_pid(0x04, v_load_pct);
            break;
        case 0x05: // Coolant Temp
            buf[0] = 3; // comprimento (Modo + PID + 1 byte de dados)
            buf[3] = (uint8_t)encode_pid(0x05, v_coolant_temp_c);
            break;
        case 0x0C: { // Engine RPM
            buf[0] = 4; // comprimento (Modo + PID + 2 bytes de dados)
            uint16_t rpm_raw = (uint16_t)encode_pid(0x0C, v_rpm);
            buf[3] = (uint8_t)(rpm_raw >> 8);
            buf[4] = (uint8_t)(rpm_raw & 0xFF);
            break;
        }
        case 0x0D: // Vehicle Speed
            buf[0] = 3; // comprimento (Modo + PID + 1 byte de dados)
            buf[3] = (uint8_t)encode_pid(0x0D, v_speed_kmh);
            break;
        case 0x10: { // MAF Air Flow Rate
            buf[0] = 4; // comprimento (Modo + PID + 2 bytes de dados)
            uint16_t maf_raw = (uint16_t)encode_pid(0x10, v_maf_g_s);
            buf[3] = (uint8_t)(maf_raw >> 8);
            buf[4] = (uint8_t)(maf_raw & 0xFF);
            break;
        }
        case 0x11: // Throttle Position
            buf[0] = 3; // comprimento (Modo + PID + 1 byte de dados)
            buf[3] = (uint8_t)encode_pid(0x11, v_throttle_pct);
            break;
        default:
            buf[0] = 0; // PID não suportado
            break;
    }
}

#endif // OBD_RESPONSES_H
