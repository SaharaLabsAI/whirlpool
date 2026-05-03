use alloy_primitives::{BlockNumber, B256};
use reth_db::Database;
use reth_db_api::{cursor::DbCursorRO, transaction::DbTx};

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use crate::reth::rpc_reader::RpcCanonicalTip;
use reth_db_api::tables::CanonicalHeaders;

#[derive(Clone, Copy, Debug)]
pub struct RpcCanonicalBlockReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcCanonicalBlockReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn block_hash(&self, number: BlockNumber) -> Result<Option<B256>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<CanonicalHeaders>(number)
            .map_err(RethStateError::Database)
    }

    pub fn canonical_hashes_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> Result<Vec<B256>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut hashes = Vec::new();
        for number in start..end {
            if let Some(hash) = tx
                .get::<CanonicalHeaders>(number)
                .map_err(RethStateError::Database)?
            {
                hashes.push(hash);
            }
        }
        Ok(hashes)
    }

    pub fn canonical_tip(&self) -> Result<Option<RpcCanonicalTip>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let Some((best_number, best_hash)) = tx
            .cursor_read::<CanonicalHeaders>()
            .map_err(RethStateError::Database)?
            .last()
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        Ok(Some(RpcCanonicalTip {
            best_number,
            best_hash,
        }))
    }
}
