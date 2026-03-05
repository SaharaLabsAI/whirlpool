# Manifest

## Inputs consumed
- `Cargo.toml` — workspace crate membership and architectural scope.
- `crates/whirlpool-node/src/main.rs` — runtime wiring, lifecycle insertion points, shared handles.
- `crates/whirlpool-node/src/config.rs` — node config defaults.
- `crates/whirlpool-node/Cargo.toml` — existing dependency surface.
- `crates/app/src/lib.rs` — exported app API surface.
- `crates/app/src/traits.rs` — interface traits (`Application`, `TxSource`).
- `crates/app/src/tx_source.rs` — in-memory tx pool behavior and concurrency guarantees.
- `crates/app/src/types.rs` — block/result fields relevant to receipt/root semantics.
- `crates/app-evm/src/config.rs` — chain-id constant and chain spec behavior.
- `crates/app-evm/src/executor.rs` — proposal/verification execution paths.
- `crates/app-evm/src/traits.rs` — state provider bridge.
- `crates/state/src/traits.rs` — state interface contract.
- `crates/state-memory/src/db.rs` — concrete in-memory account/nonce/balance access semantics.
- `crates/consensus-simplex/src/sink.rs` — finalized height sink behavior.
- `llmdocs/index.md`, `llmdocs/crates/app.md`, `llmdocs/crates/whirlpool-node.md`, `llmdocs/architecture/simplex-adapter.md`, `llmdocs/architecture/consensus-traits.md` — architecture and boundary references.
- `vendor/reth/examples/node-custom-rpc/src/main.rs`, `vendor/reth/examples/rpc-db/src/myrpc_ext.rs`, `vendor/reth/Cargo.toml` — jsonrpsee style and version references.
- Delegated explore session `ses_3433cbef1fferPlNb6utxNk3bV` — node insertion point corroboration.
- Delegated librarian session `ses_3433cbed5ffeo4oQhq0696q3c4` — alloy/reth method contract expectations.

## Outputs produced
- Scratch context and digests:
  - `scratch/run-state.md`
  - `scratch/shared-baseline.md`
  - `scratch/shared-crate-index.md`
  - `scratch/shared-domain-map.md`
  - `scratch/shared-wiring-skeleton.md`
  - `scratch/shared-flows-index.md`
  - `scratch/shared-librarian.md`
  - `scratch/shared-vendor-patterns.md`
  - `scratch/shared-prior-constraints.md`
  - `scratch/shared-intent-splits.md`
  - `scratch/testid-registry.md`
  - `scratch/STATE_DELTA.md`
  - `scratch/BUILD_DIGEST.md`
  - `scratch/final-self-check.md`
  - `scratch/finalization-notes.md`
  - `scratch/digests/*.md`
  - `scratch/inputs/*.md`
  - `scratch/*-context.md`
- Design docs:
  - `docs/INTENT.md`
  - `docs/CRATES.md`
  - `docs/WORKSPACE.md`
  - `docs/STRATEGY.md`
  - `docs/DOMAINS.md`
  - `docs/FLOWS.md`
  - `docs/TESTS.md`
  - `docs/BLOCKERS.md`
  - `docs/INDEX.md`
  - `docs/SUMMARY.md`
  - `docs/app/README.md`
  - `docs/whirlpool-node/README.md`

## Session IDs
- phase_2_explore_session: `ses_3433cbef1fferPlNb6utxNk3bV`
- phase_2_librarian_session: `ses_3433cbed5ffeo4oQhq0696q3c4`
- phase_3_session: main orchestration session (current)
- phase_4_session: main orchestration session (current)
