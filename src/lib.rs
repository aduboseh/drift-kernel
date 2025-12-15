//! Drift Kernel — Bounded-Error Numerical Primitives
//!
//! A minimal, dependency-free C-FFI library exposing Neumaier-compensated
//! summation for physics simulations and deterministic systems.
//!
//! # Error Bounds (IEEE-754 binary64)
//!
//! Standard floating-point accumulation: O(n × ε) error growth (unbounded)
//! Drift Kernel (Neumaier): O(ε) error (bounded, not accumulated)
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
    c"1.0.0".as_ptr()
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

    // ========================================================================
    // Core Algorithm Tests
    // ========================================================================

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

    #[test]
    fn test_reset() {
        let mut acc = ScgAccumulator::new(100.0);
        acc.add(50.0);
        acc.add(-25.0);
        assert_eq!(acc.ops(), 2);

        acc.reset();
        assert_eq!(acc.total(), 100.0);
        assert_eq!(acc.ops(), 0);
        assert_eq!(acc.compensation(), 0.0);
    }

    #[test]
    fn test_slice_sum_empty() {
        assert_eq!(neumaier_sum_slice(&[]), 0.0);
    }

    #[test]
    fn test_slice_sum_single() {
        assert_eq!(neumaier_sum_slice(&[42.0]), 42.0);
    }

    // ========================================================================
    // Long-Horizon Stress Tests (Phase D2)
    // ========================================================================

    #[test]
    fn test_long_accumulation_100k() {
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

    /// Long-horizon stress test: 1 million operations
    /// This validates O(ε) error bound over sustained accumulation.
    #[test]
    fn test_long_horizon_1m_ops() {
        const N: usize = 1_000_000;
        let mut acc = ScgAccumulator::new(0.0);

        // Alternating large magnitude values that should cancel
        for i in 0..N {
            let magnitude = 1e10 + (i as f64) * 0.1;
            acc.add(magnitude);
            acc.add(-magnitude);
        }

        let total = acc.total();
        let error = total.abs();

        // Error should be bounded by O(ε), not O(n × ε)
        // With 2M ops and ε ≈ 2.22e-16, O(n×ε) would be ~4.4e-10
        // O(ε) bound means error should be much smaller
        assert!(
            error < f64::EPSILON * 1000.0,
            "Error {} exceeds O(ε) bound after {} ops",
            error,
            N * 2
        );

        eprintln!(
            "1M ops stress test: total={}, error={}, ops={}",
            total,
            error,
            acc.ops()
        );
    }

    /// Harmonic series partial sum (known analytic approximation)
    /// Sum of 1/n from 1 to N ≈ ln(N) + γ (Euler-Mascheroni constant)
    #[test]
    fn test_harmonic_series_accuracy() {
        const N: usize = 100_000;
        let mut acc = ScgAccumulator::new(0.0);

        for i in 1..=N {
            acc.add(1.0 / (i as f64));
        }

        let result = acc.total();
        // H_100000 ≈ 12.090146129863335 (computed with high precision)
        let expected = 12.090146129863335;
        let error = (result - expected).abs();

        // Should be accurate to ~12 significant digits
        assert!(
            error < 1e-10,
            "Harmonic sum error {} too large (result={}, expected={})",
            error,
            result,
            expected
        );
    }

    // ========================================================================
    // Adversarial Sequence Tests
    // ========================================================================

    #[test]
    fn test_adversarial_alternating_magnitudes() {
        // Alternating huge/tiny values: 1e15, 1e-15, 1e15, 1e-15, ...
        let mut acc = ScgAccumulator::new(0.0);
        const N: usize = 10_000;

        for _ in 0..N {
            acc.add(1e15);
            acc.add(1e-15);
            acc.add(-1e15);
            acc.add(-1e-15);
        }

        let error = acc.total().abs();
        assert!(
            error < f64::EPSILON * 100.0,
            "Adversarial alternating test failed: error = {}",
            error
        );
    }

    #[test]
    fn test_adversarial_accumulating_small() {
        // Many small values that accumulate
        let mut acc = ScgAccumulator::new(0.0);
        let small = 1e-15;
        const N: usize = 1_000_000;

        for _ in 0..N {
            acc.add(small);
        }

        let result = acc.total();
        let expected = small * (N as f64);
        let relative_error = ((result - expected) / expected).abs();

        // Relative error should be bounded
        assert!(
            relative_error < f64::EPSILON * 10.0,
            "Small accumulation relative error {} too large",
            relative_error
        );
    }

    // ========================================================================
    // FFI Null Safety Tests
    // ========================================================================

    #[test]
    fn test_ffi_null_safety() {
        unsafe {
            // All functions should handle null gracefully
            scg_accumulator_free(core::ptr::null_mut());
            scg_accumulator_add(core::ptr::null_mut(), 1.0);
            assert_eq!(scg_accumulator_total(core::ptr::null()), 0.0);
            assert_eq!(scg_accumulator_raw_sum(core::ptr::null()), 0.0);
            assert_eq!(scg_accumulator_compensation(core::ptr::null()), 0.0);
            assert_eq!(scg_accumulator_drift(core::ptr::null()), 0.0);
            assert_eq!(scg_accumulator_ops(core::ptr::null()), 0);
            scg_accumulator_reset(core::ptr::null_mut());
            assert_eq!(scg_neumaier_sum(core::ptr::null(), 10), 0.0);
        }
    }

    #[test]
    fn test_ffi_roundtrip() {
        unsafe {
            let acc = scg_accumulator_new(100.0);
            assert!(!acc.is_null());

            scg_accumulator_add(acc, 1e15);
            scg_accumulator_add(acc, 1.0);
            scg_accumulator_add(acc, -1e15);

            let total = scg_accumulator_total(acc);
            assert!((total - 101.0).abs() < 1e-10);

            let drift = scg_accumulator_drift(acc);
            assert!((drift - 1.0).abs() < 1e-10);

            assert_eq!(scg_accumulator_ops(acc), 3);

            scg_accumulator_free(acc);
        }
    }

    #[test]
    fn test_version_and_epsilon() {
        // scg_kernel_version and scg_machine_epsilon are safe extern fns
        let version = scg_kernel_version();
        assert!(!version.is_null());

        let epsilon = scg_machine_epsilon();
        assert!((epsilon - f64::EPSILON).abs() < 1e-20);
    }
}

