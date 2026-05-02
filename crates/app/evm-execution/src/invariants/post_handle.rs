//! Post-handle-owned receipt lifecycle invariant map.
//!
//! `ReceiptStore` owns staged/pending receipt state and finalization
//! persistence. This module keeps tiny pure predicates and names lifecycle
//! invariants without taking over storage or lock ownership.
//!
//! Invariants that stay in `ReceiptStore` orchestration:
//! - failed persistence must not clear staged receipts;
//! - successful finalization clears only the staged and pending receipts that
//!   match the finalized block;
//! - pending receipt visibility mirrors the latest staged proposal/verification
//!   result until the matching finalization clears it.

/// Staged receipts are bound to the finalized block identity they were created
/// for: height, parent id, and computed block id must all match.
pub fn staged_receipts_match_block_identity(
    staged_height: u64,
    staged_parent_id: [u8; 32],
    staged_block_id: [u8; 32],
    block_height: u64,
    block_parent_id: [u8; 32],
    block_id: [u8; 32],
) -> bool {
    staged_height == block_height
        && staged_parent_id == block_parent_id
        && staged_block_id == block_id
}

/// A finalized block may be persisted without staged receipts only for the
/// explicit empty-block fallback.
pub fn can_store_without_staged_receipts(transaction_count: usize) -> bool {
    transaction_count == 0
}
