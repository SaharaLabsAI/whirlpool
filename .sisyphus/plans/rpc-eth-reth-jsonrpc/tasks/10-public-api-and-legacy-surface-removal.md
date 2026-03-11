# Task 10: Rewrite `lib.rs` and remove legacy RPC surface

## Status
- pending

## Dependencies
- 09

## Wave
- Wave 4

## Complexity
- S

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 09 is complete and committed.
- [ ] `server.rs` already compiles with the new builder path.
- [ ] Scope is limited to the public crate API and legacy export cleanup.
- [ ] This task remains commit-ready.
- [ ] `whirlpool-node` has not yet been switched to the new API.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crate-contracts/rpc-eth.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/domains.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
- Codebase references:
  - `crates/rpc-eth/src/lib.rs`
  - `crates/rpc-eth/src/server.rs`
  - `crates/rpc-eth/src/context.rs`
  - `crates/rpc-eth/src/eth_api.rs`
  - `crates/rpc-eth/src/eth_handler.rs`
  - `crates/rpc-eth/src/receipt_store.rs`

## Vendor Usage Patterns
- Public error and handle aliases should match the reth/jsonrpsee types already chosen by the design contract.

## What to do
1. Add or update crate-level API tests first so the exported `RpcConfig`, `RpcError`, `RpcServerHandle`, and `start_rpc_server` surface is executable before removing legacy exports.
2. Rewrite `crates/rpc-eth/src/lib.rs` to expose the new public API centered on `RpcConfig` and the builder-backed `start_rpc_server`.
3. Stop exporting or using the legacy `EthRpcContext`, `EthApiServer`, `EthApiHandler`, and `receipt_store` surfaces; delete files only if the crate compiles cleanly and the removal stays within this task.
4. Keep internal adapter modules visible only as needed for `server.rs` and tests.
5. Ensure the crate remains buildable as a clean public boundary for `whirlpool-node`.

## Mock Boundary
- Use minimal API-surface tests only.
- Do not modify downstream consumers yet; that belongs to Task 11.

## AC trace
- REQ-1
- REQ-6

## Must NOT do
- Do not update `crates/whirlpool-node/**` in this task.
- Do not add new integration-test assertions here.
- Do not touch `vendor/**`.

## Acceptance Criteria
- [ ] `crates/rpc-eth/src/lib.rs` exposes `RpcConfig`, `RpcError`, `RpcServerHandle`, and `start_rpc_server` per the design contract.
- [ ] Legacy public RPC surface is removed or internalized.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `nix develop --command cargo test -p rpc-eth --lib` passes.
- [ ] `.sisyphus/evidence/task-10-public-api-and-legacy-surface-removal.md` records commands and outcomes.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth --lib`
- [ ] Evidence file records which legacy files were removed or retained as internal-only.
- [ ] Artifact Registry remains unchanged.
- [ ] Create one dedicated git commit for this task before starting Task 11.

## Post-Task Reconciliation
- Note whether any TST rows need location adjustments because tests moved during legacy surface removal.

## QA Scenarios
- Happy path: downstream crates can import the new `RpcConfig` surface.
- Failure path: legacy exports removed too early break crate tests and must be corrected.
- Boundary case: internal modules remain available without being part of the public API.

## Evidence
- `.sisyphus/evidence/task-10-public-api-and-legacy-surface-removal.md`
