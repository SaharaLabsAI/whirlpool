# Task 08: Add `convert.rs` block and transaction conversion helpers

## Status
- pending

## Dependencies
- 05
- 06

## Wave
- Wave 3

## Complexity
- M

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Tasks 05 and 06 are complete and committed.
- [ ] Scope is limited to conversion helpers needed by provider/server code.
- [ ] Artifact Registry still shows TST-8 and TST-9 pending for later integration coverage.
- [ ] This task remains commit-ready.
- [ ] Ad hoc conversion code has not already spread into unrelated modules.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
- Codebase references:
  - `crates/rpc-eth/src/convert.rs`
  - `crates/rpc-eth/src/provider.rs`
  - `crates/rpc-eth/tests/provider_contract.rs`
- Vendor references:
  - `vendor/reth/crates/primitives-traits`
  - `vendor/reth/crates/ethereum/primitives`

## Vendor Usage Patterns
- Centralize `EvmBlock` -> reth block/header conversion here so provider and server code share one transformation path.
- Keep raw tx bytes -> `TransactionSigned` decode helpers colocated with block conversion helpers.

## What to do
1. Add or extend focused conversion tests first in `crates/rpc-eth/tests/provider_contract.rs` or a new dedicated conversion test file so block/tx conversion behavior is executable before wiring it into runtime paths.
2. Create `crates/rpc-eth/src/convert.rs` with helpers for `EvmBlock` to reth sealed block/header types and raw bytes to `TransactionSigned`.
3. Refactor `provider.rs` and `pool.rs` to call these helpers instead of open-coded conversions where appropriate, without changing task scope.
4. Keep conversion logic limited to the types named in the design docs; defer any extra RPC response shaping to reth itself.
5. Export the conversion module only as needed for internal crate use.

## Mock Boundary
- Use deterministic fixture blocks and raw signed transactions in tests.
- Do not implement full RPC response serialization; reth owns that layer.

## AC trace
- REQ-1
- REQ-2
- TST-8
- TST-9

## Must NOT do
- Do not rewrite `server.rs` in this task.
- Do not add new RPC methods here.
- Do not touch `whirlpool-node` or `testing/integration-tests/**`.

## Acceptance Criteria
- [ ] `crates/rpc-eth/src/convert.rs` exists with shared block/header/transaction conversion helpers.
- [ ] Existing provider/pool code uses the new helpers where relevant.
- [ ] Conversion-focused tests pass.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `.sisyphus/evidence/task-08-conversion-layer.md` records commands and results.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth provider_contract`
- [ ] Evidence file captures at least one block conversion and one tx decode assertion.
- [ ] Artifact Registry remains unchanged because TST-8/TST-9 integration rows are still pending.
- [ ] Create one dedicated git commit for this task before starting Task 09.

## Post-Task Reconciliation
- Note in evidence whether any provider-contract tests were renamed while absorbing conversion coverage.

## QA Scenarios
- Happy path: seeded `EvmBlock` converts into the expected reth block/header form.
- Failure path: malformed tx bytes fail decode cleanly.
- Boundary case: empty block transactions convert without panicking.

## Evidence
- `.sisyphus/evidence/task-08-conversion-layer.md`
