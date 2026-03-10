# Exploration: Commonware Vendor P2P

## Core Traits (vendor/commonware/p2p)
- `Sender`: send(recipients, data, priority) — Recipients::All/Some/One
- `Receiver`: recv() -> (PublicKey, Bytes) — authenticated messages
- `Manager`: manage peer sets by epoch, subscribe to updates
- `Blocker`: disconnect/ban malicious peers
- `Channel`: u64 for multiplexing protocols

## Connection Model
- **Transport**: TCP/IP via commonware-runtime (tokio prod)
- **Handshake**: 3-way mutual auth (Syn→SynAck→Ack) with Ed25519 static + X25519 ephemeral
- **Encryption**: ChaCha20-Poly1305 authenticated encryption
- **Identity**: Cryptographic PublicKey, persistent across sessions

## Peer Discovery
- **Discovery mode**: bootstrapper-based — requires synchronized peer sets, for unknown networks
- **Lookup mode**: direct connection to known addresses, no bootstrap needed
- **Manager integration**: peer sets updated by epoch via Manager::update(), subscribers notified

## Message Primitives
- Muxer: isolated sub-channels per protocol
- Rate limiting: LimitedSender with per-second quotas
- Priority: boolean flag for high-priority bypass
- Reliability: messages dropped if recipient offline (app responsibility)
- Framing: length-prefixed, configurable size limits

## Consensus Integration
- Resolver pattern for fetching blocks/data
- Takes Manager + Blocker + Sender/Receiver pairs
- Consensus engines register dedicated channels
