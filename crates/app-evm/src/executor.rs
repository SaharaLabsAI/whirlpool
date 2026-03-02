use std::sync::{Arc, RwLock};

use alloy_consensus::TxReceipt;
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::{bytes::BufMut, Address, B256, Bytes, U256};
use alloy_trie::{root::ordered_trie_root_with_encoder, EMPTY_ROOT_HASH};
use app::{Application, EvmBlock, ExecutionResult, TxSource};
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::{execute::{BlockBuilder, BlockExecutor}, ConfigureEvm, NextBlockEnvAttributes};
use reth_primitives_traits::{Header, Recovered, SealedHeader, SignedTransaction, SignerRecoverable};
use reth_revm::State;
use revm::database::states::bundle_state::BundleRetention;
use revm::database::BundleState;
use state::InMemoryStateDb;

use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;

pub type RecoveredTx = Recovered<TransactionSigned>;

impl StateProvider for InMemoryStateDb {
    fn state_root(&self) -> B256 {
        self.state_root()
    }
    
    fn commit(&mut self, bundle: &BundleState) {
        self.commit(bundle)
    }
}

/// Trait for accessing state root from a database.
pub trait StateProvider {
    fn state_root(&self) -> B256;
    fn commit(&mut self, bundle: &BundleState);
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
    DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + std::fmt::Debug,
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
            let raw_pending = self.tx_source.pending();
            let decoded_pending: Vec<(Vec<u8>, RecoveredTx)> = raw_pending
                .iter()
                .filter_map(|raw| {
                    decode_transactions(std::slice::from_ref(raw))
                        .ok()
                        .and_then(|mut decoded| decoded.pop().map(|tx| (raw.clone(), tx)))
                })
                .collect();

            let parent_header = build_sealed_header(parent);

            let mut state_snapshot = {
                let db = self.state_db.read().unwrap();
                db.clone()
            };

            let timestamp = parent.timestamp + 12;
            let env_attributes = NextBlockEnvAttributes {
                timestamp,
                suggested_fee_recipient: Address::ZERO,
                prev_randao: B256::ZERO,
                gas_limit: 30_000_000,
                parent_beacon_block_root: Some(B256::ZERO),
                withdrawals: None,
                extra_data: Bytes::default(),
            };

            let mut state = State::builder()
                .with_database(&mut state_snapshot)
                .with_bundle_update()
                .without_state_clear()
                .build();

            let mut builder = self
                .evm_config
                .builder_for_next_block(&mut state, &parent_header, env_attributes)
                .map_err(|err| EvmAppError::Execution(err.to_string()))?;

            builder
                .apply_pre_execution_changes()
                .map_err(|err| EvmAppError::Execution(err.to_string()))?;

            let mut executed_raw_txs = Vec::new();
            for (raw_tx, tx) in decoded_pending {
                match builder.execute_transaction(tx.clone()) {
                    Ok(_) => executed_raw_txs.push(raw_tx),
                    Err(reth_evm::execute::BlockExecutionError::Validation(
                        reth_evm::execute::BlockValidationError::InvalidTx { .. },
                    )) => {
                        continue;
                    }
                    Err(err) => return Err(EvmAppError::Execution(err.to_string())),
                }
            }

            let mut executor = builder.into_executor();
            let (evm, execution_result) = executor
                .finish()
                .map_err(|err| EvmAppError::Execution(err.to_string()))?;
            
            // Drop evm to release borrow on state
            drop(evm);

            state.merge_transitions(BundleRetention::Reverts);
            let bundle = state.take_bundle();

            let state_root = {
                let mut canonical_db = self.state_db.write().unwrap();
                canonical_db.commit(&bundle);
                canonical_db.state_root()
            };

            let transactions_root = ordered_trie_root_with_encoder(&executed_raw_txs, |tx, out| {
                out.put_slice(tx.as_slice());
            });

