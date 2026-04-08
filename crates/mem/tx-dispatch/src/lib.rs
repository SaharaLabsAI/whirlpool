use app_evm::{decode_evm_transaction as decode_app_evm_transaction, RecoveredTx};
use app_mem::{decode_personality_tx, MemTxError};

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
    decode_app_evm_transaction(raw_tx)
        .map_err(|err| TxDispatchError::InvalidEvmTransaction(err.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{SignableTransaction, TxLegacy};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, Bytes, Signature, TxKind, U256};
    use app_mem::{PersonalityMarkdownTx, SignatureScheme};
    use chainspec::SAHARA_CHAIN_ID;
    use reth_ethereum_primitives::TransactionSigned;

    fn sample_evm_tx() -> Vec<u8> {
        let tx = TxLegacy {
            chain_id: Some(SAHARA_CHAIN_ID),
            nonce: 0,
            gas_price: 2_000_000_000,
            gas_limit: 21_000,
            to: TxKind::Call(Address::with_last_byte(2)),
            value: U256::from(1000),
            input: Bytes::default(),
        };
        let signed: TransactionSigned = tx.into_signed(Signature::test_signature()).into();
        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);
        encoded
    }

    fn sample_mem_tx() -> Vec<u8> {
        PersonalityMarkdownTx::new(
            b"signer-1".to_vec(),
            b"persona-1".to_vec(),
            7,
            b"# Persona\nBe precise.".to_vec(),
            SignatureScheme::RawSecp256k1,
            vec![0x11; 65],
        )
        .encode()
        .expect("mem tx should encode")
    }

    #[test]
    fn classify_evm_transaction_uses_evm_decode_path() {
        let classified = classify_transaction(&sample_evm_tx()).expect("evm tx should classify");
        assert!(matches!(classified, ClassifiedTransaction::Evm(_)));
    }

    #[test]
    fn classify_mem_transaction_uses_mem_decode_path() {
        let classified = classify_transaction(&sample_mem_tx()).expect("mem tx should classify");
        assert!(matches!(classified, ClassifiedTransaction::Mem(_)));
    }
}
