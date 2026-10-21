# Drift Kernel — Bounded-Error Numerical Primitives

**Compensated summation primitive with a versioned, feature-gated C ABI.**

[![CI](https://github.com/aduboseh/drift-kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/aduboseh/drift-kernel/actions/workflows/ci.yml)

**Validation:** The repository defines cross-platform tests, stress cases, and ABI checks. Review actual CI results before making release guarantees. See [docs/VALIDATION.md](docs/VALIDATION.md).

> **Note on Terminology:**
> Earlier drafts used imprecise language around "zero drift." This has been corrected to reflect bounded, provable error behavior under IEEE-754. See [Guarantees](#guarantees) for precise statements.

## Guarantees

### We guarantee (for Drift Kernel–scoped operations only):
- **Improved accuracy in many sums:** Neumaier compensation mitigates loss of low-order bits; no universal absolute error bound independent of input scale, conditioning, or operation count is claimed.
- **Stable ABI:** Exported C functions behave per specification; no breaking changes without major version bump.
- **Deterministic replay:** Identical results within controlled environments (same compiler, target, rounding mode, operation ordering).

### We do NOT guarantee:
- **Platform-independent bit-for-bit determinism** across uncontrolled environments (different compilers, architectures, or rounding modes).
- **Literal "zero error"**, or absolute O(ε) error independent of input size and conditioning.
- **Global simulation determinism** — that is the responsibility of the integrating runtime, not this primitive.

### You must control:
- Compiler/toolchain version and optimization flags
- Target architecture (x86-64, ARM64, etc.)
- Floating-point rounding mode (if non-default)
- Operation ordering and input sequencing
- Host runtime determinism (thread scheduling, memory allocators, etc.)

## ABI Stability

Drift Kernel exposes a stable C ABI intended for long-term integration.
Public ABI symbols are versioned and governed; they will not be renamed
or removed without a major version increment.

Rust crate names, internal modules, and implementation details may evolve
without ABI impact.

## Execution Contract

Drift Kernel guarantees numerical stability given a deterministic execution order.

The kernel does not define, enforce, or manage execution ordering, input sequencing,
frame boundaries, synchronization, or replay semantics. These concerns are the
responsibility of the integrating runtime.

Without a governed execution model, numerical stability alone is insufficient to
guarantee deterministic simulations across frames, clients, or replays.

A governed execution contract provides:
- **Deterministic tick/frame ordering** — operations execute in the same sequence every run
- **Input sequencing** — inputs are applied in a canonical order
- **Stable frame boundaries** — tick boundaries are well-defined and consistent
- **Cross-client synchronization** — all clients agree on execution order
- **Replay semantics** — identical inputs produce identical outputs

[Iter](https://github.com/aduboseh/iter) is a reference runtime that implements a governed execution contract.

## Non-Goals

Drift Kernel will **never** provide:
- Scheduling, timing, or frame management
- Synchronization or threading primitives
- Replay, rollback, or state snapshots
- Input handling or event ordering
- Network communication

These are runtime concerns. The kernel is a numerical primitive only.

## The Problem

Standard floating-point accumulation exhibits unbounded error growth:

```cpp
double energy = 1000000.0;
for (int i = 0; i < 100000; i++) {
    energy += 1e15;
    energy -= 1e15;
}
// Expected: 1000000.0
// Actual:   1000001.54... (error accumulated)
```

This causes:
- **Physics instability** in long-running simulations
- **Desync** in multiplayer deterministic lockstep
- **Energy leaks** in conservation-based systems

## The Solution

Drift Kernel uses Neumaier-compensated summation to bound error growth:

```cpp
#include "drift_kernel.h"

ScgAccumulator* acc = scg_accumulator_new(1000000.0);
for (int i = 0; i < 100000; i++) {
    scg_accumulator_add(acc, 1e15);
    scg_accumulator_add(acc, -1e15);
}
double result = scg_accumulator_total(acc);
// For this test case, the compensated sum should preserve the initial value.
scg_accumulator_free(acc);
```

## Error Bounds

- **Standard accumulation:** Roundoff can grow with length and cancellation.
- **Neumaier (this library):** Compensation often lowers error, but accuracy depends on input magnitudes, conditioning, sequence length and floating-point assumptions. See `docs/VALIDATION.md`.

This is a mathematical property of the algorithm, not a performance claim. See `cargo bench` for actual timing measurements on your hardware.

## API Reference

### Accumulator (Stateful)

```c
// Create/destroy
ScgAccumulator* scg_accumulator_new(double initial);
void scg_accumulator_free(ScgAccumulator* acc);

// Operations
void scg_accumulator_add(ScgAccumulator* acc, double value);
void scg_accumulator_reset(ScgAccumulator* acc);

// Queries
double scg_accumulator_total(const ScgAccumulator* acc);      // Compensated (rounded)
double scg_accumulator_raw_sum(const ScgAccumulator* acc);    // Uncompensated
double scg_accumulator_compensation(const ScgAccumulator* acc); // Hidden correction
double scg_accumulator_drift(const ScgAccumulator* acc);      // total - initial
uint64_t scg_accumulator_ops(const ScgAccumulator* acc);
```

### One-Shot Sum (Stateless)

```c
double scg_neumaier_sum(const double* values, size_t len);
```

### Metadata

```c
const char* scg_kernel_version(void);  // "1.0.0"
double scg_machine_epsilon(void);       // ~2.22e-16
```

## Integration

### Unreal Engine (C++)

```cpp
// In your physics component
#include "drift_kernel.h"

class UDriftFreeAccumulator : public UObject {
    ScgAccumulator* Acc;
public:
    UDriftFreeAccumulator() { Acc = scg_accumulator_new(0.0); }
    ~UDriftFreeAccumulator() { scg_accumulator_free(Acc); }
    
    void Add(double Value) { scg_accumulator_add(Acc, Value); }
    double Total() const { return scg_accumulator_total(Acc); }
};
```

### Unity (C# via P/Invoke)

```csharp
using System.Runtime.InteropServices;

public static class DriftKernel {
    [DllImport("drift_kernel")]
    public static extern IntPtr scg_accumulator_new(double initial);
    
    [DllImport("drift_kernel")]
    public static extern void scg_accumulator_free(IntPtr acc);
    
    [DllImport("drift_kernel")]
    public static extern void scg_accumulator_add(IntPtr acc, double value);
    
    [DllImport("drift_kernel")]
    public static extern double scg_accumulator_total(IntPtr acc);
}
```

## Build

```bash
# Build both Rust and C library formats (the C symbols are feature gated)
cargo build --release --features ffi

# Output:
# target/release/drift_kernel.dll      (Windows dynamic)
# target/release/drift_kernel.lib      (Windows static)
# target/release/libdrift_kernel.so    (Linux dynamic)
# target/release/libdrift_kernel.a     (Linux static)
```

Downstream applications must pin a verified source revision and commit their `Cargo.lock`; a crate version alone does not establish dependency identity.

## Why This Works

The Neumaier algorithm tracks floating-point error in a compensation buffer:

```
For each addition:
  temp = sum + value
  if |sum| >= |value|:
    compensation += (sum - temp) + value  // Lost from sum
  else:
    compensation += (value - temp) + sum  // Lost from value
  sum = temp

Result = sum + compensation  // Compensated, still subject to rounding
```

This can substantially reduce roundoff relative to naive accumulation, but does not establish a constant absolute error bound for arbitrary inputs.

## Verification

Run the test suite:

```bash
cargo test
cargo bench  # Performance measurements
```

See [docs/VALIDATION.md](docs/VALIDATION.md) for detailed coverage information.

## License

Apache-2.0

Apache-2.0 covers use, modification, and redistribution. Commercial licenses are available for organizations requiring paid support, integration assistance, or contractual assurances.

## Contact

For integration support or licensing inquiries:
- Email: armonti@onlysg.solutions
- Subject: "Drift Kernel Integration"
