# Task 02: app-inmemory-noop-alignment

**Status**: pending
**Dependencies**: 01
**Wave / Phase**: Wave 2 / Phase 2 (InMemoryTxPool + NoopTxSource)
**Complexity**: S
**Target Crate(s)**: `app`
**AC IDs**: AC-2, AC-4

## Objective
Update all in-tree `TxSource` implementors in `app` to satisfy the extended trait without changing runtime semantics.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/STRATEGY.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/app.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/TESTS.md`

## Steps
1. Implement trait-level `push()` for `InMemoryTxPool` using existing behavior.
2. Implement no-op `push()` for `NoopTxSource`.
3. Update/repair test mocks in `app` scope impacted by trait change.
4. Re-run `app` tests to confirm no regression.

## Atomic Verification
- `nix develop --command cargo test -p app`

## Done When
- `app` compiles and tests pass after trait extension.
- Existing drain/FIFO semantics remain unchanged.
