use app::EvmBlock;
use state::BlockStorage;
use validators_dkg::DkgHistory;

use crate::{open_state_db, InMemoryStateDb};

fn block_with_extra_data(height: u64, extra_data: Vec<u8>) -> EvmBlock {
    EvmBlock {
        height,
        parent_id: [height.saturating_sub(1) as u8; 32],
        state_root: [2u8.wrapping_add(height as u8); 32],
        transactions_root: [3u8.wrapping_add(height as u8); 32],
        receipts_root: [4u8.wrapping_add(height as u8); 32],
        proposer_public_key: [height as u8; 32],
        proposer_fee_recipient: [height as u8; 20],
        extra_data,
        gas_used: 0,
        base_fee_per_gas: 1_000_000_000,
        timestamp: 1_700_000_000 + height,
        transactions: Vec::new(),
    }
}

#[test]
#[serial_test::serial]
fn reth_state_db_returns_raw_header_extra_data_for_dkg_history() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let db = open_state_db(dir.path()).expect("open db");
    let raw_carrier = vec![0x01, 0x02, 0x03, 0x04];
    let block = block_with_extra_data(7, raw_carrier.clone());

    db.store_block(&block, &[]).expect("store block");

    assert_eq!(
        DkgHistory::full_dkg_at_height(&db, 7).expect("read dkg carrier"),
        Some(raw_carrier)
    );
}

#[test]
#[serial_test::serial]
fn reth_state_db_returns_none_for_missing_dkg_history_height() {
    let dir = tempfile::tempdir().expect("create temp dir");
    let db = open_state_db(dir.path()).expect("open db");

    assert_eq!(
        DkgHistory::full_dkg_at_height(&db, 999).expect("read missing dkg carrier"),
        None
    );
}

#[test]
fn in_memory_state_db_returns_raw_block_extra_data_for_dkg_history() {
    let db = InMemoryStateDb::new();
    let raw_carrier = vec![0xaa, 0xbb, 0xcc];
    let block = block_with_extra_data(3, raw_carrier.clone());

    db.store_block(&block, &[]).expect("store block");

    assert_eq!(
        DkgHistory::full_dkg_at_height(&db, 3).expect("read dkg carrier"),
        Some(raw_carrier)
    );
    assert_eq!(
        DkgHistory::full_dkg_at_height(&db, 4).expect("read missing dkg carrier"),
        None
    );
}
