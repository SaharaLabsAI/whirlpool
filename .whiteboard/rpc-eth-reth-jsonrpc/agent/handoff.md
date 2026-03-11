# Design Handoff

## Summary
Replace rpc-eth's hand-rolled 10-method stub with reth's production JSON-RPC stack via 3 adapter types.

## Implementation Order

### Phase 1: Foundation (provider.rs)
1. Create `WhirlpoolProvider` struct wrapping `Arc<RethStateDb>` + `Arc<ChainSpec>`
2. Implement stub traits first (NoopProvider-style): `StageCheckpointReader`, `ChangeSetReader`, `PruneCheckpointReader`, `HashedPostStateProvider`, `StateRootProvider`, `StorageRootProvider`, `StateProofProvider`, `BlockBodyIndicesProvider`
3. Implement real traits: `BlockHashReader`, `BlockNumReader`, `HeaderProvider`, `BlockReader`, `BlockReaderIdExt`, `TransactionsProvider`, `ReceiptProvider`, `AccountReader`, `NodePrimitivesProvider`, `ChainSpecProvider`
4. Implement `StateProviderFactory` returning `RethStateDb` as `StateProvider`
5. Implement `CanonStateSubscriptions` with noop broadcast

### Phase 2: Pool + Network (pool.rs, network.rs)
6. Create `WhirlpoolTxPool` wrapping `Arc<dyn TxSource>`, impl `TransactionPool`
7. Create `WhirlpoolNetwork`, impl `NetworkInfo + Peers` with deterministic empty-peer behavior

### Phase 3: Server Wiring (server.rs)
8. Replace `start_rpc_server` to use `RpcModuleBuilder`
9. Update `lib.rs` public API

### Phase 4: Type Conversion (convert.rs)
10. `EvmBlock` ↔ reth `SealedBlock`/`SealedHeader` conversions
11. Raw tx bytes ↔ `TransactionSigned` conversions

### Phase 5: Integration (whirlpool-node)
12. Update `whirlpool-node` to call new `start_rpc_server(RpcConfig)`
13. Remove old `ReceiptStore`, `EthRpcContext` usage

### Phase 6: Tests
14. Unit tests for adapter trait impls
15. Integration tests mirroring reth's `rpc-builder/tests/it/http.rs` patterns
16. Blob rejection test

## Key Files to Create/Modify
- `crates/rpc-eth/src/provider.rs` — NEW (~500 lines, mostly trait impls)
- `crates/rpc-eth/src/pool.rs` — NEW (~150 lines)
- `crates/rpc-eth/src/network.rs` — NEW (~50 lines)
- `crates/rpc-eth/src/convert.rs` — NEW (~100 lines)
- `crates/rpc-eth/src/server.rs` — REWRITE (~100 lines)
- `crates/rpc-eth/src/lib.rs` — REWRITE (~30 lines)
- `crates/rpc-eth/Cargo.toml` — MODIFY (add reth deps)
- `crates/whirlpool-node/src/main.rs` or similar — MODIFY (update RPC wiring)
- `testing/integration-tests/` — NEW test files

## Files to Remove
- `crates/rpc-eth/src/receipt_store.rs` — receipts now from BlockStorage
- Current `EthApiServer` trait definition — replaced by reth's
- Current `EthApiHandler` — replaced by reth's `EthApi`

## Critical Constraints
- DO NOT modify anything under `vendor/`
- `state-reth::RethStateDb` is the concrete provider backend — no new storage implementations
- All trait impls must compile against reth's exact trait signatures (pin to vendor version)
