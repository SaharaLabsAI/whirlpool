use std::sync::{Arc, Mutex, RwLock};

use alloy_consensus::TxReceipt;
use alloy_eips::eip1559::{calc_next_block_base_fee, BaseFeeParams};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{bytes::BufMut, Address, Bytes, B256, U256};
use alloy_trie::{root::ordered_trie_root_with_encoder, EMPTY_ROOT_HASH};
use app::{
    traits::{Application, TxSource},
    EvmBlock, ExecutionResult, Receipt,
};
use community_pool::COMMUNITY_POOL_ADDRESS;
use reth_evm::{
    execute::{BlockBuilder, BlockExecutor},
    ConfigureEvm, NextBlockEnvAttributes,
};
use reth_primitives_traits::{Header, SealedHeader};
use reth_revm::State;
use revm::database::states::bundle_state::BundleRetention;
use state::BlockStorage;
use tx_dispatch::{decode_evm_transactions, RecoveredTx};

use crate::config::WhirlpoolEvmConfig;
use crate::error::EvmAppError;
pub use crate::traits::StateProvider;

#[derive(Clone, Debug)]
pub struct ProposedEvmPayload {
    pub included_transactions: Vec<Vec<u8>>,
    pub inclusion_outcomes: Vec<bool>,
    pub result: ExecutionResult,
    pub base_fee_per_gas: u64,
    pub proposer_public_key: [u8; 32],
    pub proposer_fee_recipient: Address,
    pub receipts: Vec<Receipt>,
}

/// Converts an `EvmBlock` into an Ethereum `Header`.
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
        extra_data: Bytes::copy_from_slice(&block.proposer_public_key),
        excess_blob_gas: Some(0),
        blob_gas_used: Some(0),
        ..Header::default()
    }
}

fn build_sealed_header(block: &EvmBlock) -> SealedHeader {
    let header = build_header_from_evm_block(block);
    let hash = header.hash_slow();
    SealedHeader::new(header, hash)
}

pub fn decode_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError> {
    decode_evm_transactions(raw_txs).map_err(|err| EvmAppError::InvalidBlock(err.to_string()))
}

fn credit_account_balance<DB>(
    db: &mut DB,
    address: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    if amount.is_zero() {
        return Ok(());
    }

    let mut info = db
        .get_account(address)
        .map_err(Into::into)?
        .unwrap_or_default();
    info.balance += amount;
    db.insert_account(address, info).map_err(Into::into)
}

fn credit_burned_fees<DB>(
    db: &mut DB,
    gas_used: u64,
    base_fee_per_gas: u64,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let burned_amount = U256::from(gas_used) * U256::from(base_fee_per_gas);
    credit_account_balance(db, COMMUNITY_POOL_ADDRESS, burned_amount)
}

fn validate_or_recover_fee_recipient(
    evm_config: &WhirlpoolEvmConfig,
    proposer_public_key: [u8; 32],
    carried_fee_recipient: [u8; 20],
) -> Result<Address, EvmAppError> {
    let carried_fee_recipient = Address::from(carried_fee_recipient);
    match evm_config.fee_recipient_for_proposer(proposer_public_key) {
        Some(expected) if expected != carried_fee_recipient => Err(EvmAppError::InvalidBlock(
            format!(
                "proposer fee recipient mismatch for proposer {:?}: expected {expected}, got {carried_fee_recipient}",
                proposer_public_key
            ),
        )),
        Some(expected) => Ok(expected),
        None => Ok(carried_fee_recipient),
    }
}

