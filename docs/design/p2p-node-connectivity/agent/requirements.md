# Requirements

## Scope
- Depth: `module`
- Focus crates: `all`
- Intake breadth check: **too broad**
  - Crates affected: 5+ (`crates/p2p`, `crates/p2p-commonware`, `crates/whirlpool-node`, `crates/consensus-simplex`, `crates/app`)
  - Boundaries: >4 (CLI/config -> node bootstrap -> provider builder -> transport/discovery -> consensus relay)
  - Domains: >2 (configuration, networking/discovery, consensus message relay)
  - Flows: >3 (startup/listen, dialing/bootstrap, peer discovery, channel dispatch/receive, consensus relay delivery)

## Existing Implementations To Preserve
- P2P abstraction contracts in `crates/p2p` already define sender/receiver/provider behavior and channel constants; treat these as stable interfaces.
- Commonware transport/multiplexing capabilities are already present in vendor `commonware/p2p`; this effort must wire and consume them rather than redesign transport internals.

## Requirements
- REQ-1: The p2p-commonware builder path must apply validator seeding into the runtime discovery/admission path so configured validators are actually used at startup.
  - Touches: `crates/p2p-commonware` (primary), integration boundary with `crates/whirlpool-node`.
- REQ-2: The p2p-commonware builder path must populate and use bootstrap peers so nodes can discover remote peers beyond direct dial targets.
  - Touches: `crates/p2p-commonware` (primary), integration boundary with `crates/whirlpool-node`.
- REQ-3: P2P receive path must preserve channel metadata from commonware receiver into `NetworkMessage.channel` instead of hard-coding a single channel.
  - Touches: `crates/p2p-commonware` (primary), contract consumer boundary with `crates/consensus-simplex`.
- REQ-4: whirlpool-node configuration surface must accept explicit listen addresses, dial peers, and bootstrap peers from CLI/config rather than forcing ephemeral local-only defaults.
  - Touches: `crates/whirlpool-node` (primary), uses `crates/p2p-commonware` builder inputs.
- REQ-5: Node startup wiring must pass configured listen/dial/bootstrap/validator values into the P2P provider builder so multiple nodes can form a connected graph.
  - Touches: `crates/whirlpool-node` (primary), `crates/p2p-commonware`.
- REQ-6: consensus-simplex relay path must forward outbound consensus traffic over `NetworkSender` and deliver inbound `NetworkReceiver` messages into simplex channels.
  - Touches: `crates/consensus-simplex` (primary), `crates/p2p` contract boundary, `crates/p2p-commonware` channel mapping.
- REQ-7: Channel usage for vote/certificate/resolver must remain aligned with `crates/p2p` channel constants and not introduce incompatible channel IDs.
  - Touches: `crates/p2p`, `crates/consensus-simplex`, `crates/p2p-commonware`.
- REQ-8: Existing app-layer consensus adapter/finalization wiring must remain behaviorally compatible while P2P connectivity is enabled.
  - Touches: `crates/app`, integration boundary with `crates/consensus-simplex` and `crates/whirlpool-node`.

## Assumptions
- Ed25519 identities already used by node startup are the canonical peer identity inputs for commonware authentication.
- No transport-protocol redesign is needed; commonware TCP+authenticated handshake remains unchanged.
- Consensus message types already produced by simplex are sufficient for relay once network plumbing is enabled.

## Non-Goals
- Replacing `crates/p2p` trait contracts.
- Redesigning commonware vendor discovery/authentication internals.
- Changing application business logic in `crates/app` beyond compatibility safeguards.
- Producing synthesis/architecture flow documents during intake.

## Success Criteria
- A node can be started with user-specified listen, dial, and bootstrap settings and connect to at least one remote whirlpool-node.
- Discovery path can surface peers beyond static dial list when bootstrap peers are configured.
- Inbound messages preserve channel identity and route to appropriate simplex streams.
- Relay is no longer no-op: outbound consensus messages are sent over P2P and inbound consensus traffic reaches simplex processing.
