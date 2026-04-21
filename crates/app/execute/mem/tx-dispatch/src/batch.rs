use crate::{
    classify_transaction, decode_evm_transaction, ClassifiedTransaction, RecoveredTx, TxDispatchError,
};

pub fn decode_evm_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, TxDispatchError> {
    raw_txs
        .iter()
        .map(|raw_tx| decode_evm_transaction(raw_tx))
        .collect()
}

pub fn classify_transactions(
    raw_txs: &[Vec<u8>],
) -> Result<Vec<ClassifiedTransaction>, TxDispatchError> {
    raw_txs
        .iter()
        .map(|raw_tx| classify_transaction(raw_tx))
        .collect()
}
