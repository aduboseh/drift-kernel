# Contributing to Drift Kernel

## Scope Discipline

**Drift Kernel is a numerics primitive.** It provides Neumaier-compensated summation with a stable C ABI—nothing more.

PRs that propose:
- Determinism runtimes or orchestration layers
- Governance, policy, or execution contract systems
- Integration with proprietary substrates
- Thread scheduling, synchronization, or replay mechanisms

...will be closed without merge. These are **runtime concerns**, not primitive concerns.

## Development Setup

```bash
# Clone
git clone https://github.com/aduboseh/drift-kernel.git
cd drift-kernel

# Build
cargo build --release

# Test
cargo test --all-features

# Lint
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# Benchmark
cargo bench
```

## Code Style

- Run `cargo fmt` before committing
- Zero clippy warnings (`-D warnings`)
- Document all public APIs
- Add tests for new functionality

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Ensure all tests pass and lints are clean
4. Update `CHANGELOG.md` under `[Unreleased]`
5. Submit PR against `main`

## Release Checklist

When preparing a release:

1. **Update CHANGELOG.md**
   - Move `[Unreleased]` items to new version section
   - Add release date

2. **Bump version** (must be synchronized across all sources):
   - `Cargo.toml` → `version = "X.Y.Z"`
   - `src/lib.rs` → `scg_kernel_version()` returns `"X.Y.Z"`
   - `include/drift_kernel.h` → `DRIFT_KERNEL_ABI_VERSION_*` macros
   - `ABI.md` → Version header

3. **Run full CI locally**
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo bench
   ```

4. **Tag release**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

5. **Publish crate** (optional, if publishing to crates.io)
   ```bash
   cargo publish --dry-run
   cargo publish
   ```

## ABI Compatibility

- **PATCH:** Documentation, tests, build metadata only. No ABI changes.
- **MINOR:** Additive ABI only. New symbols allowed; existing symbols unchanged.
- **MAJOR:** Required for any breaking change (symbol rename/removal, signature change, layout change).

**Intentional ABI changes require:**
- Version bump per the rules above
- CHANGELOG entry describing the change
- Update to `abi/` symbol snapshots

See `ABI.md` for full policy.

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 license.
