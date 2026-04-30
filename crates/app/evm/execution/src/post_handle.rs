use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use app_primitives::{EvmBlock, Receipt};
use state::BlockStorage;

use crate::error::EvmAppError;

#[derive(Clone, Debug)]
struct StagedReceipts {
    height: u64,
    parent_id: [u8; 32],
    block_id: [u8; 32],
    receipts: Vec<Receipt>,
}

#[derive(Clone, Debug, Default)]
pub struct ReceiptStore {
    pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>,
    staged_receipts: Arc<Mutex<BTreeMap<[u8; 32], StagedReceipts>>>,
}

impl ReceiptStore {
    pub fn stage_for_block(&self, block: &EvmBlock, receipts: Vec<Receipt>) {
        {
            let mut guard = self.pending_receipts.lock().unwrap();
            *guard = Some(receipts.clone());
        }

        let staged = StagedReceipts {
            height: block.height,
            parent_id: block.parent_id,
            block_id: block.compute_id(),
            receipts,
        };
        let mut guard = self.staged_receipts.lock().unwrap();
        guard.insert(staged.block_id, staged);
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

    #[cfg(test)]
    pub fn has_staged_receipts_for(&self, block_id: [u8; 32]) -> bool {
        self.staged_receipts.lock().unwrap().contains_key(&block_id)
    }

    #[cfg(test)]
    pub fn staged_receipts_is_empty(&self) -> bool {
        self.staged_receipts.lock().unwrap().is_empty()
    }

    pub fn pending_receipts(&self) -> Vec<Receipt> {
        self.pending_receipts
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default()
    }
}
