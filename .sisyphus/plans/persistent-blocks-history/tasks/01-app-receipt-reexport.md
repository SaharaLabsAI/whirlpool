# Task 01: app-receipt-reexport

**Status**: pending
**Dependencies**: none
**Wave**: 1
**Complexity**: S
**Target Crate(s)**: app (role: interface)

## Pre-Task Gate
N/A (Wave 1 task)

## Context
Downstream crates like `state-reth` and `rpc-eth` need a shared `Receipt` type for trait signatures and API responses. The design dictates re-exporting `alloy-consensus::Receipt` from the `app` crate to maintain architectural stability and avoid direct external dependencies in most downstream modules.

## What to do

### TDD Flow
1. Update `app/Cargo.toml` to include `alloy-consensus`.
2. Add the re-export to `app/src/lib.rs`.
3. Verify the crate compiles.

### Specific steps
1. Edit `crates/app/Cargo.toml` and add `alloy-consensus = { workspace = true }` to the `[dependencies]` section.
2. Edit `crates/app/src/lib.rs` and add `pub use alloy_consensus::Receipt;`.

## Mock Boundary
N/A

## Must NOT do
- Do NOT implement any custom receipt logic.
- Do NOT change the existing `EvmBlock` or `ExecutionResult` types.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/scratch/plan/grounding-map.md`
- `docs/crates/app.md`

## Acceptance Criteria
- `nix develop --command cargo build -p app` succeeds.

## Post-Task Gate
- Command: `nix develop --command cargo build -p app`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (N/A for this task as no TestIDs are mapped).

## QA Scenarios
N/A

## Evidence
`.sisyphus/evidence/task-01-app-receipt-reexport.txt`
