use app::tx_source::InMemoryTxPool;
use state::{BlockStorage, StateDb};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

use crate::receipt_store::ReceiptStore;

/// Shared context for the Ethereum JSON-RPC server.
///
/// Holds references to the node's transaction pool, state database,
/// block storage, receipt store, and chain metadata needed by RPC method handlers.
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    pub tx_pool: Arc<InMemoryTxPool>,
    pub state_db: Arc<RwLock<S>>,
    pub block_storage: Arc<B>,
    pub receipt_store: Arc<ReceiptStore>,
    pub chain_id: u64,
    pub block_height: Arc<AtomicU64>,
}

// Manual Clone impl to avoid requiring B: Clone (since B is behind Arc)
impl<S: StateDb, B: BlockStorage> Clone for EthRpcContext<S, B> {
    fn clone(&self) -> Self {
        Self {
            tx_pool: self.tx_pool.clone(),
            state_db: self.state_db.clone(),
            block_storage: self.block_storage.clone(),
            receipt_store: self.receipt_store.clone(),
            chain_id: self.chain_id,
            block_height: self.block_height.clone(),
        }
    }
}

impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<InMemoryTxPool>,
        state_db: Arc<RwLock<S>>,
        block_storage: Arc<B>,
        chain_id: u64,
    ) -> Self {
        Self {
            tx_pool,
            state_db,
            block_storage,
            receipt_store: Arc::new(ReceiptStore::new()),
            chain_id,
            block_height: Arc::new(AtomicU64::new(0)),
        }
    }
}
