# state-reth crate — Persistent Block Storage Contract

## Purpose

**Today**: Implements `StateDb` trait for `RethStateDb` using MDBX backend (`reth-db`). Provides persistent account state, storage, bytecode, and block hash storage. Each method opens a short-lived MDBX transaction. Tables used: `PlainAccountState`, `HashedAccounts`, `HashedStorages`, `PlainStorageState`, `Bytecodes`, `CanonicalHeaders`.

**Changes**: Implement the new `BlockStorage` trait for `RethStateDb`, adding block persistence and retrieval using existing MDBX tables that are already created by `init_db()` but currently unused: `Headers`, `HeaderNumbers`, `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, `TransactionBlocks`, `Receipts`.

## Public API Changes

### New file: `src/block_storage.rs`

```rust
use alloy_consensus::Receipt;
use alloy_primitives::B256;
use app::EvmBlock;
use reth_db::Database;
use reth_db_api::transaction::{DbTx, DbTxMut};
use reth_primitives_traits::Header;
use state::BlockStorage;

use crate::db::RethStateDb;
use crate::error::RethStateError;

impl BlockStorage for RethStateDb {
    type Error = RethStateError;

    fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), RethStateError> {
        // 1. Convert EvmBlock -> Header via app_evm::executor::build_header_from_evm_block()
        // 2. Compute header hash via header.hash_slow()
        // 3. Decode transactions via app_evm::executor::decode_transactions()
        // 4. Compute next TxNumber from last BlockBodyIndices entry
        // 5. Open single MDBX write transaction
        // 6. Write to tables:
        //    - Headers(block.height -> Header)
        //    - HeaderNumbers(header_hash -> block.height)
        //    - BlockBodyIndices(block.height -> StoredBlockBodyIndices { first_tx_num, tx_count })
        //    - Transactions(tx_number -> TransactionSigned) for each tx
        //    - TransactionHashNumbers(tx_hash -> tx_number) for each tx
        //    - TransactionBlocks(first_tx_num -> block.height)
        //    - Receipts(tx_number -> Receipt) for each receipt
        // 7. Commit write transaction
        todo!()
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, RethStateError> {
        // 1. Open MDBX read transaction
        // 2. Read Headers[number] -> Header (return None if missing)
        // 3. Read BlockBodyIndices[number] -> { first_tx_num, tx_count }
        // 4. Read Transactions[first_tx_num..first_tx_num+tx_count] -> Vec<TransactionSigned>
        // 5. RLP-encode each TransactionSigned -> Vec<Vec<u8>> for EvmBlock.transactions
        // 6. Reconstruct EvmBlock from Header fields + encoded transactions
        // 7. Return Some(EvmBlock)
        todo!()
    }

    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>, RethStateError> {
        // 1. Open MDBX read transaction
        // 2. Read HeaderNumbers[B256::from(hash)] -> BlockNumber (return None if missing)
        // 3. Delegate to get_block_by_number(block_number)
        todo!()
    }

    fn get_receipts_by_block(&self, number: u64) -> Result<Vec<Receipt>, RethStateError> {
        // 1. Open MDBX read transaction
        // 2. Read BlockBodyIndices[number] -> { first_tx_num, tx_count } (return empty if missing)
        // 3. Read Receipts[first_tx_num..first_tx_num+tx_count] -> Vec<Receipt>
        // 4. Return receipts
        todo!()
    }
}
```

### Modified file: `src/lib.rs`

```rust
pub mod block_storage;  // NEW
pub mod codec;
pub mod db;
pub mod error;
pub mod init;
pub mod tables;
pub mod trie;

pub use db::RethStateDb;
pub use error::RethStateError;
pub use init::open_state_db;
```

### Modified file: `src/tables.rs`

```rust
// Add re-exports for block storage tables
pub use reth_db_api::tables::{
    // Existing
    Bytecodes, CanonicalHeaders, HashedAccounts, HashedStorages,
    PlainAccountState, PlainStorageState,
    // NEW — for block storage
    BlockBodyIndices, HeaderNumbers, Headers, Receipts,
    TransactionBlocks, TransactionHashNumbers, Transactions,
};
```

## Internal Changes

### Transaction Numbering Strategy

Global append-only `TxNumber` counter derived from `BlockBodyIndices`:

```rust
/// Compute the next available TxNumber by reading the last block's body indices.
fn next_tx_number(tx: &impl DbTx) -> Result<u64, RethStateError> {
    // Walk BlockBodyIndices table in reverse to find last entry
    // next_tx_num = last_entry.first_tx_num + last_entry.tx_count
    // If table is empty, return 0
    todo!()
}
```

### EvmBlock Reconstruction (read path)

Convert reth `Header` back to `EvmBlock` fields:

```rust
/// Reconstruct an EvmBlock from a reth Header and raw transaction bytes.
fn reconstruct_evm_block(
    header: &Header,
    raw_transactions: Vec<Vec<u8>>,
) -> EvmBlock {
    EvmBlock {
        height: header.number,
        parent_id: header.parent_hash.0,
        state_root: header.state_root.0,
        transactions_root: header.transactions_root.0,
        receipts_root: header.receipts_root.0,
        gas_used: header.gas_used,
        timestamp: header.timestamp,
        transactions: raw_transactions,
    }
}
```

### TransactionSigned to raw bytes (read path)

```rust
use alloy_eips::eip2718::Encodable2718;

