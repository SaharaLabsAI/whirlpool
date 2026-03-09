# Design Artifact Index

## Reading Order
1. `review/DESIGN.md` - review-lane summary and finalization verdict for Sub-Intent A
2. `agent/strategy.md` - overall strategy and scoping guardrails
3. `agent/crates.md` - file-level crate change specifications
4. `agent/workspace.md` - workspace integration plan and implementation ordering
5. `agent/domains.md` - ownership boundaries, invariants, and domain model
6. `agent/crate-contracts/p2p-commonware.md` - detailed primary crate contract
7. `agent/crate-contracts/whirlpool-node.md` - node wiring contract
8. `agent/flows.md` - validator, bootstrap, and message-routing flows
9. `agent/tests.md` - requirement-to-test traceability with concrete `TST-*` contracts
10. `agent/handoff.md` - implementation order and dependency guide for plan generation
11. `agent/TASK_GEN_READY.md` - task-generation readiness marker
12. `agent/blockers.md` - blocker gate status

## Agent Lane Inventory
- `agent/strategy.md`
- `agent/crates.md`
- `agent/workspace.md`
- `agent/domains.md`
- `agent/blockers.md`
- `agent/requirements.md`
- `agent/crate-contracts/p2p-commonware.md`
- `agent/crate-contracts/whirlpool-node.md`
- `agent/flows.md`
- `agent/tests.md`
- `agent/handoff.md`
- `agent/TASK_GEN_READY.md`
- `agent/run-state.md`
- `agent/exploration/p2p-crates.md`
- `agent/exploration/commonware-vendor.md`
- `agent/exploration/node-architecture.md`

## Review Lane Inventory
- `review/DESIGN.md`
- `review/INDEX.md`

## Traceability Highlights
- `REQ-1` validator seeding: strategy -> crate specs -> `agent/crate-contracts/p2p-commonware.md` -> `agent/flows.md` -> `agent/tests.md`
- `REQ-2` bootstrap peers: workspace -> `agent/crate-contracts/whirlpool-node.md` -> `agent/flows.md` -> `agent/tests.md`
- `REQ-3` channel metadata fix: domains -> `agent/crate-contracts/p2p-commonware.md` -> `agent/flows.md` -> `agent/tests.md`

## Readiness
- Current blocker state: PASS (`agent/blockers.md`)
- Task generation marker: READY (`agent/TASK_GEN_READY.md`)
- Finalization scope: Sub-Intent A only
