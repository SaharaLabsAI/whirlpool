# Skill Digest

## Grounded
- `Grounded`: Workspace root resolved to `/home/dev/sahara/web3/agent/playground/whirlpool` from `Cargo.toml` with existing members `app`, `app-evm`, `rpc-eth`, `whirlpool-node`, `state`, `state-memory`, `mempool`, and `mempool-mdbx`; `app-mem` and `rpc-mem` are not present yet. Source: `Cargo.toml`.
- `Grounded`: Current transaction ingress is generic opaque bytes via `app::traits::TxSource::{push,pending}` and `mempool-mdbx::PersistentTxPool`; application logic is EVM-only in `crates/app-evm/src/executor.rs`. Sources: `crates/app/src/traits.rs`, `crates/mempool-mdbx/src/persistent.rs`, `crates/app-evm/src/executor.rs`.
- `Grounded`: Node wiring currently starts only `rpc-eth`, constructs `EvmApplication`, and persists finalized blocks through `PersistingFinalizationSink`. Sources: `crates/whirlpool-node/src/node.rs`, `crates/whirlpool-node/src/persisting_sink.rs`.
- `Grounded`: Prior approved design proposes non-EVM personality transactions, a new `rpc-mem` crate, an `app-mem` separation boundary, generic shared mempool ingress, and finalization-only writes into a dedicated in-memory personality store. Source: `.whiteboard/personality-markdown-tx/review/DESIGN.md`.
- `Grounded`: Design artifacts now fix crate ownership, mixed-transaction classification boundaries, finalization-only personality visibility, and dual-server node composition under `.whiteboard/add-mem-tx-support/agent/` and `.whiteboard/add-mem-tx-support/review/`.
- `Grounded`: Proof artifacts were completed for sub-intent `main` with 8 acceptance criteria, 6 local invariants, 2 cross-invariants, and 8 QA scenarios, with no `[UNGROUNDED]` claims. Sources: `main/proof.md`, `main/proven-ac.md`, `xinv-index.md`, `prove-phase-digest.md`.
- `Grounded`: Plan artifacts were completed and audited under `.sisyphus/plans/add-mem-tx-support/`, producing 6 execution tasks with `req_coverage: 9/9`, `tst_coverage: 7/7`, and handoff state `ready_for_start_work: true`. Sources: `.sisyphus/plans/add-mem-tx-support.md`, `.sisyphus/plans/add-mem-tx-support/INDEX.md`, `main/plan-audit/coverage.md`, `plan-phase-digest.md`, `e2e-state.md`.

## Unknowns
- Exact signed payload codec and whether it should live entirely in `app-mem` or share a lower-level helper type.
- Whether replay protection by `(signer, nonce)` is required in v1 or remains only a documented future hook.
- Whether `rpc-mem` stays submit-only in phase 1 or grows read methods in a later scoped follow-up.
- Whether execution should now proceed directly from `.sisyphus/plans/add-mem-tx-support.md` or wait for an explicit user request to start implementation.
