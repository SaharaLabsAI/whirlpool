use alloy_consensus::{Header, TxType, Typed2718};
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::Address;
use app_primitives::header_extra_data::proposer_public_key_from_extra_data;
use app_primitives::{EvmBlock, Receipt as AppReceipt};
use reth_db::Database;
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::{DbTx, DbTxMut};
use reth_db_models::blocks::StoredBlockBodyIndices;
use reth_ethereum_primitives::{Receipt as RethReceipt, TransactionSigned};
use revm::primitives::B256;
use state::{BlockStorage, BlockStorageError};

use crate::db::RethStateDb;
use reth_db_api::tables::{
    BlockBodyIndices, CanonicalHeaders, HeaderNumbers, HeaderTerminalDifficulties, Headers,
    Receipts, TransactionBlocks, TransactionHashNumbers, Transactions,
};

impl BlockStorage for RethStateDb {
    fn store_block(
        &self,
        block: &EvmBlock,
        receipts: &[AppReceipt],
    ) -> Result<(), BlockStorageError> {
        if block.transactions.len() != receipts.len() {
            return Err(BlockStorageError::Codec(
                "transactions/receipts length mismatch".to_string(),
            ));
        }

        let tx = self
            .db
            .tx_mut()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        let block_hash = B256::from(block.compute_id());

        // Idempotency: same block number + hash means the block is already persisted.
        if let Some(existing_hash) = tx
            .get::<CanonicalHeaders>(block.height)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
        {
            if existing_hash == block_hash {
                return Ok(());
            }

            return Err(BlockStorageError::Database(format!(
                "block {} already exists with different hash",
                block.height
            )));
        }

        let mut decoded_txs = Vec::with_capacity(block.transactions.len());
        for raw_tx in &block.transactions {
            let mut input = raw_tx.as_slice();
            let tx_signed = TransactionSigned::decode_2718(&mut input)
                .map_err(|e| BlockStorageError::Codec(e.to_string()))?;
            decoded_txs.push(tx_signed);
        }

        let header = Header {
            parent_hash: B256::from(block.parent_id),
            beneficiary: Address::from(block.proposer_fee_recipient),
            state_root: B256::from(block.state_root),
            transactions_root: B256::from(block.transactions_root),
            receipts_root: B256::from(block.receipts_root),
            gas_used: block.gas_used,
            base_fee_per_gas: Some(block.base_fee_per_gas),
            extra_data: block.extra_data.clone().into(),
            timestamp: block.timestamp,
            number: block.height,
            // Post-Cancun fields required by the EVM environment.
            // Set to zero since we do not support blob transactions.
            excess_blob_gas: Some(0),
            blob_gas_used: Some(0),
            ..Header::default()
        };

        tx.put::<Headers>(block.height, header)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;
        tx.put::<CanonicalHeaders>(block.height, block_hash)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;
        tx.put::<HeaderNumbers>(block_hash, block.height)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;
        tx.put::<HeaderTerminalDifficulties>(block.height, Default::default())
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        let first_tx_num = tx
            .cursor_read::<BlockBodyIndices>()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
            .last()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
            .map(|(_, indices)| indices.next_tx_num())
            .unwrap_or(0);

        let body_indices = StoredBlockBodyIndices {
            first_tx_num,
            tx_count: block.transactions.len() as u64,
        };

        tx.put::<BlockBodyIndices>(block.height, body_indices)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        if body_indices.tx_count > 0 {
            tx.put::<TransactionBlocks>(body_indices.next_tx_num() - 1, block.height)
                .map_err(|e| BlockStorageError::Database(e.to_string()))?;
        }

        for (i, tx_signed) in decoded_txs.iter().enumerate() {
            let tx_num = first_tx_num + i as u64;
            tx.put::<Transactions>(tx_num, tx_signed.clone())
                .map_err(|e| BlockStorageError::Database(e.to_string()))?;
            tx.put::<TransactionHashNumbers>(*tx_signed.hash(), tx_num)
                .map_err(|e| BlockStorageError::Database(e.to_string()))?;
        }

        for (i, (receipt, tx_signed)) in receipts.iter().zip(decoded_txs.iter()).enumerate() {
            let reth_receipt = RethReceipt {
                tx_type: tx_type_from_signed(tx_signed),
                success: receipt.status.coerce_status(),
                cumulative_gas_used: receipt.cumulative_gas_used,
                logs: receipt.logs.clone(),
            };

            tx.put::<Receipts>(first_tx_num + i as u64, reth_receipt)
                .map_err(|e| BlockStorageError::Database(e.to_string()))?;
        }

        tx.commit()
            .map_err(|e| BlockStorageError::Database(e.to_string()))
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, BlockStorageError> {
        let tx = self
            .db
            .tx()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        let Some(header) = tx
            .get::<Headers>(number)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        let body_indices = tx
            .get::<BlockBodyIndices>(number)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
            .unwrap_or_default();

        let mut transactions = Vec::with_capacity(body_indices.tx_count as usize);
        for tx_num in body_indices.tx_num_range() {
            let tx_signed = tx
                .get::<Transactions>(tx_num)
                .map_err(|e| BlockStorageError::Database(e.to_string()))?
                .ok_or_else(|| {
                    BlockStorageError::Database(format!(
                        "missing transaction at tx number {tx_num} for block {number}"
                    ))
                })?;

            let mut raw = Vec::new();
            tx_signed.encode_2718(&mut raw);
            transactions.push(raw);
        }

        let extra_data: Vec<u8> = header.extra_data.to_vec();
        let proposer_public_key =
            proposer_public_key_from_extra_data(&extra_data).map_err(|err| {
                BlockStorageError::Codec(format!(
                    "failed to decode proposer public key from block {number} extra_data: {err}"
                ))
            })?;
        let base_fee_per_gas = header.base_fee_per_gas.ok_or_else(|| {
            BlockStorageError::Codec(format!(
                "missing base_fee_per_gas while reconstructing block {number}"
            ))
        })?;

        Ok(Some(EvmBlock {
            height: number,
            parent_id: header.parent_hash.into(),
            state_root: header.state_root.into(),
            transactions_root: header.transactions_root.into(),
            receipts_root: header.receipts_root.into(),
            proposer_public_key,
            proposer_fee_recipient: header.beneficiary.into_array(),
            extra_data,
            gas_used: header.gas_used,
            base_fee_per_gas,
            timestamp: header.timestamp,
            transactions,
        }))
    }

    fn get_block_by_hash(&self, hash: B256) -> Result<Option<EvmBlock>, BlockStorageError> {
        let tx = self
            .db
            .tx()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        let Some(number) = tx
            .get::<HeaderNumbers>(hash)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        self.get_block_by_number(number)
    }

    fn get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError> {
        let tx = self
            .db
            .tx()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        let latest = tx
            .cursor_read::<CanonicalHeaders>()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
            .last()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
            .map(|(number, _hash)| number);

        Ok(latest)
    }

    fn get_receipts_by_block(
        &self,
        number: u64,
    ) -> Result<Option<Vec<AppReceipt>>, BlockStorageError> {
        let tx = self
            .db
            .tx()
            .map_err(|e| BlockStorageError::Database(e.to_string()))?;

        let Some(body_indices) = tx
            .get::<BlockBodyIndices>(number)
            .map_err(|e| BlockStorageError::Database(e.to_string()))?
        else {
            return Ok(None);
        };

        let mut receipts = Vec::with_capacity(body_indices.tx_count as usize);
        for tx_num in body_indices.tx_num_range() {
            let reth_receipt = tx
                .get::<Receipts>(tx_num)
                .map_err(|e| BlockStorageError::Database(e.to_string()))?
                .ok_or_else(|| {
                    BlockStorageError::Database(format!(
                        "missing receipt at tx number {tx_num} for block {number}"
                    ))
                })?;

            receipts.push(AppReceipt {
                status: reth_receipt.success.into(),
                cumulative_gas_used: reth_receipt.cumulative_gas_used,
                logs: reth_receipt.logs,
            });
        }

        Ok(Some(receipts))
    }
}

fn tx_type_from_signed(tx_signed: &TransactionSigned) -> TxType {
    match tx_signed.ty() {
        0 => TxType::Legacy,
        1 => TxType::Eip2930,
        2 => TxType::Eip1559,
        3 => TxType::Eip4844,
        4 => TxType::Eip7702,
        _ => TxType::Legacy,
    }
}

#[path = "tests/block_storage.rs"]
#[cfg(test)]
mod tests;