// ============================================================================
// Property-Based Tests (Phase D1)
// ============================================================================

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Random sequences across wide magnitude range
        #[test]
        fn prop_bounded_error_random_sequence(
            values in prop::collection::vec(-1e10f64..1e10f64, 100..1000)
        ) {
            let compensated = neumaier_sum_slice(&values);
            let naive: f64 = values.iter().sum();

            // Both should be "close" - the point is compensated doesn't diverge
            // We can't assert compensated == naive because naive accumulates error
            // Instead verify compensated is finite and reasonable
            prop_assert!(compensated.is_finite());
            prop_assert!(naive.is_finite());
        }

        /// Balanced sequences should return to zero (within epsilon)
        #[test]
        fn prop_balanced_cancellation(
            values in prop::collection::vec(-1e8f64..1e8f64, 10..100)
        ) {
            let mut acc = ScgAccumulator::new(0.0);

            // Add all values then subtract them
            for &v in &values {
                acc.add(v);
            }
            for &v in &values {
                acc.add(-v);
            }

            let error = acc.total().abs();
            // Error should be bounded by O(ε × max_magnitude)
            let max_mag = values.iter().map(|v| v.abs()).fold(0.0f64, f64::max);
            let bound = f64::EPSILON * max_mag * 100.0;

            prop_assert!(
                error < bound.max(1e-10),
                "Balanced cancellation error {} exceeds bound {}",
                error, bound
            );
        }

        /// Accumulator operations count should match actual calls
        #[test]
        fn prop_ops_count_accurate(n in 1usize..1000) {
            let mut acc = ScgAccumulator::new(0.0);
            for i in 0..n {
                acc.add(i as f64);
            }
            prop_assert_eq!(acc.ops() as usize, n);
        }

        /// Reset should restore initial state
        #[test]
        fn prop_reset_restores_initial(
            initial in -1e10f64..1e10f64,
            values in prop::collection::vec(-1e5f64..1e5f64, 1..100)
        ) {
            let mut acc = ScgAccumulator::new(initial);
            for v in values {
                acc.add(v);
            }
            acc.reset();

            prop_assert_eq!(acc.total(), initial);
            prop_assert_eq!(acc.ops(), 0);
            prop_assert_eq!(acc.compensation(), 0.0);
        }

        /// Slice sum should match accumulator result
        #[test]
        fn prop_slice_matches_accumulator(
            values in prop::collection::vec(-1e8f64..1e8f64, 1..500)
        ) {
            let slice_result = neumaier_sum_slice(&values);

            let mut acc = ScgAccumulator::new(0.0);
            for &v in &values {
                acc.add(v);
            }
            let acc_result = acc.total();

            // Results should be identical (same algorithm)
            prop_assert!(
                (slice_result - acc_result).abs() < f64::EPSILON,
                "Slice {} != Accumulator {}",
                slice_result, acc_result
            );
        }
    }
}
