## `04-app-txsource-split`

> Move concrete tx-source types out of `app::traits` into `app::tx_source`.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `03-p2p-traits-stabilization` |
| **Wave** | 2 |
| **Complexity** | M |
| **Target Crate(s)** | `app` (both), `app-evm` (consumer) |
| **Migration Step** | #4 |
| **Change Type** | move |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

`app::traits` currently mixes interfaces and concrete tx-source implementations. This task enforces interface-only boundaries while preserving compatibility through re-exports.

### What to do

#### Phase 1 - Update/create test expectations

1. Update app tests for moved paths (`TB-006`) and dual-path compile checks (`TN-005`).

```bash
cargo test -p app
```

#### Phase 2 - Implement the change

2. Add `crates/app/src/tx_source.rs` with `NoopTxSource` and `InMemoryTxPool`.
3. Remove concrete definitions from `crates/app/src/traits.rs` (keep `Application`, `TxSource`).
4. Update `crates/app/src/lib.rs` with `pub mod tx_source;` and compatibility re-exports.

```bash
cargo check -p app && cargo check -p app-evm
```

### Rollback

```bash
git restore crates/app/src/lib.rs crates/app/src/traits.rs crates/app/src/tests.rs crates/app-evm/src/lib.rs
rm -f crates/app/src/tx_source.rs
```

**Rollback dependencies**: revert this task before reverting task `03`.

### Mock Boundary

**Allowed to mock**: tx submission producers in unit tests.
**Must NOT mock**: tx pool behavior (`pending()` drain semantics).

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 4
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-006`, `TN-005`)
- `docs/refactor/split-interface-implementation/app/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p app -p app-evm
cargo test -p app
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/04-app-txsource-split.log`
