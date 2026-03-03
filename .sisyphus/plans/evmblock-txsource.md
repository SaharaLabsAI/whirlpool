# evmblock-txsource
> Entry point. Full plan: [.sisyphus/plans/evmblock-txsource/INDEX.md](./evmblock-txsource/INDEX.md)
**Plan directory**: `.sisyphus/plans/evmblock-txsource/`
**Tasks**: See INDEX.md for task list and execution strategy.
<!-- PLAN_DIR: .sisyphus/plans/evmblock-txsource/ -->

## TL;DR
Implement an in-memory transaction pool for the EVM application, replacing the no-op source with a thread-safe buffer that collects submitted transactions and drains them during block proposal.

## Deliverables
- `InMemoryTxPool` implementation in `app` crate
- Node wiring updates in `whirlpool-node` to inject the pool handle
- Integration test for push-propose cycle in `app-evm`
- Workspace-wide compliance audit and documentation sync

## Effort
[S — 4 tasks, all complexity S]

## Critical Path
Implementation of the core pool and node wiring (Wave 1) enables the integration test (Wave 2), followed by a final compliance audit (Wave 3).

## Context
- Design docs: `docs/design/evmblock-txsource/`
- Primary crates: `crates/app/`, `crates/whirlpool-node/`, `crates/app-evm/`
- Scope: Transaction submission and pool management for block production.

## Task Index
[Link to INDEX.md](./evmblock-txsource/INDEX.md)