#[derive(Clone)]
pub struct EvmApplication<DB> {
    evm_config: WhirlpoolEvmConfig,
    state_db: Arc<RwLock<DB>>,
    tx_source: Arc<dyn TxSource + Send + Sync>,
    pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>,
    last_proposed: Arc<Mutex<Option<(u64, EvmBlock, ExecutionResult, Vec<Receipt>)>>>,
}

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn new(
        evm_config: WhirlpoolEvmConfig,
        state_db: Arc<RwLock<DB>>,
        tx_source: Arc<dyn TxSource + Send + Sync>,
    ) -> Self {
        Self {
            evm_config,
            state_db,
            tx_source,
            pending_receipts: Arc::new(Mutex::new(None)),
            last_proposed: Arc::new(Mutex::new(None)),
        }
    }

    pub fn store_finalized_block(
        &self,
        block: &EvmBlock,
        storage: &dyn BlockStorage,
    ) -> Result<(), EvmAppError> {
        let receipts = {
            let mut guard = self.pending_receipts.lock().unwrap();
            guard.take().unwrap_or_default()
        };
        storage
            .store_block(block, &receipts)
            .map_err(|e| EvmAppError::State(e.to_string()))
    }

    pub fn pending_receipts(&self) -> Vec<Receipt> {
        self.pending_receipts
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default()
    }

    pub fn propose_evm_transactions(
        &self,
        parent: &EvmBlock,
        raw_txs: &[Vec<u8>],
        timestamp: u64,
    ) -> Result<ProposedEvmPayload, EvmAppError>
    where
        DB: StateProvider + Clone + revm::Database,
        <DB as StateProvider>::Error: Into<EvmAppError>,
    {
        let decoded_txs = decode_transactions(raw_txs)?;
        let parent_header = build_sealed_header(parent);

        let mut state_snapshot = {
            let db = self.state_db.read().unwrap();
            db.clone()
        };

        let env_attributes = NextBlockEnvAttributes {
            timestamp,
            suggested_fee_recipient: self.evm_config.fee_recipient(),
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

        let mut included_transactions = Vec::new();
        let mut inclusion_outcomes = Vec::with_capacity(raw_txs.len());
        for (raw_tx, tx) in raw_txs.iter().cloned().zip(decoded_txs) {
            match builder.execute_transaction(tx) {
                Ok(_) => {
                    included_transactions.push(raw_tx);
                    inclusion_outcomes.push(true);
                }
                Err(reth_evm::execute::BlockExecutionError::Validation(
                    reth_evm::execute::BlockValidationError::InvalidTx { .. },
                )) => inclusion_outcomes.push(false),
                Err(err) => return Err(EvmAppError::Execution(err.to_string())),
            }
        }

        let executor = builder.into_executor();
        let (evm, execution_result) = executor
            .finish()
            .map_err(|err| EvmAppError::Execution(err.to_string()))?;
        drop(evm);

        state.merge_transitions(BundleRetention::Reverts);
        let bundle = state.take_bundle();

        let gas_used = execution_result
            .receipts
            .iter()
            .map(TxReceipt::cumulative_gas_used)
            .sum::<u64>();

        let receipts: Vec<Receipt> = execution_result
            .receipts
            .iter()
            .map(|r| Receipt {
                status: r.status().into(),
                cumulative_gas_used: r.cumulative_gas_used(),
                logs: r.logs().to_vec(),
            })
            .collect();

        {
            let mut guard = self.pending_receipts.lock().unwrap();
            *guard = Some(receipts.clone());
        }

        let base_fee_per_gas = calc_next_block_base_fee(
            parent.gas_used,
            30_000_000,
            parent.base_fee_per_gas,
            BaseFeeParams::ethereum(),
        );

        let state_root = {
            let mut canonical_db = self.state_db.write().unwrap();
            canonical_db.commit(&bundle).map_err(Into::into)?;
            credit_burned_fees(&mut *canonical_db, gas_used, base_fee_per_gas)?;
            canonical_db.state_root().map_err(Into::into)?
        };

        let receipts_root =
            ordered_trie_root_with_encoder(&execution_result.receipts, |receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            });

        Ok(ProposedEvmPayload {
            included_transactions,
            inclusion_outcomes,
            result: ExecutionResult {
                state_root: state_root.0,
                receipts_root: receipts_root.0,
                gas_used,
                receipt_count: execution_result.receipts.len(),
            },
            base_fee_per_gas,
            proposer_public_key: self.evm_config.local_proposer_public_key(),
            proposer_fee_recipient: self.evm_config.fee_recipient(),
            receipts,
        })
    }

    pub fn verify_evm_transactions(
        &self,
        parent: &EvmBlock,
        block: &EvmBlock,
        raw_txs: &[Vec<u8>],
    ) -> Result<ExecutionResult, EvmAppError>
    where
        DB: StateProvider + Clone + revm::Database,
        <DB as StateProvider>::Error: Into<EvmAppError>,
    {
        let decoded_txs = decode_transactions(raw_txs)?;

        let mut exec_state = {
            let db = self.state_db.read().unwrap();
            db.clone()
        };

        let parent_header = build_sealed_header(parent);
        let suggested_fee_recipient = validate_or_recover_fee_recipient(
            &self.evm_config,
            block.proposer_public_key,
            block.proposer_fee_recipient,
        )?;

        let env_attributes = NextBlockEnvAttributes {
            timestamp: block.timestamp,
            suggested_fee_recipient,
            prev_randao: B256::ZERO,
            gas_limit: 30_000_000,
            parent_beacon_block_root: Some(B256::ZERO),
            withdrawals: None,
            extra_data: Bytes::default(),
        };

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

        for tx in decoded_txs {
            builder.execute_transaction(tx).map_err(|err| {
                EvmAppError::Execution(format!("Transaction execution failed: {err}"))
            })?;
        }

        let executor = builder.into_executor();
        let (evm, execution_result) = executor
            .finish()
            .map_err(|err| EvmAppError::Execution(err.to_string()))?;
        drop(evm);

        state.merge_transitions(BundleRetention::Reverts);
        let bundle = state.take_bundle();
        exec_state.commit(&bundle).map_err(Into::into)?;
        credit_burned_fees(&mut exec_state, block.gas_used, block.base_fee_per_gas)?;

        let computed_state_root = exec_state.state_root().map_err(Into::into)?;
        let computed_receipts_root =
            ordered_trie_root_with_encoder(&execution_result.receipts, |receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            });
        let computed_gas_used = execution_result
            .receipts
            .iter()
            .map(TxReceipt::cumulative_gas_used)
            .sum::<u64>();

        if computed_state_root.0 != block.state_root {
            return Err(EvmAppError::StateRootMismatch {
                expected: block.state_root,
                computed: computed_state_root.0,
            });
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

        let receipts: Vec<Receipt> = execution_result
            .receipts
            .iter()
            .map(|r| Receipt {
                status: r.status().into(),
                cumulative_gas_used: r.cumulative_gas_used(),
                logs: r.logs().to_vec(),
            })
            .collect();

        {
            let mut guard = self.pending_receipts.lock().unwrap();
            *guard = Some(receipts);
        }

        Ok(ExecutionResult {
            state_root: block.state_root,
            receipts_root: block.receipts_root,
            gas_used: block.gas_used,
            receipt_count: execution_result.receipts.len(),
        })
    }
}

