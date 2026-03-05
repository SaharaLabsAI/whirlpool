# Shared Flows Index

## Grounded facts

### Flow A: Node boot and steady state
1. Initialize tracing and finalization height tracker.
2. Start tokio runner.
3. Build network provider and consensus config.
4. Build state DB, chain spec, EVM config, tx pool, EVM app, application adapter.
5. Start consensus engine.
6. Keep process alive with `pending::<()>().await`.

Evidence: `crates/whirlpool-node/src/main.rs::main`.

### Flow B: Transaction consumption and block proposal
1. `EvmApplication::propose` drains tx source via `pending()`.
2. Decode raw tx bytes to signed recovered txs.
3. Execute transactions with reth EVM builder; skip invalid tx validation errors.
4. Commit resulting bundle to canonical DB.
5. Build block roots and `ExecutionResult`.

Evidence: `crates/app-evm/src/executor.rs::EvmApplication::propose`.

### Flow C: Verification of proposed block
1. Decode all transactions in block; fail verification if decode fails.
2. Clone state and execute all txs against cloned state.
3. Compute and compare roots + gas used.
4. Return `ExecutionResult` on success.

Evidence: `crates/app-evm/src/executor.rs::EvmApplication::verify`.

### Flow D: Finalization event reporting
1. Consensus emits finalized event.
2. `FinalizationSink` stores latest finalized height in atomic.

Evidence: `crates/consensus-simplex/src/sink.rs::EventSink::handle`.

## [PROPOSED] deltas

### Flow E: RPC server lifecycle in node
1. After engine start, construct RPC context from cloned handles (`tx_pool`, `state_db`, `height`, chain id).
2. Start `jsonrpsee` server in a spawned task.
3. Keep server handle alive alongside consensus runtime.

### Flow F: Raw tx submission
1. `eth_sendRawTransaction(bytes)` validates/decode-lite and computes tx hash.
2. Push raw bytes into shared `InMemoryTxPool`.
3. Track pending tx metadata for receipt lookup.

### Flow G: Receipt polling for tests
1. `eth_getTransactionReceipt(hash)` checks receipt index.
2. If tx not yet executed/finalized, return `None`.
3. If executed, synthesize/return receipt with status and gas fields.

## Cross-crate seams
- Node-local RPC logic reads state from `state_db` via `RwLock` and `revm::Database` accessors.
- Node-local RPC logic writes txs only through `InMemoryTxPool::push`.
- Consensus/application trait boundaries remain unchanged.
