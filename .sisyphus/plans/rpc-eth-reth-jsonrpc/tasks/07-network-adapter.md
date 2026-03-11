# Task 07: Implement `WhirlpoolNetwork`

## Status
- pending

## Dependencies
- 05

## Wave
- Wave 3

## Complexity
- S

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 05 is complete and committed.
- [ ] Artifact Registry shows TST-3 pending for this task.
- [ ] Scope is limited to the minimal `NetworkInfo` adapter.
- [ ] No real P2P integration is required by the design docs.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/strategy.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/domains.md`
- Codebase references:
  - `crates/rpc-eth/src/network.rs`
  - `crates/rpc-eth/tests/network_contract.rs`
  - `crates/rpc-eth/src/lib.rs`

## Vendor Usage Patterns
- Implement only the static informational methods required by `reth_network_api::NetworkInfo`: chain ID, syncing status, and minimal peer/network status.

## What to do
1. Create TST-3-first coverage in `crates/rpc-eth/tests/network_contract.rs` for configured `chain_id`, non-syncing behavior, and static no-peer network status.
2. Create `crates/rpc-eth/src/network.rs` with `WhirlpoolNetwork { chain_id: u64, ... }` per the design.
3. Implement the required `NetworkInfo` methods with static values suitable for RPC informational calls.
4. Export the module minimally for later `server.rs` wiring.
5. Keep all actual networking and peer management out of scope.

## Mock Boundary
- No external network system is connected; the adapter is intentionally static.
- Use direct constructor-based tests instead of spinning up any peer subsystem.

## AC trace
- REQ-4
- TST-3

## Must NOT do
- Do not add P2P wiring or peer discovery.
- Do not modify `whirlpool-node` networking code.
- Do not rewrite server startup in this task.

## Acceptance Criteria
- [ ] `WhirlpoolNetwork` exists and satisfies required `NetworkInfo` bounds.
- [ ] TST-3 coverage passes with static informational semantics.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `.sisyphus/evidence/task-07-network-adapter.md` records commands and outputs.
- [ ] The result is a dedicated, reviewable checkpoint.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth network_contract`
- [ ] Evidence file records configured chain ID and static network-status assertions.
- [ ] Artifact Registry updates TST-3 with actual test names/locations.
- [ ] Create one dedicated git commit for this task before starting Task 08 or Task 09.

## Post-Task Reconciliation
- Update the TST-3 row with exact test names in `crates/rpc-eth/tests/network_contract.rs`.

## QA Scenarios
- Happy path: configured chain ID is surfaced correctly.
- Failure path: no-peer status remains well-formed rather than panicking on absent network state.
- Boundary case: syncing flag stays false for startup informational calls.

## Evidence
- `.sisyphus/evidence/task-07-network-adapter.md`
