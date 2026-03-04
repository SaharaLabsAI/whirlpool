# Shared Module Structure (Phase 02 Step 2)

## Scope
Focus on crates that contribute the critical interfaces and their implementations: `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, and `app-evm`. Highlight where core traits live versus where concrete wiring/behaviors are implemented so we can split interfaces cleanly.

## Crate `app`
- **Public facade:** `src/lib.rs` re-exports `adapter`, `error`, `traits`, and `types`, keeping the surface limited to `ApplicationAdapter`, `ApplicationError`, trait definitions, and block/execution types.
- **Interface layer:** `src/traits.rs` defines the `Application` trait plus supporting traits/impls (`TxSource`, `NoopTxSource`, `InMemoryTxPool`). These are pure interfaces or helpers that downstream crates consume directly.
- **Implementation layer:** `src/adapter.rs` implements `ApplicationAdapter`, turning any `app::Application` into a `consensus::ConsensusApp`, while tests live in cfg-gated submodules.
- **Data types:** `src/types.rs` contains `EvmBlock`, `ExecutionResult`, codec helpers, `CoreBlock` impl, and other shared DTOs used across runtimes.

## Crate `consensus`
- **Facade:** `src/lib.rs` re-exports trait-heavy modules (`block.rs`, `app.rs`, `event.rs`, `engine.rs`), making `ConsensusApp`, `ConsensusEngine`, `EventSink`, `ConsensusStatus`, etc., available without exposing implementation details.
- **Interface modules:**
  - `block.rs` defines the `Block` trait (no executable logic).
  - `app.rs` declares the `ConsensusApp` trait that consensus engines drive.
  - `event.rs` defines `ConsensusEvent` and the `EventSink` trait.
  - `engine.rs` defines the `ConsensusEngine` trait along with helper structs (`ConsensusStatus`, `RunningEngine`) but no concrete engine implementations.
- **Conditional harnesses:** `mock` module exposes fake implementations (tests or `mock` feature) while keeping the trait surfaces intact.

## Crate `p2p`
- **Facade:** `src/lib.rs` re-exports `errors.rs`, `traits.rs`, and `types.rs`, so consumers only depend on `P2pError`, the trait trio (`NetworkProvider`, `NetworkSender`, `NetworkReceiver`, `PeerId`), and channel/message types.
- **Interface vs. data:** `traits.rs` defines the abstract networking contracts, while `types.rs` provides value objects (`Channel`, `NetworkChannel`, `NetworkMessage`, `Recipients`) referenced by those traits.
- **Implementation boundary:** actual implementations live in other crates (e.g., `p2p-commonware`); the `mock` module wraps fake implementations behind cfg guards, preserving the clean interface.

## Crate `state`
- **Facade:** `src/lib.rs` re-exports `InMemoryStateDb`, `DbAccount`, and `StateError`, so consumer crates link against these concrete helpers without reaching into internals.
- **Implementation:** `db.rs` implements the in-memory database, constructors (`new`, `with_genesis`), `Database`/`DatabaseRef` traits, and helper state operations.
- **Error handling:** `error.rs` defines `StateError` (implements `revm::database::DBErrorMarker`) and is re-exported for visibility control.

## Crate `consensus-simplex`
- **Facade:** `src/lib.rs` exposes `CommonwareBlock`, configuration (`CommonwareConfig`), wiring (`AppAdapter`, `CommonwareEngine`), mailbox/sink helpers, and channel constants while keeping vendor details private.
- **Interface layer:** `types.rs` defines `CommonwareBlock` as the super-trait bundling `consensus::Block` and `commonware_consensus::Block`, plus a blanket impl.
- **Implementation layer:** `adapter.rs` and `engine.rs` provide the glue between `ConsensusApp`, `EventSink`, and Commonware's engine; `mailbox.rs`, `sink.rs`, `config.rs`, and helper structs house the operational logic.
- **Test harness:** `tests.rs` and cfg-guarded modules keep verification code out of the published facade.

## Crate `p2p-commonware`
- **Facade:** `src/lib.rs` re-exports `CommonwarePeerId`, `map_send_error`/`map_recv_error`, `CommonwareSender`/`Receiver`, `CommonwareNetworkProvider`, builder/handle types, and the `Bootstrapper` helper.
- **Interface layer:** `provider.rs` implements the `NetworkProvider` trait via `CommonwareNetworkProvider`, but exposes only the multiplexed sender/receiver adapters, not the vendor internals.
- **Implementation layer:** `sender.rs` and `receiver.rs` adapt Commonware sink/source types into `p2p` traits; `peer_id.rs` provides the peer identifier newtype; `error.rs` translates vendor errors.
- **Multiplex helpers:** `MultiplexSender`/`MultiplexReceiver` orchestrate per-channel Commonware senders/receivers into the single imported traits.
- **Tests:** `tests.rs` exercises start-up, per-channel pumping, and multiplexing without leaking into the published API.

## Crate `app-evm`
- **Facade:** `src/lib.rs` exports configuration helpers (`SAHARA_CHAIN_ID`, `WhirlpoolEvmConfig`, `build_sahara_chain_spec`) and `EvmAppError`.
- **Interface layer:** `executor.rs` defines the `StateProvider` trait (abstracts `state_root`/`commit`) plus transforms `state::InMemoryStateDb` into a `StateProvider` via a thin impl.
- **Implementation layer:** `EvmApplication` (in `executor.rs`) ties `WhirlpoolEvmConfig`, `StateProvider`, and `TxSource` into an `app::Application` implementation; helper functions (`build_header_from_evm_block`, `decode_transactions`, etc.) live here.
- **Tests:** cfg-gated modules in `executor.rs` and separate test files demonstrate how this application is configured and executed, keeping test details isolated from public exports.

---
By keeping trait definitions in `traits.rs`/`app.rs`/`types.rs` modules and implementation adapters/providers/executors elsewhere, each crate already supports the desired interface/implementation split. The new documentation can guide where to add separate crates or modules for interface-only definitions versus concrete implementations during the refactor.
