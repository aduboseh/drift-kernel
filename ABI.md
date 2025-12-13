# Drift Kernel ABI

The stable C ABI surface is defined by the exported `extern "C"` symbols in `src/lib.rs` and the declarations in `include/drift_kernel.h`.

## ABI version macros
The header publishes compile-time ABI version macros:
- `DRIFT_KERNEL_ABI_VERSION` (current ABI generation)
- `DRIFT_KERNEL_ABI_VERSION_MAJOR`, `DRIFT_KERNEL_ABI_VERSION_MINOR`, `DRIFT_KERNEL_ABI_VERSION_PATCH`

## Compatibility rules
- PATCH: documentation/tests/build metadata changes only (no ABI changes)
- MINOR: additive ABI only (new symbols may be added; existing symbols/signatures remain unchanged)
- MAJOR: required for any breaking change (symbol rename/removal, signature change, calling convention change, or layout/behavior changes that would break existing consumers)

## Policy
No exported symbol renames without a MAJOR bump.

Note: For v1 compatibility, exported C symbols remain `scg_*` even though the public identity is Drift Kernel. A future MAJOR may introduce a `drift_*` namespace with a deprecation window.