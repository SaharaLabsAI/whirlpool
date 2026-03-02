# whirlpool-node

## Purpose
The `whirlpool-node` crate serves as the entry point and orchestration layer for the Sahara EVM node. It defines the node binary, initializes the commonware runtime, wires together the consensus engine with the EVM application, and provides a development-ready state database bridge. <!-- GROUNDED -->

## Public API at a glance (crate root exports)
- `config`: Global constants for network identity and block timing. <!-- GROUNDED -->
- `block::EmptyBlock`: A minimal block representation used for stateless consensus verification. <!-- GROUNDED -->
- `app::EmptyBlockApp`: A stateless consensus application implementation for `EmptyBlock`. <!-- GROUNDED -->

## Modules
- `app`: Implements the `EmptyBlockApp` consensus logic. <!-- GROUNDED -->
- `block`: Defines the `EmptyBlock` type and its codec/cryptography trait implementations. <!-- GROUNDED -->
- `config`: Centralized configuration constants. <!-- GROUNDED -->

## Types & traits (public contract)
- `EmptyBlock`: Struct containing `height` (u64) and `parent_id` ([u8; 32]). <!-- GROUNDED -->
    - Implements `consensus::Block`, `commonware_codec::{EncodeSize, Read, Write}`, `commonware_cryptography::{Digestible, Committable}`, and `commonware_consensus::{Heightable, Block as VendorBlock}`. <!-- GROUNDED -->
- `EmptyBlockApp`: Unit struct implementing `consensus::ConsensusApp` for `EmptyBlock`. <!-- GROUNDED -->
- `TestStateDb`: Private struct in `main.rs` wrapping `InMemoryStateDb` to bridge between the state crate and `revm::Database`. <!-- GROUNDED -->

## Functions & macros
- `EmptyBlock::genesis()`: Returns a block at height 0 with a zeroed parent ID. <!-- GROUNDED -->
- `EmptyBlock::new(height, parent_id)`: Constructs a new block instance. <!-- GROUNDED -->
- `EmptyBlockApp::new()`: Constructs a new application instance. <!-- GROUNDED -->

## Config schema
Configuration is currently managed via public constants in `config.rs` and hardcoded values within `main.rs` that initialize the `CommonwareConfig` struct. <!-- GROUNDED -->

## Config defaults table
| Field | Type | Default | Source | Override path | Evidence |
|---|---|---|---|---|---|
| `NAMESPACE` | `&[u8]` | `b"sahara-chain-v0"` | `config.rs` | N/A | <!-- GROUNDED --> |
| `namespace` | `String` | `"sahara-chain-v0"` (from `config::NAMESPACE`) | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `BLOCK_INTERVAL` | `Duration` | `5s` | `config.rs` | N/A | <!-- GROUNDED --> |
| `BIND_ADDR` | `&str` | `"127.0.0.1:0"` | `config.rs` | N/A | <!-- GROUNDED --> |
| `VALIDATOR_SEED` | `u64` | `0` | `config.rs` | N/A | <!-- GROUNDED --> |
| `leader_timeout` | `Duration` | `5s` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `notarization_timeout` | `Duration` | `5s` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `nullify_retry` | `Duration` | `500ms` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `activity_timeout` | `u64` | `10` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `skip_timeout` | `u64` | `5` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `mailbox_size` | `usize` | `100` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `replay_buffer` | `NonZeroUsize` | `100` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `write_buffer` | `NonZeroUsize` | `100` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `epoch` | `u64` | `0` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `fetch_timeout` | `Duration` | `5s` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |
| `fetch_concurrent` | `usize` | `4` | `main.rs` | `CommonwareConfig` | <!-- GROUNDED --> |

## Provider interfaces & swap points
- `TxSource`: Provided by `app::NoopTxSource` in `main.rs`. <!-- GROUNDED -->
- `StateProvider`: Provided by `TestStateDb` in `main.rs`, bridging to `InMemoryStateDb`. <!-- GROUNDED -->
- `Database`: `revm::Database` implemented by `TestStateDb`. <!-- GROUNDED -->
- `ConsensusApp`: Orchestrated via `ApplicationAdapter<EvmApplication<TestStateDb>>`. <!-- GROUNDED -->

## Feature flags & cfg
- `test`: Used for unit tests in `block.rs` and `app.rs`. <!-- GROUNDED -->

## SemVer & stability
This crate is currently in a pre-alpha/development state. Internal APIs like `TestStateDb` are private to the binary and subject to change as the node architecture evolves. <!-- PROPOSED -->

## Primary flows

### Runtime Bootstrap
The `main` function initializes tracing and the `tokio::Runner`. Within the runner context, it builds the network provider, configures the consensus engine, and initializes the `EvmApplication` with a `TestStateDb`. The flow concludes by starting the `CommonwareEngine`. <!-- GROUNDED -->

### Block Proposal
Proposals are triggered by the consensus engine calling `ConsensusApp::propose`. In the current wiring, this delegates through `ApplicationAdapter` to `EvmApplication::propose`. <!-- GROUNDED -->
- **BLOCKER (INV-01)**: Currently only produces empty blocks without transaction execution. <!-- PROPOSED -->

### Block Verification
Verification occurs when the engine receives a block and calls `ConsensusApp::verify`. This delegates to `EvmApplication::verify` which checks the state root against the local database. <!-- GROUNDED -->
- **BLOCKER (INV-02)**: Verification is restricted to state root comparison and lacks transaction/receipt replay. <!-- PROPOSED -->

### Genesis Chain
The `genesis` flow initializes the chain state. `main.rs` calls `build_sahara_chain_spec` and provides it to construct `WhirlpoolEvmConfig`; the genesis state root comes from the DB via `StateProvider`, not from the chain spec. <!-- GROUNDED -->

## API omissions report
- **BLOCKER (INV-05)**: `finalize-to-commit` callback: There is no visible mechanism in the node wiring to trigger a state commit upon block finalization. <!-- PROPOSED -->
- **BLOCKER (INV-04)**: `Snapshot/Rollback`: The runtime wiring does not yet explicitly handle state snapshots or rollbacks for block verification failures. <!-- PROPOSED -->

## Open questions / TODOs
- **BLOCKER (INV-05)**: Implement a finalization callback that triggers `InMemoryStateDb::commit` when a block is notarized/finalized. <!-- PROPOSED -->
- **BLOCKER (INV-01)**: Integrate real transaction ingress to replace `NoopTxSource`. <!-- PROPOSED -->
- Enhance `TestStateDb` or replace it with a persistent storage solution as development progresses. <!-- PROPOSED -->
