# Task 03: rpc-eth-context-generification

**Status**: pending
**Dependencies**: 02
**Wave / Phase**: Wave 3 / Phase 3 (EthRpcContext generification)
**Complexity**: S
**Target Crate(s)**: `rpc-eth`
**AC IDs**: AC-2, AC-4

## Objective
Switch RPC context transaction pool dependency from concrete `Arc<InMemoryTxPool>` to `Arc<dyn TxSource + Send + Sync>`.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/STRATEGY.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/rpc-eth.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/proven-ac.md`

## Steps
1. Update `EthRpcContext` field type and constructor signature to trait object type.
2. Update imports and test helpers relying on concrete pool types.
3. Keep handler behavior unchanged (`send_raw_transaction` continues to call `push()`).
4. Validate `rpc-eth` build/tests.

## Atomic Verification
- `nix develop --command cargo build -p rpc-eth`
- `nix develop --command cargo test -p rpc-eth`

## Done When
- `rpc-eth` compiles with trait-object tx pool.
- Existing RPC tests remain green.
