#include <unity.h>
#include "../src/obd_responses.h"

// Definição das variáveis globais necessárias para compilação/teste
volatile float    v_speed_kmh      = 0.0;
volatile uint16_t v_rpm            = 0;
volatile uint8_t  v_throttle_pct   = 0;
volatile uint8_t  v_load_pct       = 0;
volatile float    v_maf_g_s        = 0.0;
volatile int8_t   v_coolant_temp_c = 0;

void setUp(void) {
    // Reset das variáveis globais antes de cada teste
    v_speed_kmh      = 0.0;
    v_rpm            = 0;
    v_throttle_pct   = 0;
    v_load_pct       = 0;
    v_maf_g_s        = 0.0;
    v_coolant_temp_c = 0;
}

void tearDown(void) {
}

void test_encode_engine_load(void) {
    TEST_ASSERT_EQUAL(0, encode_pid(0x04, 0));
    TEST_ASSERT_EQUAL(128, encode_pid(0x04, 50));
    TEST_ASSERT_EQUAL(255, encode_pid(0x04, 100));
}

void test_encode_coolant_temp(void) {
    TEST_ASSERT_EQUAL(0, encode_pid(0x05, -40));
    TEST_ASSERT_EQUAL(40, encode_pid(0x05, 0));
    TEST_ASSERT_EQUAL(127, encode_pid(0x05, 87));
    TEST_ASSERT_EQUAL(255, encode_pid(0x05, 215));
}

void test_encode_rpm(void) {
    TEST_ASSERT_EQUAL(0, encode_pid(0x0C, 0));
    TEST_ASSERT_EQUAL(4000, encode_pid(0x0C, 1000));
    TEST_ASSERT_EQUAL(24000, encode_pid(0x0C, 6000));
}

void test_encode_speed(void) {
    TEST_ASSERT_EQUAL(0, encode_pid(0x0D, 0));
    TEST_ASSERT_EQUAL(60, encode_pid(0x0D, 60));
    TEST_ASSERT_EQUAL(120, encode_pid(0x0D, 120));
}

void test_encode_maf(void) {
    TEST_ASSERT_EQUAL(0, encode_pid(0x10, 0.0));
    TEST_ASSERT_EQUAL(450, encode_pid(0x10, 4.5));
    TEST_ASSERT_EQUAL(65535, encode_pid(0x10, 655.35));
}

void test_encode_throttle(void) {
    TEST_ASSERT_EQUAL(0, encode_pid(0x11, 0));
    TEST_ASSERT_EQUAL(128, encode_pid(0x11, 50));
    TEST_ASSERT_EQUAL(255, encode_pid(0x11, 100));
}

void test_montar_resposta_obd(void) {
    uint8_t buf[8];

    // Testa PID 0x0C (RPM) com valor 2000 RPM (2000 * 4 = 8000 = 0x1F40)
    v_rpm = 2000;
    montar_resposta_obd(0x0C, buf);
    TEST_ASSERT_EQUAL(4, buf[0]);
    TEST_ASSERT_EQUAL(0x41, buf[1]);
    TEST_ASSERT_EQUAL(0x0C, buf[2]);
    TEST_ASSERT_EQUAL(0x1F, buf[3]);
    TEST_ASSERT_EQUAL(0x40, buf[4]);

    // Testa PID 0x0D (Velocidade) com valor 85 km/h
    v_speed_kmh = 85.0;
    montar_resposta_obd(0x0D, buf);
    TEST_ASSERT_EQUAL(3, buf[0]);
    TEST_ASSERT_EQUAL(0x41, buf[1]);
    TEST_ASSERT_EQUAL(0x0D, buf[2]);
    TEST_ASSERT_EQUAL(85, buf[3]);
}

int main(int argc, char **argv) {
    UNITY_BEGIN();
    RUN_TEST(test_encode_engine_load);
    RUN_TEST(test_encode_coolant_temp);
    RUN_TEST(test_encode_rpm);
    RUN_TEST(test_encode_speed);
    RUN_TEST(test_encode_maf);
    RUN_TEST(test_encode_throttle);
    RUN_TEST(test_montar_resposta_obd);
    return UNITY_END();
}
