# app

## Purpose
Abstract application layer bridging consensus to EVM execution. It defines interfaces for block proposal/verification and transaction sourcing.

## Key Types
- `EvmBlock`: Block type implementing `consensus::Block` and commonware codec/commitment traits.
- `ExecutionResult`: Block execution output (state roots, gas usage).
- `ApplicationAdapter`: Adapts `Application` to `consensus::ConsensusApp`.
- `ApplicationError`: Application-layer error type.
- `TxSource`: Transaction source trait exposing `pending()`.
- `NoopTxSource`: `TxSource` implementation that always returns no transactions.
- `InMemoryTxPool`: Mutex-backed in-memory `TxSource` implementation that stores raw tx bytes.

## Key Methods
- `Application::propose(parent, height)`: Produces the next block and execution result.
- `Application::verify(parent, block)`: Verifies a proposed block and returns execution result.
- `ApplicationAdapter::new(app)`: Wraps an `Application` implementation for consensus wiring.
- `InMemoryTxPool::new()`: Creates an empty tx pool.
- `InMemoryTxPool::push(tx)`: Appends a raw EIP-2718 transaction.
- `InMemoryTxPool::pending()`: Drains and returns queued transactions (FIFO, at-most-once delivery).

## Exports
- `crates/app/src/lib.rs:8` re-exports `InMemoryTxPool` with `Application`, `NoopTxSource`, and `TxSource`.

## Dependencies
- `consensus`: Core consensus traits (`Block`, `ConsensusApp`).
- `commonware-codec`: Serialization traits.
- `commonware-cryptography`: Hashing and commitment traits.
- `commonware-consensus`: Vendor consensus traits.

## Status
Complete. Provides consensus/execution bridge traits plus a usable in-memory tx source for node wiring.
