# SUMMARY — Real Simplex Consensus Wiring

## What

Replace the stub consensus engine in `CommonwareEngine::start()` with real wiring to the vendor `commonware_consensus::simplex::Engine`, enabling whirlpool-node to produce real EVM blocks through BFT consensus.

## Why

The whirlpool-node currently runs in stub mode — a thread increments block height every 5 seconds without actually executing any consensus or EVM logic. All the components needed for real block production are already built and tested:

- **EvmApplication** (app-evm): Fully implements propose/verify using reth's EVM block builder
- **ApplicationAdapter** (app): Bridges EvmApplication to the ConsensusApp trait
- **InMemoryTxPool** (app): Transaction sourcing for block building
- **AppAdapter** (consensus-simplex): Bridges ConsensusApp to vendor simplex Application/Reporter
- **Mailbox/MailboxActor** (consensus-simplex): Bridges ConsensusApp to vendor simplex Automaton/Relay
- **FinalizationSink** (consensus-simplex): Tracks finalized block height

The only missing piece is the `start()` method that wires all these together with the vendor simplex engine.

## Scope

**3 crates affected**, ordered by change magnitude:

1. **consensus-simplex** (major): Replace stub loop with real simplex engine wiring — create Mailbox, spawn MailboxActor, assemble simplex Config, start vendor Engine, return RunningEngine wrapping the Handle
2. **p2p-commonware** (moderate): Add `start_per_channel()` method that exposes 3 separate (Sender, Receiver) channel pairs instead of multiplexed channels — the vendor simplex engine requires individual channel pairs for vote, certificate, and resolver protocols
3. **whirlpool-node** (minor): Pass runtime context and validator config to engine constructor; use oracle handle for Blocker creation

## Key Design Decisions

1. **P2P channel splitting**: Add concrete `start_per_channel()` on `CommonwareNetworkProvider` rather than changing the generic `NetworkProvider` trait — simplex already depends on concrete commonware types
2. **Runtime context**: Store commonware runtime context in `CommonwareEngine` at construction time, keeping `ConsensusEngine::start()` signature unchanged
3. **Validator set**: Single-validator dev mode — `CommonwareConfig` extended with validators field, `RoundRobinElector` for leader election
4. **Blocker**: Obtain from P2P `OracleHandle.control()` which returns an `Oracle` implementing the vendor `Blocker` trait
5. **Shutdown**: Abort vendor `Handle<()>` on shutdown instead of the current thread/AtomicBool polling

## Risks and Unknowns

- Vendor simplex Config has **9 generic type parameters** — complex type threading but all concrete types are known (ed25519, Sequential, RoundRobinElector, etc.)
- **Single-validator BFT** behavior is untested in vendor code (tests use n≥3) — needs empirical verification
- **Runtime context ownership** between network builder and engine requires careful ordering

## Implementation Order

1. p2p-commonware: `start_per_channel()` method
2. consensus-simplex/config: Extend CommonwareConfig with signer/validators
3. consensus-simplex/engine: Replace stub `start()` body
4. whirlpool-node/main: Wire context, signer, validators, oracle handle
5. Tests: Unit tests for channel splitting, integration test for real block finalization
