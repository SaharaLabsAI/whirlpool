# Task 01: Add RPC dependencies to Cargo manifests

**Status**: [ ] pending
**Dependencies**: none
**Wave**: 1
**Complexity**: S

## AC Coverage
- AC-1 through AC-12 (enabler task; establishes crate dependencies required by all RPC behavior and tests)

## Pre-Task Gate
N/A (no dependencies)

## Context
Establish dependency pins before any code edits so all later slices compile against consistent versions and types.

## What to do
1. Update `crates/whirlpool-node/Cargo.toml` dependencies:
   - `jsonrpsee = { version = "0.26.0", features = ["server", "macros"] }`
   - `alloy-primitives = { version = "1.5.0", features = ["map-foldhash"] }`
   - `alloy-rpc-types = { version = "1.4.3", features = ["eth"] }`
2. Add any required test-side dependencies for alloy integration tests in `crates/whirlpool-node/Cargo.toml` `[dev-dependencies]` (ProviderBuilder-compatible set).
3. If needed, align workspace manifest dependency declarations so whirlpool-node can consume pinned versions without drift.
4. Do not change source code in this task.

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/INTENT.md`
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/SUMMARY.md`
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/whirlpool-node/README.md`

## Acceptance Criteria
- Cargo manifests contain the pinned versions: jsonrpsee 0.26.0, alloy-primitives 1.5.0, alloy-rpc-types 1.4.3.
- No source files under `crates/whirlpool-node/src/` are modified.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/01-add-rpc-dependencies.log`
