use alloy_eips::eip2718::Decodable2718;
use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::SignedTransaction;

use crate::error::EvmAppError;

use super::super::RecoveredTx;

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
