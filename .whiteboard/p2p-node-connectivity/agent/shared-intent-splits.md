# Shared Intent Splits

Reason for split: intake thresholds exceeded (crates>3, boundaries>4, domains>2, flows>3), so the original intent is too broad for one module-depth design pass.

## Split A — P2P Provider Completeness
- Goal: close `crates/p2p-commonware` feature gaps so provider behavior matches existing `crates/p2p` contracts.
- Includes:
  - validator seeding application
  - bootstrap peer population
  - channel metadata preservation on receive
- Primary crates: `crates/p2p-commonware`, `crates/p2p`

## Split B — Node Config and Startup Wiring
- Goal: expose and wire network topology inputs needed for multi-node connectivity.
- Includes:
  - CLI/config for listen addresses, dial peers, bootstrap peers
  - startup wiring from config into provider builder
- Primary crates: `crates/whirlpool-node`, `crates/p2p-commonware`

## Split C — Consensus Relay Activation
- Goal: make simplex relay actually exchange consensus traffic over the P2P network.
- Includes:
  - outbound relay send via `NetworkSender`
  - inbound relay receive/routing into simplex mailbox streams
  - channel-constant alignment validation
- Primary crates: `crates/consensus-simplex`, `crates/p2p`, `crates/p2p-commonware`

## Shared Invariants Across Splits
- Preserve `crates/p2p` trait interfaces and channel constants.
- Reuse existing commonware transport/multiplexing behavior rather than replacing vendor networking internals.
- Maintain compatibility with existing app consensus adapter flow in `crates/app`.
