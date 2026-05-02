use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, Signature, TxKind, U256};
use app_primitives::{
    header_extra_data::{build_header_extra_data, DkgHeaderSections},
    Receipt as AppReceipt,
};
use reth_db::Database;
use reth_db_api::transaction::DbTx;
use reth_ethereum_primitives::TransactionSigned;
use revm::primitives::B256;
use state::BlockStorage;

use crate::init::open_state_db;
use crate::tables::BlockBodyIndices;

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

fn make_block(height: u64, tx_count: usize) -> app_primitives::EvmBlock {
    let proposer_public_key = [height as u8; 32];
    app_primitives::EvmBlock {
        height,
        parent_id: [height.saturating_sub(1) as u8; 32],
        state_root: [2u8.wrapping_add(height as u8); 32],
        transactions_root: [3u8.wrapping_add(height as u8); 32],
        receipts_root: [4u8.wrapping_add(height as u8); 32],
        proposer_public_key,
        proposer_fee_recipient: [height as u8; 20],
        extra_data: build_header_extra_data(proposer_public_key, DkgHeaderSections::default())
            .expect("canonical extra_data"),
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
            cumulative_gas_used: 21_000 * (i as u64 + 1),
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
    assert!(matches!(err, state::BlockStorageError::Codec(_)));
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
fn get_block_by_number_errors_on_legacy_raw_32_byte_extra_data() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let db = open_state_db(dir.path()).expect("open db");
    let mut block = make_block(12, 1);
    block.extra_data = block.proposer_public_key.to_vec();
    let receipts = make_receipts(1);

    db.store_block(&block, &receipts).expect("store block");
    let err = db
        .get_block_by_number(12)
        .expect_err("legacy raw 32-byte extra_data should fail block reconstruction");
    assert!(matches!(err, state::BlockStorageError::Codec(_)));
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
