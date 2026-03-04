# Phase 02 Step 2 — Structural Dependency Graph

## Scope
- Workspace root: `/home/dev/sahara/web3/agent/playground/whirlpool`.
- Focus crates: `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, `app-evm` (all members of the workspace listed in `Cargo.toml`).
- Depth: `structural` level (traits/interfaces vs. implementations).
- Intake rule met: every workspace `Cargo.toml` was read to build the dependency map, and only the focus crate sources were scanned for `#[cfg(...)]` gating.

## Direct / Reverse / Dev Dependencies & Features

### `app`
- **Direct deps**: `consensus` (path dependency), `commonware-consensus`, `commonware-codec`, `commonware-cryptography` (all vendor path deps), `sha2` 0.10, `bytes` 1, `thiserror` 2.
- **Reverse deps**: `app-evm`, `whirlpool-node`.
- **Dev deps**: `futures` 0.3.
- **Features**: none.
- **cfg usage**: `src/traits.rs:77`, `src/adapter.rs:57`, `src/types.rs:110/189/222`, `src/error.rs:11` all `#[cfg(test)]` blocks around test helpers.
- **Circular risk**: None; `app` stays at the foundation so maintain clean trait-only exports.

### `consensus`
- **Direct deps**: `thiserror` 2, `tokio` 1 (default-features=false, features `rt`, `sync`).
- **Reverse deps**: `app`, `app-evm`, `consensus-simplex`, `whirlpool-node`, `whirlpool-node-simple`.
- **Dev deps**: `tokio` 1 with features `rt`, `macros`.
- **Features**: `mock` (exposes fake engine implementations via `#[cfg(any(test, feature = "mock"))]`).
- **cfg usage**: `src/lib.rs:20` (`#[cfg(any(test, feature = "mock"))]`), `src/lib.rs:23` (`#[cfg(test)]`).
- **Circular risk**: Keep trait definitions in `app.rs`/`block.rs`/`engine.rs` as interface-only so downstream crates never need to feed behavior back.

### `p2p`
- **Direct deps**: `bytes` 1.5, `serde` 1.0 (derive), `thiserror` 1.0, `tokio` 1.42 (features `sync`, `macros`, `rt`).
- **Reverse deps**: `consensus-simplex` (mock-enabled), `p2p-commonware`.
- **Dev deps**: none.
- **Features**: `mock` (activates the cfg-gated `mock` module).
- **cfg usage**: `src/lib.rs:46` (`#[cfg(any(test, feature = "mock"))]`), `src/mock.rs:101` (`#[cfg(test)]`).
- **Circular risk**: None; definitions remain upstream of Commonware adapters.

### `state`
- **Direct deps**: `revm` 34, `sha2` 0.10, `thiserror` 2, `alloy-genesis` 1.5.
- **Reverse deps**: `app-evm`, `whirlpool-node`.
- **Dev deps**: none.
- **Features**: none.
- **cfg usage**: `src/db.rs:188` (`#[cfg(test)]`).
- **Circular risk**: Already a leaf crate; interface extraction merely needs a traits module for `StateDb` without back references.

### `consensus-simplex`
- **Direct deps**: `consensus`, `p2p` (features `["mock"]`), `p2p-commonware`, `commonware-consensus`, `commonware-broadcast`, `commonware-cryptography`, `commonware-p2p`, `commonware-runtime`, `commonware-storage`, `commonware-codec`, `commonware-utils`, `commonware-parallel`, `tokio` 1 (features `rt`, `sync`, `macros`), `tracing` 0.1, `futures` 0.3, `rand` 0.8, `rand_core` 0.6.
- **Reverse deps**: `whirlpool-node`, `whirlpool-node-simple`.
- **Dev deps**: `bytes` 1.
- **Features**: none.
- **cfg usage**: `src/lib.rs:22`, `src/engine.rs:195`, `src/mailbox.rs:187`, `src/sink.rs:65`, `src/config.rs:58` (all `#[cfg(test)]`).
- **Circular risk**: None; it sits above `consensus`/`p2p` but below node crates, so trait movements must keep dependencies acyclic.

### `p2p-commonware`
- **Direct deps**: `p2p`, `commonware-p2p`, `commonware-cryptography`, `commonware-runtime`, `commonware-utils`, `commonware-stream`, `thiserror` 2, `bytes` 1, `tracing` 0.1, `rand_core` 0.6.
- **Reverse deps**: `consensus-simplex`, `whirlpool-node`, `whirlpool-node-simple`.
- **Dev deps**: `tokio` 1 (features `sync`, `macros`, `rt`).
- **Features**: none.
- **cfg usage**: `src/lib.rs:16`, `src/lib.rs:78`, `src/provider.rs:305`, `src/tests.rs:3` (`#[cfg(test)]` blocks around integration helpers).
- **Circular risk**: Keep the adapters strictly downstream; trait definitions should stay inside `p2p` to avoid upward dependencies from Commonware.

### `app-evm`
- **Direct deps**: `app`, `state`, `consensus`, `reth-evm`, `reth-evm-ethereum`, `reth-ethereum-primitives`, `reth-revm`, `reth-execution-types`, `reth-execution-errors`, `reth-primitives-traits`, `reth-chainspec`, `alloy-consensus` 1.4.3, `alloy-eips` 1.4.3, `alloy-genesis` 1.0.31, `alloy-primitives` 1.5.0, `revm` 34, `thiserror` 2, `alloy-trie` 0.9.`
- **Reverse deps**: `whirlpool-node`.
- **Dev deps**: `futures` 0.3, `tokio` 1 (features `rt`, `macros`).
- **Features**: none.
- **cfg usage**: `src/config.rs:83`, `src/executor.rs:368` (`#[cfg(test)]`).
- **Circular risk**: Already aggregates multiple layers; trait extraction must avoid pulling traits from `app-evm` back into `app`/`state` to prevent cycles.

## Structural Graph Notes
- Layering is stable: `app`, `consensus`, `p2p`, `state`, and `consensus-simplex` offer upward interfaces with no downstream reverse edges introduced by this exploration. `p2p-commonware` and `app-evm` implement/bridge those traits, while `whirlpool-node*` consume them all. Any new interface modules must preserve this acyclicity.
- Mock features (`mock`) in `consensus` and `p2p` are gatekeepers for test harnesses consumed by downstream crates (`consensus-simplex` uses the `p2p` mock). Keep the cfg annotations aligned when splitting traits out.
- `app` already centralizes trait exports in `traits.rs` and re-exports them from `lib.rs`; new splits should re-export from the interface module to keep public APIs stable.
- `state` currently has only concrete implementations; introducing a dedicated `StateDb` trait in `state::traits` would keep `app-evm` and `whirlpool-node` depending on the interface before reaching for `InMemoryStateDb`.
- `consensus-simplex` and `p2p-commonware` stay implementation-heavy but rely on the upstream traits above them. Extracting new traits or re-exporting existing ones must avoid referencing back to these crates.

## Output
- Saved this analysis to `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-dependency-graph.md` for downstream planning.
