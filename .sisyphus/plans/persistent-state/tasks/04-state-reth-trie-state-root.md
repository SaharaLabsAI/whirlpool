## `04-state-reth-trie-state-root`

> Implement trie-backed root computation with deterministic semantics over MDBX plain state.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 4 |
| **Dependencies** | `03-state-reth-core-db-tables-codec` |
| **Goal** | Deliver canonical `state_root()` behavior via `reth_trie::StateRoot::overlay_root` |
| **AC/INV** | `AC-1`, `AC-9`, `INV-4`, `INV-8` |

### Files to modify

- `crates/state-reth/src/trie.rs`
- `crates/state-reth/src/db.rs`
- `crates/state-reth/src/tables.rs`
- `crates/state-reth/src/error.rs`

### Work

1. Implement hashed-state preparation from `PlainAccountState` and `PlainStorageState`.
2. Enforce normalization rules from contract docs (exclude zero-value storage slots).
3. Implement root computation with `StateRoot::overlay_root` and typed error mapping.
4. Expose helper(s) used by `state_root()` and genesis/commit pathways.
5. Keep root semantics explicitly distinct from `state-memory` keccak state root.

### Verification command

```bash
nix develop --command cargo build -p state-reth
```
