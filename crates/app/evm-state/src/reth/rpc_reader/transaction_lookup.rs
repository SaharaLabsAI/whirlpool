use alloy_primitives::{TxHash, TxNumber};
use reth_db::Database;
use reth_db_api::transaction::DbTx;

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use reth_db_api::tables::{TransactionHashNumbers, Transactions};

#[derive(Clone, Copy, Debug)]
pub struct RpcTransactionLookupReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcTransactionLookupReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn transaction_id(&self, hash: TxHash) -> Result<Option<TxNumber>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<TransactionHashNumbers>(hash)
            .map_err(RethStateError::Database)
    }

    pub fn transaction_by_id(
        &self,
        tx_num: TxNumber,
    ) -> Result<Option<reth_ethereum_primitives::TransactionSigned>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<Transactions>(tx_num)
            .map_err(RethStateError::Database)
    }

    pub fn transactions_by_tx_range(
        &self,
        start: TxNumber,
        end: TxNumber,
    ) -> Result<Vec<reth_ethereum_primitives::TransactionSigned>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut transactions = Vec::new();
        for tx_num in start..end {
            if let Some(transaction) = tx
                .get::<Transactions>(tx_num)
                .map_err(RethStateError::Database)?
            {
                transactions.push(transaction);
            }
        }
        Ok(transactions)
    }
}
