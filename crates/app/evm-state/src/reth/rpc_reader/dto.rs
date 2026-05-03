use std::ops::Range;

use alloy_consensus::Header;
use alloy_primitives::{BlockNumber, TxNumber, B256};
use reth_db_models::blocks::StoredBlockBodyIndices;

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
