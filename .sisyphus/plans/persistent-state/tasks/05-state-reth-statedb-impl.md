## `05-state-reth-statedb-impl`

> Implement all `state::StateDb` methods for `RethStateDb`, including durable write path and rollback safety.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 5 |
| **Dependencies** | `04-state-reth-trie-state-root` |
| **Goal** | Complete persistent trait implementation with atomic commit behavior |
| **AC/INV** | `AC-1`, `AC-10`, `INV-5` |

### Files to modify

- `crates/state-reth/src/db.rs`
- `crates/state-reth/src/init.rs`
- `crates/state-reth/src/tables.rs`
- `crates/state-reth/src/error.rs`

### Work

1. Implement constructors (`new`, `with_genesis`) with fallible trait signatures.
2. Implement all reads (`get_account`, `get_storage`, `get_code_by_hash`, `get_block_hash`) via short-lived read transactions.
3. Implement write methods (`insert_account`, `insert_block_hash`, `commit`) via short-lived write transactions.
4. Ensure `commit(BundleState)` applies account/storage/code changes and preserves atomic rollback on any failure.
5. Integrate `state_root()` and genesis bootstrap path to use trie helpers.

### Verification command

```bash
nix develop --command cargo build -p state-reth
```
