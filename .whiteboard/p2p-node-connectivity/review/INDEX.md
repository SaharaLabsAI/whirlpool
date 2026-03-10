# Design Artifact Index

## Reading Order
1. `review/DESIGN.md` - review-lane verdict and source-grounded finalization summary for Sub-Intent C
2. `agent/strategy.md` - scope guardrails and high-level relay-activation design intent
3. `agent/crate-contracts/consensus-simplex.md` - primary relay activation contract for mailbox and engine wiring
4. `agent/crate-contracts/p2p.md` - payload channel constant contract
5. `agent/crate-contracts/p2p-commonware.md` - payload channel registration and per-channel bundle contract
6. `agent/crate-contracts/whirlpool-node.md` - node compatibility boundary
7. `agent/flows.md` - proposal broadcast, payload receive, verification cache, and channel alignment flows
8. `agent/domains-wiring.md` - cross-crate ownership and wiring for relay payload movement
9. `agent/tests.md` - requirement-to-test mapping with relay, transport, and compatibility coverage
10. `agent/handoff.md` - implementation order, dependencies, and verification steps
11. `agent/shared-intent-splits.md` - canonical split ledger showing Sub-Intent C scope
12. `agent/requirements.md` - original requirement ledger for full intent set

## Agent Lane Inventory
- `agent/strategy.md`
- `agent/crate-contracts/consensus-simplex.md`
- `agent/crate-contracts/p2p.md`
- `agent/crate-contracts/p2p-commonware.md`
- `agent/crate-contracts/whirlpool-node.md`
- `agent/flows.md`
- `agent/domains-wiring.md`
- `agent/tests.md`
- `agent/handoff.md`
- `agent/shared-intent-splits.md`
- `agent/requirements.md`

## Review Lane Inventory
- `review/DESIGN.md`
- `review/INDEX.md`

## Traceability Highlights
- `REQ-6` relay activation:
  - `agent/strategy.md`
  - `agent/crate-contracts/consensus-simplex.md`
  - `agent/crate-contracts/p2p-commonware.md`
  - `agent/flows.md`
  - `agent/domains-wiring.md`
  - `agent/tests.md`
  - `agent/handoff.md`
- `REQ-7` channel alignment:
  - `agent/crate-contracts/p2p.md`
  - `agent/crate-contracts/p2p-commonware.md`
  - `agent/flows.md`
  - `agent/domains-wiring.md`
  - `agent/tests.md`
- `REQ-8` compatibility preservation:
  - `agent/crate-contracts/whirlpool-node.md`
  - `agent/flows.md`
  - `agent/domains-wiring.md`
  - `agent/tests.md`
  - `agent/handoff.md`
  - `review/DESIGN.md`

## Scope Notes
- Finalization scope is Sub-Intent C only: `consensus-relay-activation`.
- Primary design ownership sits in `crates/consensus-simplex`.
- `crates/p2p` and `crates/p2p-commonware` change only additively to expose a dedicated payload channel.
- `crates/whirlpool-node` is preserved as a compatibility consumer rather than a relay-logic owner.

## Readiness
- Review verdict: PASS (`review/DESIGN.md`)
- Design status: execution-ready for Sub-Intent C
- Vendor boundary: preserved
