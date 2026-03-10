# Shared Dependency Graph — mempool-split-interface

## Workspace Members
- crates/consensus
- crates/consensus-simplex
- crates/p2p
- crates/p2p-commonware
- crates/rpc-eth
- crates/whirlpool-node
- crates/state
- crates/state-memory
- crates/state-reth
- crates/app
- crates/app-evm
- crates/mempool
- testing/integration-tests

## mempool Dependencies
### Runtime
- `app` (path `crates/app`): used for `app::traits::TxSource` (trait implemented by `PersistentTxPool`).
- `reth-libmdbx` (path `vendor/reth/crates/storage/libmdbx-rs`): provides `Environment`, `Database`, and `WriteFlags` used by `MempoolStore`.

### Dev
- `tempfile` 3.x: used in `store` + `persistent` unit tests to create temporary MDBX directories.

## Reverse Dependencies (who depends on mempool)
| Crate | Dep Type | Specific Imports Used |
| --- | --- | --- |
| `crates/whirlpool-node` | path dependency (runtime) | Imports `mempool::PersistentTxPool` and creates a `dyn TxSource` pointing to it in `src/main.rs`; relies on `PersistentTxPool::open` to mount `data/mempool` and re-exports the `TxSource` trait object for RPC and application wiring.

## Feature Flags
- None defined in `crates/mempool/Cargo.toml` (no `[features]` section).

## Conditional Compilation
- `#[cfg(test)]` guards the `tests` modules inside both `src/store.rs` and `src/persistent.rs`; there are no other `cfg` or `cfg_attr` annotations in the crate.

## Dependency Graph (simplified)
```
           +--------------------+
           |  crates/whirlpool-node |
           +--------------------+
                     |
                     v
             +---------------+
             |   mempool     |
             +---------------+
              /             \
             v               v
        app (traits)   reth-libmdbx (path ../../vendor/reth/crates/storage/libmdbx-rs)
                           |
                           v
                 (native MDBX bindings + libmdbx-sys transitives)
```
- transitive note: `reth-libmdbx` is vendored under `vendor/reth/crates/storage/libmdbx-rs` and ultimately pulls in `mdbx` native bindings.

## Circular Dependency Risks
- Future `mempool` interface crate must not depend on `mempool-mdbx`. The implementation crate will depend on the interface, so any reverse reference would create a cycle.
- The current runtime dependency on `reth-libmdbx` belongs with the concrete MDBX-backed implementation. Keeping it in the interface crate would force every consumer (e.g., `whirlpool-node`) to compile MDBX, defeating the point of the split and coupling the interface to native bindings.
- `app::traits::TxSource` is used by the interface today, so the trait definitions stay in the interface while MDBX-specific APIs move to `mempool-mdbx`. That keeps the interface lightweight and prevents both crates from depending on each other through shared concrete logic.
