/**
 * Drift Kernel FFI Integration Test
 * 
 * This verifies the C ABI works correctly from a pure C consumer.
 * 
 * Build and run (Linux):
 *   gcc -o test_abi tests/ffi/test_abi.c -L target/release -ldrift_kernel -Iinclude
 *   LD_LIBRARY_PATH=target/release ./test_abi
 * 
 * Build and run (Windows):
 *   cl /I include tests\ffi\test_abi.c target\release\drift_kernel.dll.lib
 *   set PATH=%PATH%;target\release
 *   test_abi.exe
 */

#include "drift_kernel.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

#define TEST(name) static int test_##name(void)
#define RUN_TEST(name) do { \
    printf("  %-40s ", #name); \
    if (test_##name()) { \
        printf("[PASS]\n"); \
        passed++; \
    } else { \
        printf("[FAIL]\n"); \
        failed++; \
    } \
} while(0)

static int passed = 0;
static int failed = 0;

/* Helper: check if two doubles are approximately equal */
static int approx_eq(double a, double b, double epsilon) {
    return fabs(a - b) < epsilon;
}

/* ========================================================================== */
/* Tests */
/* ========================================================================== */

TEST(accumulator_create_free) {
    ScgAccumulator* acc = scg_accumulator_new(100.0);
    if (acc == NULL) return 0;
    
    double total = scg_accumulator_total(acc);
    scg_accumulator_free(acc);
    
    return approx_eq(total, 100.0, 1e-15);
}

TEST(accumulator_add_basic) {
    ScgAccumulator* acc = scg_accumulator_new(0.0);
    if (acc == NULL) return 0;
    
    scg_accumulator_add(acc, 1.0);
    scg_accumulator_add(acc, 2.0);
    scg_accumulator_add(acc, 3.0);
    
    double total = scg_accumulator_total(acc);
    uint64_t ops = scg_accumulator_ops(acc);
    scg_accumulator_free(acc);
    
    return approx_eq(total, 6.0, 1e-15) && (ops == 3);
}

TEST(accumulator_catastrophic_cancellation) {
    /* This is the key test: naive summation would fail here */
    ScgAccumulator* acc = scg_accumulator_new(0.0);
    if (acc == NULL) return 0;
    
    scg_accumulator_add(acc, 1e16);
    scg_accumulator_add(acc, 1.0);
    scg_accumulator_add(acc, -1e16);
    
    double total = scg_accumulator_total(acc);
    scg_accumulator_free(acc);
    
    /* Should be exactly 1.0, not 0.0 */
    return approx_eq(total, 1.0, 1e-10);
}

TEST(accumulator_compensation_nonzero) {
    ScgAccumulator* acc = scg_accumulator_new(0.0);
    if (acc == NULL) return 0;
    
    scg_accumulator_add(acc, 1e16);
    scg_accumulator_add(acc, 1.0);
    scg_accumulator_add(acc, -1e16);
    
    double comp = scg_accumulator_compensation(acc);
    scg_accumulator_free(acc);
    
    /* Compensation should have captured the lost 1.0 */
    return fabs(comp) > 0.0;
}

TEST(accumulator_drift) {
    ScgAccumulator* acc = scg_accumulator_new(100.0);
    if (acc == NULL) return 0;
    
    scg_accumulator_add(acc, 50.0);
    
    double drift = scg_accumulator_drift(acc);
    scg_accumulator_free(acc);
    
    return approx_eq(drift, 50.0, 1e-15);
}

TEST(accumulator_reset) {
    ScgAccumulator* acc = scg_accumulator_new(42.0);
    if (acc == NULL) return 0;
    
    scg_accumulator_add(acc, 100.0);
    scg_accumulator_add(acc, -50.0);
    scg_accumulator_reset(acc);
    
    double total = scg_accumulator_total(acc);
    uint64_t ops = scg_accumulator_ops(acc);
    scg_accumulator_free(acc);
    
    return approx_eq(total, 42.0, 1e-15) && (ops == 0);
}

