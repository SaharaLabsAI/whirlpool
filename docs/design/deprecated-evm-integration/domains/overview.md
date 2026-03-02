# Domains

## Scope & method
- Evidence rule: every non-UNKNOWN claim cites a `path::Symbol` or a quoted snippet
- Domain assignment rule: 2 independent evidence points minimum

## Domain map

| Domain | Summary | Owning crates | Key public entrypoints | Evidence |
|---|---|---|---|---|
| Consensus | Block ordering, finalization, fault detection | `consensus`, `consensus-simplex` | `ConsensusApp`, `ConsensusEngine`, `Block` trait, `ConsensusEvent` | `crates/consensus/src/app.rs::ConsensusApp`, `crates/consensus/src/engine.rs::ConsensusEngine` |
| Application | Block proposal, verification, state transitions | `app` [PROPOSED], `whirlpool-node` (current) | `Application` trait [PROPOSED], `EmptyBlockApp` (current) | `crates/whirlpool-node/src/app.rs::EmptyBlockApp`, `crates/consensus/src/app.rs::ConsensusApp` |
| EVM Execution | EVM-based transaction execution, block building | `app-evm` [PROPOSED] | `WhirlpoolEvmConfig` [PROPOSED] | `vendor/reth/crates/evm/evm/src/lib.rs::ConfigureEvm`, `vendor/reth/crates/ethereum/evm/src/lib.rs::EthEvmConfig` |
| Networking | P2P message transport, peer management | `p2p`, `p2p-commonware` | Network provider traits | `crates/p2p/src/lib.rs`, `crates/p2p-commonware/src/lib.rs` |
<!-- continuation round 2 -->
| State Storage | In-memory EVM state database, BundleState commitment, state root computation | `state` [PROPOSED] | `InMemoryStateDb` [PROPOSED] | round 2 |
