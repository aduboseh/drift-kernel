# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-12-15

### Added
- Neumaier-compensated accumulator with O(ε) bounded error
- Stable C ABI with null-safety checks on all exported functions
- `scg_accumulator_new`, `scg_accumulator_free`, `scg_accumulator_add`
- `scg_accumulator_total`, `scg_accumulator_raw_sum`, `scg_accumulator_compensation`
- `scg_accumulator_drift`, `scg_accumulator_ops`, `scg_accumulator_reset`
- `scg_neumaier_sum` for stateless array summation
- `scg_kernel_version`, `scg_machine_epsilon` metadata functions
- C header `drift_kernel.h` with ABI version macros
- Property-based tests (proptest) for numerical invariants
- Long-horizon stress tests (10^6+ operations)
- FFI integration tests (C harness)
- CI matrix: fmt, clippy, test (Linux/Windows/macOS), audit
- Criterion benchmarks for performance validation

### Changed
- Documentation: "Zero Drift" language replaced with "Bounded Error" to reflect mathematical reality
- README: Added explicit Guarantees section with IEEE-754 compliance statements

### Notes
- C symbols retain `scg_*` prefix for v1 ABI compatibility
- A future major version may introduce `drift_*` namespace with deprecation window

## [Unreleased]

### Added
- (future changes go here)
