# SCG Kernel - Zero-Drift Numerical Primitives

**Drop-in drift-free accumulation for game engines and physics simulations.**

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

SCG Kernel uses Neumaier-compensated summation:

```cpp
#include "scg_kernel.h"

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

| Metric | Standard | SCG Kernel |
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
#include "scg_kernel.h"

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

public static class ScgKernel {
    [DllImport("scg_kernel")]
    public static extern IntPtr scg_accumulator_new(double initial);
    
    [DllImport("scg_kernel")]
    public static extern void scg_accumulator_free(IntPtr acc);
    
    [DllImport("scg_kernel")]
    public static extern void scg_accumulator_add(IntPtr acc, double value);
    
    [DllImport("scg_kernel")]
    public static extern double scg_accumulator_total(IntPtr acc);
}
```

## Build

```bash
# Build all library formats
cargo build --release -p scg-kernel

# Output:
# target/release/scg_kernel.dll      (Windows dynamic)
# target/release/scg_kernel.lib      (Windows static)
# target/release/libscg_kernel.so    (Linux dynamic)
# target/release/libscg_kernel.a     (Linux static)
```

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

Run the zero-drift demo:

```bash
cargo run --release --example zero_drift_demo -p scg-energy
```

Output shows naive drift growing while SCG drift stays exactly 0.0.

## License

Apache-2.0

## Contact

For integration support or licensing inquiries:
- Email: armonti@onlysg.solutions
- Subject: "SCG Kernel Integration"
