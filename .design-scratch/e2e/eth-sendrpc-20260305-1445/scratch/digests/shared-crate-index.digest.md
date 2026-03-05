## Grounded facts
- Focus crates are `app` and `whirlpool-node` from e2e state (`.design-scratch/e2e/eth-sendrpc-20260305-1445/e2e-state.md`).
- `app` exports interface traits `Application` and `TxSource` and concrete tx pool `InMemoryTxPool` (`crates/app/src/traits.rs`, `crates/app/src/tx_source.rs`, `crates/app/src/lib.rs`).
- `whirlpool-node` is the only node binary and currently owns runtime wiring (`crates/whirlpool-node/src/main.rs`).
- Node creates `tx_pool: Arc<InMemoryTxPool>` and passes clone into `EvmApplication` (`crates/whirlpool-node/src/main.rs`).
- No JSON-RPC dependency currently appears in workspace crate manifests (`crates/whirlpool-node/Cargo.toml`, `crates/app/Cargo.toml`).

## [PROPOSED] deltas
- Implement RPC as node-local modules under `whirlpool-node` in implementation phase.
- Keep `app` as interface crate with no required trait expansion for minimum RPC scope.
- Add JSON-RPC deps to node crate only (`jsonrpsee` 0.26, supporting ETH rpc types as needed).
