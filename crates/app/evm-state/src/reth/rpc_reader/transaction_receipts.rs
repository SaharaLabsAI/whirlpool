use alloy_primitives::TxNumber;
use reth_db::Database;
use reth_db_api::transaction::DbTx;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use reth_db_api::tables::Receipts;

#[derive(Clone, Copy, Debug)]
pub struct RpcReceiptReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcReceiptReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn receipt(
        &self,
        tx_num: TxNumber,
    ) -> Result<Option<reth_ethereum_primitives::Receipt>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<Receipts>(tx_num).map_err(RethStateError::Database)
    }

    pub fn receipts_by_tx_range(
        &self,
        start: TxNumber,
        end: TxNumber,
    ) -> Result<Vec<reth_ethereum_primitives::Receipt>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut receipts = Vec::new();
        for tx_num in start..end {
            if let Some(receipt) = tx
                .get::<Receipts>(tx_num)
                .map_err(RethStateError::Database)?
            {
                receipts.push(receipt);
            }
        }
        Ok(receipts)
    }
}
