use alloy_primitives::{BlockNumber, TxHash, TxNumber};
use reth_db::Database;
use reth_db_api::{cursor::DbCursorRO, transaction::DbTx};

use crate::error::RethStateError;
use crate::reth::db::RethStateDb;
use crate::reth::rpc_reader::RpcTransactionMetaInputs;
use reth_db_api::tables::{
    BlockBodyIndices, CanonicalHeaders, Headers, TransactionBlocks, TransactionHashNumbers,
    Transactions,
};

#[derive(Clone, Copy, Debug)]
pub struct RpcTransactionMetaReader<'a> {
    db: &'a RethStateDb,
}

impl<'a> RpcTransactionMetaReader<'a> {
    pub(in crate::reth::rpc_reader) fn new(db: &'a RethStateDb) -> Self {
        Self { db }
    }

    pub fn block_number_by_transaction_id(
        &self,
        tx_num: TxNumber,
    ) -> Result<Option<BlockNumber>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut cursor = tx
            .cursor_read::<TransactionBlocks>()
            .map_err(RethStateError::Database)?;
        let entry = cursor.seek(tx_num).map_err(RethStateError::Database)?;
        Ok(entry.map(|(_, block_number)| block_number))
    }

    pub fn transaction_by_hash_with_meta_inputs(
        &self,
        hash: TxHash,
    ) -> Result<Option<RpcTransactionMetaInputs>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let Some(tx_num) = tx
            .get::<TransactionHashNumbers>(hash)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let Some(transaction) = tx
            .get::<Transactions>(tx_num)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let Some(block_number) = tx
            .get::<TransactionBlocks>(tx_num)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let Some(header) = tx
            .get::<Headers>(block_number)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let block_hash = tx
            .get::<CanonicalHeaders>(block_number)
            .map_err(RethStateError::Database)?
            .unwrap_or_default();
        let body_indices = tx
            .get::<BlockBodyIndices>(block_number)
            .map_err(RethStateError::Database)?
            .unwrap_or_default()
            .into();

        Ok(Some(RpcTransactionMetaInputs {
            transaction,
            tx_num,
            block_number,
            header,
            block_hash,
            body_indices,
        }))
    }
}
