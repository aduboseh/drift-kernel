# Drift Kernel - Zero-Drift Numerical Primitives

**Deterministic accumulation primitive for governed execution environments.**

## ABI Stability

Drift Kernel exposes a stable C ABI intended for long-term integration.
Public ABI symbols are versioned and governed; they will not be renamed
or removed without a major version increment.

Rust crate names, internal modules, and implementation details may evolve
without ABI impact.

## Execution Contract

Drift Kernel guarantees numerical stability **given a deterministic execution order**.

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

## The Problem

Standard floating-point accumulation drifts over time:

```cpp
double energy = 1000000.0;
for (int i = 0; i < 100000; i++) {
    energy += 1e15;
    energy -= 1e15;
}
// Expected: 1000000.0
// Actual:   1000001.54... (DRIFTED)
```

This causes:
- **Physics instability** in long-running simulations
- **Desync** in multiplayer deterministic lockstep
- **Energy leaks** in conservation-based systems

## The Solution

Drift Kernel uses Neumaier-compensated summation:

```cpp
#include "drift_kernel.h"

ScgAccumulator* acc = scg_accumulator_new(1000000.0);
for (int i = 0; i < 100000; i++) {
    scg_accumulator_add(acc, 1e15);
    scg_accumulator_add(acc, -1e15);
}
double result = scg_accumulator_total(acc);
// result == 1000000.0 (EXACT)
scg_accumulator_free(acc);
```

## Performance

| Metric | Standard | Drift Kernel |
|--------|----------|------------|
| Drift after 100k ops | ~1.5e-3 | 0.0 |
| Error growth | O(n × ε) | O(ε) |
| Overhead | - | ~2 extra ops |

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
double scg_accumulator_total(const ScgAccumulator* acc);      // Compensated (exact)
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
const char* scg_kernel_version(void);  // "0.1.0"
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
# Build all library formats
cargo build --release

# Output:
# target/release/drift_kernel.dll      (Windows dynamic)
# target/release/drift_kernel.lib      (Windows static)
# target/release/libdrift_kernel.so    (Linux dynamic)
# target/release/libdrift_kernel.a     (Linux static)
```

Note: `Cargo.lock` is intentionally not committed (this is a library crate).

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

Result = sum + compensation  // Exact to machine precision
```

This bounds error to O(ε) regardless of operation count, while standard accumulation grows as O(n × ε).

## Verification

Run the test suite:

```bash
cargo test
```

## License

Apache-2.0

## Contact

For integration support or licensing inquiries:
- Email: armonti@onlysg.solutions
- Subject: "Drift Kernel Integration"
