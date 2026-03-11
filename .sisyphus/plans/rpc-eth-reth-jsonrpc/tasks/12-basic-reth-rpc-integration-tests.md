# Task 12: Add basic reth-backed RPC integration coverage

## Status
- pending

## Dependencies
- 11

## Wave
- Wave 5

## Complexity
- L

## Target crates
- `testing/integration-tests` - end-to-end verification crate
- `rpc-eth` - tested API provider
- `whirlpool-node` - startup integration boundary

## Pre-Task Gate
- [ ] Task 11 is complete and committed.
- [ ] TST-4, TST-5, TST-6, and TST-7 are pending for creation in this task.
- [ ] The builder-backed server path already compiles end to end.
- [ ] Scope is limited to startup plus basic `eth_*` behavior coverage.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/tests.md`
- Codebase references:
  - `testing/integration-tests/tests/rpc_integration.rs`
  - `testing/integration-tests/Cargo.toml`
  - `crates/whirlpool-node/src/main.rs`
- Vendor references:
  - `vendor/reth/crates/rpc/rpc-builder/tests/it/http.rs`

## Vendor Usage Patterns
- Mirror reth `rpc-builder` HTTP tests for server startup, typed clients, and simple param-driven assertions.
- Reuse alloy `ProviderBuilder`-style clients only where they still match the reth test pattern.

## What to do
1. Rewrite or extend `testing/integration-tests/tests/rpc_integration.rs` first so TST-4, TST-5, TST-6, and TST-7 exist before any follow-on integration adjustments.
2. Add startup coverage that proves the reth-backed server accepts HTTP connections and exposes `eth_chainId`.
3. Add basic latest-block and balance-path coverage using seeded `RethStateDb` / block-storage fixtures that flow through the new provider adapter.
4. Align the harness structure with the reth `rpc-builder` HTTP test style called out in the design docs.
5. Keep block retrieval permutations, raw transaction submission, and blob exclusion coverage for Task 13.

## Mock Boundary
- Use seeded local state/block fixtures and in-process HTTP servers only.
- Do not mock the reth builder stack itself; the goal is real end-to-end wiring.

## AC trace
- REQ-1
- REQ-2
- REQ-7
- TST-4
- TST-5
- TST-6
- TST-7

## Must NOT do
- Do not add blob exclusion assertions here.
- Do not add all parameter permutations for block and raw-transaction methods yet.
- Do not change implementation code outside fixes required for these tests to pass.

## Acceptance Criteria
- [ ] `testing/integration-tests/tests/rpc_integration.rs` includes TST-4, TST-5, TST-6, and TST-7 coverage.
- [ ] The harness shape mirrors reth HTTP test patterns.
- [ ] `nix develop --command cargo test -p integration-tests --test rpc_integration` passes.
- [ ] `.sisyphus/evidence/task-12-basic-reth-rpc-integration-tests.md` records commands and outcomes.
- [ ] The result is a coherent checkpoint suitable for a dedicated commit.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo build -p whirlpool-node`
- [ ] `nix develop --command cargo test -p integration-tests --test rpc_integration`
- [ ] Evidence file records actual test names for TST-4 through TST-7.
- [ ] Artifact Registry updates TST-4, TST-5, TST-6, and TST-7 with actual names/locations/status.
- [ ] Create one dedicated git commit for this task before starting Task 13.

## Post-Task Reconciliation
- Update Artifact Registry rows for TST-4 through TST-7 with exact test names and locations in `testing/integration-tests/tests/rpc_integration.rs`.

## QA Scenarios
- Happy path: server boots and returns configured chain ID, latest block number, and funded balances.
- Failure path: startup succeeds but provider-backed block/state queries fail due to miswired adapters.
- Boundary case: empty chain / zero-balance queries still return valid RPC responses.

## Evidence
- `.sisyphus/evidence/task-12-basic-reth-rpc-integration-tests.md`
