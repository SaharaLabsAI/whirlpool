use app::tx_source::InMemoryTxPool;
use state::StateDb;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

use super::receipt_store::ReceiptStore;

/// Shared context for the Ethereum JSON-RPC server.
///
/// Holds references to the node's transaction pool, state database,
/// receipt store, and chain metadata needed by RPC method handlers.
#[derive(Clone)]
pub struct EthRpcContext<S: StateDb> {
    pub tx_pool: Arc<InMemoryTxPool>,
    pub state_db: Arc<RwLock<S>>,
    pub receipt_store: Arc<ReceiptStore>,
    pub chain_id: u64,
    pub block_height: Arc<AtomicU64>,
}

impl<S: StateDb> EthRpcContext<S> {
    pub fn new(
        tx_pool: Arc<InMemoryTxPool>,
        state_db: Arc<RwLock<S>>,
        chain_id: u64,
    ) -> Self {
        Self {
            tx_pool,
            state_db,
            receipt_store: Arc::new(ReceiptStore::new()),
            chain_id,
            block_height: Arc::new(AtomicU64::new(0)),
        }
    }
}
