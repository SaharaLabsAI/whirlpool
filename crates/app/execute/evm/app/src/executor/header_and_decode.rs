use super::*;

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

pub fn decode_evm_transaction(raw_tx: &[u8]) -> Result<RecoveredTx, EvmAppError> {
    let mut input = raw_tx;
    let tx = TransactionSigned::decode_2718(&mut input)
        .map_err(|err| EvmAppError::InvalidBlock(err.to_string()))?;

    let signer = tx
        .try_recover()
        .map_err(|err| EvmAppError::InvalidBlock(err.to_string()))?;

    Ok(tx.with_signer(signer))
}

pub fn decode_evm_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError> {
    raw_txs
        .iter()
        .map(|raw_tx| decode_evm_transaction(raw_tx))
        .collect()
}

pub(super) fn build_sealed_header(block: &EvmBlock) -> SealedHeader {
    let header = build_header_from_evm_block(block);
    let hash = header.hash_slow();
    SealedHeader::new(header, hash)
}
