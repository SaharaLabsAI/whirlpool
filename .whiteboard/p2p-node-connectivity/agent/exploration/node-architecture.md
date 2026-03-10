# Exploration: Node Architecture

## Startup Flow (whirlpool-node/src/main.rs)
1. Init tracing, spawn Commonware tokio::Runner
2. Create deterministic ed25519 signer + validator set
3. Build P2P: CommonwareNetworkProviderBuilder (namespace='whirlpool-dev', 1MB max msg, OS-assigned listen/dial ports 127.0.0.1:0)
4. Open state-reth + mempool-mdbx databases, recover tip → AtomicU64 height
5. Create FinalizationSink → PersistingFinalizationSink
6. Wrap app_evm::EvmApplication with ApplicationAdapter → ConsensusApp
7. Hand app + sink + config + network_provider to consensus_simplex::CommonwareEngine
8. Start JSON-RPC server, main task awaits, oracle_handle kept alive

## Component Wiring
- `app/adapter.rs`: maps Application → ConsensusApp
- `PersistingFinalizationSink`: decorates FinalizationSink, persists finalized blocks
- `app/traits.rs`: TxSource(InMemory/Noop), Application traits — consensus depends only on traits

## Consensus-Simplex Engine
- Creates mailbox (Automaton/Relay), actor, reporter (AppAdapter)
- Sets up simplex::Config, starts vendor engine with per-channel vote/cert/resolver streams
- Config expects: signer/validator keys, buffers, timeouts, height
- **Relay is currently no-op** — multi-node relay not implemented

## Integration Points for P2P
1. main.rs — P2P builder construction (add peer config, bootstrap, validator seeding)
2. consensus-simplex — per-channel streams (fix channel metadata, wire relay)
3. Relay/mailbox — replace no-op with real multi-node message passing
