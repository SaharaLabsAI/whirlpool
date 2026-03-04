# impact-analysis.digest

- **Grounded**: Core application traits and concrete tx-source types are co-located in `crates/app/src/traits.rs` (`Application`, `TxSource`, `NoopTxSource`, `InMemoryTxPool`) and are consumed by adapter/runtime paths in `crates/app/src/adapter.rs`, `crates/app-evm/src/executor.rs`, and `crates/whirlpool-node/src/main.rs`.
- **Grounded**: Consensus traits are defined in separate files but not yet under a unified `traits` module (`crates/consensus/src/app.rs::ConsensusApp`, `block.rs::Block`, `event.rs::EventSink`, `engine.rs::ConsensusEngine`), with heavy downstream generic-bound usage in `crates/consensus-simplex/src/{adapter.rs,engine.rs}` and node binaries.
- **Grounded**: P2P interface traits are already centralized (`crates/p2p/src/traits.rs`) and implemented in mocks plus adapters (`crates/p2p/src/mock.rs`, `crates/p2p-commonware/src/{sender.rs,receiver.rs,provider.rs,lib.rs}`).
- **Grounded**: `StateProvider` is currently defined in `crates/app-evm/src/executor.rs` and used both locally and in node wiring (`crates/whirlpool-node/src/main.rs`).
- **[PROPOSED]**: Introduce compatibility re-exports during moves so public symbol paths remain valid through transition.
- **UNKNOWN**: Exhaustive downstream callsite validation from the still-running delegated impact session.
- **BLOCKER**: None at explore stage; planned-only symbols (`StateDb`, `CommonwareTransport`) are explicitly marked as not yet present.
