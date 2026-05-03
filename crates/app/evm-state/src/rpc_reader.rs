use std::ops::Range;

use alloy_consensus::Header;
use alloy_primitives::{Address, BlockNumber, TxHash, TxNumber, B256};
use reth_db::Database;
use reth_db_api::{cursor::DbCursorRO, transaction::DbTx};
use reth_db_models::blocks::StoredBlockBodyIndices;
use reth_primitives_traits::Account;

use crate::{db::RethStateDb, error::RethStateError};
use reth_db_api::tables::{
    BlockBodyIndices, CanonicalHeaders, HeaderNumbers, Headers, PlainAccountState, Receipts,
    TransactionBlocks, TransactionHashNumbers, Transactions,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RpcBlockBodyIndices {
    first_tx_num: TxNumber,
    tx_count: u64,
}

impl RpcBlockBodyIndices {
    pub fn first_tx_num(&self) -> TxNumber {
        self.first_tx_num
    }

    pub fn tx_count(&self) -> u64 {
        self.tx_count
    }

    pub fn next_tx_num(&self) -> TxNumber {
        self.first_tx_num.saturating_add(self.tx_count)
    }

    pub fn tx_num_range(&self) -> Range<TxNumber> {
        self.first_tx_num..self.first_tx_num.saturating_add(self.tx_count)
    }
}

impl From<StoredBlockBodyIndices> for RpcBlockBodyIndices {
    fn from(indices: StoredBlockBodyIndices) -> Self {
        Self {
            first_tx_num: indices.first_tx_num(),
            tx_count: indices.tx_count(),
        }
    }
}

impl From<RpcBlockBodyIndices> for StoredBlockBodyIndices {
    fn from(indices: RpcBlockBodyIndices) -> Self {
        Self {
            first_tx_num: indices.first_tx_num,
            tx_count: indices.tx_count,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RpcStoredBlock {
    pub header: Header,
    pub transactions: Vec<reth_ethereum_primitives::TransactionSigned>,
}

#[derive(Clone, Debug)]
pub struct RpcCanonicalTip {
    pub best_number: BlockNumber,
    pub best_hash: B256,
}

#[derive(Clone, Debug)]
pub struct RpcTransactionMetaInputs {
    pub transaction: reth_ethereum_primitives::TransactionSigned,
    pub tx_num: TxNumber,
    pub block_number: BlockNumber,
    pub header: Header,
    pub block_hash: B256,
    pub body_indices: RpcBlockBodyIndices,
}

#[derive(Clone, Copy, Debug)]
pub struct RpcStateReader<'a> {
    db: &'a RethStateDb,
}

impl RethStateDb {
    pub fn rpc_reader(&self) -> RpcStateReader<'_> {
        RpcStateReader { db: self }
    }
}

impl RpcStateReader<'_> {
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

    pub fn block_number(&self, hash: B256) -> Result<Option<BlockNumber>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<HeaderNumbers>(hash)
            .map_err(RethStateError::Database)
    }

    pub fn header_by_hash(&self, hash: B256) -> Result<Option<Header>, RethStateError> {
        let Some(number) = self.block_number(hash)? else {
            return Ok(None);
        };
        self.header_by_number(number)
    }

    pub fn header_by_number(&self, number: BlockNumber) -> Result<Option<Header>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<Headers>(number).map_err(RethStateError::Database)
    }

    pub fn headers_range(
        &self,
        start: BlockNumber,
        end: BlockNumber,
    ) -> Result<Vec<Header>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let mut headers = Vec::new();
        for number in start..end {
            if let Some(header) = tx
                .get::<Headers>(number)
                .map_err(RethStateError::Database)?
            {
                headers.push(header);
            }
        }
        Ok(headers)
    }

    pub fn header_with_hash(
        &self,
        number: BlockNumber,
    ) -> Result<Option<(Header, B256)>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let Some(header) = tx
            .get::<Headers>(number)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        let Some(hash) = tx
            .get::<CanonicalHeaders>(number)
            .map_err(RethStateError::Database)?
        else {
            return Ok(None);
        };
        Ok(Some((header, hash)))
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

    pub fn basic_account(&self, address: Address) -> Result<Option<Account>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        tx.get::<PlainAccountState>(address)
            .map_err(RethStateError::Database)
    }

    pub fn header_extra_data_at_height(
        &self,
        height: u64,
    ) -> Result<Option<Vec<u8>>, RethStateError> {
        let tx = self.db.db.tx().map_err(RethStateError::Database)?;
        let header = tx
            .get::<Headers>(height)
            .map_err(RethStateError::Database)?;
        Ok(header.map(|header| header.extra_data.to_vec()))
    }
}
