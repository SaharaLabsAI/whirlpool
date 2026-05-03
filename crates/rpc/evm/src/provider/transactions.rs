use std::ops::{RangeBounds, RangeInclusive};

use alloy_consensus::{transaction::TransactionMeta, BlockHeader};
use alloy_eips::BlockHashOrNumber;
use alloy_primitives::{Address, BlockNumber, TxHash, TxNumber};
use reth_ethereum_primitives::{Receipt, TransactionSigned};
use reth_primitives_traits::SignerRecoverable;
use reth_storage_api::{
    BlockBodyIndicesProvider, BlockNumReader, ReceiptProvider, ReceiptProviderIdExt,
    TransactionsProvider,
};
use reth_storage_errors::provider::{ProviderError, ProviderResult};

use crate::provider_impl::{
    map_db_err, range_to_exclusive_bounds, tx_range_to_exclusive_bounds, WhirlpoolProvider,
};

impl TransactionsProvider for WhirlpoolProvider {
    type Transaction = TransactionSigned;

    fn transaction_id(&self, tx_hash: TxHash) -> ProviderResult<Option<TxNumber>> {
        self.state_db
            .rpc_reader()
            .transactions()
            .lookup()
            .transaction_id(tx_hash)
            .map_err(map_db_err)
    }

    fn transaction_by_id(&self, id: TxNumber) -> ProviderResult<Option<Self::Transaction>> {
        self.state_db
            .rpc_reader()
            .transactions()
            .lookup()
            .transaction_by_id(id)
            .map_err(map_db_err)
    }

    fn transaction_by_id_unhashed(
        &self,
        id: TxNumber,
    ) -> ProviderResult<Option<Self::Transaction>> {
        self.transaction_by_id(id)
    }

    fn transaction_by_hash(&self, hash: TxHash) -> ProviderResult<Option<Self::Transaction>> {
        let Some(id) = self.transaction_id(hash)? else {
            return Ok(None);
        };
        self.transaction_by_id(id)
    }

    fn transaction_by_hash_with_meta(
        &self,
        hash: TxHash,
    ) -> ProviderResult<Option<(Self::Transaction, TransactionMeta)>> {
        let Some(inputs) = self
            .state_db
            .rpc_reader()
            .transactions()
            .meta()
            .transaction_by_hash_with_meta_inputs(hash)
            .map_err(map_db_err)?
        else {
            return Ok(None);
        };
        let tx_index = inputs
            .tx_num
            .saturating_sub(inputs.body_indices.first_tx_num());

        let meta = TransactionMeta {
            tx_hash: hash,
            index: tx_index,
            block_hash: inputs.block_hash,
            block_number: inputs.block_number,
            base_fee: inputs.header.base_fee_per_gas(),
            excess_blob_gas: inputs.header.excess_blob_gas(),
            timestamp: inputs.header.timestamp(),
        };
        Ok(Some((inputs.transaction, meta)))
    }

