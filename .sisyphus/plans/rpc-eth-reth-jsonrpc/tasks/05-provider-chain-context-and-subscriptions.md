# Task 05: Implement provider chain context and subscription adapters

## Status
- pending

## Dependencies
- 04

## Wave
- Wave 2

## Complexity
- M

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 04 is complete and committed.
- [ ] The provider contract suite from TST-1 exists and passes.
- [ ] Scope is limited to `ChainSpecProvider`, `CanonStateSubscriptions`, `AccountReader`, and `NodePrimitivesProvider` completion.
- [ ] This task remains commit-ready.
- [ ] Server wiring is still deferred to later tasks.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crate-contracts/rpc-eth.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
- Codebase references:
  - `crates/rpc-eth/src/provider.rs`
  - `crates/rpc-eth/tests/provider_contract.rs`

## Vendor Usage Patterns
- `CanonStateSubscriptions` uses a noop `tokio::sync::broadcast::Sender<CanonStateNotification>` that never fires but satisfies builder bounds.
- `NodePrimitivesProvider` should expose the expected Ethereum primitive set without inventing new node types.

## What to do
1. Extend the provider contract tests first so TST-1 also covers chain-spec access, account reads, node primitive exposure, and noop canonical subscription setup.
2. Implement `ChainSpecProvider` to return the stored `Arc<ChainSpec>`.
3. Implement `AccountReader` by delegating to `StateDb::get_account` through the `RethStateDb` backend.
4. Implement `NodePrimitivesProvider` and `CanonStateSubscriptions`, using the noop broadcast channel design from `agent/crates.md`.
5. Confirm `WhirlpoolProvider` now satisfies the full adapter contract needed before pool/network/server work begins.

## Mock Boundary
- Canonical state notifications remain intentionally noop because real subscription feeds are out of scope for this RPC-focused slice.
- Do not add background tasks or live chain event propagation.

## AC trace
- REQ-2
- REQ-4
- TST-1

## Must NOT do
- Do not rewrite `server.rs` or `lib.rs` here.
- Do not implement `WhirlpoolTxPool` or `WhirlpoolNetwork` in this task.
- Do not touch `whirlpool-node`.

## Acceptance Criteria
- [ ] `WhirlpoolProvider` now includes the remaining real adapter traits and noop canonical subscription support.
- [ ] TST-1 provider contract coverage passes against the completed provider surface.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract` passes.
- [ ] `.sisyphus/evidence/task-05-provider-chain-context-and-subscriptions.md` records gates and remaining downstream dependencies.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract`
- [ ] Evidence file confirms provider work is complete and server wiring can begin.
- [ ] Artifact Registry marks TST-1 as `implemented` with final test names/locations.
- [ ] Create one dedicated git commit for this task before starting Task 06.

## Post-Task Reconciliation
- Mark the TST-1 Artifact Registry row as implemented and record the exact provider contract test names.

## QA Scenarios
- Happy path: chain spec and account reads are surfaced through the provider adapter.
- Failure path: unknown account returns empty/default provider semantics.
- Boundary case: canonical subscription stream exists but emits no notifications.

## Evidence
- `.sisyphus/evidence/task-05-provider-chain-context-and-subscriptions.md`
