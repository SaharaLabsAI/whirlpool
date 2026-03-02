<!-- PLAN_DIR: .sisyphus/plans/evm-tx-execution -->
# EVM Transaction Execution

## TL;DR
Implement full EVM transaction execution in `app-evm` using reth's block executor, ensuring state changes commit to `state::InMemoryStateDb` with correct block field computation.

## Deliverables
- Real `propose()` and `verify()` implementations in `app-evm`
- Transaction decoding and sender recovery helper
- Unit tests for state commitment and clone isolation
- Integration test for full propose-verify cycle

## Effort
[XL — estimated ~535 lines changed across 7 tasks]

## Critical Path
State verification tests and transaction decoding (Wave 1) enable core execution logic for proposal and verification (Wave 2), culminating in full system integration (Wave 3).

## Context
- Design docs: `docs/design/evm-tx-execution/`
- Primary crate: `crates/app-evm/`
- Secondary crate: `crates/state/`
- Scope: Sub-Intent 1 of broader EVM block production design

## Task Index
[Link to INDEX.md](evm-tx-execution/INDEX.md)
