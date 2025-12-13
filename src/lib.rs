//! Drift Kernel - Zero-Drift Numerical Primitives
//!
//! A minimal, dependency-free C-FFI library exposing Neumaier-compensated
//! summation for game engines and physics simulations.
//!
//! # Machine Precision Guarantee
//!
//! Standard floating-point accumulation: O(n × ε_machine) drift
//! Drift Kernel (Neumaier): O(ε_machine) bounded, non-accumulating
//!
//! # C Integration
//!
//! ```c
//! #include "drift_kernel.h"
//!
//! ScgAccumulator* acc = scg_accumulator_new(1000000.0);
//! for (int i = 0; i < 100000; i++) {
//!     scg_accumulator_add(acc, 1e15);
//!     scg_accumulator_add(acc, -1e15);
//! }
//! double drift = scg_accumulator_total(acc) - 1000000.0;
//! // drift == 0.0 (exact)
//! scg_accumulator_free(acc);
//! ```

// NOTE: We use std for heap allocation (Box) required by C FFI.
// The core algorithm has no dependencies and is pure Rust.

// ============================================================================
// CORE: Neumaier Compensated Accumulator
// ============================================================================

/// Neumaier-compensated accumulator for drift-free summation.
///
/// Unlike standard floating-point addition which accumulates error at O(n × ε),
/// Neumaier summation bounds error to O(ε) regardless of operation count.
#[repr(C)]
pub struct ScgAccumulator {
    /// Running sum (may contain floating-point error)
    sum: f64,
    /// Compensation buffer - captures lost precision
    compensation: f64,
    /// Initial value (for drift calculation)
    initial: f64,
    /// Operation count (for diagnostics)
    ops: u64,
}

impl ScgAccumulator {
    /// Create a new accumulator with initial value
    #[inline]
    pub fn new(initial: f64) -> Self {
        Self {
            sum: initial,
            compensation: 0.0,
            initial,
            ops: 0,
        }
    }

    /// Add a value using Neumaier compensated summation
    ///
    /// This is the core algorithm that eliminates drift:
    /// - Tracks lost precision in compensation buffer
    /// - Handles catastrophic cancellation (large + small values)
    /// - Maintains O(ε) error bound regardless of operation count
    #[inline]
    pub fn add(&mut self, value: f64) {
        let temp = self.sum + value;

        // Neumaier's improvement over Kahan:
        // Check which operand has larger magnitude and compensate accordingly
        self.compensation += if self.sum.abs() >= value.abs() {
            // Standard case: sum is larger
            (self.sum - temp) + value
        } else {
            // Edge case: value is larger (handles catastrophic cancellation)
            (value - temp) + self.sum
        };

        self.sum = temp;
        self.ops += 1;
    }

    /// Get the compensated total (sum + compensation)
    ///
    /// This is the physically correct value with machine precision.
    #[inline]
    pub fn total(&self) -> f64 {
        self.sum + self.compensation
    }

    /// Get the raw sum (without compensation)
    ///
    /// Useful for comparing drift between naive and compensated sums.
    #[inline]
    pub fn raw_sum(&self) -> f64 {
        self.sum
    }

    /// Get the compensation buffer value
    ///
    /// This is the "hidden correction term" that makes drift-free accumulation possible.
    /// Exposing it proves the algorithm is working, not smoothing.
    #[inline]
    pub fn compensation(&self) -> f64 {
        self.compensation
    }

    /// Calculate drift from initial value
    ///
    /// For balanced operations (equal adds/subtracts), this should be ~0.0
    #[inline]
    pub fn drift(&self) -> f64 {
        self.total() - self.initial
    }

    /// Get operation count
    #[inline]
    pub fn ops(&self) -> u64 {
        self.ops
    }

    /// Reset to initial state
    #[inline]
    pub fn reset(&mut self) {
        self.sum = self.initial;
        self.compensation = 0.0;
        self.ops = 0;
    }
}

// ============================================================================
// C FFI - Extern Functions
// ============================================================================

/// Create a new Neumaier-compensated accumulator.
///
/// # Safety
/// Returns a heap-allocated pointer. Caller MUST free with `scg_accumulator_free`.
#[no_mangle]
pub extern "C" fn scg_accumulator_new(initial: f64) -> *mut ScgAccumulator {
    let acc = ScgAccumulator::new(initial);
    let boxed = Box::new(acc);
    Box::into_raw(boxed)
}

/// Free an accumulator.
///
/// # Safety
/// `acc` must be a valid pointer from `scg_accumulator_new`, or null.
/// After this call, `acc` is invalid.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_free(acc: *mut ScgAccumulator) {
    if !acc.is_null() {
        let _ = Box::from_raw(acc);
    }
}

