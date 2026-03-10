# MANIFEST — mempool-split-interface

## Inputs Consumed
- `crates/mempool/src/lib.rs` — module declarations, re-exports
- `crates/mempool/src/store.rs` — MempoolStore struct + methods
- `crates/mempool/src/persistent.rs` — PersistentTxPool + TxSource impl
- `crates/mempool/src/error.rs` — MempoolError enum
- `crates/mempool/Cargo.toml` — dependencies
- `crates/mempool/tests/integration.rs` — integration tests
- `crates/app/src/traits.rs` — TxSource trait definition
- `crates/state/` — reference split pattern (interface crate)
- `crates/state-memory/` — reference split pattern (impl crate)
- `agent-docs/index.md` — project architecture overview
- `agent-docs/crates/mempool.md` — mempool documentation

## Outputs Produced
- `INTENT.md` — Step 1
- `scratch/run-state.md` — Step 1
- `scratch/MANIFEST.md` — Step 1
- `scratch/STATE_DELTA.md` — Step 1
- `scratch/shared-refactor-splits.md` — Step 1

## Session IDs
(populated during Phase 2)
