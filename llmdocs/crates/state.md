# state

## Purpose
In-memory EVM state database implementing revm's Database trait for testing and fast execution.

## Key Types
- `InMemoryStateDb`: HashMap-based storage for accounts, bytecodes, and block hashes. Implements `revm::Database` and `revm::DatabaseRef`.
- `DbAccount`: Container for account information (`AccountInfo`) and storage slots.
- `StateError`: Error type for database operations, implements `revm::database::DBErrorMarker`.

## Key Functions
- `InMemoryStateDb::new()`: Creates an empty database instance.
- `InMemoryStateDb::with_genesis(alloc)`: Initializes database from a genesis allocation map.
- `InMemoryStateDb::commit(bundle)`: Processes `BundleState` changes from revm into the database.
- `InMemoryStateDb::state_root()`: Computes a deterministic flat keccak256 hash of the entire state.
- `InMemoryStateDb::insert_block_hash(number, hash)`: Records a block hash for the `BLOCKHASH` opcode.

## Dependencies
- `revm`: Core EVM implementation traits and primitives.
- `alloy-genesis`: Genesis account types.
- `thiserror`: Error derivation.

## Status
Complete. Fully implemented with unit test coverage for all core functionality.
