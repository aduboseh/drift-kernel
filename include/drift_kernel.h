/**
 * Drift Kernel — Bounded-Error Numerical Primitives
 *
 * Minimal C API for Neumaier-compensated summation.
 * Provides bounded-error accumulation with a stable ABI.
 *
 * EXECUTION CONTRACT:
 * Drift Kernel guarantees bounded numerical error given a deterministic
 * execution order. This library does not provide scheduling, synchronization,
 * input ordering, or replay mechanisms. Integrators are responsible for
 * enforcing a stable execution contract.
 *
 * ERROR BOUNDS (IEEE-754 binary64):
 *   - Standard accumulation: O(n × ε) error growth
 *   - Neumaier summation:    O(ε) bounded error
 *
 * THREAD SAFETY:
 * ScgAccumulator instances are not thread-safe. Use one instance per thread
 * or synchronize externally.
 *
 * License: Apache-2.0
 * Copyright (c) 2025 Drift Kernel Project
 */

#ifndef DRIFT_KERNEL_H
#define DRIFT_KERNEL_H

#include <stdint.h>
#include <stddef.h>

#ifndef DRIFT_KERNEL_ABI_VERSION
#define DRIFT_KERNEL_ABI_VERSION 1
#endif

#define DRIFT_KERNEL_ABI_VERSION_MAJOR 1
#define DRIFT_KERNEL_ABI_VERSION_MINOR 0
#define DRIFT_KERNEL_ABI_VERSION_PATCH 0

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * TYPES
 * ============================================================================ */

/**
 * Opaque handle to a Neumaier-compensated accumulator.
 * 
 * Internal structure (for reference, do not access directly):
 *   - sum: f64          Running sum
 *   - compensation: f64 Error correction buffer
 *   - initial: f64      Initial value (for drift calculation)
 *   - ops: u64          Operation count
 */
typedef struct ScgAccumulator ScgAccumulator;

/* ============================================================================
 * ACCUMULATOR API
 * ============================================================================ */

/**
 * Create a new Neumaier-compensated accumulator.
 * 
 * @param initial  Initial energy/value
 * @return         Pointer to accumulator (caller must free with scg_accumulator_free)
 */
ScgAccumulator* scg_accumulator_new(double initial);

/**
 * Free an accumulator.
 * 
 * @param acc  Accumulator to free (may be NULL)
 */
void scg_accumulator_free(ScgAccumulator* acc);

/**
 * Add a value with Neumaier compensation.
 *
 * @param acc    Accumulator (must not be NULL)
 * @param value  Value to add (positive or negative)
 */
void scg_accumulator_add(ScgAccumulator* acc, double value);

/**
 * Get the compensated total.
 *
 * @param acc  Accumulator (must not be NULL)
 * @return     Compensated total (sum + compensation)
 */
double scg_accumulator_total(const ScgAccumulator* acc);

/**
 * Get the raw sum (without compensation).
 * 
 * Useful for comparing naive vs compensated results.
 * 
 * @param acc  Accumulator (must not be NULL)
 * @return     Raw sum (may have accumulated error)
 */
double scg_accumulator_raw_sum(const ScgAccumulator* acc);

/**
 * Get the compensation buffer value.
 *
 * @param acc  Accumulator (must not be NULL)
 * @return     Compensation buffer value
 */
double scg_accumulator_compensation(const ScgAccumulator* acc);

/**
 * Get drift from initial value.
 * 
 * For balanced operations, this should be ~0.0 (within machine epsilon).
 * 
 * @param acc  Accumulator (must not be NULL)
 * @return     total() - initial
 */
double scg_accumulator_drift(const ScgAccumulator* acc);

/**
 * Get operation count.
 * 
 * @param acc  Accumulator (must not be NULL)
 * @return     Number of add() calls since creation/reset
 */
uint64_t scg_accumulator_ops(const ScgAccumulator* acc);

/**
 * Reset accumulator to initial state.
 * 
 * @param acc  Accumulator (must not be NULL)
 */
void scg_accumulator_reset(ScgAccumulator* acc);

/* ============================================================================
 * UTILITY FUNCTIONS
 * ============================================================================ */

/**
 * Compute Neumaier sum of an array (stateless, one-shot).
 * 
 * @param values  Array of doubles
 * @param len     Number of elements
 * @return        Neumaier-compensated sum
 */
double scg_neumaier_sum(const double* values, size_t len);

/**
 * Get library version string.
 * 
 * @return  Static string pointer (do NOT free)
 */
const char* scg_kernel_version(void);

/**
 * Get machine epsilon for double precision.
 * 
 * @return  ~2.22e-16 (IEEE 754)
 */
double scg_machine_epsilon(void);

#ifdef __cplusplus
}
#endif

#endif /* DRIFT_KERNEL_H */
