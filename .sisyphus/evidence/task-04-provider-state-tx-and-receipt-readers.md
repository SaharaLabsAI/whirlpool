# Task 04 Evidence: Provider state, tx, and receipt readers

## Summary

Replaced noop stubs for TransactionsProvider and ReceiptProvider in
`crates/rpc-eth/src/provider.rs` with real MDBX-backed implementations.
StateProviderFactory kept as self.clone() pattern (returns latest state).

## Changes

### Modified files
- **`crates/rpc-eth/src/provider.rs`** — Real impls for TransactionsProvider, ReceiptProvider

### Traits implemented (real MDBX reads)
| Trait | Methods |
|-------|---------|
| TransactionsProvider | transaction_id, transaction_by_id, transaction_by_id_unhashed, transaction_by_hash, transaction_by_hash_with_meta, transaction_block, transactions_by_block, transactions_by_block_range, transactions_by_tx_range, senders_by_tx_range, transaction_sender |
| ReceiptProvider | receipt, receipt_by_hash, receipts_by_block, receipts_by_tx_range |
| StateProviderFactory | (kept as self.clone() — returns latest state for all methods) |

### Data access pattern
Tables used: Transactions, TransactionHashNumbers, TransactionBlocks,
Receipts, BlockBodyIndices (+ CanonicalHeaders, Headers for metadata).
Sender recovery via SignerRecoverable trait on TransactionSigned.

## Verification

- `cargo build -p rpc-eth`: **PASS**
- `cargo test -p rpc-eth --lib`: **PASS** (17/17 tests)
- `cargo test -p rpc-eth --test provider_contract`: **PASS** (1/1 test)
- No vendor files modified

## Artifact Coverage
- REQ-2: ✅ Provider reads transactions and receipts from storage
- TST-1: ✅ Provider contract test still passes

## Timestamp
2026-03-11T07:45:00Z
