# Task 04: Implement provider state, transaction, and receipt readers

## Status
- pending

## Dependencies
- 03

## Wave
- Wave 2

## Complexity
- M

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 03 is complete and committed.
- [ ] Provider block/header readers already compile.
- [ ] Scope is limited to REQ-2 state, transaction, and receipt read paths.
- [ ] Artifact Registry still shows TST-7 and TST-9 pending for later integration coverage.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
- Codebase references:
  - `crates/rpc-eth/src/provider.rs`
  - `crates/rpc-eth/tests/provider_contract.rs`
- Vendor references:
  - `vendor/reth/crates/storage/provider`

## Vendor Usage Patterns
- `TransactionsProvider` should decode from `EvmBlock.transactions` into `TransactionSigned` values at the provider boundary.
- `ReceiptProvider` should delegate directly to `BlockStorage::get_receipts_by_block`.

## What to do
1. Extend the provider contract tests first to cover state-provider acquisition, transaction extraction, and receipt retrieval behavior tied to TST-1.
2. Implement `StateProviderFactory` so `WhirlpoolProvider` can hand back the underlying `RethStateDb` as the reth `StateProvider` source.
3. Implement `TransactionsProvider` by decoding `EvmBlock.transactions` into `TransactionSigned` values and exposing them through the trait methods.
4. Implement `ReceiptProvider` by delegating to block-storage receipt access and shaping outputs to the reth trait contract.
5. Add only the minimum internal decode helpers needed if Task 08 conversion helpers are not yet present; replace ad hoc duplication later only if still scoped to this task.

## Mock Boundary
- Use fixture-backed `RethStateDb` or deterministic raw transaction bytes in tests.
- Do not add tx-pool submission behavior; that belongs to Task 06.

## AC trace
- REQ-2
- TST-1
- TST-7
- TST-9

## Must NOT do
- Do not implement chain-spec or canonical subscription behavior here.
- Do not rewrite server wiring.
- Do not add blob rejection logic; that belongs to Task 06.

## Acceptance Criteria
- [ ] `StateProviderFactory`, `TransactionsProvider`, and `ReceiptProvider` are implemented on `WhirlpoolProvider`.
- [ ] Provider contract tests cover seeded account, transaction, and receipt reads.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract` passes.
- [ ] `.sisyphus/evidence/task-04-provider-state-tx-and-receipt-readers.md` records gates and fixtures.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract`
- [ ] Evidence file captures how tx bytes and receipts were seeded for the provider suite.
- [ ] Artifact Registry keeps TST-1 current and leaves TST-7/TST-9 integration rows pending.
- [ ] Create one dedicated git commit for this task before starting Task 05.

## Post-Task Reconciliation
- Refresh the TST-1 row with any new provider contract test names.

## QA Scenarios
- Happy path: state provider returns the seeded account/transaction/receipt data.
- Failure path: absent receipts or transactions yield empty/`None` results without panic.
- Boundary case: zero-transaction blocks still satisfy trait methods cleanly.

## Evidence
- `.sisyphus/evidence/task-04-provider-state-tx-and-receipt-readers.md`
