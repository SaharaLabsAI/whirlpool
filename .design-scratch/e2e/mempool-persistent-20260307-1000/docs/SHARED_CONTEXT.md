# SHARED_CONTEXT

## Workspace Overview
- **Root**: `/home/dev/sahara/web3/agent/playground/whirlpool`
- **Type**: Rust workspace (Cargo.toml at root)
- **Build**: `nix develop --command cargo build/test`
- **Architecture**: 3-layer — abstract traits → BFT adapter → node binary

## Relevant Crates

### app (crates/app/)
- **Role**: Application-layer traits and shared types
- **Key files**: `src/traits.rs` (TxSource trait), `src/tx_source.rs` (InMemoryTxPool, NoopTxSource), `src/types.rs` (EvmBlock, ExecutionResult)
- **TxSource trait** (traits.rs:23): `pub trait TxSource { fn pending(&self) -> Vec<Vec<u8>>; }`
- **InMemoryTxPool**: `Mutex<Vec<Vec<u8>>>`, `push()` appends, `pending()` drains FIFO, concurrent-safe
- **NoopTxSource**: Returns empty vec (used in tests)
- **EvmBlock**: Raw `Vec<Vec<u8>>` txs + header fields, implements codec Read/Write

### app-evm (crates/app-evm/)
- **Role**: EVM execution engine, block building
- **Key files**: `src/executor.rs` (EvmApplication)
- **EvmApplication**: Stores `Arc<dyn TxSource + Send + Sync>` (already trait-object abstracted)
- **propose()**: Drains `tx_source.pending()`, decodes EIP-2718, executes in REVM, captures receipts
- **verify()**: Replays deterministically, compares roots/gas
- **store_finalized_block()**: Takes pending receipts, writes block+receipts via BlockStorage

### whirlpool-node (crates/whirlpool-node/)
- **Role**: Binary entrypoint, wires all crates together
- **Wiring**: `let tx_pool = Arc::new(InMemoryTxPool::new())` → passed to EvmApplication + EthRpcContext
- **Storage dir**: Uses `persistent_storage_dir` from commonware runtime for data persistence
- **Existing persistence**: state-reth DB, block storage, receipt store all use persistent paths
- **PersistingFinalizationSink**: Wraps consensus finalization with BlockStorage writes

### rpc-eth (crates/rpc-eth/)
- **Role**: Ethereum JSON-RPC handler
- **EthRpcContext**: Holds `tx_pool: Arc<InMemoryTxPool>` (concrete type, NOT trait object)
- **Usage**: `ctx.tx_pool.push(bytes.to_vec())` in `send_raw_transaction`
- **Issue**: Concrete type coupling — needs generification or trait object for swappable impls

### state-reth (crates/state-reth/)
- **Role**: Persistent state storage via reth-db (MDBX/libmdbx)
- **Pattern**: `reth_db::init_db(path)` → `DatabaseEnv`, wraps in `RethStateDb`
- **CRUD**: `tx()?.get::<Table>(key)`, `tx_mut()?.put::<Table>(key, val)?.commit()`
- **Tables**: Uses reth's built-in `Tables` enum — no custom tables declared
- **Custom table challenge**: `Table` trait tightly coupled to reth's `Tables` enum

### consensus-simplex (crates/consensus-simplex/)
- **Role**: BFT consensus adapter (wraps commonware Simplex)
- **Flow**: `ConsensusApp::propose/verify` callbacks, `Reporter::report` on finalization events

## Full Transaction Lifecycle

```
eth_sendRawTransaction (rpc-eth)
  → InMemoryTxPool.push(raw_bytes)
  → EvmApplication.propose() drains tx_source.pending()
  → decode EIP-2718, execute in REVM, compute state/receipt/tx roots
  → ApplicationAdapter wraps for ConsensusApp interface
  → Simplex BFT consensus
  → AppAdapter::Reporter::report on ConsensusEvent::Finalized
  → PersistingFinalizationSink persists block
  → EvmApplication.store_finalized_block writes to BlockStorage
```

## Existing Persistence Patterns
1. **MDBX via reth-db**: state-reth uses `reth_db::open_db(path, args)` for MDBX databases
2. **Path management**: `whirlpool-node` manages storage paths, passes to constructors
3. **Arc wrapping**: All shared resources wrapped in `Arc<>` for concurrent access
4. **Trait-based abstraction**: `StateDb`, `BlockStorage` traits with concrete impls
5. **Multi-DB coexistence**: Each `init_db` with distinct path yields separate `DatabaseEnv`

## Design Constraints (from exploration)
1. **TxSource trait needs `push` method** — currently only has `pending()`. Persistent pool needs push on trait.
2. **Drain semantics must be preserved** — consensus expects `pending()` to drain.
3. **No vendor modification** — cannot extend reth-db `Tables` enum for custom mempool tables.
4. **Custom table options**: raw MDBX, redb, sled, or file-based — avoid reth-db table coupling.
5. **Dedup absent** — InMemoryTxPool stores duplicates. Persistence could add tx-hash keying for natural dedup.

## Technology Stack
- Rust edition 2021
- Tokio async runtime (via commonware)
- reth-db for MDBX databases (state-reth only)
- serde for serialization where needed
- EIP-2718 encoding for all transaction data
