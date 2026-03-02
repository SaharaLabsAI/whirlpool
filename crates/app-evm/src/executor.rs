use std::sync::{Arc, RwLock};

use alloy_eips::eip2718::Decodable2718;
use alloy_primitives::{B256, Bytes, U256};
use app::{Application, EvmBlock, ExecutionResult, TxSource};
use alloy_trie::EMPTY_ROOT_HASH;

use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::{Header, Recovered, SealedHeader, SignedTransaction};
use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;

pub type RecoveredTx = Recovered<TransactionSigned>;

/// Trait for accessing state root from a database.
pub trait StateProvider {
    fn state_root(&self) -> B256;
}

/// Converts an `EvmBlock` into an Ethereum `Header`.
fn build_header_from_evm_block(block: &EvmBlock) -> Header {
    Header {
        number: block.height,
        parent_hash: B256::from(block.parent_id),
        state_root: B256::from(block.state_root),
        transactions_root: B256::from(block.transactions_root),
        receipts_root: B256::from(block.receipts_root),
        gas_limit: 30_000_000,
        gas_used: block.gas_used,
        timestamp: block.timestamp,
        difficulty: U256::ZERO,
        extra_data: Bytes::default(),
        ..Header::default()
    }
}

/// Builds a sealed header from an `EvmBlock` by hashing it.
fn build_sealed_header(block: &EvmBlock) -> SealedHeader {
    let header = build_header_from_evm_block(block);
    let hash = header.hash_slow();
    SealedHeader::new(header, hash)
}

pub fn decode_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError> {
    raw_txs
        .iter()
        .map(|raw_tx| {
            let mut input = raw_tx.as_slice();
            let tx = TransactionSigned::decode_2718(&mut input)
                .map_err(|err| EvmAppError::InvalidBlock(err.to_string()))?;

            let signer = tx
                .try_recover()
                .map_err(|err| EvmAppError::InvalidBlock(err.to_string()))?;

            Ok(tx.with_signer(signer))
        })
        .collect()
}

#[derive(Clone)]
pub struct EvmApplication<DB> {
    evm_config: WhirlpoolEvmConfig,
    state_db: Arc<RwLock<DB>>,
    tx_source: Arc<dyn TxSource + Send + Sync>,
}

impl<DB> EvmApplication<DB> {
    pub fn new(
        evm_config: WhirlpoolEvmConfig,
        state_db: Arc<RwLock<DB>>,
        tx_source: Arc<dyn TxSource + Send + Sync>,
    ) -> Self {
        Self {
            evm_config,
            state_db,
            tx_source,
        }
    }
}

impl<DB> Application for EvmApplication<DB>
where
    DB: StateProvider + Clone + Send + Sync + 'static,
{
    type Block = EvmBlock;
    type Result = ExecutionResult;
    type Error = EvmAppError;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        async move {
            let state_root = {
                let db = self.state_db.read().unwrap();
                db.state_root()
            };

            EvmBlock {
                height: 0,
                parent_id: [0u8; 32],
                state_root: state_root.0,
                transactions_root: EMPTY_ROOT_HASH.0,
                receipts_root: EMPTY_ROOT_HASH.0,
                gas_used: 0,
                timestamp: 0,
                transactions: vec![],
            }
        }
    }

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl std::future::Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send {
        async move {
            // MVP: Empty block execution (no transaction processing)
            let state_root = {
                let db = self.state_db.read().unwrap();
                db.state_root()
            };

            let block = EvmBlock {
                height,
                parent_id: parent.compute_id(),
                state_root: state_root.0,
                transactions_root: EMPTY_ROOT_HASH.0,
                receipts_root: EMPTY_ROOT_HASH.0,
                gas_used: 0,
                timestamp: parent.timestamp + 12,
                transactions: vec![],
            };

            let result = ExecutionResult {
                state_root: state_root.0,
                receipts_root: EMPTY_ROOT_HASH.0,
                gas_used: 0,
                receipt_count: 0,
            };

            Ok((block, result))
        }
    }

    fn verify(
        &self,
        _parent: &Self::Block,
        block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
        async move {
            // Compute expected state root
            let computed_state_root = {
                let db = self.state_db.read().unwrap();
                db.state_root()
            };

            // Verify state root matches
            if computed_state_root.0 != block.state_root {
                return Err(EvmAppError::StateRootMismatch {
                    expected: block.state_root,
                    computed: computed_state_root.0,
                });
            }

            Ok(ExecutionResult {
                state_root: block.state_root,
                receipts_root: block.receipts_root,
                gas_used: block.gas_used,
                receipt_count: 0,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{SignableTransaction, TxLegacy};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, Signature, TxKind};

    #[test]
    fn test_header_conversion() {
        let evm_block = EvmBlock {
            height: 42,
            parent_id: [1u8; 32],
            state_root: [2u8; 32],
            transactions_root: [3u8; 32],
            receipts_root: [4u8; 32],
            gas_used: 21_000,
            timestamp: 1_234_567_890,
            transactions: vec![],
        };

        let header = build_header_from_evm_block(&evm_block);
        assert_eq!(header.number, 42);
        assert_eq!(header.parent_hash, B256::from([1u8; 32]));
        assert_eq!(header.state_root, B256::from([2u8; 32]));
        assert_eq!(header.transactions_root, B256::from([3u8; 32]));
        assert_eq!(header.receipts_root, B256::from([4u8; 32]));
        assert_eq!(header.gas_used, 21_000);
        assert_eq!(header.timestamp, 1_234_567_890);

        let expected_hash = header.hash_slow();
        let sealed = build_sealed_header(&evm_block);
        assert_eq!(sealed.number, 42);
        assert_eq!(sealed.hash(), expected_hash);
    }

    #[test]
    fn decode_transactions_valid_rlp() {
        let tx = TxLegacy {
            chain_id: Some(1),
            nonce: 0,
            gas_price: 1_000_000_000,
            gas_limit: 21_000,
            to: TxKind::Call(Address::ZERO),
            value: U256::from(1u64),
            input: Bytes::default(),
        };
        let signed: TransactionSigned = tx.into_signed(Signature::test_signature()).into();

        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);

        let decoded = decode_transactions(&[encoded]).expect("valid tx should decode");
        assert_eq!(decoded.len(), 1);

        let expected_signer = signed.try_recover().expect("signature should recover");
        assert_eq!(decoded[0].signer(), expected_signer);
    }

    #[test]
    fn decode_transactions_invalid_rlp() {
        let err = decode_transactions(&[vec![0x01, 0x02, 0x03]]).expect_err("invalid RLP should fail");
        assert!(matches!(err, EvmAppError::InvalidBlock(_)));
    }

    #[test]
    fn decode_transactions_empty_input() {
        let decoded = decode_transactions(&[]).expect("empty input should be valid");
        assert!(decoded.is_empty());
    }
}
