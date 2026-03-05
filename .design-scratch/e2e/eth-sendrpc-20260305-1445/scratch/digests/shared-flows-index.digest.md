## Grounded facts
- Existing runtime lifecycle: consensus engine starts then process waits indefinitely (`crates/whirlpool-node/src/main.rs`).
- Proposal flow drains tx pool and executes decoded txs (`crates/app-evm/src/executor.rs::EvmApplication::propose`).
- Verification flow replays full tx list and checks roots/gas (`crates/app-evm/src/executor.rs::EvmApplication::verify`).
- Finalized height is externally tracked by atomic sink (`crates/consensus-simplex/src/sink.rs`).

## [PROPOSED] deltas
- Add parallel RPC server lifecycle after engine startup in node main.
- Route raw tx submission through `InMemoryTxPool::push` only.
- Introduce node-local receipt tracking and poll flow for `eth_getTransactionReceipt`.
