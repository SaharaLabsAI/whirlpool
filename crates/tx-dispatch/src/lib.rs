use alloy_eips::eip2718::Decodable2718;
use app_mem::{decode_personality_tx, MemTxError};
use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::{Recovered, SignedTransaction};

pub type RecoveredTx = Recovered<TransactionSigned>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassifiedTransaction {
    Evm(Vec<u8>),
    Mem(Vec<u8>),
}

#[derive(Debug, thiserror::Error)]
pub enum TxDispatchError {
    #[error("invalid evm transaction: {0}")]
    InvalidEvmTransaction(String),
    #[error("invalid mem transaction: {0}")]
    InvalidMemTransaction(#[from] MemTxError),
}

pub fn decode_evm_transaction(raw_tx: &[u8]) -> Result<RecoveredTx, TxDispatchError> {
    let mut input = raw_tx;
    let tx = TransactionSigned::decode_2718(&mut input)
        .map_err(|err| TxDispatchError::InvalidEvmTransaction(err.to_string()))?;

    let signer = tx
        .try_recover()
        .map_err(|err| TxDispatchError::InvalidEvmTransaction(err.to_string()))?;

    Ok(tx.with_signer(signer))
}

pub fn decode_evm_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, TxDispatchError> {
    raw_txs
        .iter()
        .map(|raw_tx| decode_evm_transaction(raw_tx))
        .collect()
}

pub fn classify_transaction(raw_tx: &[u8]) -> Result<ClassifiedTransaction, TxDispatchError> {
    if decode_evm_transaction(raw_tx).is_ok() {
        return Ok(ClassifiedTransaction::Evm(raw_tx.to_vec()));
    }

    decode_personality_tx(raw_tx)?;
    Ok(ClassifiedTransaction::Mem(raw_tx.to_vec()))
}

pub fn classify_transactions(
    raw_txs: &[Vec<u8>],
) -> Result<Vec<ClassifiedTransaction>, TxDispatchError> {
    raw_txs
        .iter()
        .map(|raw_tx| classify_transaction(raw_tx))
        .collect()
}
