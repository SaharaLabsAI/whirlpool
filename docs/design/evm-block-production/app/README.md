# app

## Purpose
This crate defines the abstract interface between the consensus engine and the execution environment. It provides core types for block representation and execution results, along with traits for transaction sourcing and application logic. A primary role is providing the `ApplicationAdapter`, which bridges generic application logic to the `ConsensusApp` requirements of the underlying consensus engine. <!-- GROUNDED -->

## Public API at a glance (crate root exports)
- `traits::{Application, TxSource, NoopTxSource}` <!-- GROUNDED -->
- `types::{EvmBlock, ExecutionResult}` <!-- GROUNDED -->
- `adapter::ApplicationAdapter` <!-- GROUNDED -->
- `error::ApplicationError` <!-- GROUNDED -->

## Modules
- `adapter`: Bridges `Application` to `ConsensusApp`. <!-- GROUNDED -->
- `error`: Defines the `ApplicationError` enum. <!-- GROUNDED -->
- `traits`: Contains `Application` and `TxSource` trait definitions. <!-- GROUNDED -->
- `types`: Defines `EvmBlock` and `ExecutionResult` structures. <!-- GROUNDED -->

## Types & traits (public contract)

### `Application` trait
Defines the lifecycle methods for a blockchain application. <!-- GROUNDED -->
- **Constraints**: `Send + Sync + Clone + 'static` <!-- GROUNDED -->
- **Associated Types**:
  - `Block`: Must implement `consensus::Block`. <!-- GROUNDED -->
  - `Result`: Must implement `Clone + Send`. <!-- GROUNDED -->
  - `Error`: Must implement `std::error::Error + Send + Sync`. <!-- GROUNDED -->
- **Methods**:
  - `genesis() -> Block`: Returns the initial block. <!-- GROUNDED -->
  - `propose(parent: &Block, height: u64) -> Result<(Block, Result), Error>`: Produces a new block candidate and its execution result. <!-- GROUNDED -->
  - `verify(parent: &Block, block: &Block) -> Result<Result, Error>`: Validates a block candidate and returns its execution result. <!-- GROUNDED -->

### `TxSource` trait
Provides an interface for retrieving pending transactions. <!-- GROUNDED -->
- `pending() -> Vec<Vec<u8>>`: Returns a list of opaque transaction bytes. <!-- GROUNDED -->
- `NoopTxSource`: A default implementation that always returns an empty list. <!-- GROUNDED -->

### `EvmBlock` struct
The primary block representation for the EVM application. <!-- GROUNDED -->
- **Fields**: `height` (u64), `parent_id` ([u8;32]), `state_root` ([u8;32]), `transactions_root` ([u8;32]), `receipts_root` ([u8;32]), `gas_used` (u64), `timestamp` (u64), `transactions` (Vec<Vec<u8>>). <!-- GROUNDED -->
- **Integrations**: Implements `CoreBlock`, `Codec`, `Digestible`, `Committable`, `Heightable`, and `VendorBlock`. <!-- GROUNDED -->

### `ExecutionResult` struct
Captures the side effects of block execution. <!-- GROUNDED -->
- **Fields**: `state_root` ([u8;32]), `receipts_root` ([u8;32]), `gas_used` (u64), `receipt_count` (usize). <!-- GROUNDED -->

### `ApplicationAdapter<A>` struct
Wraps an `Application` implementation where `Block = EvmBlock` to satisfy consensus engine requirements. <!-- GROUNDED -->
- **Implementation**: Maps `propose` results (Ok → Some, Err → None) and `verify` results (Ok → Ok, Err → `ConsensusError::InvalidBlock`). <!-- GROUNDED -->

## Functions & macros
- `EvmBlock::compute_id() -> [u8;32]`: Computes block hash via SHA-256 over height, parent ID, state root, and transactions root. <!-- GROUNDED -->

## Config schema
This crate has NO config schema as it consists of pure trait and type definitions. <!-- GROUNDED -->

## Config defaults table
| Field | Type | Default | Source | Override path | Evidence |
|---|---|---|---|---|---|
| N/A | N/A | N/A | N/A | N/A | No config schema defined <!-- GROUNDED --> |

## Provider interfaces & swap points
- `TxSource`: Pluggable transaction pool interface. Current default is `NoopTxSource`. <!-- GROUNDED -->
- `Application`: Pluggable execution logic. Usually implemented by `app-evm`. <!-- GROUNDED -->

## Feature flags & cfg
None defined. <!-- GROUNDED -->

## SemVer & stability
- **Status**: Alpha / Internal. <!-- PROPOSED -->
- **Breaking Changes**: Any modification to `EvmBlock` fields or `Application` trait signatures will break downstream crates `app-evm` and `whirlpool-node`. <!-- GROUNDED -->

## Primary flows

### Genesis Initialization
The `ApplicationAdapter` receives a call to `genesis()`, which it delegates to the inner `Application`. This returns the initial `EvmBlock` required by the consensus engine. <!-- GROUNDED -->

### Block Proposal
1. Consensus engine calls `ApplicationAdapter::propose`. <!-- GROUNDED -->
2. Adapter delegates to `inner.propose(parent, height)`. <!-- GROUNDED -->
3. Execution logic (e.g., in `app-evm`) pulls transactions from `TxSource`. <!-- GROUNDED -->
4. Resulting `(EvmBlock, ExecutionResult)` is returned. <!-- GROUNDED -->
5. Adapter strips the result and returns `Some(EvmBlock)` to consensus. <!-- GROUNDED -->

### Block Verification
1. Consensus engine calls `ApplicationAdapter::verify(parent, block)`. <!-- GROUNDED -->
2. Adapter delegates to `inner.verify(parent, block)`. <!-- GROUNDED -->
3. Verification logic re-executes or validates state roots. <!-- GROUNDED -->
4. On success, `Ok(())` is returned to consensus. On failure, the error is converted to `ConsensusError::InvalidBlock`. <!-- GROUNDED -->

## API omissions report
- `ExecutionResult` does not currently include logs or bloom filters, which are standard for EVM execution. <!-- PROPOSED -->
- `EvmBlock` lacks a `beneficiary` or `coinbase` field, though these might be handled at the state layer. <!-- GROUNDED -->

## Open questions / TODOs
- BLOCKER: The `ApplicationAdapter` currently discards the `ExecutionResult` during proposal, which may prevent the node from caching execution side effects (like state changes) until finalization. <!-- GROUNDED -->
- UNKNOWN: How the `Application` trait will handle asynchronous execution or state pre-fetching in future iterations. <!-- PROPOSED -->
- TODO: Add support for more complex `TxSource` implementations with priority or gas-price awareness. <!-- PROPOSED -->
- **UNKNOWN (INV-07)**: The `TxSource::pending()` interface provides no ordering guarantees; deterministic ordering policy for proposals is undefined. <!-- PROPOSED -->
