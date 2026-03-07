# SKILL_DIGEST

## Grounded
- **Current mempool**: `InMemoryTxPool` in `crates/app/src/tx_source.rs` — `Mutex<Vec<Vec<u8>>>`, push/pending/drain semantics, raw EIP-2718 bytes. [Source: crates/app/src/tx_source.rs]
- **TxSource trait**: `crates/app/src/traits.rs:23` — `fn pending(&self) -> Vec<Vec<u8>>`. [Source: crates/app/src/traits.rs]
- **Concrete coupling**: `EthRpcContext` uses `Arc<InMemoryTxPool>` (NOT trait object). `whirlpool-node/main.rs` creates `InMemoryTxPool` directly. [Source: crates/rpc-eth/src/context.rs, crates/whirlpool-node/src/main.rs]
- **Existing persistence pattern**: state-reth uses MDBX/libmdbx via `reth-db`. `open_state_db(path)` entry point. [Source: llmdocs/crates/state-reth.md]
- **Node runtime**: Uses commonware tokio runtime with persistent storage dir. Block storage via reth_db. [Source: crates/whirlpool-node/src/main.rs]

## [PROPOSED]
- Intent: Add persistent storage to mempool so transactions survive node restarts.
- Approach TBD: Design phase will determine storage backend, API changes, and integration strategy.

## Unknowns
- Whether TxSource trait needs extending for persistence (e.g., remove, count, iterate)
- Storage backend choice (MDBX like state-reth? Separate DB? Simple file-based?)
- Whether EthRpcContext should use trait object instead of concrete InMemoryTxPool
