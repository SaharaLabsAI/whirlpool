# Shared Crate Index

## In-Scope Crates

| Crate | Path | Purpose | Change Type |
|-------|------|---------|-------------|
| consensus-simplex | crates/consensus-simplex | Simplex BFT engine wiring | Major (replace stub) |
| p2p-commonware | crates/p2p-commonware | Commonware P2P network provider | Moderate (expose channel pairs) |
| whirlpool-node | crates/whirlpool-node | Node binary | Minor (pass runtime context) |

## Adjacent Crates (read-only reference)

| Crate | Path | Purpose | Relevance |
|-------|------|---------|-----------|
| consensus | crates/consensus | Core consensus traits | Defines ConsensusEngine, ConsensusApp |
| app | crates/app | Application adapter layer | ApplicationAdapter wraps EvmApplication |
| app-evm | crates/app-evm | EVM execution engine | EvmApplication (propose/verify) |
| p2p | crates/p2p | P2P trait layer | Defines NetworkProvider, Channel |
| state | crates/state | State database | InMemoryStateDb for EVM |

## Vendor Dependencies

| Vendor | Path | Usage |
|--------|------|-------|
| commonware-consensus | vendor/commonware/consensus | `simplex::Engine`, `simplex::Config` |
| commonware-runtime | vendor/commonware/runtime | `tokio::Runner`, Spawner/Clock/Metrics traits |
| commonware-p2p | vendor/commonware/p2p | `discovery::Network`, `Blocker` trait, `Oracle` |
| commonware-parallel | vendor/commonware/parallel | `Sequential` (Strategy trait impl) |
| commonware-cryptography | vendor/commonware/cryptography | `ed25519`, `Digest`, `Digestible` |
