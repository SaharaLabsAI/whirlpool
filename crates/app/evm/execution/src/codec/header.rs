use alloy_primitives::{Address, Bytes, B256, U256};
use app::EvmBlock;
use reth_primitives_traits::Header;

pub fn build_header_from_evm_block(block: &EvmBlock) -> Header {
    Header {
        number: block.height,
        parent_hash: B256::from(block.parent_id),
        state_root: B256::from(block.state_root),
        transactions_root: B256::from(block.transactions_root),
        receipts_root: B256::from(block.receipts_root),
        beneficiary: Address::from(block.proposer_fee_recipient),
        gas_limit: 30_000_000,
        gas_used: block.gas_used,
        base_fee_per_gas: Some(block.base_fee_per_gas),
        timestamp: block.timestamp,
        difficulty: U256::ZERO,
        extra_data: Bytes::copy_from_slice(&block.extra_data),
        excess_blob_gas: Some(0),
        blob_gas_used: Some(0),
        ..Header::default()
    }
}
