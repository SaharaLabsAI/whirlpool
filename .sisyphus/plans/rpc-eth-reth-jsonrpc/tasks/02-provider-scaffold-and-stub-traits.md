# Task 02: Scaffold `WhirlpoolProvider` and stub trait surface

## Status
- pending

## Dependencies
- 01

## Wave
- Wave 2

## Complexity
- L

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 01 is complete and committed.
- [ ] Scope still matches REQ-2 and excludes vendor edits.
- [ ] Artifact Registry shows TST-1 pending for creation by this task.
- [ ] `crates/rpc-eth/src/provider.rs` does not already contain the full provider adapter.
- [ ] This task remains commit-ready despite touching a large trait surface.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/strategy.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/tests.md`
- Codebase references:
  - `crates/rpc-eth/src/lib.rs`
  - `crates/rpc-eth/src/server.rs`
  - `crates/rpc-eth/src/provider.rs`
  - `crates/rpc-eth/tests/provider_contract.rs`
- Vendor references:
  - `vendor/reth/crates/storage/provider`
  - `vendor/reth/crates/rpc/rpc-eth-api/src/core.rs`

## Vendor Usage Patterns
- Follow reth provider trait signatures exactly from the pinned vendor crates.
- Use noop/stub semantics patterned after reth's noop provider implementations where applicable.

## What to do
1. Author TST-1-first compile-contract coverage in `crates/rpc-eth/tests/provider_contract.rs` that proves a `WhirlpoolProvider` instance can satisfy the `RpcModuleBuilder` provider bounds once implemented.
2. Create `crates/rpc-eth/src/provider.rs` with the `WhirlpoolProvider` struct holding `Arc<RethStateDb>`, `Arc<ChainSpec>`, and the noop canonical-state broadcast sender.
3. Implement the stub-first trait set from `agent/handoff.md`: `StageCheckpointReader`, `ChangeSetReader`, `PruneCheckpointReader`, `HashedPostStateProvider`, `StateRootProvider`, `StorageRootProvider`, `StateProofProvider`, and `BlockBodyIndicesProvider`.
4. Export the new module through `crates/rpc-eth/src/lib.rs` only as much as needed for the contract test to compile; do not rewrite the public API yet.
5. Keep all real data-backed trait impls deferred to Tasks 03-05, but ensure the module compiles cleanly with stubbed returns.

## Mock Boundary
- Mock only the behavior that the design explicitly marks as noop or empty-provider behavior.
- Do not fabricate real block, receipt, or account reads in this task; those belong to later provider slices.

## AC trace
- REQ-2
- TST-1

## Must NOT do
- Do not implement `BlockReader`, `TransactionsProvider`, `ReceiptProvider`, or `StateProviderFactory` here.
- Do not rewrite `server.rs` to use `RpcModuleBuilder` yet.
- Do not modify `crates/whirlpool-node/**`.
- Do not touch `vendor/**`.

## Acceptance Criteria
- [ ] `WhirlpoolProvider` exists in `crates/rpc-eth/src/provider.rs` with the designed field layout.
- [ ] The stub trait surface compiles and returns noop/empty values per design.
- [ ] TST-1 scaffolding exists and exercises provider trait bounds.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `.sisyphus/evidence/task-02-provider-scaffold-and-stub-traits.md` captures commands and outcomes.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract`
- [ ] Evidence file includes which stub traits were added and any remaining real-trait TODO seams.
- [ ] Artifact Registry updates TST-1 from `pending` to `created` with actual test location/name.
- [ ] Create one dedicated git commit for this task before starting Task 03.

## Post-Task Reconciliation
- Update the Artifact Registry row for TST-1 with the actual test name(s) created in `crates/rpc-eth/tests/provider_contract.rs`.

## QA Scenarios
- Happy path: provider stubs satisfy compile-time bounds with noop behavior.
- Failure path: trait signature drift from vendor reth requires correcting imports or associated types.
- Boundary case: noop methods return empty/`None` responses without leaking fake chain data.

## Evidence
- `.sisyphus/evidence/task-02-provider-scaffold-and-stub-traits.md`
