# Shared Baseline

## Prior Art

- **evm-tx-execution** (Sub-Intent 1): Implemented EvmApplication in app-evm with real reth-based propose/verify. IMPLEMENTED.
- **evmblock-txsource** (Sub-Intent 2): Implemented InMemoryTxPool in app crate. IMPLEMENTED.
- **Current task** is effectively Sub-Intent 3: Wire real simplex consensus engine.

## Current State

The consensus-simplex crate has ALL adapter components built and tested:
- `AppAdapter` — bridges ConsensusApp↔commonware Application/VerifyingApplication/Reporter
- `Mailbox` + `MailboxActor` — bridges ConsensusApp↔commonware Automaton/Relay
- `FinalizationSink` — handles finalization events, tracks height
- `CommonwareConfig` — holds simplex engine configuration

The ONLY missing piece is `CommonwareEngine::start()` which is stubbed:
- Creates Mailbox, MailboxActor, FinalizationSink (all unused with `_` prefix)
- Spawns a `std::thread` incrementing height every 5 seconds
- Never calls vendor `simplex::Engine` at all

## Vendor API

The vendor `commonware_consensus::simplex::Engine`:
- `new(context: E, config: Config<S,L,B,D,A,R,F,T>)` — E implements Clock+Rng+Spawner+Storage+Metrics
- `start(self, vote_network, cert_network, resolver_network)` — takes 3 separate (Sender, Receiver) channel pairs
- Returns `Handle<()>` (commonware-runtime handle for lifecycle management)

## P2P Layer Gap

Current `NetworkProvider::start()` returns `(MultiplexSender, MultiplexReceiver)` — multiplexed.
Vendor simplex engine needs 3 SEPARATE (Sender, Receiver) pairs.
Options: (A) Bypass trait, expose per-channel pairs from provider, (B) Change trait.
