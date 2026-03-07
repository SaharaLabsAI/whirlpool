# Task 04: mempool-crate-scaffold

**Status**: pending
**Dependencies**: 02
**Wave / Phase**: Wave 4 / Phase 4 (new mempool crate)
**Complexity**: S
**Target Crate(s)**: `mempool` (new)
**AC IDs**: AC-3

## Objective
Create the new `mempool` crate skeleton with public API surface for `PersistentTxPool` and store layer.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/CRATES.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/crates/mempool.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/STRATEGY.md`

## Steps
1. Add workspace crate structure: `Cargo.toml`, `src/lib.rs`, `src/store.rs`, `src/persistent.rs`.
2. Add dependencies (`app`, `libmdbx-rs`, `parking_lot`, tracing as needed).
3. Define initial types (`PersistentTxPool`, `MempoolStore`, `MempoolError`) and compile-safe stubs.
4. Ensure crate is wired into workspace manifests.

## Atomic Verification
- `nix develop --command cargo build -p mempool`

## Done When
- New crate builds with declared API scaffolding.
- No behavior is wired into node yet.
