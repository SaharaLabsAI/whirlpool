# CRATES

## Scope and Blocker Context

This crate allocation covers the affected crates for persistent MDBX-backed state integration:

- `state-reth` (new crate)
- `state` (interface contract updates)
- `whirlpool-node` (runtime wiring replacement)

Hard blockers from `BLOCKERS.md` that constrain crate design:

- **BLK-001 (hard):** `StateDb` fallibility contract must be finalized across crates.
- **BLK-002 (hard):** trie root input/normalization contract is not fully pinned.
- **BLK-003 (hard):** MDBX host prerequisites contract is not yet specified.

Design choices below assume the strategy direction is accepted and call out where blockers prevent full finalization.

---

## Crate: `state-reth` (new)

### Purpose

Provide a persistent `StateDb` implementation backed by `reth-db` (MDBX), with revm-compatible database traits for EVM execution and RPC-backed reads.

### Dependencies

#### Workspace dependencies

- `state` (trait authority for `StateDb`)

#### External / vendored dependencies

- `reth-db` (MDBX environment + transactions + table APIs)
- `reth-db-api` (table traits/models)
- `reth-storage-errors` (database error surface)
- `reth-trie` (state root computation)
- `reth-codecs` (compact codecs used by reth storage stack)
- `revm` (Database / DatabaseRef trait integration)
- `alloy-primitives` (shared primitive types)
- `thiserror` (error type derivations)

### Module structure (mod tree)

```text
state-reth/
  src/
    lib.rs
    db.rs
    tables.rs
    codec.rs
    trie.rs
    init.rs
    error.rs
```

- `lib.rs`: crate root, public exports
- `db.rs`: `RethStateDb` type + `StateDb` + revm trait impls
- `tables.rs`: typed table access helpers (accounts/storage/code/block-hash)
- `codec.rs`: revm <-> reth type conversion
- `trie.rs`: state-root derivation via `reth_trie::StateRoot`
- `init.rs`: db creation/init + genesis bootstrapping helpers
- `error.rs`: unified `RethStateError`

### Public API surface (key `pub` items)

- `pub struct RethStateDb { ... }`
- `impl state::StateDb for RethStateDb`
- `impl revm::DatabaseRef for RethStateDb`
- `impl revm::Database for RethStateDb`
- `pub enum RethStateError`
- `pub fn create_db(...) -> Result<..., RethStateError>`
- `pub fn init_db(...) -> Result<..., RethStateError>`
- Optional convenience constructor:
  - `pub fn open(path: impl AsRef<std::path::Path>) -> Result<RethStateDb, RethStateError>`

### Changes required

- **New:** crate `crates/state-reth/` with `Cargo.toml` and module skeleton above.
- **New:** `StateDb` method mappings to `PlainAccountState`, `PlainStorageState`, `Bytecodes`.
- **New:** per-method transaction acquisition policy (short-lived read/write tx, commit on write path).
- **New:** trie-backed `state_root()` implementation.
- **New:** error taxonomy and conversion surface to revm `DBError`.
- **Deferred by blocker:** exact `get_block_hash`/`insert_block_hash` table selection (`CanonicalHeaders` vs `HeaderNumbers`) remains soft-blocked (BLK-101).
- **Constrained by hard blockers:**
  - BLK-002 prevents finalizing exact hashed-state contract in `trie.rs`.
  - BLK-003 prevents finalizing deployment/ops contract around MDBX prerequisites.

---

## Crate: `state` (modified)

### Purpose

Remain the backend-agnostic interface crate defining the `StateDb` contract used by EVM app and RPC layers.

### Dependencies

#### Workspace dependencies

- None required for trait boundary itself.

#### External dependencies

- Existing: `revm`, `alloy-genesis`, `thiserror` (per current crate usage)

### Module structure (mod tree)

```text
state/
  src/
    lib.rs
    traits.rs
    error.rs
```

### Public API surface (key `pub` items)

- `pub trait StateDb`
- `pub type GenesisAccount = alloy_genesis::GenesisAccount`
- exported error marker/types used by revm integration (from `error.rs`)

### Changes required

- **Modified (contract change):** make `StateDb` fallible.
  - Add associated error type:
    - `type Error: std::error::Error + Send + Sync + 'static;`
  - Change all trait methods to return `Result<..., Self::Error>`.
- **Modified:** preserve method semantics and names; only add fallibility to support MDBX/reth I/O.
- **No removals:** retain existing conceptual surface (`new`, `with_genesis`, `state_root`, `commit`, getters, inserts).
- **Cross-crate migration impact:** `state-memory`, `app-evm`, and `rpc-eth` generic paths must compile with new `Result`-based contract.

### Blocker impact

- **BLK-001 (hard):** this crate cannot be finalized until canonical fallible signature and trait bounds are explicitly approved as shared contract.

---

## Crate: `whirlpool-node` (modified)

### Purpose

Compose runtime components and wire a concrete `StateDb` backend into EVM execution and RPC serving.

### Dependencies

#### Workspace dependencies

- Existing: `app`, `app-evm`, `rpc-eth`, `state`
- **Change:** replace active state backend dependency from `state-memory` path usage to `state-reth`

#### External dependencies

- Existing runtime dependencies (`tokio`, `tracing`, `revm`, etc.)
- No direct mandatory reth-table dependency expected in this crate (prefer encapsulation in `state-reth`)

### Module structure (mod tree)

```text
whirlpool-node/
  src/
    main.rs
    (existing runtime/config modules unchanged unless needed for DB path config)
```

### Public API surface (key `pub` items)

- Primarily binary wiring crate; key externally visible surface is startup behavior and configuration handling.
- Key wiring constructs affected:
  - shared state container `Arc<RwLock<S>>`
  - `EvmApplication<DB>` state provider wiring
  - `EthRpcContext<S>` construction

### Changes required

- **Modified:** replace `TestStateDb` + `InMemoryStateDb` wiring with `RethStateDb`.
- **Modified:** initialize/open MDBX database at startup (using configured path).
- **Modified:** run genesis initialization on first startup path.
- **Modified:** ensure shared state object remains `Arc<RwLock<...>>` and satisfies `Send + Sync + 'static` bounds for EVM and RPC paths.
- **Potentially removed/simplified:** `TestStateDb` wrapper if it becomes redundant once `RethStateDb` provides both `StateDb` and revm traits directly.
- **Operational addition:** startup validation/reporting for MDBX prerequisites and directory readiness.

### Blocker impact

- **BLK-003 (hard):** startup behavior and failure policy are incomplete until host prerequisite contract is specified.

---

## Cross-Crate Contract Notes

- `state-reth` is the persistent implementation; `state` remains interface authority.
- `whirlpool-node` only composes/wires and should not absorb reth table logic.
- Consumer crates (`app-evm`, `rpc-eth`) stay generic, but must accept `StateDb` fallibility migration once BLK-001 is resolved.
- Root semantics switch from in-memory deterministic hash to trie-root semantics for persistent backend paths; this is an intentional behavioral divergence requiring explicit validation contract (BLK-002).
