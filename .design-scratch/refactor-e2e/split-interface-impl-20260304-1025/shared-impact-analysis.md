# Shared Impact Analysis — split interface vs implementation

## Structural depth / cross-crate seams
- Splitting traits away from their concrete helpers will leave the `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, and `app-evm` crates with explicit interface-only modules and dedicated implementation modules (e.g., `app::traits` vs `app::tx_source`).
- Each symbol below names the current home, one concrete callsite, and the dependent crates that rely on it; high-risk callsites highlight where rewiring the interfaces could break execution paths or add new dependency edges.
- Future migrations must preserve the existing publicly exported paths (via re-exports) because downstream crates (especially `app-evm`, `whirlpool-node`, and `consensus-simplex`) already import these symbols from their current modules.

## Symbol impacts

### Application (`app::traits::Application`)
- Definition: trait declared in `crates/app/src/traits.rs:4-22` exposes `genesis`, `propose`, and `verify` with async `Future` results.
- Call site: `crates/app-evm/src/executor.rs:99-239` implements `Application` for `EvmApplication`, using the trait to run EVM proposals/verification and to commit state via associated `Block`/`Result` types.
- Cross-crate dependencies: `app-evm` (execution logic), `app::adapter` (maps `Application` → `consensus::ConsensusApp`), and `whirlpool-node` (uses `ApplicationAdapter` inside `CommonwareEngine`).
- High-risk call site: `EvmApplication::propose` (`app-evm/src/executor.rs:127-239`) drains `TxSource`, executes transactions, updates the `StateProvider`, and returns both `EvmBlock` and `ExecutionResult`; moving `Application` into a new interface-only module risks breaking the concrete `EvmApplication` impl unless re-exports and trait paths stay stable.

### TxSource (`app::traits::TxSource`)
- Definition: `TxSource::pending()` declared in `crates/app/src/traits.rs:24-26` returns raw EIP-2718 bytes.
- Call site: `app-evm/src/executor.rs:133-187` calls `self.tx_source.pending()` inside `EvmApplication::propose`, then decodes and replays transactions.
- Cross-crate dependencies: `app-evm` (transaction ingress), node tests (use concrete `NoopTxSource`/`InMemoryTxPool`), and any future tx-pool crate that will provide a `TxSource` implementation.
- High-risk call site: the `Arc<dyn TxSource + Send + Sync>` stored on `EvmApplication` needs the exact trait path; splitting the trait into a new module means `Arc::new(tx_source)` constructors and `use` statements must be updated across both production and test code.

### NoopTxSource (`app::traits::NoopTxSource`)
- Definition: struct + `TxSource` impl in `crates/app/src/traits.rs:28-34` producing an empty vector.
- Call site: `crates/app-evm/tests/application_integration.rs:3-17` builds adapters with `Arc::new(NoopTxSource)` to exercise empty-block behavior.
- Cross-crate dependencies: `app-evm` tests, any other project relying on the default no-op source, and the re-export from `app::lib.rs` (`pub use traits::NoopTxSource`).
- High-risk call site: tests expect to refer to `app::NoopTxSource`; moving the struct to `app::tx_source` requires a re-export or the consumers must change their `use` paths immediately, so the intermediate migration step should still expose `pub use tx_source::NoopTxSource` or similar.

### InMemoryTxPool (`app::traits::InMemoryTxPool`)
- Definition: FIFO, mutex-based pool declared `crates/app/src/traits.rs:43-75` and currently exported from `app::lib.rs`.
- Call site: `crates/whirlpool-node/src/main.rs:129-134` constructs `Arc::new(InMemoryTxPool::new())` and hands it to `EvmApplication::new` so the runtime executes real transactions.
- Cross-crate dependencies: `whirlpool-node` (runtime wiring), `app-evm/tests` (where the pool could be injected once exported), and any future tx source provider.
- High-risk call site: consensus proposals now depend on the pool being thread-safe and in the same crate; moving `InMemoryTxPool` into a new implementation module requires re-export so `whirlpool-node` binding (`use app::InMemoryTxPool`) does not break and so the pool’s constructor remains visible to the node for future tx ingress.

### ConsensusApp (`consensus::app::ConsensusApp`)
- Definition: trait in `crates/consensus/src/app.rs:6-27` with associated `Block` and methods `genesis`, `propose`, `verify` returning futures.
- Call sites: `crates/app/src/adapter.rs:20-55` implements `ConsensusApp` for `ApplicationAdapter<A>` by delegating to `Application`, and `consensus-simplex/src/engine.rs:48-182` accepts `A: ConsensusApp` to drive the vendor simplex engine.
- Cross-crate dependencies: `app` (adapter), `consensus-simplex` (engine), and any downstream node that instantiates a `ConsensusEngine` with an `ApplicationAdapter`.
- High-risk call site: `CommonwareEngine::start` (`consensus-simplex/src/engine.rs:90-192`) calls `ConsensusApp::propose`/`verify`; if the trait is rehomed without re-exporting, `CommonwareEngine` would need updated imports, and any future interface split must preserve the existing trait signature to avoid breaking the engine’s bounds.

### Block (`consensus::block::Block`)
- Definition: trait in `crates/consensus/src/block.rs:4-16` providing `id`, `parent_id`, `height`.
- Call site: `consensus-simplex/src/sink.rs:8-63` uses `Block` as the sink’s type parameter, and `app::adapter.rs` ties `ConsensusApp::Block` = `EvmBlock`.
- Cross-crate dependencies: `consensus-simplex` (FinalizationSink, Mailbox, AppAdapter) and `app` (through `ConsensusApp` and `EvmBlock`).
- High-risk call site: finalization logging and state tracking depend on the block trait staying in the same module so the adapter/engine pair can share the same block type without adjusting their `use` paths.

### EventSink (`consensus::event::EventSink`)
- Definition: trait in `crates/consensus/src/event.rs:29-37` with `handle` consuming `ConsensusEvent<Self::Block>`.
- Call sites: `consensus-simplex/src/sink.rs:37-63` implements `EventSink` for `FinalizationSink`, and `consensus-simplex/src/engine.rs:48-96` constrains `S: EventSink<Block = A::Block>` when wiring the simplex engine.
- Cross-crate dependencies: `consensus-simplex` provides both the sink implementation and the engine, and `whirlpool-node` instantiates `FinalizationSink` before handing it to `CommonwareEngine`.
- High-risk call site: `FinalizationSink::handle` is invoked continually by the consensus loop; any change to the interface path must keep this trait accessible without changing the sink’s `use` statements or node wiring.

### ConsensusEngine (`consensus::engine::ConsensusEngine`)
- Definition: trait in `crates/consensus/src/engine.rs:64-68` with a single `start` method returning `RunningEngine`.
- Call site: `consensus-simplex/src/engine.rs:90-191` implements `ConsensusEngine` for `CommonwareEngine<A, S, E, C>`, calling `start()` to spawn the vendor engine and return a `RunningEngine` handle.
- Cross-crate dependencies: `whirlpool-node` depends on the trait by using `CommonwareEngine` (the struct implements the trait) to satisfy the consensus crate’s expectations.
- High-risk call site: the new interface module for `ConsensusEngine` must expose the trait without altering the node’s import path; otherwise the binary’s `use consensus::ConsensusEngine;` would break.

### PeerId (`p2p::traits::PeerId`)
- Definition: trait in `crates/p2p/src/traits.rs:10-16` abstracting peer identity.
- Call site: `crates/p2p-commonware/src/peer_id.rs:22-53` defines `CommonwarePeerId<P>` that implements `p2p::PeerId` by wrapping a Commonware `PublicKey`.
- Cross-crate dependencies: `p2p-commonware` provides the bridging type; any other crate assuming generic `PeerId` can rely on this implementation.
- High-risk call site: moving the trait into a narrower interface module requires `p2p-commonware` to re-import the new path, but as long as the crate re-exports `PeerId`, existing uses stay untouched.

### NetworkSender (`p2p::traits::NetworkSender`)
- Definition: trait in `crates/p2p/src/traits.rs:22-44` describing async `send` with `Channel`, `Bytes`, and `Recipients`.
- Call site: `crates/p2p-commonware/src/lib.rs:31-62` implements `NetworkSender` for `MultiplexSender`, routing to per-channel `CommonwareSender` instances.
- Cross-crate dependencies: `p2p-commonware` (core implementation), `consensus-simplex` (expects `NetworkProvider` to hand it a sender implementing this trait through the provider interface), and node wiring (which receives the sender from `CommonwareNetworkProvider`).
- High-risk call site: the multiplex sender is created during `CommonwareNetworkProvider::start` and stored in the node; splitting the trait without preserving the original path would require touching node, tests, and provider logic simultaneously.

### NetworkReceiver (`p2p::traits::NetworkReceiver`)
- Definition: trait in `crates/p2p/src/traits.rs:46-65` with async `recv` returning `Option<NetworkMessage<Self::PeerId>>`.
- Call site: `crates/p2p-commonware/src/lib.rs:65-128` implements `NetworkReceiver` for `MultiplexReceiver`, polling multiple per-channel receivers.
- Cross-crate dependencies: `p2p-commonware` provides the consumer, `consensus-simplex` consumes the receiver through `CommonwareEngine`, and `whirlpool-node` keeps the returned receiver alive for the engine’s network loops.
- High-risk call site: the receiver is passed into `CommonwareEngine::start` (vendor engine uses per-channel receivers); reorganizing the trait could break this wiring if the node or engine imports the wrong module.

### NetworkProvider (`p2p::traits::NetworkProvider`)
- Definition: trait in `crates/p2p/src/traits.rs:67-95` with associated `Sender`, `Receiver`, and `start()`.
- Call site: `crates/p2p-commonware/src/provider.rs:246-303` implements `NetworkProvider` for `CommonwareNetworkProvider`, returning `MultiplexSender`/`MultiplexReceiver` for consensus use.
- Cross-crate dependencies: `whirlpool-node` (takes `CommonwareNetworkProvider` via builder), `consensus-simplex` (requires a provider to start the network), and future variants of the node or tests.
- High-risk call site: the trait is the normalized interface between vendor networking and the rest of the stack; splitting it requires carefully re-exporting the symbol so the node (which imports via `use p2p::NetworkProvider`) is unaffected.

### StateDb (`state::traits::StateDb`)
- **INCOMPLETE / MISSING**: there is no `StateDb` trait today; the `state` crate only exports `InMemoryStateDb` (e.g., `crates/state/src/db.rs:19-136`). Introducing `state::traits::StateDb` is a planned addition to provide an interface for `state_root`/`commit`. Until that trait exists, there are no concrete call sites to list, so downstream crates depend on the struct directly.

### CommonwareBlock (`consensus-simplex::types::CommonwareBlock`)
- Definition: trait in `crates/consensus-simplex/src/types.rs:1-20` combining `consensus::Block`, `commonware_consensus::Block`, and `Clone`.
- Call site: `consensus-simplex/src/engine.rs:50-95` constrains `A::Block` with `CommonwareBlock`/`Digestible`/`Committable` when instantiating the simplex engine and `AppAdapter` (via `crate::adapter::AppAdapter`).
- Cross-crate dependencies: `app` (the concrete `EvmBlock` needs to satisfy both core and vendor block traits) and `consensus-simplex` (requires the abstraction for vendor compatibility).
- High-risk call site: re-exporting `CommonwareBlock` in a new traits module will require updating every `use crate::types::CommonwareBlock` to the new path, starting with the engine and adapter modules.

### CommonwareTransport (`p2p-commonware::traits::CommonwareTransport`)
- **INCOMPLETE / MISSING**: this trait is currently only proposed (docs mention `[PROPOSED] p2p-commonware::traits::CommonwareTransport`), so no implementation or call site exists in the codebase. The new trait would presumably describe the vendor transport primitives exposed via `p2p-commonware`, but without concrete code, we cannot cite a callsite.

### StateProvider (`app-evm::executor::StateProvider`)
- Definition: trait declared in `crates/app-evm/src/executor.rs:31-35` exposing `state_root`/`commit` to the executor.
- Call site: `crates/app-evm/src/executor.rs:21-29` implements `StateProvider` for `state::InMemoryStateDb`, and `crates/whirlpool-node/src/main.rs:27-43` implements `StateProvider` for `TestStateDb` (wrapper around `InMemoryStateDb`). `EvmApplication` further bounds its database parameter with `StateProvider` in `impl<DB> Application for EvmApplication<DB>` (`app-evm/src/executor.rs:99-365`).
- Cross-crate dependencies: `state` crate (provides the struct), `whirlpool-node` (provides a thin shim), and `app-evm` (consumes the trait to commit pending bundles).
- High-risk call site: `EvmApplication::propose`/`verify` (`app-evm/src/executor.rs:145-354`) clones or locks the database via `StateProvider`; moving the trait into a separate `traits` module must keep the trait path stable so that both the executor and the node’s shim continue to satisfy the bound.

## Summary of cross-crate impact
- Interface-only modules must continue to re-export the same symbols so downstream crates (`app-evm`, `whirlpool-node`, `consensus-simplex`, `p2p-commonware`) do not need ripple changes.
- Missing traits (`StateDb`, `CommonwareTransport`) introduce work: the `state` crate will need an interface trait before `app-evm` can depend on it abstractly, and `p2p-commonware` must settle on a transport trait before exposing vendor-specific channels to the rest of the system.
- High-risk callsites are clustered around `EvmApplication::propose/verify` (state + tx sourcing) and `CommonwareEngine::start` (consensus + networking). Any interface move must keep their `use` paths intact or provide transitional re-exports, otherwise runtime wiring (node startup, engine startup) will break at build time.
