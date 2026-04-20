use alloy_consensus::{Header, TxType, Typed2718};
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::Address;
use app::{proposer_public_key_from_extra_data, EvmBlock, Receipt as AppReceipt};
use reth_db::Database;
use reth_db_api::cursor::DbCursorRO;
use reth_db_api::transaction::{DbTx, DbTxMut};
use reth_db_models::blocks::StoredBlockBodyIndices;
use reth_ethereum_primitives::{Receipt as RethReceipt, TransactionSigned};
use revm::primitives::B256;
use state::{BlockStorage, BlockStorageError};

use crate::db::RethStateDb;
use crate::tables::{
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
            .inner()
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
            .inner()
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
            proposer_public_key_from_extra_data(&extra_data).ok_or_else(|| {
                BlockStorageError::Codec(format!(
                    "failed to decode proposer public key from block {number} extra_data"
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
            base_fee_per_gas: header.base_fee_per_gas.unwrap_or(0),
            timestamp: header.timestamp,
            transactions,
        }))
    }

    fn get_block_by_hash(&self, hash: B256) -> Result<Option<EvmBlock>, BlockStorageError> {
        let tx = self
            .inner()
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
            .inner()
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
            .inner()
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

#[cfg(test)]
mod tests {
    use alloy_consensus::{SignableTransaction, TxLegacy};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, Bytes, Signature, TxKind, U256};
    use app::Receipt as AppReceipt;
    use reth_db::Database;
    use reth_db_api::transaction::DbTx;
    use state::BlockStorage;

    use crate::init::open_state_db;
    use crate::tables::BlockBodyIndices;

    use super::*;

    fn make_raw_tx(nonce: u64) -> Vec<u8> {
        let tx = TxLegacy {
            chain_id: Some(1),
            nonce,
            gas_price: 1_000_000_000,
            gas_limit: 21_000,
            to: TxKind::Call(Address::ZERO),
            value: U256::from(nonce + 1),
            input: Bytes::default(),
        };

        let signed: TransactionSigned = tx.into_signed(Signature::test_signature()).into();
        let mut raw = Vec::new();
        signed.encode_2718(&mut raw);
        raw
    }

    fn make_block(height: u64, tx_count: usize) -> EvmBlock {
        EvmBlock {
            height,
            parent_id: [height.saturating_sub(1) as u8; 32],
            state_root: [2u8.wrapping_add(height as u8); 32],
            transactions_root: [3u8.wrapping_add(height as u8); 32],
            receipts_root: [4u8.wrapping_add(height as u8); 32],
            proposer_public_key: [height as u8; 32],
            proposer_fee_recipient: [height as u8; 20],
            extra_data: vec![height as u8; 32],
            gas_used: 21_000 * tx_count as u64,
            base_fee_per_gas: 1_000_000_000,
            timestamp: 1_700_000_000 + height,
            transactions: (0..tx_count)
                .map(|i| make_raw_tx((height * 1000) + i as u64))
                .collect(),
        }
    }

    fn make_receipts(count: usize) -> Vec<AppReceipt> {
        (0..count)
            .map(|i| AppReceipt {
                status: true.into(),
                cumulative_gas_used: (21_000 * (i as u64 + 1)),
                logs: Vec::new(),
            })
            .collect()
    }

    // TC-SR-01
    #[test]
    #[serial_test::serial]
    fn tc_sr_01_store_block_with_transactions_and_receipts() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");
        let block = make_block(1, 3);
        let receipts = make_receipts(3);

        let result = db.store_block(&block, &receipts);
        assert!(result.is_ok());
    }

    // TC-SR-02
    #[test]
    #[serial_test::serial]
    fn tc_sr_02_store_block_mismatched_receipts_returns_error() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");
        let block = make_block(1, 3);
        let receipts = make_receipts(2);

        let err = db
            .store_block(&block, &receipts)
            .expect_err("mismatched lengths should fail");
        assert!(matches!(err, BlockStorageError::Codec(_)));
    }

    // TC-SR-03
    #[test]
    #[serial_test::serial]
    fn tc_sr_03_get_block_by_number_round_trip() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");
        let block = make_block(7, 3);
        let receipts = make_receipts(3);

        db.store_block(&block, &receipts).expect("store block");
        let loaded = db
            .get_block_by_number(7)
            .expect("read block")
            .expect("block exists");

        assert_eq!(loaded.height, block.height);
        assert_eq!(loaded.parent_id, block.parent_id);
        assert_eq!(loaded.state_root, block.state_root);
        assert_eq!(loaded.transactions_root, block.transactions_root);
        assert_eq!(loaded.receipts_root, block.receipts_root);
        assert_eq!(loaded.gas_used, block.gas_used);
        assert_eq!(loaded.timestamp, block.timestamp);
        assert_eq!(loaded.transactions, block.transactions);
    }

    #[test]
    #[serial_test::serial]
    fn get_block_by_number_errors_on_malformed_extra_data() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");
        let mut block = make_block(12, 1);
        block.extra_data = vec![0x01, 0x02];
        let receipts = make_receipts(1);

        db.store_block(&block, &receipts).expect("store block");
        let err = db
            .get_block_by_number(12)
            .expect_err("malformed extra_data should fail block reconstruction");
        assert!(matches!(err, BlockStorageError::Codec(_)));
    }

    // TC-SR-04
    #[test]
    #[serial_test::serial]
    fn tc_sr_04_get_block_by_number_missing_returns_none() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");

        let loaded = db.get_block_by_number(999).expect("query should succeed");
        assert!(loaded.is_none());
    }

    // TC-SR-05
    #[test]
    #[serial_test::serial]
    fn tc_sr_05_get_block_by_hash_round_trip() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");
        let block = make_block(5, 2);
        let receipts = make_receipts(2);

        db.store_block(&block, &receipts).expect("store block");
        let hash = B256::from(block.compute_id());
        let loaded = db
            .get_block_by_hash(hash)
            .expect("read by hash")
            .expect("block exists");

        assert_eq!(loaded.height, block.height);
        assert_eq!(loaded.transactions, block.transactions);
    }

    // TC-SR-06
    #[test]
    #[serial_test::serial]
    fn tc_sr_06_get_block_by_hash_unknown_returns_none() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");

        let loaded = db
            .get_block_by_hash(B256::from([0xabu8; 32]))
            .expect("query should succeed");
        assert!(loaded.is_none());
    }

    // TC-SR-07
    #[test]
    #[serial_test::serial]
    fn tc_sr_07_sequential_blocks_have_monotonic_tx_numbers() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");

        let block_one = make_block(1, 2);
        let block_two = make_block(2, 3);
        db.store_block(&block_one, &make_receipts(2))
            .expect("store first block");
        db.store_block(&block_two, &make_receipts(3))
            .expect("store second block");

        let tx = db.inner().tx().expect("open read tx");
        let idx_one = tx
            .get::<BlockBodyIndices>(1)
            .expect("read first indices")
            .expect("first indices exist");
        let idx_two = tx
            .get::<BlockBodyIndices>(2)
            .expect("read second indices")
            .expect("second indices exist");

        assert_eq!(idx_two.first_tx_num, idx_one.next_tx_num());
        assert!(idx_two.first_tx_num >= idx_one.first_tx_num);
    }

    // TC-SR-08
    #[test]
    #[serial_test::serial]
    fn tc_sr_08_get_receipts_by_block_preserves_order() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");

        let block = make_block(11, 3);
        let mut receipts = make_receipts(3);
        receipts[0].cumulative_gas_used = 100;
        receipts[1].cumulative_gas_used = 200;
        receipts[2].cumulative_gas_used = 300;

        db.store_block(&block, &receipts).expect("store block");
        let loaded = db
            .get_receipts_by_block(11)
            .expect("read receipts")
            .expect("receipts exist");

        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].cumulative_gas_used, 100);
        assert_eq!(loaded[1].cumulative_gas_used, 200);
        assert_eq!(loaded[2].cumulative_gas_used, 300);
    }

    // TC-SR-09
    #[test]
    #[serial_test::serial]
    fn tc_sr_09_get_latest_block_number_empty_db_returns_none() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");

        let latest = db.get_latest_block_number().expect("query latest");
        assert_eq!(latest, None);
    }

    // TC-SR-10
    #[test]
    #[serial_test::serial]
    fn tc_sr_10_get_latest_block_number_returns_highest_stored() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db = open_state_db(dir.path()).expect("open db");

        // Store three blocks at heights 0, 5, 10
        for h in [0, 5, 10] {
            let block = make_block(h, 1);
            let receipts = make_receipts(1);
            db.store_block(&block, &receipts).expect("store block");
        }

        let latest = db
            .get_latest_block_number()
            .expect("query latest")
            .expect("should have blocks");
        assert_eq!(latest, 10);
    }
}