TEST(accumulator_raw_sum) {
    ScgAccumulator* acc = scg_accumulator_new(0.0);
    if (acc == NULL) return 0;
    
    scg_accumulator_add(acc, 1.0);
    scg_accumulator_add(acc, 2.0);
    
    double raw = scg_accumulator_raw_sum(acc);
    double total = scg_accumulator_total(acc);
    scg_accumulator_free(acc);
    
    /* For simple sums, raw and total should be very close */
    return approx_eq(raw, 3.0, 1e-10) && approx_eq(total, 3.0, 1e-15);
}

TEST(neumaier_sum_array) {
    double values[] = {1.0, 2.0, 3.0, 4.0, 5.0};
    double result = scg_neumaier_sum(values, 5);
    return approx_eq(result, 15.0, 1e-15);
}

TEST(neumaier_sum_empty) {
    double result = scg_neumaier_sum(NULL, 0);
    return approx_eq(result, 0.0, 1e-15);
}

TEST(null_safety_free) {
    /* Should not crash */
    scg_accumulator_free(NULL);
    return 1;
}

TEST(null_safety_add) {
    /* Should not crash */
    scg_accumulator_add(NULL, 1.0);
    return 1;
}

TEST(null_safety_total) {
    double result = scg_accumulator_total(NULL);
    return approx_eq(result, 0.0, 1e-15);
}

TEST(null_safety_ops) {
    uint64_t result = scg_accumulator_ops(NULL);
    return result == 0;
}

TEST(version_string) {
    const char* version = scg_kernel_version();
    if (version == NULL) return 0;
    /* Version should be non-empty and start with a digit */
    return strlen(version) > 0 && version[0] >= '0' && version[0] <= '9';
}

TEST(machine_epsilon) {
    double epsilon = scg_machine_epsilon();
    /* Should be approximately 2.22e-16 for IEEE-754 double */
    return epsilon > 2e-16 && epsilon < 3e-16;
}

TEST(long_accumulation_stability) {
    /* 100k balanced operations should not accumulate drift */
    ScgAccumulator* acc = scg_accumulator_new(1000000.0);
    if (acc == NULL) return 0;
    
    for (int i = 0; i < 100000; i++) {
        double large = 1e15 + (double)i * 1e-5;
        scg_accumulator_add(acc, large);
        scg_accumulator_add(acc, -large);
    }
    
    double drift = scg_accumulator_drift(acc);
    scg_accumulator_free(acc);
    
    return fabs(drift) < 1e-10;
}

/* ========================================================================== */
/* Main */
/* ========================================================================== */

int main(void) {
    printf("Drift Kernel FFI Tests\n");
    printf("======================\n\n");
    
    printf("Version: %s\n", scg_kernel_version());
    printf("Machine epsilon: %.16e\n\n", scg_machine_epsilon());
    
    printf("Running tests:\n");
    
    RUN_TEST(accumulator_create_free);
    RUN_TEST(accumulator_add_basic);
    RUN_TEST(accumulator_catastrophic_cancellation);
    RUN_TEST(accumulator_compensation_nonzero);
    RUN_TEST(accumulator_drift);
    RUN_TEST(accumulator_reset);
    RUN_TEST(accumulator_raw_sum);
    RUN_TEST(neumaier_sum_array);
    RUN_TEST(neumaier_sum_empty);
    RUN_TEST(null_safety_free);
    RUN_TEST(null_safety_add);
    RUN_TEST(null_safety_total);
    RUN_TEST(null_safety_ops);
    RUN_TEST(version_string);
    RUN_TEST(machine_epsilon);
    RUN_TEST(long_accumulation_stability);
    
    printf("\n");
    printf("Results: %d passed, %d failed\n", passed, failed);
    
    return failed > 0 ? 1 : 0;
}
