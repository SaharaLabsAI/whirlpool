# Task 10: whirlpool-node-wiring

**Status**: pending
**Dependencies**: 04, 06, 08, 09
**Wave**: 5
**Complexity**: M
**Target Crate(s)**: whirlpool-node (role: wiring)

## Pre-Task Gate
- `nix develop --command cargo build -p app-evm` succeeds.
- `nix develop --command cargo build -p state-reth` succeeds.
- `nix develop --command cargo build -p rpc-eth` succeeds.

## Context
Wiring the persistence flow into the Whirlpool node is the final integration task. This requires creating a `PersistingFinalizationSink` that calls the application-level `store_finalized_block` upon block finalization and updating the RPC context to use the same persistent database for block queries.

## What to do

### TDD Flow
1. Create the `PersistingFinalizationSink` struct and its implementation.
2. Update the `whirlpool-node/src/main.rs` to use the new sink wrapper.
3. Update the `EthRpcContext::new` call to pass the `RethStateDb` as the `block_storage`.
4. Verify the node builds and starts correctly.

### Specific steps
1. Edit `crates/whirlpool-node/src/main.rs`:
   - Create `struct PersistingFinalizationSink<S: StateDb + BlockStorage, A: Application>`.
   - Implement a `on_finalized` callback that calls `app.store_finalized_block(block, &state_db)` before forwarding to the original sink.
   - Update `main()` to wrap the existing `FinalizationSink` in a `PersistingFinalizationSink`.
   - Update `EthRpcContext::new()` to pass the shared `Arc<RwLock<RethStateDb>>` as the new `block_storage` argument.
2. Add necessary imports for `app_evm::EvmApplication`, `state_reth::RethStateDb`, and `rpc_eth::EthRpcContext`.
3. Update the `EthRpcContext` generic signature in `main()`.

## Mock Boundary
N/A (actual wiring)

## Must NOT do
- Do NOT change the `CommonwareEngine` or other consensus-simplex logic.
- Do NOT modify the `PersistingFinalizationSink` to be asynchronous unless the trait requires it.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/scratch/plan/grounding-map.md`
- `docs/crates/whirlpool-node.md`

## Acceptance Criteria
- `nix develop --command cargo build -p whirlpool-node` succeeds.
- Node starts without panic (log check: "Node initialized with persistent block storage").

## Post-Task Gate
- Command: `nix develop --command cargo build -p whirlpool-node`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-INT-02, TC-FLW-04 status: created).

## QA Scenarios
- QA-11: Node starts with block storage wired.

## Evidence
`.sisyphus/evidence/task-10-whirlpool-node-wiring.txt`
