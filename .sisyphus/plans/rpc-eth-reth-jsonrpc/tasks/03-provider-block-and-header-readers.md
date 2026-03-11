# Task 03: Implement provider block/header/hash readers

## Status
- pending

## Dependencies
- 02

## Wave
- Wave 2

## Complexity
- M

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 02 is complete and committed.
- [ ] TST-1 already exists and can be extended in place.
- [ ] Scope is limited to block/hash/header reads required by REQ-2.
- [ ] `RethStateDb` remains the canonical backend for block lookup.
- [ ] This task remains independently commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/domains.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
- Codebase references:
  - `crates/rpc-eth/src/provider.rs`
  - `crates/rpc-eth/tests/provider_contract.rs`
- Vendor references:
  - `vendor/reth/crates/storage/provider`

## Vendor Usage Patterns
- Delegate real header and block readers to `BlockStorage`/`RethStateDb`-backed calls, then shape results into the exact reth types expected by the trait signatures.

## What to do
1. Extend the provider contract tests first so TST-1 asserts `BlockHashReader`, `BlockNumReader`, `HeaderProvider`, `BlockReader`, and `BlockReaderIdExt` compile and return values from seeded block-storage fixtures.
2. Implement `BlockHashReader` using `RethStateDb::get_block_hash` or the corresponding block-storage delegate.
3. Implement `BlockNumReader`, `HeaderProvider`, `BlockReader`, and `BlockReaderIdExt` by delegating to block-storage reads and header extraction.
4. Add only the minimal helper functions needed inside `provider.rs` to translate storage reads into the provider trait outputs; leave full block conversion utilities for Task 08.
5. Keep all state/account/receipt/provider-factory work out of this task.

## Mock Boundary
- Use seeded in-memory or test `RethStateDb` fixtures in crate tests.
- Do not mock reth trait signatures; only mock storage contents necessary to exercise lookup behavior.

## AC trace
- REQ-2
- TST-1
- TST-6
- TST-8

## Must NOT do
- Do not implement `StateProviderFactory`, `TransactionsProvider`, or `ReceiptProvider` here.
- Do not rewrite RPC server startup.
- Do not update `whirlpool-node`.

## Acceptance Criteria
- [ ] Provider block/hash/header trait impls delegate to the real storage backend.
- [ ] TST-1 coverage expands to include seeded block lookup behavior.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract` passes.
- [ ] `.sisyphus/evidence/task-03-provider-block-and-header-readers.md` records the gate results.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract`
- [ ] Evidence file notes the storage fixture used for block/header verification.
- [ ] Artifact Registry keeps TST-1 current and notes that TST-6/TST-8 remain integration-pending.
- [ ] Create one dedicated git commit for this task before starting Task 04.

## Post-Task Reconciliation
- Update the TST-1 Artifact Registry row if test names changed while extending the provider contract suite.

## QA Scenarios
- Happy path: latest block number and header are returned from storage.
- Failure path: missing block/hash yields `None`/not-found behavior without panic.
- Boundary case: block-id extension logic resolves latest and explicit identifiers consistently.

## Evidence
- `.sisyphus/evidence/task-03-provider-block-and-header-readers.md`
