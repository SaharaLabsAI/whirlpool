# State Commitment

## Trigger

Canonical timing rule: `propose()` is speculative (no canonical commit). [PROPOSED] Canonical commit happens only after finalization through a finalize->commit seam, which is currently a `BLOCKER`.

## Stages

1. [PROPOSED] Execution path produces a commit-ready state artifact (`BundleState` or equivalent).
2. State crate exposes `InMemoryStateDb::commit(&BundleState)` for applying effects.
3. State crate exposes `state_root()` for deriving post-commit root.
4. Current proposal/verification traces do not show commit-ready artifact handoff into canonical commit.
5. Finalization path currently updates finalized height via sink but does not groundedly call commit.

## Outputs

- Grounded API output: state DB can apply `commit` and derive `state_root`.
- `UNKNOWN` end-to-end output: canonical block effects persisted exactly once on finalization.

## Stage ownership

- Commit API and root derivation: `state`.
- Upstream commit-ready artifact production: [PROPOSED] `app-evm` full execution path.
- Finalization-to-commit orchestration: `UNKNOWN` cross-crate seam (`consensus`/`consensus-simplex`/`whirlpool-node` integration).

## Handoff contracts

- Grounded: `state::InMemoryStateDb::commit(&BundleState)` and `state_root()` exist.
- Grounded: consensus app trait currently exposes `genesis`, `propose`, `verify` only.
- Clarification: `ExecutionResult` is summary telemetry and is not sufficient for canonical state commit.
- [PROPOSED] canonical commit requires preserving a commit-ready artifact (`BundleState` or equivalent) from proposal/execution context until finalization.
- `UNKNOWN`: where commit-ready artifact is stored/owned across propose->finalize seam.
- `BLOCKER`: no grounded finalize callback contract into app/state commit seam.

## Error propagation

- `UNKNOWN`: commit-stage errors in canonical finalization flow are not evidenced end-to-end.
- Current evidence confirms API presence, not integration-time error plumbing.

## Pseudo-code sketch

```rust
fn on_finalized_block(block) -> Result<(), CommitError> {
    // [PROPOSED] recover commit-ready artifact bound to finalized block
    let bundle = lookup_bundle_for_block(block.hash())?;
    state_db.commit(&bundle)?;
    let root = state_db.state_root()?;
    ensure_root_matches_block(root, block.state_root)?;
    Ok(())
}
```

## Open questions / TODOs

- `BLOCKER`: define and wire finalize -> commit callback seam.
- `UNKNOWN`: exact owner and storage location for commit-ready artifact before finalization.
- `UNKNOWN`: reorg/fault semantics for commit rollback or compensation.
- [PROPOSED] add atomicity checks and idempotency guards for exactly-once commit behavior.
