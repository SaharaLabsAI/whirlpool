## `07-p2p-commonware-transport-introduction`

> Introduce additive `CommonwareTransport` interface and wire provider implementations.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `06-consensus-simplex-commonwareblock-relocation` |
| **Wave** | 3 |
| **Complexity** | M |
| **Target Crate(s)** | `p2p-commonware` (target), `consensus-simplex` (consumer) |
| **Migration Step** | #7 |
| **Change Type** | introduce |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

This high-risk adapter step introduces a new local transport contract without changing current provider semantics or associated type signatures.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust `TB-009` parity tests and `TN-008` contract coverage.

```bash
cargo test -p p2p-commonware
```

#### Phase 2 - Implement the change

2. Add `crates/p2p-commonware/src/traits.rs` with `CommonwareTransport`.
3. Implement trait for transport/provider types in `provider.rs` (or focused impl module).
4. Expose trait in `crates/p2p-commonware/src/lib.rs` while preserving current API.

```bash
cargo check -p p2p-commonware && cargo check -p consensus-simplex
```

### Rollback

```bash
git restore crates/p2p-commonware/src/lib.rs crates/p2p-commonware/src/provider.rs crates/p2p-commonware/src/tests.rs crates/consensus-simplex/src
rm -f crates/p2p-commonware/src/traits.rs
```

**Rollback dependencies**: revert this task before reverting task `06`.

### Mock Boundary

**Allowed to mock**: network IO fakes for transport tests.
**Must NOT mock**: provider-to-transport send/recv contract parity.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 7
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-009`, `TN-008`)
- `docs/refactor/split-interface-implementation/p2p-commonware/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p p2p-commonware -p consensus-simplex
cargo test -p p2p-commonware
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/07-p2p-commonware-transport-introduction.log`
