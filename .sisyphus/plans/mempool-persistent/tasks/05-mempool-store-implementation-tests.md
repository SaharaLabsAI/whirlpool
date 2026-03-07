# Task 05: mempool-store-implementation-tests

**Status**: pending
**Dependencies**: 04
**Wave / Phase**: Wave 4 / Phase 4 (MempoolStore)
**Complexity**: M
**Target Crate(s)**: `mempool`
**AC IDs**: AC-1, AC-3, AC-5

## Objective
Implement MDBX-backed `MempoolStore` operations (`open`, `push`, `drain_pending`) with FIFO and restart recovery guarantees.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/FLOWS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/TESTS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/mempool.md`

## Steps
1. Implement MDBX open/init and key encoding strategy (u64 big-endian).
2. Implement `push()` with monotonic key assignment and committed write transaction.
3. Implement `drain_pending()` as atomic read-all + delete-all + commit.
4. Add unit tests for push/drain/empty behavior and restart persistence.

## Atomic Verification
- `nix develop --command cargo test -p mempool --test store_persistence`

## Done When
- Store-level tests prove persistence across reopen and FIFO drain order.
- Public store API behavior is test-backed.
