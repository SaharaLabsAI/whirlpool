## `07-state-reth-tests`

> Add unit/integration/property tests for state-reth contracts: persistence, rollback, genesis, root determinism, concurrency, and no-op commit behavior.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 7 |
| **Dependencies** | `06-state-reth-revm-impls` |
| **Goal** | Validate state-reth behavior against AC/INV/QA contracts |
| **AC/INV/QA** | `AC-2`, `AC-8`, `AC-9`, `AC-10`, `INV-3`, `INV-4`, `INV-5`, `INV-6`, `INV-8`, `QA-1`, `QA-3` |

### Files to create/modify

- `crates/state-reth/src/db.rs` (unit tests)
- `crates/state-reth/src/codec.rs` (unit tests)
- `crates/state-reth/src/init.rs` (unit tests)
- `crates/state-reth/tests/persistence.rs`
- `crates/state-reth/tests/genesis.rs`
- `crates/state-reth/tests/concurrency.rs`
- `crates/state-reth/tests/state_root.rs`

### Work

1. Implement unit tests `TC-SR-U001`..`TC-SR-U017` as applicable to crate modules.
2. Implement integration tests `TC-SR-I001`..`TC-SR-I008` with temp DB lifecycle and reopen checks.
3. Ensure `test_commit_rollback_on_error` verifies no partial persistence.
4. Ensure concurrency test covers single writer + multiple readers (`QA-1`).
5. Add explicit empty `BundleState` no-op coverage (`QA-3`).

### Verification command

```bash
nix develop --command cargo test -p state-reth
```
