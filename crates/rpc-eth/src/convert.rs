use alloy_eips::eip2718::{Decodable2718, Eip2718Error};
use alloy_primitives::{Address, B256};
use app::types::EvmBlock;
use reth_ethereum_primitives::{Block, BlockBody, TransactionSigned};
use reth_primitives_traits::Header;

/// Decode raw EIP-2718 transaction bytes into a signed transaction.
pub fn decode_transaction(bytes: &[u8]) -> Result<TransactionSigned, Eip2718Error> {
    TransactionSigned::decode_2718(&mut &bytes[..])
}

/// Convert an EvmBlock into a reth header.
pub fn evmblock_to_header(block: &EvmBlock) -> Header {
    Header {
        number: block.height,
        parent_hash: B256::from_slice(&block.parent_id),
        beneficiary: Address::from(block.proposer_fee_recipient),
        state_root: B256::from_slice(&block.state_root),
        transactions_root: B256::from_slice(&block.transactions_root),
        receipts_root: B256::from_slice(&block.receipts_root),
        gas_used: block.gas_used,
        base_fee_per_gas: Some(block.base_fee_per_gas),
        extra_data: block.proposer_public_key.to_vec().into(),
        timestamp: block.timestamp,
        ..Default::default()
    }
}

/// Convert an EvmBlock into a reth block with decoded transactions.
pub fn evmblock_to_block(block: &EvmBlock) -> Result<Block, Eip2718Error> {
    let header = evmblock_to_header(block);
    let transactions: Result<Vec<TransactionSigned>, _> = block
        .transactions
        .iter()
        .map(|raw| decode_transaction(raw))
        .collect();
    let body = BlockBody {
        transactions: transactions?,
        ommers: vec![],
        withdrawals: None,
    };
    Ok(Block::new(header, body))
}
