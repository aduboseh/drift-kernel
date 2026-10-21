# APEX-DRIFT-TRIO-001 — Numerical Integrity, Replay, and Release Hardening

**Directive date:** 2026-10-20
**Authority:** Repository-scoped hardening proposal; no production-release attestation implied
**Owner:** Only SG Solutions
**Change boundary:** `aduboseh/drift-kernel`, `aduboseh/drift-linalg`, `aduboseh/drift-harness`
**Status:** Implemented on review branches; CI and release gates must be satisfied before merge or publication

## 1. Mission

Make the Drift Trio mathematically honest, reproducibly buildable and demonstrably testable as independent reusable libraries. Do not conflate numerical stability with deterministic execution, global optimality, proof of correct simulation, or regulator certification. FADO may consume the trio only through explicit checked numerical and trace-verification contracts.

## 2. Architectural authority

- **Drift Kernel** owns scalar compensated accumulation and a versioned C ABI. It never owns scheduling, graph topology, governance, or replay.
- **Drift Linalg** owns spatial vector types and component-wise compensated *addition*. It does not claim compensated multiplication, compensated dot products, or drift-free general integration.
- **Drift Harness** owns execution fingerprint checks and path-sensitive trace verification. It does not own runtime scheduling and cannot certify the completeness of user-provided state snapshots.
- **FADO and SCG** remain separate consumers. Each retains authority for graph invariants, policy checks, ordering, transaction semantics, and provenance.

## 3. Numerical accuracy contract

1. Neumaier compensation is an established summation algorithm, not a new mathematical discovery.
2. No absolute `O(machine_epsilon)` error independent of number of terms, input magnitude, conditioning, overflow, and representation is claimed.
3. Any precise error theorem used in a release must include its floating-point model, input assumptions, relative or absolute measure, constants, and higher-order terms.
4. Non-finite operands, operation-counter overflow, and non-finite intermediate totals must be rejected by a **checked** API before internal state mutation.
5. Legacy unchecked calls are retained only for compatibility; they are not suitable for FADO's authoritative decision path without additional checks.
6. Negative zero and NaN bit patterns are not automatically normalized by raw IEEE-754 serialization. Consumers must define canonical semantics appropriate to their use case.

## 4. Release and dependency authority

- Existing published tags are immutable. Never retag `drift-kernel/v1.0.0` or rewrite an existing release.
- Release source identity is a Git tag or immutable commit SHA combined with a checksum and toolchain fingerprint; the `Cargo.toml` version string alone is insufficient.
- Confirm a new version and tag in each repository **after** its PR passes required checks. Record exact commit and signed digest. Versioning must respect SemVer compatibility.
- FADO consumers must use reviewed versions pinned by a committed `Cargo.lock`; development Git dependencies must use immutable revisions.
- CI builds the `ffi` feature for the C ABI and checks exported symbols against the public header. Default `cargo build --release` without that feature is not a C ABI verification.
- Security audit failures may not be silently skipped for a release candidate.

## 5. Replay and verification contract

- The legacy `compute_fingerprint` hashes a final-state snapshot only. Equal final hashes do not prove identical intermediate execution.
- The additional trace API hashes domain/version, declared tick count, initial state, ordered inputs, and each post-tick state, using length-delimited canonical framing.
- Non-vacuum trace callers must supply exactly one input frame per tick, including explicitly empty frames. Partial input arrays are rejected.
- The integrating simulation must serialize **all** future-affecting mutable state with a canonical, versioned byte schema.
- Replay claims must state operating system, architecture, Rust version, target features, optimization flags, floating-point mode and input ordering.
- A trace match is an observed equivalence of supplied bytes for one execution contract, not a proof of universal determinism.

## 6. Acceptance gates

**Kernel:** `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, checked-API rejection cases, release build `cargo build --release --features ffi`, and symbol verification on the generated C ABI artifact.

**Linalg:** `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, cross-platform serialization/cancellation regression checks, and verified immutable Kernel dependency identity.

**Harness:** `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`; trace test where two executions share a final state but differ in path; strict-input-length rejection; replay equality and inequality; and an immutable Linalg dev dependency.

**Integration:** Verify all three PRs independently before release; run a pinned consumer build and independently replay a documented workload. Publish the authoritative source hashes and test evidence. Do not mark complete because a documentation file claims CI exists.

## 7. FADO-specific entry requirements

- The FADO optimizer must not delegate graph validity, scheduling, transactional commit, or runtime policy authority to the Drift Trio.
- Accumulate financial and objective metrics using dimensionally consistent, finite checked values; use integer minor currency units where exact monetary accounting is required.
- Before an optimizer commits a floating-node transformation, validate DAG acyclicity, resource feasibility, effect semantics, policy, idempotency key and available rollback.
- Every accepted activation must contain graph hash before/after, proposal identity, exact candidate set, solver version, policy hash, numerics mode and path-sensitive replay evidence.
- Drift Linalg is an optional dependency only for genuinely spatial optimizations. Do not add it to every FADO deployment.

## 8. Release-blocking risks

1. **Overstated guarantees:** Remove or scope any surviving absolute drift-free / zero-error / universal bit-exact claims; block releases if marketing claims outrun code tests.
2. **ABI divergence:** Fail the build when C symbols are missing, signatures diverge, or FFI fails across supported targets.
3. **False replay assurance:** Reject incomplete input sequences in the strict trace interface; require snapshot completeness reviews and path-sensitive tests.
4. **Mutable dependency identity:** Lock source revisions and document release hashes. Treat an untagged library version or floating Git ref as insufficient evidence.

## 9. Completion certificate

This directive itself is not a completion certificate. Close it only when all three repository pull requests have passed their specified CI gates, all required tags and dependency revisions are verified, the cross-repository integration harness has passed, and the resulting source/runner artifacts have been attested. Until then, use **under review** as the release status.

## 10. Required evidence package

- Exact Git SHA and changed-file diff for each repository
- Link to each completed CI run, including platform matrix, clippy, unit/property tests and audit results
- C ABI symbol manifest and generated shared/static artifact checksum
- Evidence that different intermediate paths with the same terminal state have different ordered-trace fingerprints
- Version, release tag and dependency closure for any consumer integration
- Reviewer sign-off on numerics, API compatibility and release claims