/// Add a value with Neumaier compensation.
///
/// # Safety
/// `acc` must be a valid, non-null pointer from `scg_accumulator_new`.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_add(acc: *mut ScgAccumulator, value: f64) {
    if acc.is_null() {
        return;
    }
    (*acc).add(value);
}

/// Get the compensated total (high precision).
///
/// # Safety
/// `acc` must be a valid, non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_total(acc: *const ScgAccumulator) -> f64 {
    if acc.is_null() {
        return 0.0;
    }
    (*acc).total()
}

/// Get the raw sum (without compensation).
///
/// # Safety
/// `acc` must be a valid, non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_raw_sum(acc: *const ScgAccumulator) -> f64 {
    if acc.is_null() {
        return 0.0;
    }
    (*acc).raw_sum()
}

/// Get the compensation buffer value.
///
/// # Safety
/// `acc` must be a valid, non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_compensation(acc: *const ScgAccumulator) -> f64 {
    if acc.is_null() {
        return 0.0;
    }
    (*acc).compensation()
}

/// Get drift from initial value.
///
/// # Safety
/// `acc` must be a valid, non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_drift(acc: *const ScgAccumulator) -> f64 {
    if acc.is_null() {
        return 0.0;
    }
    (*acc).drift()
}

/// Get operation count.
///
/// # Safety
/// `acc` must be a valid, non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_ops(acc: *const ScgAccumulator) -> u64 {
    if acc.is_null() {
        return 0;
    }
    (*acc).ops()
}

/// Reset accumulator to initial state.
///
/// # Safety
/// `acc` must be a valid, non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn scg_accumulator_reset(acc: *mut ScgAccumulator) {
    if acc.is_null() {
        return;
    }
    (*acc).reset();
}

// ============================================================================
// UTILITY: Standalone Neumaier Sum
// ============================================================================

/// Compute Neumaier sum of an array (one-shot, no state).
///
/// # Safety
/// `values` must point to a valid array of at least `len` f64 elements.
#[no_mangle]
pub unsafe extern "C" fn scg_neumaier_sum(values: *const f64, len: usize) -> f64 {
    if values.is_null() || len == 0 {
        return 0.0;
    }

    let slice = core::slice::from_raw_parts(values, len);
    neumaier_sum_slice(slice)
}

/// Rust-native Neumaier sum for slices.
#[inline]
pub fn neumaier_sum_slice(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sum = values[0];
    let mut compensation = 0.0;

    for &value in &values[1..] {
        let temp = sum + value;
        compensation += if sum.abs() >= value.abs() {
            (sum - temp) + value
        } else {
            (value - temp) + sum
        };
        sum = temp;
    }

    sum + compensation
}

// ============================================================================
// VERSION & METADATA
// ============================================================================

/// Get library version string.
///
/// Returns a static string pointer. Do NOT free.
#[no_mangle]
pub extern "C" fn scg_kernel_version() -> *const core::ffi::c_char {
    c"0.1.0".as_ptr()
}

/// Get machine epsilon for f64.
#[no_mangle]
pub extern "C" fn scg_machine_epsilon() -> f64 {
    f64::EPSILON
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catastrophic_cancellation() {
        // This test demonstrates where naive summation fails
        let mut acc = ScgAccumulator::new(0.0);

        // Add 1e16, then 1.0, then -1e16
        // Mathematically: 0 + 1e16 + 1.0 - 1e16 = 1.0
        acc.add(1e16);
        acc.add(1.0);
        acc.add(-1e16);

        let result = acc.total();
        assert!((result - 1.0).abs() < 1e-10, "Expected 1.0, got {}", result);

        // Compare to naive sum (would lose the 1.0)
        let naive = 1e16_f64 + 1.0 + (-1e16_f64);
        assert!(
            (naive - 1.0).abs() > 1e-10,
            "Naive should fail, got {}",
            naive
        );
    }

    #[test]
    fn test_long_accumulation() {
        let mut acc = ScgAccumulator::new(1_000_000.0);

        // 100,000 balanced operations
        for i in 0..100_000 {
            let large = 1e15 + (i as f64) * 1e-5;
            let small = (i as f64).sin() * 1e-10;
            acc.add(large);
            acc.add(-large);
            acc.add(small);
            acc.add(-small);
        }

        let drift = acc.drift();
        assert!(drift.abs() < 1e-10, "Drift should be ~0, got {}", drift);
    }

    #[test]
    fn test_compensation_exposed() {
        let mut acc = ScgAccumulator::new(0.0);
        acc.add(1e16);
        acc.add(1.0);
        acc.add(-1e16);

        // Compensation should be non-zero (it captured the lost 1.0)
        let comp = acc.compensation();
        assert!(
            comp.abs() > 0.0,
            "Compensation should capture error, got {}",
            comp
        );
    }
}
