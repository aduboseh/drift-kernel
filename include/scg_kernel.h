/**
 * SCG Kernel - Zero-Drift Numerical Primitives
 * 
 * A minimal C API for Neumaier-compensated summation.
 * Drop-in replacement for floating-point accumulation in physics engines.
 * 
 * Machine Precision Guarantee:
 *   Standard floating-point: O(n × ε) drift (accumulates)
 *   SCG Kernel (Neumaier):   O(ε) drift (bounded)
 * 
 * Example:
 *   ScgAccumulator* acc = scg_accumulator_new(1000000.0);
 *   for (int i = 0; i < 100000; i++) {
 *       scg_accumulator_add(acc, 1e15);
 *       scg_accumulator_add(acc, -1e15);
 *   }
 *   double drift = scg_accumulator_drift(acc);
 *   // drift == 0.0 (exact, verified)
 *   scg_accumulator_free(acc);
 * 
 * License: Apache-2.0
 * Copyright (c) 2025 SCG Project
 */

#ifndef SCG_KERNEL_H
#define SCG_KERNEL_H

#include <stdint.h>
#include <stddef.h>

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
 * This is the core drift-killing operation:
 * - Tracks lost precision in hidden compensation buffer
 * - Handles catastrophic cancellation (1e15 + 1.0 - 1e15 = 1.0, not 0.0)
 * - O(ε) error bound regardless of operation count
 * 
 * @param acc    Accumulator (must not be NULL)
 * @param value  Value to add (positive or negative)
 */
void scg_accumulator_add(ScgAccumulator* acc, double value);

/**
 * Get the compensated total (high precision).
 * 
 * Returns sum + compensation, which is the physically correct value.
 * 
 * @param acc  Accumulator (must not be NULL)
 * @return     Compensated total
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
 * This is the "hidden correction term" that makes SCG physics.
 * Exposing it proves the algorithm is working, not smoothing.
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

#endif /* SCG_KERNEL_H */