    fn transactions_by_block(
        &self,
        block_id: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<Self::Transaction>>> {
        let Some(block_number) = self.convert_hash_or_number(block_id)? else {
            return Ok(None);
        };
        let Some(body_indices) = self.block_body_indices(block_number)? else {
            return Ok(None);
        };

        if body_indices.tx_num_range().is_empty() {
            return Ok(Some(Vec::new()));
        }

        self.transactions_by_tx_range(body_indices.tx_num_range())
            .map(Some)
    }

    fn transactions_by_block_range(
        &self,
        range: impl RangeBounds<BlockNumber>,
    ) -> ProviderResult<Vec<Vec<Self::Transaction>>> {
        let (start, end) = range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        let mut transactions = Vec::with_capacity(end.saturating_sub(start) as usize);
        for block_number in start..end {
            match self.block_body_indices(block_number)? {
                Some(body_indices) if !body_indices.tx_num_range().is_empty() => {
                    transactions.push(self.transactions_by_tx_range(body_indices.tx_num_range())?);
                }
                _ => transactions.push(Vec::new()),
            }
        }
        Ok(transactions)
    }

    fn transactions_by_tx_range(
        &self,
        range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Self::Transaction>> {
        let (start, end) = tx_range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        self.state_db
            .rpc_reader()
            .transactions()
            .lookup()
            .transactions_by_tx_range(start, end)
            .map_err(map_db_err)
    }

    fn senders_by_tx_range(
        &self,
        range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Address>> {
        let transactions = self.transactions_by_tx_range(range)?;
        transactions
            .into_iter()
            .map(|transaction| {
                transaction
                    .recover_signer()
                    .map_err(|_| ProviderError::SenderRecoveryError)
            })
            .collect()
    }

    fn transaction_sender(&self, id: TxNumber) -> ProviderResult<Option<Address>> {
        let Some(transaction) = self.transaction_by_id(id)? else {
            return Ok(None);
        };
        transaction
            .recover_signer()
            .map(Some)
            .map_err(|_| ProviderError::SenderRecoveryError)
    }
}

impl ReceiptProvider for WhirlpoolProvider {
    type Receipt = Receipt;

    fn receipt(&self, id: TxNumber) -> ProviderResult<Option<Self::Receipt>> {
        self.state_db
            .rpc_reader()
            .transactions()
            .receipts()
            .receipt(id)
            .map_err(map_db_err)
    }

    fn receipt_by_hash(&self, hash: TxHash) -> ProviderResult<Option<Self::Receipt>> {
        let Some(id) = self.transaction_id(hash)? else {
            return Ok(None);
        };
        self.receipt(id)
    }

    fn receipts_by_block(
        &self,
        block: BlockHashOrNumber,
    ) -> ProviderResult<Option<Vec<Self::Receipt>>> {
        let Some(block_number) = self.convert_hash_or_number(block)? else {
            return Ok(None);
        };
        let Some(body_indices) = self.block_body_indices(block_number)? else {
            return Ok(None);
        };

        if body_indices.tx_num_range().is_empty() {
            return Ok(Some(Vec::new()));
        }

        self.receipts_by_tx_range(body_indices.tx_num_range())
            .map(Some)
    }

    fn receipts_by_tx_range(
        &self,
        range: impl RangeBounds<TxNumber>,
    ) -> ProviderResult<Vec<Self::Receipt>> {
        let (start, end) = tx_range_to_exclusive_bounds(range);
        if start >= end {
            return Ok(Vec::new());
        }

        self.state_db
            .rpc_reader()
            .transactions()
            .receipts()
            .receipts_by_tx_range(start, end)
            .map_err(map_db_err)
    }

    fn receipts_by_block_range(
        &self,
        block_range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<Vec<Vec<Self::Receipt>>> {
        if block_range.is_empty() {
            return Ok(Vec::new());
        }

        let mut block_body_indices =
            Vec::with_capacity(block_range.end().saturating_sub(*block_range.start()) as usize + 1);
        for block_number in block_range {
            block_body_indices.push(self.block_body_indices(block_number)?.unwrap_or_default());
        }

        let non_empty_blocks: Vec<_> = block_body_indices
            .iter()
            .filter(|indices| indices.tx_count() > 0)
            .collect();
        if non_empty_blocks.is_empty() {
            return Ok(vec![Vec::new(); block_body_indices.len()]);
        }

        let first_tx = non_empty_blocks[0].first_tx_num();
        let last_tx = non_empty_blocks[non_empty_blocks.len() - 1].last_tx_num();
        let mut receipts_iter = self.receipts_by_tx_range(first_tx..=last_tx)?.into_iter();

        let mut receipts = Vec::with_capacity(block_body_indices.len());
        for indices in &block_body_indices {
            if indices.tx_count() == 0 {
                receipts.push(Vec::new());
            } else {
                receipts.push(
                    receipts_iter
                        .by_ref()
                        .take(indices.tx_count() as usize)
                        .collect(),
                );
            }
        }

        Ok(receipts)
    }
}

impl ReceiptProviderIdExt for WhirlpoolProvider {}
