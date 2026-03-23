# Task 13: Add blob exclusion and remaining RPC integration coverage

## Status
- pending

## Dependencies
- 12

## Wave
- Wave 5

## Complexity
- M

## Target crates
- `testing/integration-tests` - end-to-end verification crate
- `rpc-eth` - tested API provider

## Pre-Task Gate
- [ ] Task 12 is complete and committed.
- [ ] Artifact Registry shows TST-8, TST-9, TST-10, and TST-11 pending for this task.
- [ ] The basic HTTP integration harness already passes.
- [ ] Scope is limited to the remaining supported RPC methods and blob exclusion behavior.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/strategy.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/tests.md`
- Codebase references:
  - `testing/integration-tests/tests/rpc_evm_integration.rs`
  - `crates/rpc-eth/src/server.rs`
  - `crates/rpc-eth/src/pool.rs`
- Vendor references:
  - `vendor/reth/crates/rpc/rpc-builder/tests/it/http.rs`

## Vendor Usage Patterns
- Reuse the same HTTP harness style from Task 12, adding the parameter permutations and unsupported-method checks highlighted by reth's tests.

## What to do
1. Extend `testing/integration-tests/tests/rpc_evm_integration.rs` first so TST-8, TST-9, TST-10, and TST-11 are defined before implementation fixes are applied.
2. Add `eth_getBlockByNumber` coverage using seeded block fixtures and parameter permutations modeled after reth HTTP tests.
3. Add `eth_sendRawTransaction` coverage that proves accepted transactions hit `TxSource` and blob/Type-3 payloads are rejected.
4. Add explicit `eth_blobBaseFee` unsupported-method assertions and any remaining request-shape permutations required by TST-11.
5. Keep changes tightly scoped to the integration harness and only the implementation fixes needed to satisfy these assertions.

## Mock Boundary
- Continue using local seeded fixtures and in-process HTTP startup.
- Do not introduce blob execution or any vendor modifications; unsupported behavior is the contract.

## AC trace
- REQ-3
- REQ-5
- REQ-7
- TST-8
- TST-9
- TST-10
- TST-11

## Must NOT do
- Do not expand into admin/debug/engine namespaces.
- Do not add new non-`eth_*` coverage.
- Do not touch `vendor/**`.

## Acceptance Criteria
- [ ] TST-8, TST-9, TST-10, and TST-11 coverage exists and passes in `testing/integration-tests/tests/rpc_evm_integration.rs`.
- [ ] Blob requests receive the unsupported behavior required by the design.
- [ ] `nix develop --command cargo test -p integration-tests --test rpc_evm_integration` passes.
- [ ] `.sisyphus/evidence/task-13-blob-and-remaining-rpc-integration-tests.md` records commands and outcomes.
- [ ] The result is a coherent checkpoint suitable for a dedicated commit.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo build -p whirlpool-node`
- [ ] `nix develop --command cargo test -p integration-tests --test rpc_evm_integration`
- [ ] Evidence file records actual test names for TST-8 through TST-11.
- [ ] Artifact Registry updates TST-8, TST-9, TST-10, and TST-11 with actual names/locations/status.
- [ ] Create one dedicated git commit for this task before starting Task 14.

## Post-Task Reconciliation
- Update Artifact Registry rows for TST-8 through TST-11 with exact test names and final statuses.

## QA Scenarios
- Happy path: block retrieval and raw transaction submission work end to end.
- Failure path: blob/Type-3 payloads and `eth_blobBaseFee` fail with the expected unsupported response.
- Boundary case: block-by-number permutations and full-vs-hash-only transaction responses match the chosen reth defaults.

## Evidence
- `.sisyphus/evidence/task-13-blob-and-remaining-rpc-integration-tests.md`
