## `06-state-reth-revm-impls`

> Add `revm::Database` and `revm::DatabaseRef` integrations on top of the completed `StateDb` implementation.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 6 |
| **Dependencies** | `05-state-reth-statedb-impl` |
| **Goal** | Enable EVM execution engine interoperability with persistent backend |
| **AC/INV** | `AC-1`, `INV-6` |

### Files to modify

- `crates/state-reth/src/db.rs`
- `crates/state-reth/src/error.rs`
- `crates/state-reth/src/lib.rs`

### Work

1. Implement `revm::DatabaseRef` (`basic_ref`, `storage_ref`, `block_hash_ref`, `code_by_hash_ref`) delegating to fallible `StateDb` paths.
2. Implement `revm::Database` mutable methods (`basic`, `storage`, `block_hash`, `code_by_hash`) with matching semantics.
3. Ensure error type compatibility (`type Error = RethStateError`) and trait bounds remain `Send + Sync + 'static` compatible.
4. Confirm `RethStateDb` trait/object bounds required by node wiring (`Clone + Send + Sync + Debug`) are satisfied.

### Verification command

```bash
nix develop --command cargo build -p state-reth
```
