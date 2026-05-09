//! Compatibility facade for fee-pool storage helpers.
//!
//! The canonical claim-ledger storage owner is `fee_pool::claim_ledger`.
//! This module preserves the public `fee_pool::storage::claimable_balance_slot`
//! path and gives reviewers a stable storage-oriented start path without
//! introducing a second storage authority.

pub use crate::fee_pool::claim_ledger::claimable_balance_slot;