            let receipts_root = ordered_trie_root_with_encoder(&execution_result.receipts, |receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            });

            let gas_used = execution_result
                .receipts
                .iter()
                .map(TxReceipt::cumulative_gas_used)
                .sum::<u64>();

            let block = EvmBlock {
                height,
                parent_id: parent.compute_id(),
                state_root: state_root.0,
                transactions_root: transactions_root.0,
                receipts_root: receipts_root.0,
                gas_used,
                timestamp,
                transactions: executed_raw_txs,
            };

            let result = ExecutionResult {
                state_root: state_root.0,
                receipts_root: receipts_root.0,
                gas_used,
                receipt_count: execution_result.receipts.len(),
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

    struct MockTxSource {
        txs: Vec<Vec<u8>>,
    }

    impl app::TxSource for MockTxSource {
        fn pending(&self) -> Vec<Vec<u8>> {
            self.txs.clone()
        }
    }

    async fn setup_app(txs: Vec<Vec<u8>>) -> (EvmApplication<InMemoryStateDb>, Arc<RwLock<InMemoryStateDb>>) {
        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let source = Arc::new(MockTxSource { txs });
        
        let app = EvmApplication::new(config, db.clone(), source);
        (app, db)
    }

    #[tokio::test]
    async fn propose_empty_txsource_produces_empty_block() {
        let (app, _) = setup_app(vec![]).await;
        let parent = app.genesis().await;
        
        let (block, result) = app.propose(&parent, 1).await.unwrap();
        
        assert!(block.transactions.is_empty());
        assert_eq!(block.gas_used, 0);
        assert_eq!(block.transactions_root, EMPTY_ROOT_HASH.0);
        assert_eq!(block.receipts_root, EMPTY_ROOT_HASH.0);
        assert_eq!(result.gas_used, 0);
    }

    #[tokio::test]
    async fn propose_executes_transfer_transaction() {
        let receiver = Address::with_last_byte(2);
        
        let tx = TxLegacy {
            chain_id: Some(crate::config::SAHARA_CHAIN_ID),
            nonce: 0,
            gas_price: 10,
            gas_limit: 21_000,
            to: TxKind::Call(receiver),
            value: U256::from(1000),
            input: Bytes::default(),
        };
        let signature = Signature::test_signature(); 
        let signed: TransactionSigned = tx.into_signed(signature).into();
        let recovered = signed.recover_signer().unwrap();
        
        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);

        let (app, db) = setup_app(vec![encoded]).await;

        // Fund the sender
        {
            let mut db = db.write().unwrap();
            let info = revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            };
            db.insert_account(recovered, info);
        }

        let parent = app.genesis().await;
        let (block, _) = app.propose(&parent, 1).await.unwrap();

        assert_eq!(block.transactions.len(), 1);
        assert!(block.gas_used > 0);
        assert_ne!(block.transactions_root, EMPTY_ROOT_HASH.0);
        assert_ne!(block.receipts_root, EMPTY_ROOT_HASH.0);
        assert_ne!(block.state_root, parent.state_root);
    }

    #[tokio::test]
    async fn propose_executes_contract_deployment() {
         // Simple contract that returns 42
         let bytecode = Bytes::from(vec![0x60, 0x2a, 0x60, 0x00, 0x52, 0x60, 0x20, 0x60, 0x00, 0xf3]);
         
         let tx = TxLegacy {
            chain_id: Some(crate::config::SAHARA_CHAIN_ID),
            nonce: 0,
            gas_price: 10,
            gas_limit: 100_000,
            to: TxKind::Create,
            value: U256::ZERO,
            input: bytecode,
        };
        
        let signature = Signature::test_signature();
        let signed: TransactionSigned = tx.into_signed(signature).into();
        let recovered = signed.recover_signer().unwrap();
        
        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);

        let (app, db) = setup_app(vec![encoded]).await;
        
        // Fund
        {
            let mut db = db.write().unwrap();
            let info = revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            };
            db.insert_account(recovered, info);
        }
        
        let parent = app.genesis().await;
        let (block, _) = app.propose(&parent, 1).await.unwrap();
        
        assert_eq!(block.transactions.len(), 1);
        assert!(block.gas_used > 0);
    }

    #[tokio::test]
    async fn propose_skips_invalid_transactions() {
        let (app, _) = setup_app(vec![vec![0xde, 0xad, 0xbe, 0xef]]).await;
        let parent = app.genesis().await;
        
        let (block, _) = app.propose(&parent, 1).await.unwrap();
        
        // Should produce empty block, not fail
        assert!(block.transactions.is_empty());
    }
}
