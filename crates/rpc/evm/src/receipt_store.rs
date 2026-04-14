use alloy_primitives::B256;
use alloy_rpc_types::TransactionReceipt;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory receipt store for confirmed transactions.
///
/// Maps transaction hash -> receipt. Thread-safe via `RwLock`.
pub struct ReceiptStore {
    inner: RwLock<HashMap<B256, TransactionReceipt>>,
}

impl ReceiptStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// Look up a receipt by transaction hash.
    pub fn get(&self, tx_hash: &B256) -> Option<TransactionReceipt> {
        let store = self.inner.read().expect("receipt store poisoned");
        store.get(tx_hash).cloned()
    }
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::new()
    }
}
