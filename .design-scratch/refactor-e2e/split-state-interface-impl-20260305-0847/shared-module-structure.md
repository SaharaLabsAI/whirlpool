# Shared Module Structure Exploration

## Module Trees
- `app` (`crates/app`)
  - `adapter`, `error`, `traits`, `tx_source`, `types`
  - Root re-exports: `ApplicationAdapter`, `ApplicationError`, `InMemoryTxPool`, `NoopTxSource`, `EvmBlock`, `ExecutionResult`.
- `app-evm` (`crates/app-evm`)
  - `config` (chain constants/config builders), `error` (`EvmAppError`), `executor` (`EvmApplication` + helpers), `traits` (`StateProvider` bridging to `state::traits::StateDb`).
  - Root re-exports: `SAHARA_CHAIN_ID`, `WhirlpoolEvmConfig`, `build_sahara_chain_spec`, `EvmAppError`.
- `consensus` (`crates/consensus`)
  - Modules: `block`, `error`, `app`, `event`, `engine`, `traits`, optional `mock`, `tests`.
  - Re-exports: `ConsensusError`, `ConsensusEvent`, `ConsensusStatus`, `RunningEngine`.
- `consensus-simplex` (`crates/consensus-simplex`)
  - Modules: `traits`, `types`, `config`, `adapter`, `engine`, `mailbox`, `sink` plus `Channel` constants (VOTE/CERTIFICATE/RESOLVER) referencing `p2p::Channel`.
  - Re-exports: `CommonwareConfig`, `AppAdapter`, `CommonwareEngine`, `Mailbox`, `MailboxActor`, `Message`, channel constants.
- `p2p` (`crates/p2p`)
  - Modules: `errors`, `traits`, `types`, optional `mock`.
  - Re-exports: `P2pError`, `NetworkProvider`, `NetworkReceiver`, `NetworkSender`, `PeerId`, `Channel`, `NetworkChannel`, `NetworkMessage`, `Recipients`.
- `p2p-commonware` (`crates/p2p-commonware`)
  - Public modules: `provider`, `traits`, `sender`, `receiver`; private helpers: `peer_id`, `error`; optional `tests`.
  - Re-exports: `CommonwarePeerId`, `map_send_error`, `map_recv_error`, `CommonwareTransport`, `CommonwareSender`, `CommonwareReceiver`, `CommonwareNetworkProvider`, `CommonwareNetworkProviderBuilder`, `OracleHandle`, `Bootstrapper`.
  - Defines `MultiplexSender` + `MultiplexReceiver` in the crate root to multiplex per-channel senders/receivers.
- `state` (`crates/state`)
  - `lib.rs` exposes `pub mod db`, `pub mod error`, `pub mod traits` and re-exports `DbAccount`, `InMemoryStateDb`, `StateError`, `StateDb` for downstream use.
  - `state::db`: hosts `DbAccount` (account info + storage map), `InMemoryStateDb` (accounts/bytecodes/block hashes), convenience wrappers over `StateDb` trait methods, and `revm::DatabaseRef`/`revm::Database` impls that use `StateError` for error bookkeeping.
  - `state::error`: defines `StateError` (`Internal(String)`) and the `revm::database::DBErrorMarker` impl so that `StateError` anchors the `Database` trait.
  - `state::traits`: defines `StateDb` trait (constructor helpers, `state_root`, `commit`, captures getters like `get_account`, `get_code_by_hash`, `get_storage`, `get_block_hash`, plus the insert helpers).
- `whirlpool-node` (`crates/whirlpool-node`)
  - `config` module, `main.rs` binary.
  - `main.rs` wires consensus, networking, app layers; defines `TestStateDb(InMemoryStateDb)` wrapper, implements `app_evm::traits::StateProvider` by delegating to `state::InMemoryStateDb`, and implements `revm::Database` using `state::StateError` as the error type.
- `whirlpool-node-simple` (`crates/whirlpool-node-simple`)
  - Modules: `app`, `block` (likely lightweight stubs for simplified node).
- `[PROPOSED]/state-memory` (missing)
  - Intended to host concrete state storage (probably the `db` and `error` modules that currently live in `state`).
  - Would depend on `state` for `StateDb` and re-export implementations: `DbAccount`, `InMemoryStateDb`, `StateError`, `impl DatabaseRef for InMemoryStateDb`, `impl Database for InMemoryStateDb`, plus any helpers that remain implementation-only.
  - `state` crate would retain the `traits` module and re-export `StateDb` only so that interface-only consumers keep a lightweight dependency.

