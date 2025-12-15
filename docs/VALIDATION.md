# Drift Kernel Validation & Guarantees

**Version:** 1.0.0

This document describes the validation approach and guarantee boundaries for Drift Kernel.

## Scope

Drift Kernel is a **numerics primitive**: Neumaier-compensated summation with a stable C ABI.

It is **not**:
- A determinism runtime or orchestration layer
- A governance or policy engine
- A simulation framework

## Guarantee Boundaries

### We Guarantee (for Drift Kernel–scoped operations only)
- **Bounded numerical error:** O(ε) error growth vs. O(n × ε) for naive summation, under IEEE-754 binary64
- **Stable ABI:** Exported C functions behave per specification; no breaking changes without major version bump
- **Deterministic replay:** Identical results within controlled environments (same compiler, target, rounding mode, operation ordering)

### We Do NOT Guarantee
- **Platform-independent bit-for-bit determinism** across uncontrolled environments
- **Literal "zero error"** — impossible under IEEE-754
- **Global simulation determinism** — responsibility of the integrating runtime

### You Must Control
- Compiler/toolchain version and optimization flags
- Target architecture (x86-64, ARM64, etc.)
- Floating-point rounding mode (if non-default)
- Operation ordering and input sequencing
- Host runtime determinism (thread scheduling, memory allocators, etc.)

## Validation Coverage

### Unit Tests
- Catastrophic cancellation handling
- Long-horizon accumulation (100k+ ops)
- FFI null-safety
- Reset behavior
- Slice sum consistency

### Property-Based Tests (proptest)
- Random sequence stability (wide magnitude range)
- Balanced cancellation invariant
- Operation count accuracy
- Reset state invariants
- Slice/accumulator equivalence

### Stress Tests
- 1M operation long-horizon test
- Harmonic series accuracy (analytic reference)
- Adversarial alternating magnitude sequences
- Small value accumulation precision

### FFI Integration Tests
- C harness exercising full API
- Null pointer safety verification
- Cross-platform validation (Linux + Windows)

### ABI Verification
- Symbol snapshot comparison
- Breaking change detection in CI

## CI Pipeline

Every PR runs:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --all-features` (Linux, Windows, macOS)
4. `cargo audit` (security advisory check)
5. ABI symbol verification
6. FFI integration tests (C harness)
7. Coverage measurement

## NorthStar Principle

> _"Provable bounds. Visible validation. No magical thinking."_

All claims in documentation can be reconstructed from:
- Mathematical properties of Neumaier summation (1974)
- Test coverage exercising stated invariants
- CI evidence visible in public repository
