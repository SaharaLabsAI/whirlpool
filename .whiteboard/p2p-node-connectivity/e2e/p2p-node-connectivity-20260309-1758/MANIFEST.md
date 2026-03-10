# MANIFEST

## Inputs
- User intent: P2P node connectivity for whirlpool-node
- Codebase exploration: crates/p2p, crates/p2p-commonware, crates/whirlpool-node, crates/app, crates/consensus-simplex, vendor/commonware
- agent-docs: index.md, crates/p2p-commonware.md, overview/project-overview.md
- Intake constraints: module depth, all focus crates, grounded-facts-only execution

## Outputs
- `e2e-state.md` — initial state (phase: align)
- `SKILL_DIGEST.md` — grounded facts from exploration
- `STATE_DELTA.md` — initial delta + intake updates
- `MANIFEST.md` — this file
- `scratch/agent/requirements.md` — normalized REQ-1..REQ-8 and intake framing
- `scratch/agent/shared-intent-splits.md` — split plan due to breadth threshold exceedance
- `scratch/agent/run-state.md` — intake run-state (`phase=align`, `step=intake`)
