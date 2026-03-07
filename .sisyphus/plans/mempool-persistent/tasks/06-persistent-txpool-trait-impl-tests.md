# Task 06: persistent-txpool-trait-impl-tests

**Status**: pending
**Dependencies**: 05
**Wave / Phase**: Wave 4 / Phase 4 (PersistentTxPool)
**Complexity**: M
**Target Crate(s)**: `mempool`
**AC IDs**: AC-1, AC-3, AC-5

## Objective
Implement `TxSource` for `PersistentTxPool` and validate trait-level semantics (FIFO drain, concurrency, duplicate acceptance).

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/DOMAINS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/TESTS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/mempool.md`

## Steps
1. Implement `PersistentTxPool::open(path)` around `MempoolStore` with interior synchronization.
2. Implement `TxSource::push()` delegation to store writes.
3. Implement `TxSource::pending()` delegation to store drain.
4. Add tests for trait-object usage, concurrent push safety, and FIFO return order.

## Atomic Verification
- `nix develop --command cargo test -p mempool`

## Done When
- `PersistentTxPool` is a drop-in `TxSource` implementation.
- Trait semantics match in-memory expectations where specified.
