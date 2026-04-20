use super::*;

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn new(
        evm_config: WhirlpoolEvmConfig,
        state_db: Arc<RwLock<DB>>,
        tx_source: Arc<dyn TxSource + Send + Sync>,
    ) -> Self {
        Self {
            evm_config,
            state_db,
            tx_source,
            pending_receipts: Arc::new(Mutex::new(None)),
            staged_receipts: Arc::new(Mutex::new(BTreeMap::new())),
            last_proposed: Arc::new(Mutex::new(None)),
        }
    }

    pub fn store_finalized_block(
        &self,
        block: &EvmBlock,
        storage: &dyn BlockStorage,
    ) -> Result<(), EvmAppError> {
        let block_id = block.compute_id();
        let staged = {
            let guard = self.staged_receipts.lock().unwrap();
            guard.get(&block_id).cloned()
        };
        let Some(staged) = staged else {
            if block.transactions.is_empty() {
                storage
                    .store_block(block, &[])
                    .map_err(|e| EvmAppError::State(e.to_string()))?;
                return Ok(());
            }
            return Err(EvmAppError::InvalidBlock(format!(
                "missing staged receipts for finalized block {} ({:?})",
                block.height, block_id
            )));
        };

        if staged.height != block.height || staged.parent_id != block.parent_id {
            return Err(EvmAppError::InvalidBlock(format!(
                "staged receipts do not match finalized block identity: staged(height={}, parent={:?}, id={:?}), block(height={}, parent={:?}, id={:?})",
                staged.height, staged.parent_id, staged.block_id, block.height, block.parent_id, block_id
            )));
        }

        storage
            .store_block(block, &staged.receipts)
            .map_err(|e| EvmAppError::State(e.to_string()))?;

        {
            let mut guard = self.staged_receipts.lock().unwrap();
            if let Some(current) = guard.get(&block_id) {
                if current.height == block.height
                    && current.parent_id == block.parent_id
                    && current.block_id == block_id
                {
                    guard.remove(&block_id);
                }
            }
        }
        {
            let mut guard = self.pending_receipts.lock().unwrap();
            if guard
                .as_ref()
                .map(|receipts| receipts == &staged.receipts)
                .unwrap_or(false)
            {
                guard.take();
            }
        };
        Ok(())
    }

    pub fn pending_receipts(&self) -> Vec<Receipt> {
        self.pending_receipts
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default()
    }
}
