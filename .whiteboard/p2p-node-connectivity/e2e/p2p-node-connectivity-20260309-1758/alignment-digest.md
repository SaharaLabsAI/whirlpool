# Alignment Digest — P2P Node Connectivity

## Approved Intent
Enable whirlpool-node instances to connect to each other via P2P networking by fixing p2p-commonware gaps, adding node configuration, and activating the consensus relay.

## Confirmed Scope
- **Crates affected**: p2p-commonware, whirlpool-node, consensus-simplex, p2p (minor), app (compatibility check)
- **Depth**: module — touches internals of 3-4 crates, adds no new crates
- **3 sub-intents proposed**:
  - Split A: P2P Provider Completeness (validator seeding, bootstrap peers, channel metadata fix)
  - Split B: Node Config & Startup Wiring (CLI/config, builder integration)
  - Split C: Consensus Relay Activation (relay wiring, channel alignment, app compatibility)

## Approach Direction
1. Fix the foundational p2p-commonware bugs first (Split A) — nothing else works without correct channel metadata and peer discovery
2. Add configuration support (Split B) — so nodes can specify real addresses and peers
3. Wire the relay (Split C) — connect consensus to the now-functional P2P layer

## Risks
- 2 high-severity risks resolved directly by requirements (channel bug, validator seeding)
- 4 low/medium risks accepted (CLI framework, relay scope, vendor stability, NAT)
- 0 blockers

## Iteration Count
- Alignment iteration: 1
