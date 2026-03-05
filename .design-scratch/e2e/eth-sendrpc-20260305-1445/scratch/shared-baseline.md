# Shared Baseline

## Grounded facts
- Workspace is a Rust workspace rooted at `Cargo.toml` with members including `crates/app` and `crates/whirlpool-node` (`Cargo.toml::[workspace].members`).
- `whirlpool-node` binary bootstraps runtime, networking, consensus engine, EVM app, and then waits forever (`crates/whirlpool-node/src/main.rs::main`).
- `whirlpool-node` currently has no JSON-RPC server wiring (`crates/whirlpool-node/src/main.rs::main`).
- Node already constructs shareable state handles:
  - `state_db: Arc<RwLock<TestStateDb>>` (`crates/whirlpool-node/src/main.rs::main`)
  - `tx_pool: Arc<InMemoryTxPool>` (`crates/whirlpool-node/src/main.rs::main`)
- `InMemoryTxPool` is thread-safe and supports `push(Vec<u8>)` + drain `pending()` (`crates/app/src/tx_source.rs::InMemoryTxPool`).
- EVM execution includes only successful decoded transactions in proposed blocks (`crates/app-evm/src/executor.rs::EvmApplication::propose`).
- Chain id constant is fixed in app-evm config (`crates/app-evm/src/config.rs::SAHARA_CHAIN_ID`).
- Finalized height is externally tracked via `FinalizationSink` and `Arc<AtomicU64>` (`crates/consensus-simplex/src/sink.rs::FinalizationSink`).

## [PROPOSED] deltas
- Add an `eth` JSON-RPC module in the node binary layer (not consensus traits layer, not adapter layer).
- Use `jsonrpsee` 0.26 and macro-generated trait server pattern to align with vendor reth examples.
- Expose the minimum method set for alloy-provider transfer tests:
  - `eth_chainId`
  - `eth_getBalance`
  - `eth_getTransactionCount`
  - `eth_estimateGas`
  - `eth_gasPrice`
  - `eth_sendRawTransaction`
  - `eth_getTransactionReceipt`
- Keep design test-oriented: prioritize correctness for single-transfer integration tests first, then extension path for fuller receipt fidelity.
