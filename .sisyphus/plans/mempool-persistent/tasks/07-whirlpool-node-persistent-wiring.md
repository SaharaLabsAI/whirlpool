# Task 07: whirlpool-node-persistent-wiring

**Status**: pending
**Dependencies**: 03, 06
**Wave / Phase**: Wave 5 / Phase 5 (node wiring)
**Complexity**: M
**Target Crate(s)**: `whirlpool-node`
**AC IDs**: AC-1, AC-4

## Objective
Wire `PersistentTxPool::open()` into node startup and inject `Arc<dyn TxSource + Send + Sync>` into EVM and RPC context.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/whirlpool-node.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/STRATEGY.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/FLOWS.md`

## Steps
1. Add `mempool` dependency to `whirlpool-node/Cargo.toml`.
2. Compute mempool path as `{persistent_storage_dir}/mempool` in startup wiring.
3. Replace `InMemoryTxPool::new()` wiring with `PersistentTxPool::open(path)`.
4. Pass same trait object to `EvmApplication` and `EthRpcContext`.
5. Validate node crate build.

## Atomic Verification
- `nix develop --command cargo build -p whirlpool-node`

## Done When
- Node compiles and uses persistent tx source wiring only.
- Startup path logic keeps mempool storage isolated from other DB directories.
