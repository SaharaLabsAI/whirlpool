use std::sync::{Arc, RwLock};

use alloy_primitives::{B256, Bytes, U256};
use app::{EvmBlock, TxSource};
use reth_primitives_traits::{Header, SealedHeader};

use crate::config::WhirlpoolEvmConfig;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
