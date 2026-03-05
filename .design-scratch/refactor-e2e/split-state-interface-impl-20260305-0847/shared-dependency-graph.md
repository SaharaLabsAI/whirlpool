# Shared dependency graph exploration

## Workspace members
- `crates/consensus`
- `crates/consensus-simplex`
- `crates/p2p`
- `crates/p2p-commonware`
- `crates/whirlpool-node`
- `crates/whirlpool-node-simple`
- `crates/state`
- `crates/app`
- `crates/app-evm`

## Dependency map (in-scope crates)
### `state`
- Path: `crates/state/Cargo.toml` (workspace member).
- Direct dependencies: `revm = "34"`, `sha2 = "0.10"`, `thiserror = "2"`, `alloy-genesis = "1.5"` (no `[features]` table in `Cargo.toml`).
- Exports: `db::{DbAccount, InMemoryStateDb}`, `error::StateError`, `traits::StateDb` from `src/lib.rs`, keeping the concrete types in view for existing consumers.
- Interfaces: `StateDb` trait in `src/traits.rs` defines `new`, `with_genesis`, `state_root`, `commit`, accessor helpers, and requires `revm` types (`BundleState`, `AccountInfo`, `Bytecode`, `Address`, `B256`, `U256`), which preserves the `revm` dependency even after the split.
- Concrete implementation: `src/db.rs` currently implements `DbAccount`, `InMemoryStateDb`, `StateDb for InMemoryStateDb`, and `revm::Database` / `DatabaseRef` (each returning `StateError` from `src/error.rs`). `StateError` implements `revm::database::DBErrorMarker` directly in the interface crate.
- Notes: the current module tree keeps both interface and concrete code together; splitting introduces separate crates but keeps these responsibilities clearly defined.

### `state-memory` [PROPOSED/missing]
- Status: crate absent today; plan documents (`INTENT.md`, `shared-refactor-splits.md`) describe introducing it so interface-only consumers can avoid depending on `InMemoryStateDb` internals.
- Expected dependencies: it will depend on `state` (for `StateDb`, `StateError`/`DBErrorMarker`, shared types), `revm` (for `Database`, `DatabaseRef`, `state` primitives, `BundleState`, `AccountInfo`, `Bytecode`, `Address`, `B256`, `U256`, `Database` traits), and `alloy-genesis` (for `GenesisAccount` used in `with_genesis`). It will also use `std` collections (`HashMap`).
- Responsibilities: host `DbAccount`, `InMemoryStateDb`, and their `revm` trait implementations (currently in `crates/state/src/db.rs`); mirror existing behavior so that `state` stays interface-only.
- Status note: mark as `[PROPOSED]/missing` in the graph and keep mapping of current consumers around the existing `state` crate until the split is materialized.

## Reverse dependencies
- `crates/app-evm`: depends on `state` (`state::traits::StateDb` in `crates/app-evm/src/traits.rs` and `InMemoryStateDb` in `crates/app-evm/src/executor.rs` plus four test files under `crates/app-evm/tests/` that instantiate `state::InMemoryStateDb`). Any split will need to update these references so tests/dev-dependencies pull `state-memory` for the concrete DB.
- `crates/whirlpool-node`: depends on `state` because `src/main.rs` uses `state::InMemoryStateDb` to build `TestStateDb` and `state::StateError` for the `Database`/`StateProvider` impls. This binary explicitly wires the in-memory implementation; after the split it should continue pulling `state-memory` for runtime behavior while `state` continues to provide the trait/error contract.
- Additional consumers: internal documentation and design notes reference both trait and concrete types, but no other workspace crates currently declare a direct dependency via Cargo (search for `state = { path = ... }` only returned `app-evm` and `whirlpool-node`).

## Feature flags
- `state`: no `[features]` section in `crates/state/Cargo.toml`; everything ships with the default feature set of its dependencies (`revm`, `sha2`, `thiserror`, `alloy-genesis`). Because `state` itself exposes no optional features, the interface split will not add cargo-feature variations.
- `state-memory` (proposed): will likely also avoid extra features and rely on the defaults of `state`/`revm`/`alloy-genesis`; ensures consumers such as `app-evm` and `whirlpool-node` can opt into the concrete crate without toggling new flags.
- Workspace-wide: the root `Cargo.toml` uses `resolver = "2"` but defines no shared features; the interface split only adds a crate rather than additional feature flags.

## cfg / conditional compilation summary
- `crates/state/src/db.rs` guards its 300+ line test module with `#[cfg(test)] mod tests { ... }`, ensuring the concrete implementation’s unit tests don’t ship in production artifacts.
- `crates/app-evm/src/executor.rs` and several other workspace crates trigger `#[cfg(test)]` (e.g., `crates/app-evm/src/config.rs`, `crates/consensus/src/lib.rs`, `crates/p2p-commonware/src/lib.rs`), but there are no `cfg` gates around the `state` dependency itself—`state::StateDb` is always available.
- Broader scan (`rg '#\[cfg'` across the workspace) finds the expected widespread use of `#[cfg(test)]` and occasional `#[cfg(any(test, feature = "mock"))]`, but not in ways that would disrupt the interface/concrete split. The new crate can reuse existing conditional testing scaffolding.

## Simplified dependency graph
```
state (interface crate)
  ├─ deps: revm, sha2, thiserror, alloy-genesis
  ├─ exports: StateDb trait, StateError (+ DBErrorMarker)
  └─ consumed by: app-evm (trait + tests), whirlpool-node (StateError/TestStateDb)
state-memory [PROPOSED]
  ├─ deps: state (StateDb/StateError), revm, alloy-genesis, std collections
  └─ supplies: InMemoryStateDb, DbAccount, Database/DatabaseRef ERR types to runtime consumers
app-evm
  └─ depends on state for interfaces/tests, will also depend on state-memory when it needs the concrete DB
whirlpool-node
  └─ depends on state for interfaces and runtime wiring; after refactor it will depend on state-memory for actual implementation
```

## Circular dependency risks
- Avoid letting `state` depend on `state-memory`; if `state` re-exports `InMemoryStateDb` or `Database` impls from the new crate, the interface/implementation split would create a cycle. Keep `state` interface-only so the only dependency edge is `state-memory -> state`.
- The `StateError` type currently implements `revm::database::DBErrorMarker` inside `state`; keep that impl there so `state-memory` depends only on the trait and error without forcing `revm` into a reverse dependency from `state` back to `state-memory`.
- Ensure downstream crates (e.g., `app-evm` tests, `whirlpool-node`) take their concrete `InMemoryStateDb` from `state-memory` rather than via `state` once the split lands, preventing any attempt by `state` to pull in runtime bells from its own consumers.
