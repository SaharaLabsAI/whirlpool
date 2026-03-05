## `02-state-reth-scaffold`

> Create the new `state-reth` crate skeleton with public module surface, dependencies, and error type foundation.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 2 |
| **Dependencies** | `01-state-trait-fallible-and-state-memory` |
| **Goal** | Introduce compilable crate scaffold for MDBX-backed backend |
| **AC/INV** | `AC-1`, `INV-6` |

### Files to create/modify

- `Cargo.toml` (workspace members)
- `crates/state-reth/Cargo.toml`
- `crates/state-reth/src/lib.rs`
- `crates/state-reth/src/error.rs`
- `crates/state-reth/src/db.rs`
- `crates/state-reth/src/tables.rs`
- `crates/state-reth/src/codec.rs`
- `crates/state-reth/src/trie.rs`
- `crates/state-reth/src/init.rs`

### Work

1. Add `crates/state-reth` to workspace membership.
2. Add `state-reth` manifest with vendored reth dependencies and `mdbx` feature posture from design docs.
3. Scaffold module files and public exports (`RethStateDb`, `RethStateError`, `create_db`, `init_db`).
4. Define `RethStateError` taxonomy (`Database`, `Init`, `Codec`, `StateRoot`) and revm DB error marker implementation.
5. Add minimal struct skeleton for `RethStateDb` (`Arc<DatabaseEnv>`, path/config fields) with constructor placeholders.

### Verification command

```bash
nix develop --command cargo build -p state-reth
```
