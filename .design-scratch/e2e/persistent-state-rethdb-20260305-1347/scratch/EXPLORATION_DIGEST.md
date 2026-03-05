# EXPLORATION DIGEST

## 1. Architecture Summary
- Crate topology is stable and already backend-agnostic: `state` trait boundary -> `state-memory` reference impl -> `app-evm` generic executor -> `rpc-eth` generic API/context -> `whirlpool-node` runtime wiring; proposed `state-reth` can slot in as an additional backend crate without API redesign. [source: `docs/EXPLORATION.md` Architecture Findings; `docs/SHARED_CONTEXT.md` Workspace/Architecture Summary]
- State flow hinges on `StateProvider` blanket impl over `T: StateDb`, so consumers stay abstracted from concrete storage. [source: `docs/EXPLORATION.md` Generic state flow; `docs/SHARED_CONTEXT.md` Architecture Summary]
- `EvmApplication<DB>` requires `DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + Debug`; RPC requires `S: StateDb + Send + Sync + 'static`; node shares DB via `Arc<RwLock<...>>`, implying thread-safe shared ownership for any persistent backend wrapper. [source: `docs/EXPLORATION.md` lines on bounds/wiring; `docs/SHARED_CONTEXT.md` Consumer bounds]

## 2. Type Surface
- Canonical `StateDb` contract to preserve is the infallible 10-method surface (`new`, `with_genesis`, `state_root`, `commit`, account/code/storage/block-hash getters, and insertors). [source: `docs/SHARED_CONTEXT.md` Type System Summary; `docs/EXPLORATION.md` Types addendum]
- Boundary types: `GenesisAccount` (`alloy_genesis`), plus shared execution/state primitives `BundleState`, `Address`, `B256`, `U256`, `AccountInfo`, `Bytecode`; `DbAccount` remains implementation-local and not trait-facing. [source: `docs/EXPLORATION.md` Type findings; `docs/SHARED_CONTEXT.md` Type disambiguation]
- New impl must mirror `revm::DatabaseRef` + `revm::Database` method shapes and use an error type satisfying `DBErrorMarker` for revm compatibility. [source: `docs/EXPLORATION.md` Types addendum; `docs/SHARED_CONTEXT.md` state-memory behavioral reference]

## 3. Dependency Posture
- No blocking version skew found for core shared deps: `alloy-primitives` aligns at `1.5.0`; `revm` aligns on major line `34`/`34.0.0`. [source: `docs/EXPLORATION.md` Dependency gap addendum; `docs/SHARED_CONTEXT.md` Gap-focused checks]
- `reth-db` default feature enables MDBX (`mdbx`) and pulls a moderate storage surface (`reth-libmdbx`, `reth-db-api`, fs/errors), while `reth-provider` carries a much larger transitive/provider stack. [source: `docs/EXPLORATION.md` reth storage stack shape; `docs/SHARED_CONTEXT.md` reth MDBX dependency shape]
- Sys/native requirements for MDBX path are C compiler + clang/libclang due to `cc` + `bindgen` in `reth-mdbx-sys`; no CMake declared. [source: `docs/EXPLORATION.md` dependency addendum; `docs/SHARED_CONTEXT.md` gap checks]

## 4. reth-db Integration Path
- Recommended approach: implement `state-reth` directly over raw `reth-db` tables (not `reth-provider`) to match current `StateDb` contract with lower dependency/abstraction overhead. [source: decision notes in `docs/EXPLORATION.md` and `docs/SHARED_CONTEXT.md`]
- Table mapping for parity: `PlainAccountState` (accounts), `PlainStorageState` dupsort (storage), `Bytecodes` (code), plus block-hash/trie-relevant tables for root workflow. [source: `docs/EXPLORATION.md` table mappings; `docs/SHARED_CONTEXT.md` reth-db API patterns]
- Key APIs/patterns: init via `create_db`/`init_db`; reads via `get_by_encoded_key` + dupsort cursors; writes via `put` and `cursor_dup_write().upsert(...)`; durability through `tx.commit()`. [source: `docs/EXPLORATION.md` API pattern findings; `docs/SHARED_CONTEXT.md` reth-db API pattern summary]

## 5. Risks & Unknowns
- State root parity risk: current in-memory root is deterministic hash-over-sorted material, whereas reth path uses trie-based `StateRoot::overlay_root`; exact parity/expected semantics must be pinned before correctness claims. [source: `docs/EXPLORATION.md` state_root notes + trie root pattern; `docs/SHARED_CONTEXT.md` state_root pattern]
- Infallibility mismatch risk: `StateDb` trait is infallible, but MDBX/IO ops are fallible; adapter policy for mapping storage failures into current trait surface (and revm error path) remains an open design constraint. [source: `docs/SHARED_CONTEXT.md` infallible trait + revm error surface; type/dependency exploration categories]
- MDBX concurrency/threading risk: node architecture expects `Arc<RwLock<S>>` shared across execution and RPC; transaction lifetime and thread-safety constraints in MDBX-backed access patterns need explicit validation under that sharing model. [source: `docs/EXPLORATION.md` node wiring + tx patterns; `docs/SHARED_CONTEXT.md` consumer bounds/wiring]
