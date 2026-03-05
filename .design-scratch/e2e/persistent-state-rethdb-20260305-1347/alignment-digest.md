# Alignment Digest

## Intent
Add persistent state backed by reth-db (MDBX). New crate `state-reth` implementing StateDb trait. Wire into whirlpool-node replacing TestStateDb.

## Scope
- Primary: `state-reth` (new crate)
- Modified: `whirlpool-node` (replace TestStateDb with persistent DB)
- Touched: `state` (minor trait adjustments for fallibility)
- Reference: `state-memory` (behavioral baseline)
- Unmodified: `app-evm`, `rpc-eth` (generic consumers)

## Approach
Raw reth-db tables (not reth-provider). Tables: PlainAccountState, PlainStorageState (dupsort), Bytecodes, block hashes. Init via create_db/init_db. MDBX transactions for reads/writes.

## Risk Decisions (User-Approved)
1. State root: adopt reth trie semantics (StateRoot::overlay_root)
2. Infallibility: acknowledged — StateDb trait methods are infallible but MDBX I/O is fallible; must design error strategy
3. Concurrency: Arc<RwLock<>> vs MDBX transaction model; must validate compatibility

## Key Type Requirements
- Implement: StateDb, revm::Database, revm::DatabaseRef, Clone, Send, Sync, Debug
- StateProvider blanket impl (app-evm/src/traits.rs) bridges automatically

## Dep Posture
- alloy-primitives 1.5 and revm 34 compatible across workspace + vendored reth
- Sys deps: C compiler + clang for reth-mdbx-sys (in Nix dev shell)
- No workspace patches needed
