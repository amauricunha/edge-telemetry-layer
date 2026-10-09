#ifndef CAN_PROFILES_H
#define CAN_PROFILES_H

#ifdef NATIVE_TEST
#include <stdint.h>
#define PROGMEM
#define pgm_read_byte(addr) (*(addr))
#else
#include <Arduino.h>
#include <avr/pgmspace.h>
#endif

struct PerfilConfig {
    uint8_t speed_min, speed_max;       // km/h
    uint16_t rpm_min, rpm_max;          // RPM
    uint8_t throttle_min, throttle_max; // %
};

static const PerfilConfig PERFIS[4] = {
    { 0,   0,    0,    0,  0,   0 }, // 0 não usado
    { 40,  80, 1200, 2500, 10,  30 }, // ECONOMICO (0x01)
    { 60, 120, 2000, 4000, 20,  60 }, // NORMAL (0x02)
    { 80, 160, 3500, 6500, 50, 100 }  // ESPORTIVO (0x03)
};

// Tabela senoidal (64 pontos, 0–255 mapeados para 0.0–1.0)
const uint8_t SINE_TABLE[64] PROGMEM = {
    128, 140, 152, 165, 176, 188, 198, 208, 218, 226, 234, 240, 245, 250, 253, 254,
    255, 254, 253, 250, 245, 240, 234, 226, 218, 208, 198, 188, 176, 165, 152, 140,
    128, 115, 103, 90, 79, 67, 57, 47, 37, 29, 21, 15, 10, 5, 2, 1,
    0, 1, 2, 5, 10, 15, 21, 29, 37, 47, 57, 67, 79, 90, 103, 115
};

inline uint8_t ler_sine_table(uint16_t index) {
    return pgm_read_byte(&SINE_TABLE[index % 64]);
}

inline float interpolar_senoide(float val_min, float val_max, uint16_t index) {
    uint8_t sine_val = ler_sine_table(index);
    return val_min + (val_max - val_min) * ((float)sine_val / 255.0f);
}

#endif // CAN_PROFILES_H
