# Skill Digest

## Grounded
- **Workspace**: 11 crates, 3-layer consensus arch (traits → simplex adapter → node) — Grounded (Cargo.toml)
- **Current block storage**: Ephemeral `BlockStore` (Arc<RwLock<HashMap<Digest,B>>>) in consensus-simplex, dropped after finalization — Grounded (consensus-simplex/src/lib.rs)
- **StateDb block hashes**: `insert_block_hash/get_block_hash` stores number→hash mapping only (not full blocks) — Grounded (state/src/traits.rs)
- **state-reth exists**: MDBX-backed StateDb for accounts/storage/code, includes CanonicalHeaders table — Grounded (state-reth/src/db.rs)
- **Finalization**: FinalizationSink only updates Arc<AtomicU64> height counter, no persistence — Grounded (consensus-simplex/src/sink.rs)
- **Prior e2e**: persistent-state-rethdb completed (10/10 tasks), established MDBX patterns — Grounded (e2e-state.md)

## [PROPOSED]
- Extend state-reth or create new block storage module using reth-db MDBX tables
- Hook finalization to persist full blocks
- Expose via eth_getBlock* in rpc-eth crate

## Unknowns
- Which reth-db tables are available for block headers/bodies/transactions/receipts
- Current rpc-eth implementation scope (what endpoints exist today)
- Block type generics — how to store arbitrary Block types durably