/// Encode a TransactionSigned back to raw bytes for EvmBlock.transactions.
fn encode_transaction(tx: &TransactionSigned) -> Vec<u8> {
    let mut buf = Vec::new();
    tx.encode_2718(&mut buf);
    buf
}
```

### Write Path (single MDBX transaction)

All table writes for a single block are batched in one MDBX write transaction following the existing pattern in `RethStateDb::commit()` which opens `self.db.tx_mut()`, performs all writes, then calls `tx.commit()`.

### Error Handling

Reuses existing `RethStateError` enum — all MDBX operations already map to `RethStateError::Database(DatabaseError)`. No new error variants needed.

## Dependencies

### No new dependencies

All required crates are already in `Cargo.toml`:

- `reth-db` (MDBX backend) — existing
- `reth-db-api` (table definitions, `DbTx`, `DbTxMut`) — existing
- `reth-primitives-traits` (`Header`, `StoredBlockBodyIndices`) — existing
- `alloy-primitives` (`B256`) — existing
- `state` (for `BlockStorage` trait) — existing

### New internal usage

- `app-evm` (for `build_header_from_evm_block` and `decode_transactions` conversion functions)

Add to `Cargo.toml`:

```toml
[dependencies]
app = { path = "../app" }        # For EvmBlock type
app-evm = { path = "../app-evm" }  # For build_header_from_evm_block, decode_transactions
alloy-consensus = "1.4.3"       # For Receipt type
alloy-eips = "1.4.3"            # For Encodable2718 (TransactionSigned -> raw bytes)
reth-ethereum-primitives = { path = "../../vendor/reth/crates/ethereum/primitives" }  # For TransactionSigned
```

## Error Types

No new error variants. Existing `RethStateError` covers all cases:

| Variant | Usage in BlockStorage |
|---------|----------------------|
| `Database(DatabaseError)` | All MDBX read/write failures |
| `Codec(String)` | Transaction decode/encode failures |
| `Init(String)` | Not used by BlockStorage |
| `StateRoot(String)` | Not used by BlockStorage |

## Test Surface

### Unit tests (in `src/block_storage.rs`)

1. **Round-trip: store and retrieve by number** — Store a block with receipts, retrieve by number, verify all EvmBlock fields match
2. **Round-trip: store and retrieve by hash** — Store a block, retrieve by header hash, verify match
3. **Receipts round-trip** — Store block with N receipts, `get_receipts_by_block()` returns exactly N receipts in order
4. **Missing block returns None** — `get_block_by_number(999)` returns `Ok(None)` on empty database
5. **Missing hash returns None** — `get_block_by_hash(&[0u8; 32])` returns `Ok(None)` on empty database
6. **Empty receipts** — Store block with 0 transactions/receipts, verify retrieval works
7. **Multiple blocks sequential** — Store blocks 1, 2, 3 sequentially, verify TxNumber continuity across blocks
8. **Transaction numbering** — Verify `BlockBodyIndices` correctly tracks `first_tx_num` and `tx_count` across multiple blocks

### Integration tests

- Propose block via `EvmApplication`, extract receipts, call `store_block`, then `get_block_by_number` — verify round-trip fidelity

## Integration Points

| Connected Crate | Direction | Interface | Data Flow |
|-----------------|-----------|-----------|-----------|
| `state` | Implements | `BlockStorage` trait | Trait definition |
| `app-evm` | Called by | `store_block()` | EvmApp finalization handler calls `state_db.store_block(&block, &receipts)` |
| `app-evm` | Imports from | `build_header_from_evm_block()`, `decode_transactions()` | Conversion functions for EvmBlock -> reth types |
| `rpc-eth` | Called by | `get_block_by_number()`, `get_block_by_hash()`, `get_receipts_by_block()` | RPC endpoints query persistent block data |
| `whirlpool-node` | Wired by | `RethStateDb` instance | Shared `Arc<RwLock<RethStateDb>>` implements both `StateDb` and `BlockStorage` |

**MDBX Tables Used** (all created by `init_db()`, currently empty):

| Table | Key | Value | Operation |
|-------|-----|-------|-----------|
| `Headers` | `BlockNumber` (u64) | `Header` (Compact) | Write in `store_block`, Read in `get_block_by_number` |
| `HeaderNumbers` | `BlockHash` (B256) | `BlockNumber` (u64) | Write in `store_block`, Read in `get_block_by_hash` |
| `BlockBodyIndices` | `BlockNumber` (u64) | `StoredBlockBodyIndices` | Write in `store_block`, Read in all getters |
| `Transactions` | `TxNumber` (u64) | `TransactionSigned` (Compact) | Write in `store_block`, Read in `get_block_by_number` |
| `TransactionHashNumbers` | `TxHash` (B256) | `TxNumber` (u64) | Write in `store_block` |
| `TransactionBlocks` | `TxNumber` (u64) | `BlockNumber` (u64) | Write in `store_block` |
| `Receipts` | `TxNumber` (u64) | `Receipt` (Compact) | Write in `store_block`, Read in `get_receipts_by_block` |

**Source**: STRATEGY.md Stream 1, CRATES.md state-reth section, DOMAINS.md Integration Point 2
