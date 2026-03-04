# Manifest — Refactor Intake

## Inputs Consumed
- User intake request (phase=intake) for interface/implementation split in app, consensus, p2p, state.
- Workspace files:
  - crates/app/src/traits.rs
  - crates/app/src/lib.rs
  - crates/consensus/src/{app.rs,block.rs,event.rs,engine.rs,lib.rs}
  - crates/p2p/src/traits.rs
  - crates/p2p/src/lib.rs
  - crates/state/src/{db.rs,lib.rs}
- Skill protocol references:
  - rust-whiteboard-refactor/phases/01-intake.md
  - rust-whiteboard-refactor/shared/conventions.md
  - rust-whiteboard-refactor/shared/state-protocol.md

## Outputs Produced
- docs/refactor/split-interface-implementation/INTENT.md
- .design-scratch/refactor-e2e/split-interface-impl-20260304-1025/run-state.md
- .design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-refactor-splits.md
- .design-scratch/refactor-e2e/split-interface-impl-20260304-1025/STATE_DELTA.md
