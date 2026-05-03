use app_primitives::header_extra_data::HeaderExtraDataHistory;
use app_primitives::EvmBlock;
use state::BlockStorage;

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
fn in_memory_state_db_returns_raw_block_extra_data_for_dkg_history() {
    let db = crate::InMemoryStateDb::new();
    let raw_carrier = vec![0xaa, 0xbb, 0xcc];
    let block = block_with_extra_data(3, raw_carrier.clone());

    db.store_block(&block, &[]).expect("store block");

    assert_eq!(
        HeaderExtraDataHistory::header_extra_data_at_height(&db, 3)
            .expect("read header extra_data"),
        Some(raw_carrier)
    );
    assert_eq!(
        HeaderExtraDataHistory::header_extra_data_at_height(&db, 4)
            .expect("read missing dkg carrier"),
        None
    );
}
