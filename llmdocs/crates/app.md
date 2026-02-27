# app

## Purpose
Abstract application layer bridging consensus to EVM execution. It defines the interface for state transitions and block validation.

## Key Types
- `EvmBlock`: Custom block type implementing `consensus::Block` and several commonware traits (`CodecWrite`, `CodecRead`, `Digestible`, `Committable`, `Heightable`).
- `ExecutionResult`: Outcome of block execution, containing state roots and gas usage.
- `ApplicationAdapter`: Adapts the generic `Application` trait to the concrete `consensus::ConsensusApp` trait.
- `ApplicationError`: Error type for application-layer operations.
- `NoopTxSource`: A default implementation of `TxSource` that returns no transactions.

## Key Functions
- `Application::propose()`: Produces a new block and its execution result given a parent and height.
- `Application::verify()`: Validates a block against its parent and returns the execution result.
- `ApplicationAdapter::new(app)`: Wraps an `Application` instance for use with the consensus engine.

## Dependencies
- `consensus`: Core consensus traits (`Block`, `ConsensusApp`).
- `commonware-codec`: Serialization traits.
- `commonware-cryptography`: Hashing and commitment traits.
- `commonware-consensus`: Vendor-specific consensus traits.

## Status
Complete. Defines the core abstractions for the bridge between consensus and execution layers.
