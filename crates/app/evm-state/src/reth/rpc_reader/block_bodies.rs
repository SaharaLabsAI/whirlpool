use alloy_primitives::BlockNumber;
use reth_db::Database;
use reth_db_api::transaction::DbTx;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use crate::reth::rpc_reader::{RpcBlockBodyIndices, RpcStoredBlock};
use reth_db_api::tables::{BlockBodyIndices, Headers, Transactions};

#[derive(Clone, Copy, Debug)]
pub struct RpcBlockBodyReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcBlockBodyReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn read_block_by_number(
        &self,
        number: BlockNumber,
    ) -> Result<Option<RpcStoredBlock>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let Some(header) = tx
            .get::<Headers>(number)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let body_indices = tx
            .get::<BlockBodyIndices>(number)
            .map_err(RethStateError::Database)?
            .unwrap_or_default();

        let mut transactions = Vec::with_capacity(body_indices.tx_count() as usize);
        for tx_num in body_indices.tx_num_range() {
            let Some(transaction) = tx
                .get::<Transactions>(tx_num)
                .map_err(RethStateError::Database)?
            else {
                return Ok(None);
            };
            transactions.push(transaction);
        }

        Ok(Some(RpcStoredBlock {
            header,
            transactions,
        }))
    }

    pub fn block_body_indices(
        &self,
        number: BlockNumber,
    ) -> Result<Option<RpcBlockBodyIndices>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<BlockBodyIndices>(number)
            .map(|indices| indices.map(Into::into))
            .map_err(RethStateError::Database)
    }

    pub fn block_body_indices_range(
        &self,
        start: BlockNumber,
        end_inclusive: BlockNumber,
    ) -> Result<Vec<RpcBlockBodyIndices>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut indices = Vec::new();
        for number in start..=end_inclusive {
            if let Some(body_indices) = tx
                .get::<BlockBodyIndices>(number)
                .map_err(RethStateError::Database)?
            {
                indices.push(body_indices.into());
            }
        }
        Ok(indices)
    }
}