## Re-export Chains
- `state` root makes the `StateDb` trait available via `pub use traits::StateDb`, while the concrete helpers (`DbAccount`, `InMemoryStateDb`, `StateError`) live in `db`/`error` but are currently re-exported alongside the trait.
- `state::db` pulls in `crate::traits::StateDb` and `crate::error::StateError`, tying the implementation to the interface and the error surface.
- `state::error` also pulls in `revm::database::DBErrorMarker` and exposes `StateError` (with `Internal` variant) through `state::StateError`.
- `app` exposes `ApplicationAdapter`, `ApplicationError`, `InMemoryTxPool`, `NoopTxSource`, `EvmBlock`, and `ExecutionResult` so that downstream binaries can reach those facades through `app::...` paths.
- `app-evm` exposes the configuration helpers and `EvmAppError` plus the `StateProvider` trait (backed by `state::traits::StateDb`).
- `p2p` and `p2p-commonware` expose networking abstractions (`NetworkProvider`, `NetworkSender`, `NetworkReceiver`, `CommonwareTransport`, `Channel`, etc.) so that higher-level crates (`consensus-simplex`, `whirlpool-node`) can rely on trait facades instead of concrete dependencies.
- `whirlpool-node` currently consumes `state::InMemoryStateDb` and `state::StateError` directly; after the split it will be able to depend on `state-memory::InMemoryStateDb` (or a renamed path) for the concrete `revm::Database` implementation while still referencing `state::traits::StateDb` for interface bounds.

## Visibility Map for Changed Symbols
- `StateDb`: defined in `state::traits::StateDb` (public trait) and re-exported at `state::StateDb`. Downstream consumers (`app-evm::traits::StateProvider`, tests in `app-evm`, `whirlpool-node` wrappers) import via `state::traits::StateDb` or `state::StateDb` depending on path preference.
- `StateError`: defined (and made `pub`) in `state::error::StateError`, re-exported as `state::StateError` so that `TestStateDb` and any `revm::Database` impls can refer to it without reaching into the `state::error` module.
- `DBErrorMarker` impl for `StateError`: located inside `state::error` (inline `impl revm::database::DBErrorMarker for StateError {}`), giving `StateError` the marker trait required by `revm::Database`.
- `DbAccount`: `pub struct DbAccount` is defined inside `state::db` and re-exported as `state::DbAccount`. It carries `AccountInfo` + storage map used by `InMemoryStateDb` and unit tests.
- `InMemoryStateDb`: `pub struct` lives in `state::db` (hashmaps for accounts/bytecodes/block hashes) and is re-exported at `state::InMemoryStateDb`. The struct implements `StateDb`, and the module also exposes helper methods forwarding to the trait impl.
- `impl DatabaseRef for InMemoryStateDb`: located near the bottom of `state::db`, with associated `type Error = StateError` and read methods that translate to the inner hashmaps.
- `impl Database for InMemoryStateDb`: immediately after the `DatabaseRef` impl, also uses `StateError` and delegates to the reference-based APIs.

## Facade Analysis
- Today the `state` crate is both the trait-level facade (`StateDb`) and the concrete implementation provider (`InMemoryStateDb`, `StateError`, revm adapters). Any consumer pulling `state` gets both interface and implementation. App layers such as `app-evm` only need the trait, but node binaries (`whirlpool-node`, its tests) reach for the concrete `InMemoryStateDb` and the `StateError`/`revm::Database` surface.
- `app-evm::traits::StateProvider` simply forwards `state_root`/`commit` to anything that implements `StateDb`, so keeping the trait in `state` preserves a lightweight interface that does not drag in hash map-heavy `revm` dependencies. This trait is consumed by `EvmApplication`, `StateProvider` impls in binaries, and the shared tests under `crates/app-evm/tests`.
- `whirlpool-node` currently wraps `state::InMemoryStateDb` with a local `TestStateDb` to satisfy `StateProvider` and `revm::Database`. After the split, `TestStateDb` could either wrap `state-memory::InMemoryStateDb` or be replaced entirely by `state-memory` exposing the `revm::Database` impl directly; the node would then depend on both `state` (for the trait) and `state-memory` (for the concrete DB).
- Introducing `[PROPOSED]/state-memory` lets interface-only dependers keep depending on `state`, while runtime components (node, `EvmApplication`, integration tests) point at `state-memory` for the `DbAccount`/`InMemoryStateDb`/`StateError` trio and the `revm` implementations. The `state-memory` crate would therefore define the `db` and `error` modules that currently sit inside `state`, re-export them, and `use state::traits::StateDb` to keep the `impl StateDb for InMemoryStateDb` near the concrete data.
- Visibility-wise, `state::lib` will need to stop re-exporting `DbAccount`, `InMemoryStateDb`, and `StateError` once they migrate; the new crate should re-export them so callers only update one import path each (e.g., `state-memory::InMemoryStateDb`). The `StateDb` trait remains the sole export from `state` that interface-only consumers reach for, keeping the interface crate minimal.
