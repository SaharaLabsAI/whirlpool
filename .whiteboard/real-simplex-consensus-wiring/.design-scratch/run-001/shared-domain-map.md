# Shared Domain Map

## Domain: Consensus Engine Wiring

**Owner**: consensus-simplex crate

**Boundary**: Takes ConsensusApp + EventSink + NetworkProvider → produces RunningEngine

**Components**:
1. **Mailbox/MailboxActor** — bridges app to commonware Automaton trait
2. **AppAdapter** — bridges app+sink to commonware Application/Reporter traits
3. **FinalizationSink** — tracks finalized height from reporter events
4. **CommonwareEngine** — orchestrates component creation and simplex engine startup

**Cross-Boundary Interfaces**:
- `consensus::ConsensusApp` (app → engine) — propose, verify, genesis
- `consensus::EventSink` (engine → app) — finalization events
- `p2p::NetworkProvider` (network → engine) — P2P channel pairs
- `commonware_consensus::simplex::Engine` (vendor) — the actual BFT engine
- `commonware_runtime` context (runtime → engine) — Spawner/Clock/Metrics for vendor engine

## Domain: P2P Network

**Owner**: p2p-commonware crate

**Boundary**: Manages discovery network, provides channel pairs for consensus

**Key Change**: Must expose per-channel (Sender, Receiver) pairs instead of/in addition to multiplexed channels. The vendor simplex engine directly consumes individual channel pairs.
