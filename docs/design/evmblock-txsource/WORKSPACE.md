# WORKSPACE — EvmBlock TxSource

## Crate Dependency Graph (relevant subset)

```
whirlpool-node
  └── app-evm
        ├── app          ← TxSource trait + InMemoryTxPool [PROPOSED]
        ├── state
        ├── consensus
        └── reth-*, alloy-*, revm (vendor)
```

## Key File Entrypoints

| File | Purpose |
|---|---|
| `crates/app/src/traits.rs` | `TxSource` trait, `NoopTxSource`, **[PROPOSED]** `InMemoryTxPool` |
| `crates/app/src/lib.rs` | Re-exports (add `InMemoryTxPool`) |
| `crates/whirlpool-node/src/main.rs:130` | Node wiring — swap `NoopTxSource` → `InMemoryTxPool` |
| `crates/app-evm/src/executor.rs` | Consumer — `EvmApplication.tx_source` (no changes) |
| `crates/app-evm/tests/integration.rs` | Integration tests (add `InMemoryTxPool` test) |

## Build / Test

```sh
nix develop --command cargo build
nix develop --command cargo test
```
