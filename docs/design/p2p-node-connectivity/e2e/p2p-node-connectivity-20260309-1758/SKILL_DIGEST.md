# SKILL_DIGEST

## Grounded
- **Workspace**: Whirlpool modular consensus framework at `/home/dev/sahara/web3/agent/playground/whirlpool` (cite: `agent-docs/index.md`)
- **P2P trait crate**: `crates/p2p` defines `PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider` traits (cite: `crates/p2p/src/traits.rs`)
- **P2P impl crate**: `crates/p2p-commonware` bridges to Commonware vendor; `CommonwareNetworkProviderBuilder` + `CommonwareNetworkProvider` (cite: `crates/p2p-commonware/src/provider.rs`)
- **Gap: Validator seeding**: `initial_validators` stored but never applied to oracle (cite: `crates/p2p-commonware/src/provider.rs`)
- **Gap: Bootstrap peers**: listen/dial addresses ephemeral (127.0.0.1:0), bootstrappers never populated (cite: `crates/whirlpool-node/src/main.rs`)
- **Gap: Channel metadata**: `CommonwareReceiver` hard-codes Channel(0), loses real channel info (cite: `crates/p2p-commonware/src/receiver.rs`)
- **Gap: Relay no-op**: consensus-simplex relay not implemented for multi-node (cite: `crates/consensus-simplex/`)
- **Commonware vendor**: TCP+ChaCha20-Poly1305 transport, bootstrapper-based discovery, Manager/Blocker patterns, muxed channels (cite: `vendor/commonware/p2p/`)

## [PROPOSED]
- Sub-Intent B: Node Config & Startup Wiring (REQ-4, REQ-5) — pending design
- Sub-Intent C: Consensus Relay Activation (REQ-6, REQ-7, REQ-8) — pending design

## Unknowns
- Exact CLI framework used by whirlpool-node (clap? structopt? custom?)
- Whether NAT traversal is needed for initial scope
