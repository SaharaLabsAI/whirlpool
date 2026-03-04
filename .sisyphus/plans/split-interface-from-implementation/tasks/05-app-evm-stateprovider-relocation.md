## `05-app-evm-stateprovider-relocation`

> Move `StateProvider` from executor module to `app-evm::traits` with compatibility shim.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `04-app-txsource-split` |
| **Wave** | 2 |
| **Complexity** | M |
| **Target Crate(s)** | `app-evm` (both), `whirlpool-node` (consumer) |
| **Migration Step** | #5 |
| **Change Type** | move |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

This step isolates the EVM state interface while preserving existing executor behavior and generic bounds. Compatibility re-exports keep downstream crates unbroken until later consumer migration.

### What to do

#### Phase 1 - Update/create test expectations

1. Add/adjust `TB-007` and `TN-006` dual-path compile tests.

```bash
cargo test -p app-evm
```

#### Phase 2 - Implement the change

2. Add `crates/app-evm/src/traits.rs` and move `StateProvider` there.
3. Update `crates/app-evm/src/lib.rs` to expose `traits` and re-export `StateProvider`.
4. Keep temporary compatibility export from `crates/app-evm/src/executor.rs`.

```bash
cargo check -p app-evm && cargo check -p whirlpool-node
```

### Rollback

```bash
git restore crates/app-evm/src/lib.rs crates/app-evm/src/executor.rs crates/app-evm/src/tests.rs crates/whirlpool-node/src
rm -f crates/app-evm/src/traits.rs
```

**Rollback dependencies**: revert this task before reverting task `04`.

### Mock Boundary

**Allowed to mock**: state backends in app-evm tests.
**Must NOT mock**: executor path using `StateProvider` trait bounds.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 5
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-007`, `TN-006`)
- `docs/refactor/split-interface-implementation/app-evm/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p app-evm -p whirlpool-node
cargo test -p app-evm
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/05-app-evm-stateprovider-relocation.log`
