// Trie computation module.
//
// Provides state root calculation using reth's trie infrastructure.
// This uses the real Merkle Patricia Trie rather than the simplified
// keccak256-over-sorted-data approach used by state-memory.

use alloy_primitives::B256;
use reth_db_api::transaction::DbTx;
use reth_trie::StateRoot;
use reth_trie_db::DatabaseStateRoot;

use crate::error::RethStateError;

/// Compute the state root from the current database contents.
///
/// Uses `reth_trie::StateRoot` to compute a proper Merkle Patricia Trie root
/// from the HashedAccounts, HashedStorages, AccountsTrie, and StoragesTrie tables.
pub fn compute_state_root(tx: &impl DbTx) -> Result<B256, RethStateError> {
    StateRoot::from_tx(tx)
        .root()
        .map_err(|e| RethStateError::StateRoot(e.to_string()))
}
