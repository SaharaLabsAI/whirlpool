# Crate: mempool

Persistent transaction pool implementation using MDBX.

## Overview

The `mempool` crate provides a persistent storage layer for transactions, ensuring they survive node restarts. It implements the `TxSource` trait from the `app-traits` crate.

## Key Components

### MempoolStore
Raw MDBX wrapper for transaction storage.
- Path: `crates/mempool/traits/src/store.rs`
- Backend: `reth-libmdbx`
- Key Format: `u64` big-endian (FIFO ordering)
- Primary API: `put(key, value)`, `get(key)`, `delete(key)`, `iter()`

### PersistentTxPool
High-level transaction pool implementation.
- Path: `crates/mempool/traits/src/persistent.rs`
- Traits: Implements `TxSource`
- Storage: Uses `MempoolStore`
- Data Directory: Defaulted to `data/mempool` in `whirlpool-node`

### MempoolError
Error types for mempool operations.
- Path: `crates/mempool/traits/src/error.rs`

## Integration

- Used by `whirlpool-node` to provide persistent transaction sourcing.
- Replaces `InMemoryTxPool` in production configurations.

## Testing

- Unit tests: `crates/mempool/traits/src/store.rs`, `crates/mempool/traits/src/persistent.rs`
- Integration tests: `crates/mempool/traits/tests/integration.rs`
