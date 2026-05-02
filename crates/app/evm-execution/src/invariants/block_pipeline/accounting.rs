//! Pure accounting-shape predicates for block-pipeline invariant checks.
//!
//! These helpers do not own fee accounting semantics. They keep tiny reusable
//! predicates discoverable for reviewers while `block_pipeline::accounting`
//! remains the accounting owner.

/// Priority-fee aggregation is defined only when executed transactions and
/// receipt-derived gas deltas describe the same execution set.
pub fn tx_and_gas_delta_counts_match(tx_count: usize, gas_delta_count: usize) -> bool {
    tx_count == gas_delta_count
}

/// Receipt cumulative gas must be monotonic so adjacent cumulative values can
/// produce a per-transaction gas delta.
pub fn cumulative_gas_delta(previous: u64, current: u64) -> Option<u64> {
    current.checked_sub(previous)
}