impl<DB> Application for EvmApplication<DB>
where
    DB: StateProvider + Clone + Send + Sync + 'static + revm::Database + std::fmt::Debug,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    type Block = EvmBlock;
    type Result = ExecutionResult;
    type Error = EvmAppError;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        async move {
            let state_root = {
                let db = self.state_db.read().unwrap();
                db.state_root()
                    .map_err(Into::into)
                    .expect("genesis state root should not fail")
            };

            EvmBlock {
                height: 0,
                parent_id: [0u8; 32],
                state_root: state_root.0,
                transactions_root: EMPTY_ROOT_HASH.0,
                receipts_root: EMPTY_ROOT_HASH.0,
                proposer_public_key: self.evm_config.local_proposer_public_key(),
                proposer_fee_recipient: self.evm_config.fee_recipient().into_array(),
                gas_used: 0,
                base_fee_per_gas: 1_000_000_000,
                timestamp: 0,
                transactions: vec![],
            }
        }
    }

    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl std::future::Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send
    {
        async move {
            {
                let cache = self.last_proposed.lock().unwrap();
                if let Some((cached_height, ref block, ref result, ref receipts)) = *cache {
                    if cached_height == height {
                        let mut guard = self.pending_receipts.lock().unwrap();
                        *guard = Some(receipts.clone());
                        return Ok((block.clone(), result.clone()));
                    }
                }
            }

            let raw_pending = self.tx_source.pending();
            let timestamp = parent.timestamp + 12;
            let payload = self.propose_evm_transactions(parent, &raw_pending, timestamp)?;

            let transactions_root =
                ordered_trie_root_with_encoder(&payload.included_transactions, |tx, out| {
                    out.put_slice(tx.as_slice());
                });

            let block = EvmBlock {
                height,
                parent_id: parent.compute_id(),
                state_root: payload.result.state_root,
                transactions_root: transactions_root.0,
                receipts_root: payload.result.receipts_root,
                proposer_public_key: payload.proposer_public_key,
                proposer_fee_recipient: payload.proposer_fee_recipient.into_array(),
                gas_used: payload.result.gas_used,
                base_fee_per_gas: payload.base_fee_per_gas,
                timestamp,
                transactions: payload.included_transactions,
            };

            {
                let mut cache = self.last_proposed.lock().unwrap();
                *cache = Some((
                    height,
                    block.clone(),
                    payload.result.clone(),
                    payload.receipts,
                ));
            }

            Ok((block, payload.result))
        }
    }

    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<Self::Result, Self::Error>> + Send {
        async move {
            let computed_tx_root =
                ordered_trie_root_with_encoder(&block.transactions, |tx, out| out.put_slice(tx));
            if computed_tx_root.0 != block.transactions_root {
                return Err(EvmAppError::InvalidBlock(format!(
                    "Transactions root mismatch: expected {:?}, computed {:?}",
                    block.transactions_root, computed_tx_root.0
                )));
            }

            self.verify_evm_transactions(parent, block, &block.transactions)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        build_sahara_chain_spec_with_alloc_and_fee_recipients, DEFAULT_PROPOSER_FEE_RECIPIENT,
    };
    use alloy_consensus::{SignableTransaction, TxLegacy};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, Signature, TxKind};
    use community_pool::COMMUNITY_POOL_ADDRESS;
    use reth_ethereum_primitives::TransactionSigned;
    use reth_primitives_traits::SignerRecoverable;
    use state_memory::InMemoryStateDb;
    use std::collections::BTreeMap;

    struct MockTxSource {
        txs: Vec<Vec<u8>>,
    }

    impl TxSource for MockTxSource {
        fn push(&self, _tx: Vec<u8>) {}

        fn pending(&self) -> Vec<Vec<u8>> {
            self.txs.clone()
        }
    }

    async fn setup_app(
        txs: Vec<Vec<u8>>,
    ) -> (
        EvmApplication<InMemoryStateDb>,
        Arc<RwLock<InMemoryStateDb>>,
    ) {
        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec);
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let source = Arc::new(MockTxSource { txs });

        let app = EvmApplication::new(config, db.clone(), source);
        (app, db)
    }

    async fn setup_app_with_config(
        txs: Vec<Vec<u8>>,
        config: WhirlpoolEvmConfig,
    ) -> (
        EvmApplication<InMemoryStateDb>,
        Arc<RwLock<InMemoryStateDb>>,
    ) {
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let source = Arc::new(MockTxSource { txs });
        let app = EvmApplication::new(config, db.clone(), source);
        (app, db)
    }

    fn sample_evm_tx() -> (Vec<u8>, Address) {
        let receiver = Address::with_last_byte(2);
        let tx = TxLegacy {
            chain_id: Some(crate::config::SAHARA_CHAIN_ID),
            nonce: 0,
            gas_price: 2_000_000_000,
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
        (encoded, recovered)
    }

    #[tokio::test]
    async fn propose_executes_transfer_transaction() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx]).await;

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
        let (block, result) = app.propose(&parent, 1).await.unwrap();

        assert_eq!(block.transactions.len(), 1);
        assert!(result.gas_used > 0);
    }

    #[tokio::test]
    async fn propose_credits_community_pool_and_fee_recipient() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx]).await;

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
        let (block, _result) = app.propose(&parent, 1).await.unwrap();
        let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);
        let expected_priority_fees =
            U256::from(block.gas_used) * U256::from(2_000_000_000u64 - block.base_fee_per_gas);

        let db = db.read().unwrap();
        let community_pool_balance = db
            .get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance;
        let fee_recipient_balance = db
            .get_account(DEFAULT_PROPOSER_FEE_RECIPIENT)
            .unwrap_or_default()
            .balance;

        assert_eq!(community_pool_balance, burned_amount);
        assert_eq!(fee_recipient_balance, expected_priority_fees);
        assert_eq!(
            block.proposer_fee_recipient,
            DEFAULT_PROPOSER_FEE_RECIPIENT.into_array()
        );
    }

    #[tokio::test]
    async fn verify_accepts_valid_block() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx]).await;

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
        let (block, _) = app.propose(&parent, 1).await.unwrap();

        let chain_spec = Arc::new(crate::config::build_sahara_chain_spec());
        let config =
            WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
        let pre_db = Arc::new(RwLock::new(pre_state));
        let source = Arc::new(MockTxSource { txs: vec![] });
        let verifier_app = EvmApplication::new(config, pre_db, source);

        assert!(verifier_app.verify(&parent, &block).await.is_ok());
    }

    #[tokio::test]
    async fn verify_rejects_fee_recipient_that_conflicts_with_genesis_mapping() {
        let proposer_public_key = [0x11; 32];
        let expected_fee_recipient = Address::repeat_byte(0x22);
        let mut validator_fee_recipients = BTreeMap::new();
        validator_fee_recipients.insert(proposer_public_key, expected_fee_recipient);

        let chain_spec = Arc::new(build_sahara_chain_spec_with_alloc_and_fee_recipients(
            BTreeMap::new(),
            validator_fee_recipients,
        ));
        let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
            .with_local_proposer_public_key(proposer_public_key);

        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app_with_config(vec![tx], proposer_config).await;

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
        block.proposer_fee_recipient = Address::repeat_byte(0x77).into_array();

        let verifier_config =
            WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x55; 32]);
        let verifier_app = EvmApplication::new(
            verifier_config,
            Arc::new(RwLock::new(pre_state)),
            Arc::new(MockTxSource { txs: vec![] }),
        );

        let err = verifier_app
            .verify(&parent, &block)
            .await
            .expect_err("genesis mapping should reject mismatched fee recipient");
        assert!(
            matches!(err, EvmAppError::InvalidBlock(_)),
            "expected invalid block error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn store_finalized_block_stores_and_clears_receipts() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx]).await;

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

        #[derive(Default)]
        struct MockBlockStorage {
            stored: Mutex<Vec<(EvmBlock, Vec<Receipt>)>>,
        }

        impl BlockStorage for MockBlockStorage {
            fn store_block(
                &self,
                block: &EvmBlock,
                receipts: &[Receipt],
            ) -> Result<(), state::BlockStorageError> {
                self.stored
                    .lock()
                    .unwrap()
                    .push((block.clone(), receipts.to_vec()));
                Ok(())
            }

            fn get_block_by_number(
                &self,
                _number: u64,
            ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
                Ok(None)
            }

            fn get_block_by_hash(
                &self,
                _hash: B256,
            ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
                Ok(None)
            }

            fn get_receipts_by_block(
                &self,
                _number: u64,
            ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
                Ok(None)
            }

            fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
                Ok(None)
            }
        }

        let storage = MockBlockStorage::default();
        app.store_finalized_block(&block, &storage).unwrap();

        let stored = storage.stored.lock().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].0.height, 1);
        assert_eq!(stored[0].1.len(), 1);
    }
}
