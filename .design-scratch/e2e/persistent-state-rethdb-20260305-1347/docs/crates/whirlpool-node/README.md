# whirlpool-node

## Purpose / Overview

`whirlpool-node` is the runtime composition crate that selects and wires the concrete `StateDb` backend into EVM execution and RPC serving. In this iteration, it migrates from `TestStateDb(InMemoryStateDb)` to `RethStateDb` as the default runtime backend.

This document defines **changes only** for persistent MDBX integration.

## Public API Surface (Changed Runtime Contract)

This crate is primarily a binary/wiring crate; externally observable API changes are startup/configuration and backend selection behavior.

### Public config additions

```rust
pub const STATE_DB_PATH: &str = "./data/state";
pub const STATE_DB_CREATE_IF_MISSING: bool = true;
```

```rust
#[derive(Clone, Debug)]
pub struct NodeStateDbConfig {
    pub path: std::path::PathBuf,
    pub db_args: reth_db::DatabaseArguments,
    pub initialize_genesis_on_empty: bool,
}

impl Default for NodeStateDbConfig {
    fn default() -> Self;
}
```

### Startup wiring entrypoints (binary behavior contract)

```rust
fn main();
```

```rust
fn build_state_db(
    cfg: &NodeStateDbConfig,
    genesis: std::collections::HashMap<revm::primitives::Address, state::GenesisAccount>,
) -> Result<std::sync::Arc<std::sync::RwLock<state_reth::RethStateDb>>, NodeStartupError>;
```

```rust
#[derive(Debug, thiserror::Error)]
pub enum NodeStartupError {
    #[error("state db initialization failed: {0}")]
    StateDbInit(String),
    #[error("genesis initialization failed: {0}")]
    GenesisInit(String),
    #[error("invalid state db path: {0}")]
    InvalidPath(String),
    #[error("missing host prerequisites: {0}")]
    MissingPrerequisites(String),
}
```

## Internal Module Structure (Changed Areas)

- `src/config.rs`: add MDBX path + DB argument configuration surface.
- `src/main.rs`:
  - remove `TestStateDb` wrapper usage from runtime path,
  - create/init/open persistent DB,
  - construct `Arc<RwLock<RethStateDb>>`,
  - inject into `EvmApplication` and `EthRpcContext`.
- `src/lib.rs`: re-export config module remains minimal.

## Dependencies

### Internal workspace

- keep: `app`, `app-evm`, `rpc-eth`, `state`
- add: `state-reth`
- optional retention: `state-memory` for tests/fallback only (not default runtime path)

### External/vendored

- `reth-db` (through `state-reth` and/or direct config type usage)
- existing runtime dependencies (`tokio`, `tracing`, `revm`, etc.)

## Initialization Sequence Contract

Runtime startup must execute the following sequence in order:

1. **create_db/open phase**
   - validate `NodeStateDbConfig.path` is usable,
   - call `state_reth::create_db(path, db_args.clone())` when provisioning new path is needed.
2. **init_db phase**
   - call `state_reth::init_db(path, db_args.clone())` to ensure required tables exist.
3. **with_genesis phase**
   - if first startup (empty/uninitialized state), call `RethStateDb::with_genesis(genesis_alloc)` exactly once before serving traffic.
4. Wrap concrete backend in `Arc<RwLock<_>>` and wire into:
   - `EvmApplication::new(...)`
   - `rpc::context::EthRpcContext::new(...)`

This sequence resolves the required wiring contract: `create_db -> init_db -> with_genesis`.

## Swapping `TestStateDb` -> `RethStateDb`

- Remove runtime construction of `TestStateDb::new()` in `main.rs`.
- Replace `Arc<RwLock<TestStateDb>>` with `Arc<RwLock<RethStateDb>>`.
- Keep generic consumer signatures unchanged (`EvmApplication<DB>`, `EthRpcContext<S>`), relying on trait compatibility.
- Preserve shared-state ownership and lock strategy (`Arc<RwLock<_>>`) to minimize integration risk.

## Error Handling Strategy

- Startup failures in DB create/init/genesis are fatal and must abort node start.
- Errors are logged with explicit phase context (`create_db`, `init_db`, `with_genesis`).
- Runtime read/write errors from state backend propagate through EVM/RPC flows via trait/revm error channels.

## Thread Safety / Concurrency Guarantees

- Shared state remains `Arc<RwLock<...>>` across EVM and RPC tasks.
- `RethStateDb` clone behavior is cheap (`Arc<DatabaseEnv>` sharing) and safe for multi-component injection.
- Lock-hold time should remain short by relying on backend per-method transaction acquisition.

## Constructor / Builder Patterns

- Node-level DB config is explicit through `NodeStateDbConfig`.
- Backend construction is encapsulated in one startup helper (`build_state_db`) to keep `main()` deterministic and testable.

## Key Invariants

- Node must not start serving RPC/consensus until state backend is initialized successfully.
- Genesis initialization is idempotent at process level and guarded by first-run detection.
- Runtime backend for production path is persistent (`state-reth`), not in-memory.
- State object handed to EVM and RPC must be the same shared instance.

## Graceful Shutdown Considerations

- No long-lived MDBX transaction should outlive request/method scope.
- On shutdown signal, stop accepting new work, let in-flight operations complete, then drop state holder.
- Dropping `Arc<RwLock<RethStateDb>>` closes environment handles naturally; no forced flush API is required beyond committed tx durability.

## Blocker Resolution Notes

- **BLK-001 consumed:** crate assumes fallible `StateDb` contract and propagates backend errors.
- **BLK-002 consumed indirectly:** node treats `state_root` semantics as backend-defined and does not reinterpret root values.
- **BLK-003 resolved:** startup contract now explicitly requires host prerequisites and defines fatal failure policy when missing.
