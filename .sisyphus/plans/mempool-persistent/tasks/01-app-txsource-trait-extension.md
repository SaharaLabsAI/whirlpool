# Task 01: app-txsource-trait-extension

**Status**: pending
**Dependencies**: none
**Wave / Phase**: Wave 1 / Phase 1 (TxSource trait extension)
**Complexity**: S
**Target Crate(s)**: `app`
**AC IDs**: AC-4

## Objective
Extend `TxSource` with `fn push(&self, tx: Vec<u8>)` and enforce trait-object-safe bounds required for shared runtime use.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/STRATEGY.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/app.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/proven-ac.md`

## Steps
1. Update `app/src/traits.rs` to extend `TxSource` with `push()` and `Send + Sync` bounds.
2. Ensure trait signature remains infallible (`push` returns `()`, `pending` unchanged return type).
3. Compile `app` to surface implementor fallout for the next task.

## Atomic Verification
- `nix develop --command cargo build -p app`

## Done When
- `TxSource` includes `push()` and trait object bounds compile.
- No files under `vendor/` are touched.

## Notes
This task intentionally may break downstream implementors; reconciliation is handled in Task 02.
