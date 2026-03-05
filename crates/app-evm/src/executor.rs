use std::sync::{Arc, RwLock};

use alloy_consensus::TxReceipt;
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::{bytes::BufMut, Address, B256, Bytes, U256};
use alloy_trie::{root::ordered_trie_root_with_encoder, EMPTY_ROOT_HASH};
use app::{EvmBlock, ExecutionResult, traits::{Application, TxSource}};
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::{execute::{BlockBuilder, BlockExecutor}, ConfigureEvm, NextBlockEnvAttributes};
use reth_primitives_traits::{Header, Recovered, SealedHeader, SignedTransaction};
use reth_revm::State;
use revm::database::states::bundle_state::BundleRetention;

use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;
pub use crate::traits::StateProvider;

pub type RecoveredTx = Recovered<TransactionSigned>;

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

            let executor = builder.into_executor();
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
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
        async move {
            // 1. Decode ALL transactions (must succeed or fail entire verification)
            let decoded_txs = decode_transactions(&block.transactions)
                .map_err(|_| EvmAppError::InvalidTransaction("Failed to decode all transactions".into()))?;

            // 2. Clone state for isolated re-execution
            let mut exec_state = {
                let db = self.state_db.read().unwrap();
                db.clone()
            };

            // 3. Build Header and Env
            let parent_header = build_sealed_header(parent);
            let timestamp = block.timestamp; // Use block timestamp
            
            // Validate timestamp (optional but good practice)
            // if timestamp != parent.timestamp + 12 { ... }

            let env_attributes = NextBlockEnvAttributes {
                timestamp,
                suggested_fee_recipient: Address::ZERO,
                prev_randao: B256::ZERO,
                gas_limit: 30_000_000,
                parent_beacon_block_root: Some(B256::ZERO),
                withdrawals: None,
                extra_data: Bytes::default(),
            };

            // 4. Build State and Executor
            let mut state = State::builder()
                .with_database(&mut exec_state)
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

            // 5. Execute all transactions
            for tx in decoded_txs {
                builder.execute_transaction(tx)
                    .map_err(|err| EvmAppError::Execution(format!("Transaction execution failed: {}", err)))?;
            }

            // 6. Finish execution
            let executor = builder.into_executor();
            let (evm, execution_result) = executor
                .finish()
                .map_err(|err| EvmAppError::Execution(err.to_string()))?;
            
            drop(evm);

            state.merge_transitions(BundleRetention::Reverts);
            let bundle = state.take_bundle();

            // 7. Apply bundle to cloned state (NOT canonical)
            exec_state.commit(&bundle);

            // 8. Compute all 4 fields
            let computed_state_root = exec_state.state_root();
            
            let computed_tx_root = ordered_trie_root_with_encoder(&block.transactions, |tx, out| {
                out.put_slice(tx.as_slice());
            });
            
            let computed_receipts_root = ordered_trie_root_with_encoder(&execution_result.receipts, |r, out| {
                r.with_bloom_ref().encode_2718(out);
            });
            
            let computed_gas_used: u64 = execution_result
                .receipts
                .iter()
                .map(|r| r.cumulative_gas_used())
                .sum();

            // 9. Compare all 4 fields
            if computed_state_root.0 != block.state_root {
                return Err(EvmAppError::StateRootMismatch {
                    expected: block.state_root,
                    computed: computed_state_root.0,
                });
            }

            if computed_tx_root.0 != block.transactions_root {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Transactions root mismatch: expected {:?}, computed {:?}",
                    block.transactions_root, computed_tx_root.0
                )));
            }

            if computed_receipts_root.0 != block.receipts_root {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Receipts root mismatch: expected {:?}, computed {:?}",
                    block.receipts_root, computed_receipts_root.0
                )));
            }

            if computed_gas_used != block.gas_used {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Gas used mismatch: expected {}, computed {}",
                    block.gas_used, computed_gas_used
                )));
            }

            // 10. Return ExecutionResult
            Ok(ExecutionResult {
                state_root: block.state_root,
                receipts_root: block.receipts_root,
                gas_used: block.gas_used,
                receipt_count: execution_result.receipts.len(),
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
    use reth_primitives_traits::SignerRecoverable;
    use state_memory::InMemoryStateDb;

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

    impl TxSource for MockTxSource {
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

    #[tokio::test]
    async fn verify_accepts_valid_block() {
        // Setup a valid block with a transaction
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
        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);

        let (app, db) = setup_app(vec![encoded]).await;
        
        // Fund sender
        let recovered = signed.recover_signer().unwrap();
        {
            let mut db = db.write().unwrap();
            let info = revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            };
            db.insert_account(recovered, info);
        }

        // Snapshot state before propose
        let pre_state = db.read().unwrap().clone();

        let parent = app.genesis().await;
        let (block, _) = app.propose(&parent, 1).await.unwrap();
        
        // Create new app with pre-state to verify
        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let pre_db = Arc::new(RwLock::new(pre_state));
        // Source doesn't matter for verify
        let source = Arc::new(MockTxSource { txs: vec![] });
        let verifier_app = EvmApplication::new(config, pre_db, source);

        let result = verifier_app.verify(&parent, &block).await;
        assert!(result.is_ok(), "Verify failed for valid block: {:?}", result.err());
    }

    #[tokio::test]
    async fn verify_rejects_wrong_state_root() {
        let (app, db) = setup_app(vec![]).await;
        let pre_state = db.read().unwrap().clone();
        
        let parent = app.genesis().await;
        let (mut block, _) = app.propose(&parent, 1).await.unwrap();
        
        // Corrupt state root
        block.state_root = [0xde; 32];
        
        // Verifier
        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let pre_db = Arc::new(RwLock::new(pre_state));
        let source = Arc::new(MockTxSource { txs: vec![] });
        let verifier_app = EvmApplication::new(config, pre_db, source);

        let result = verifier_app.verify(&parent, &block).await;
        assert!(matches!(result, Err(EvmAppError::StateRootMismatch { .. })));
    }

    #[tokio::test]
    async fn verify_rejects_undecodable_transactions() {
        let (app, db) = setup_app(vec![]).await;
        let pre_state = db.read().unwrap().clone();

        let parent = app.genesis().await;
        let (mut block, _) = app.propose(&parent, 1).await.unwrap();
        
        // Inject invalid RLP
        block.transactions.push(vec![0xde, 0xad, 0xbe, 0xef]);
        
        // Verifier
        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let pre_db = Arc::new(RwLock::new(pre_state));
        let source = Arc::new(MockTxSource { txs: vec![] });
        let verifier_app = EvmApplication::new(config, pre_db, source);

        let result = verifier_app.verify(&parent, &block).await;
        // Should fail decoding
        assert!(matches!(result, Err(EvmAppError::InvalidTransaction(_))));
    }

    #[tokio::test]
    async fn verify_rejects_wrong_gas_used() {
        // Need a transaction to have gas used
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
        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);

        let (app, db) = setup_app(vec![encoded]).await;
        
        // Fund sender
        let recovered = signed.recover_signer().unwrap();
        {
            let mut db = db.write().unwrap();
            let info = revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            };
            db.insert_account(recovered, info);
        }

        let pre_state = db.read().unwrap().clone();

        let parent = app.genesis().await;
        let (mut block, _) = app.propose(&parent, 1).await.unwrap();
        
        // Corrupt gas used
        block.gas_used += 1;
        
        // Verifier
        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let pre_db = Arc::new(RwLock::new(pre_state));
        let source = Arc::new(MockTxSource { txs: vec![] });
        let verifier_app = EvmApplication::new(config, pre_db, source);

        let result = verifier_app.verify(&parent, &block).await;
        assert!(result.is_err());
    }
}
