# Design Artifact Index

## Reading Order
1. `review/DESIGN.md` - review-lane verdict and source-grounded finalization summary for Sub-Intent B
2. `agent/strategy.md` - scope guardrails and high-level design intent
3. `agent/crates.md` - file-level crate change specifications for `crates/whirlpool-node`
4. `agent/workspace.md` - workspace integration plan and dependency boundaries
5. `agent/domains.md` - domain model, ownership, and invariants
6. `agent/crate-contracts/whirlpool-node.md` - finalized crate contract for startup config and wiring
7. `agent/flows.md` - CLI parsing, startup wiring, bootstrap parsing, and storage derivation flows
8. `agent/tests.md` - requirement-to-test mapping with concrete unit and integration cases
9. `agent/handoff.md` - implementation order, dependencies, and verification steps
10. `agent/TASK_GEN_READY.md` - readiness marker for task generation
11. `agent/blockers.md` - blocker gate status
12. `agent/requirements.md` - original requirement ledger for full intent set

## Agent Lane Inventory
- `agent/strategy.md`
- `agent/crates.md`
- `agent/workspace.md`
- `agent/domains.md`
- `agent/blockers.md`
- `agent/requirements.md`
- `agent/crate-contracts/whirlpool-node.md`
- `agent/flows.md`
- `agent/tests.md`
- `agent/handoff.md`
- `agent/TASK_GEN_READY.md`
- `agent/exploration/node-config-startup.md`

## Review Lane Inventory
- `review/DESIGN.md`
- `review/INDEX.md`

## Traceability Highlights
- `REQ-4` configuration surface:
  - `agent/strategy.md`
  - `agent/crates.md`
  - `agent/domains.md`
  - `agent/crate-contracts/whirlpool-node.md`
  - `agent/flows.md`
  - `agent/tests.md`
- `REQ-5` startup wiring:
  - `agent/workspace.md`
  - `agent/crate-contracts/whirlpool-node.md`
  - `agent/flows.md`
  - `agent/tests.md`
  - `agent/handoff.md`
- Readiness gate:
  - `agent/blockers.md`
  - `agent/TASK_GEN_READY.md`
  - `review/DESIGN.md`

## Scope Notes
- Finalization scope is Sub-Intent B only: `node-config-startup-wiring`.
- Only `crates/whirlpool-node` is designed to change.
- `crates/p2p-commonware` remains a read-only consumed contract in this phase.

## Readiness
- Current blocker state: PASS (`agent/blockers.md`)
- Task generation marker: READY (`agent/TASK_GEN_READY.md`)
- Review verdict: PASS (`review/DESIGN.md`)
