# Manifest

## Inputs
- User intent: add mem tx support with prior design reference and `app-mem` crate separation.
- Prior design: `.whiteboard/personality-markdown-tx/review/DESIGN.md`.
- Workspace topology: `Cargo.toml`.
- Grounding files: `crates/app/src/traits.rs`, `crates/app/src/types.rs`, `crates/app/src/tx_source.rs`, `crates/app-evm/src/executor.rs`, `crates/whirlpool-node/src/node.rs`, `crates/whirlpool-node/src/persisting_sink.rs`, `crates/rpc-eth/src/lib.rs`, `crates/rpc-eth/src/server.rs`, `crates/rpc-eth/src/pool.rs`, `crates/mempool-mdbx/src/persistent.rs`, `crates/state/src/block_storage.rs`, `crates/state/src/lib.rs`, `crates/state-memory/src/lib.rs`, `docs/chain/crates/types.md`.
- Approved alignment, design, proof, and plan state under `.whiteboard/add-mem-tx-support/e2e/add-mem-tx-support-20260322-1325/` and `.whiteboard/add-mem-tx-support/`.

## Outputs
- `e2e-state.md`
- `SKILL_DIGEST.md`
- `STATE_DELTA.md`
- `MANIFEST.md`
- `review/alignment-digest.md`
- `review/design-phase-digest.md`
- `prove-phase-digest.md`
- `plan-phase-digest.md`
- `xinv-index.md`
- `main/proof.md`
- `main/proof-digest.md`
- `main/proof-challenges.md`
- `main/proven-ac.md`
- `main/plan-audit/coverage.md`
- `agent/requirements.md`
- `agent/tests.md`
- `agent/testid-registry.md`
- `agent/risk-assessment.md`
- `agent/strategy.md`
- `agent/crates.md`
- `agent/workspace.md`
- `agent/domains.md`
- `agent/blockers.md`
- `agent/flows.md`
- `agent/crate-contracts/app-mem.md`
- `agent/crate-contracts/rpc-mem.md`
- `agent/crate-contracts/whirlpool-node.md`
- `agent/handoff.md`
- `agent/TASK_GEN_READY.md`
- `review/DESIGN.md`
- `review/INDEX.md`
- `.whiteboard/add-mem-tx-support/BUILD_DIGEST.md`
- `.sisyphus/plans/add-mem-tx-support.md`
- `.sisyphus/plans/add-mem-tx-support/INDEX.md`
- `.sisyphus/plans/add-mem-tx-support/tasks/01-app-mem-crate-and-tests.md`
- `.sisyphus/plans/add-mem-tx-support/tasks/02-rpc-mem-ingress.md`
- `.sisyphus/plans/add-mem-tx-support/tasks/03-mixed-proposal-verification.md`
- `.sisyphus/plans/add-mem-tx-support/tasks/04-finalization-store.md`
- `.sisyphus/plans/add-mem-tx-support/tasks/05-workspace-and-node-wiring.md`
- `.sisyphus/plans/add-mem-tx-support/tasks/06-integration-and-audit.md`
