# INTENT — Mempool Interface/Implementation Split

## Objective

Split the `mempool` crate into two crates:
- **`mempool`** — Interface crate: exports `MempoolStore` trait (new), `MempoolError`, and re-exports `TxSource` from `app`.
- **`mempool-mdbx`** — Implementation crate: concrete MDBX-backed `MdbxMempoolStore` (renamed from current `MempoolStore`) + `PersistentTxPool` (implements `TxSource`).

## Motivation

1. **Testability** — Consumers can depend on `mempool` (interface) without pulling in `reth-libmdbx`. Tests can use mock stores.
2. **Pluggability** — Future alternative backends (RocksDB, in-memory for tests) slot in without touching the interface crate.
3. **Dependency hygiene** — `whirlpool-node` currently transitively pulls MDBX into every build. The interface crate eliminates this coupling for crates that only need the trait.
4. **Consistency** — Follows the established `state` / `state-memory` / `state-reth` pattern in this workspace.

## Scope

### Crates

| Crate | Role |
|---|---|
| `mempool` (existing, **modified**) | Becomes interface-only: trait + error |
| `mempool-mdbx` (**new**) | Concrete MDBX implementation |
| `whirlpool-node` (modified) | Updates dep from `mempool` → `mempool-mdbx` |
| `app` (read-only) | Owns `TxSource` trait — unchanged |

### Symbols

| Symbol | Current Path | Change Type | Target Path |
|---|---|---|---|
| `MempoolStore` (struct) | `mempool::store::MempoolStore` | MOVE+RENAME → trait+impl | **Trait**: `mempool::MempoolStore` / **Impl**: `mempool_mdbx::MdbxMempoolStore` |
| `PersistentTxPool` | `mempool::persistent::PersistentTxPool` | MOVE | `mempool_mdbx::PersistentTxPool` |
| `MempoolError` | `mempool::error::MempoolError` | STAYS | `mempool::MempoolError` (shared) |
| `TxSource` impl | `mempool::persistent` | MOVE | `mempool_mdbx::PersistentTxPool` |

### Depth

**Structural** — Cross-crate boundary redesign with 2 crates affected + 1 consumer. New crate created.

## Success Criteria

1. `cargo build` passes for entire workspace after split.
2. `cargo test --workspace` passes — all existing tests continue to pass.
3. `mempool` crate has zero dependency on `reth-libmdbx`.
4. `mempool-mdbx` depends on `mempool` (for trait + error) and `reth-libmdbx`.
5. `whirlpool-node` depends on `mempool-mdbx` and the node binary behavior is unchanged.
6. The `MempoolStore` trait in `mempool` exactly matches the public API surface of the old struct.
7. `mempool-mdbx::MdbxMempoolStore` implements `mempool::MempoolStore`.

## Constraints

- **No behavioral changes** — This is a pure structural refactor. All runtime behavior must be preserved.
- **No vendor modifications** — `vendor/` is read-only.
- **Follow state/state-memory pattern** — Interface crate exports trait + error. Impl crate exports concrete types.
- **Crate-level split** — NOT file-level. No `traits.rs` in mixed crate.

## Out-of-Scope

- Adding new mempool backends (in-memory, RocksDB).
- Changing the `TxSource` trait in `app`.
- Modifying error types beyond what's needed for the trait.
- Performance optimizations.
- Adding async to the trait surface.
