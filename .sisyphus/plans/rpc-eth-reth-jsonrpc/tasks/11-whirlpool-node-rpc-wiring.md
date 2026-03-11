# Task 11: Update `whirlpool-node` for `RpcConfig` startup

## Status
- pending

## Dependencies
- 10

## Wave
- Wave 5

## Complexity
- M

## Target crates
- `whirlpool-node` - integration boundary
- `rpc-eth` - consumed API boundary

## Pre-Task Gate
- [ ] Task 10 is complete and committed.
- [ ] `rpc-eth` exports the new `RpcConfig` entrypoint.
- [ ] Artifact Registry shows TST-12 pending for this task.
- [ ] Scope is limited to node startup wiring and legacy RPC dependency removal.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/domains.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crate-contracts/rpc-eth.md`
- Codebase references:
  - `crates/whirlpool-node/src/main.rs`
  - `crates/whirlpool-node/src/node.rs`
  - `crates/whirlpool-node/src/lib.rs`
  - `crates/rpc-eth/src/lib.rs`
  - `testing/integration-tests/tests/rpc_integration.rs`

## What to do
1. Add or update a node-level startup smoke test first so TST-12 becomes executable against the new `RpcConfig` boundary before fully removing the old wiring.
2. Update `crates/whirlpool-node/src/main.rs` and/or `crates/whirlpool-node/src/node.rs` to construct `rpc_eth::RpcConfig` from the existing `RethStateDb`, `TxSource`, chain ID, and bind address inputs.
3. Remove legacy `ReceiptStore`, `EthRpcContext`, and manual block-height wiring from the node startup path.
4. Keep node startup sequencing unchanged apart from the RPC-server construction path.
5. Verify both `rpc-eth` and `whirlpool-node` build with the new integration seam.

## Mock Boundary
- Use existing node startup test seams or lightweight startup smoke coverage only.
- Do not add full network or consensus integration tests here; end-to-end RPC behavior remains in Tasks 12-13.

## AC trace
- REQ-6
- TST-12

## Must NOT do
- Do not add all supported RPC method assertions here.
- Do not change consensus or P2P startup behavior outside RPC wiring.
- Do not touch `vendor/**`.

## Acceptance Criteria
- [ ] `whirlpool-node` constructs and starts the new `rpc-eth` server path via `RpcConfig`.
- [ ] Legacy node-side RPC context/receipt-store plumbing is removed.
- [ ] TST-12 startup smoke coverage exists and passes.
- [ ] `nix develop --command cargo build -p whirlpool-node` passes.
- [ ] `.sisyphus/evidence/task-11-whirlpool-node-rpc-wiring.md` records commands and outcomes.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo build -p whirlpool-node`
- [ ] `nix develop --command cargo test -p whirlpool-node`
- [ ] Evidence file records the exact node file(s) updated and the TST-12 smoke test used.
- [ ] Artifact Registry updates TST-12 with actual test name/location.
- [ ] Create one dedicated git commit for this task before starting Task 12.

## Post-Task Reconciliation
- Update the TST-12 row with actual startup smoke test details and any location changes in `testing/integration-tests/tests/rpc_integration.rs`.

## QA Scenarios
- Happy path: node startup creates the reth-backed RPC server without changing other startup phases.
- Failure path: stale legacy context wiring causes compile errors that must be removed.
- Boundary case: optional RPC bind configuration still maps cleanly into `RpcConfig`.

## Evidence
- `.sisyphus/evidence/task-11-whirlpool-node-rpc-wiring.md`
