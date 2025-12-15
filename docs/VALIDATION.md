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

### Thread Safety

`ScgAccumulator` instances are **not thread-safe**. Each instance must be confined to a single thread unless externally synchronized. This is a deliberate design choice—the struct contains no synchronization primitives and assumes single-threaded access.

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

### Fuzz Testing
Fuzz harnesses are provided in `fuzz/` for:
- `scg_neumaier_sum` — arbitrary byte sequences interpreted as f64 arrays
- `ScgAccumulator` lifecycle — create, add, query, reset sequences

Run locally with:
```bash
cargo +nightly fuzz run fuzz_neumaier_sum
cargo +nightly fuzz run fuzz_accumulator
```

Fuzzing is not part of CI (requires nightly + extended runtime), but harnesses are maintained and runnable.

## CI Pipeline

Every PR runs:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --all-features` (Linux, Windows, macOS)
4. `cargo audit` (security advisory check)
5. ABI symbol verification
6. FFI integration tests (C harness)
7. Coverage measurement

## Error Bound Basis

The O(ε) error bound claimed by Drift Kernel is not novel. It implements standard Neumaier summation as published in:

> Neumaier, A. (1974). "Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen."
> *Zeitschrift für Angewandte Mathematik und Mechanik*, 54(1), 39–51.

**Why this works:** Neumaier summation improves on Kahan summation by checking which operand has larger magnitude before computing the compensation term. This handles "catastrophic cancellation" cases where the value being added is larger than the running sum.

The algorithm maintains a compensation buffer that captures precision lost during each addition. The final result (sum + compensation) recovers the lost bits, bounding total error to O(ε) regardless of operation count—compared to O(n × ε) for naive summation.

Drift Kernel implements this algorithm exactly as described in the literature. No novel variants or modifications are claimed.

## Cross-Platform Determinism

**Observed behavior:** Identical results across Linux, Windows, and macOS for IEEE-754 binary64 operations under default rounding mode (round-to-nearest-even).

This is expected for pure floating-point arithmetic without platform-specific intrinsics. However, we explicitly disclaim bit-for-bit determinism guarantees across:
- Different compiler versions or optimization levels
- Non-default rounding modes
- Extended precision intermediates (x87 FPU)

CI validates cross-platform consistency for the test suite on every commit.

## NorthStar Principle

> _"Provable bounds. Visible validation. No magical thinking."_

All claims in documentation can be reconstructed from:
- Neumaier (1974) — canonical algorithm source
- Test coverage exercising stated invariants
- CI evidence visible in public repository
