# Task 08: cross-crate-integration-tests

**Status**: pending
**Dependencies**: 07
**Wave / Phase**: Wave 6 / Phase 6 (integration tests)
**Complexity**: M
**Target Crate(s)**: `app-evm`, `rpc-eth`, `whirlpool-node`, `integration-tests`
**AC IDs**: AC-1, AC-2, AC-4, AC-5

## Objective
Add/enable integration tests validating RPC submission, persistent drain behavior, startup recovery, and ordering across crate boundaries.

## Design Refs
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/TESTS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/FLOWS.md`
- `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/proven-ac.md`

## Steps
1. Implement integration tests for RPC->mempool and mempool->proposal flows.
2. Add restart recovery scenarios (crash-after-push, crash-during-drain semantics).
3. Validate FIFO ordering in integration path.
4. Run crate-level integration test commands and capture evidence.

## Atomic Verification
- `nix develop --command cargo test -p app-evm`
- `nix develop --command cargo test -p rpc-eth`
- `nix develop --command cargo test -p whirlpool-node`

## Done When
- Integration tests validate trait-object wiring + persistence behavior.
- Regression tests in touched crates remain passing.
