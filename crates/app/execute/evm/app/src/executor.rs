use std::sync::{Arc, Mutex, RwLock};

use alloy_consensus::{Transaction, TxReceipt};
use alloy_eips::eip1559::{calc_next_block_base_fee, BaseFeeParams};
use alloy_eips::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::{bytes::BufMut, Address, Bytes, B256, U256};
use alloy_trie::{root::ordered_trie_root_with_encoder, EMPTY_ROOT_HASH};
use app::{
    traits::{Application, TxSource},
    EvmBlock, ExecutionResult, Receipt,
};
use evm_precompiles::{
    claimable_balance_slot, COMMUNITY_POOL_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
};
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::{
    execute::{BlockBuilder, BlockExecutor},
    ConfigureEvm, NextBlockEnvAttributes,
};
use reth_primitives_traits::{Header, SealedHeader};
use reth_primitives_traits::{Recovered, SignedTransaction};
use reth_revm::State;
use revm::database::states::bundle_state::BundleRetention;
use state::BlockStorage;

use crate::config::WhirlpoolEvmConfig;
use crate::epoch_boundary::{
    apply_boundary_state_to_provider, boundary_required_for_height,
    execute_epoch_boundary_system_call_if_required, load_epoch_boundary_state,
    tx_is_reserved_epoch_namespace, BoundaryCallFailureMode,
};
use crate::error::EvmAppError;
pub use crate::traits::StateProvider;

pub type RecoveredTx = Recovered<TransactionSigned>;

#[derive(Clone, Debug)]
pub struct ProposedEvmPayload {
    pub included_user_transactions: Vec<Vec<u8>>,
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

fn credit_fee_pool_claim<DB>(
    db: &mut DB,
    recipient: Address,
    amount: U256,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    if amount.is_zero() {
        return Ok(());
    }

    let slot = claimable_balance_slot(recipient);
    let current = db
        .get_storage(FEE_POOL_PRECOMPILE_ADDRESS, slot)
        .map_err(Into::into)?;
    let next = current
        .checked_add(amount)
        .ok_or_else(|| EvmAppError::Execution("fee-pool claim ledger overflow".into()))?;

    db.insert_storage(FEE_POOL_PRECOMPILE_ADDRESS, slot, next)
        .map_err(Into::into)
}

fn gas_deltas_and_used<R>(receipts: &[R]) -> Result<(Vec<u64>, u64), EvmAppError>
where
    R: TxReceipt,
{
    let mut previous = 0_u64;
    let mut deltas = Vec::with_capacity(receipts.len());

    for receipt in receipts {
        let cumulative = receipt.cumulative_gas_used();
        let delta = cumulative.checked_sub(previous).ok_or_else(|| {
            EvmAppError::InvalidBlock(format!(
                "receipt cumulative gas must be nondecreasing: previous={previous}, current={cumulative}"
            ))
        })?;
        deltas.push(delta);
        previous = cumulative;
    }

    Ok((deltas, previous))
}

fn aggregate_priority_fees(
    txs: &[RecoveredTx],
    gas_deltas: &[u64],
    base_fee_per_gas: u64,
) -> Result<U256, EvmAppError> {
    if txs.len() != gas_deltas.len() {
        return Err(EvmAppError::Execution(format!(
            "priority-fee aggregation requires matching tx/receipt counts, got txs={}, gas_deltas={}",
            txs.len(),
            gas_deltas.len()
        )));
    }

    let mut total = U256::ZERO;
    for (tx, gas_delta) in txs.iter().zip(gas_deltas.iter()) {
        let tip_per_gas = tx.effective_tip_per_gas(base_fee_per_gas).ok_or_else(|| {
            EvmAppError::InvalidBlock("transaction tip under base fee is invalid".into())
        })?;
        let fee = U256::from(*gas_delta)
            .checked_mul(U256::from(tip_per_gas))
            .ok_or_else(|| EvmAppError::Execution("priority-fee multiplication overflow".into()))?;
        total = total
            .checked_add(fee)
            .ok_or_else(|| EvmAppError::Execution("priority-fee accumulation overflow".into()))?;
    }

    Ok(total)
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
        block_height: u64,
    ) -> Result<ProposedEvmPayload, EvmAppError>
    where
        DB: StateProvider + Clone + revm::Database,
        <DB as StateProvider>::Error: Into<EvmAppError>,
    {
        let parent_header = build_sealed_header(parent);

        let mut state_snapshot = {
            let db = self.state_db.read().unwrap();
            db.clone()
        };
        let boundary_state = load_epoch_boundary_state(&state_snapshot)?;
        let base_fee_per_gas = calc_next_block_base_fee(
            parent.gas_used,
            30_000_000,
            parent.base_fee_per_gas,
            BaseFeeParams::ethereum(),
        );
        let boundary_required = boundary_required_for_height(boundary_state, block_height);
        let decoded_txs = decode_evm_transactions(raw_txs)?;

        let env_attributes = NextBlockEnvAttributes {
            timestamp,
            suggested_fee_recipient: FEE_POOL_PRECOMPILE_ADDRESS,
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

        let boundary_state_changes = execute_epoch_boundary_system_call_if_required(
            builder.evm_mut(),
            boundary_required,
            BoundaryCallFailureMode::Propose,
        )?;

        let mut included_user_transactions = Vec::new();
        let mut executed_decoded_txs = Vec::new();
        let mut inclusion_outcomes = Vec::with_capacity(raw_txs.len());

        for (raw_tx, tx) in raw_txs.iter().cloned().zip(decoded_txs) {
            if tx_is_reserved_epoch_namespace(&tx, tx.signer()) {
                inclusion_outcomes.push(false);
                continue;
            }

            match builder.execute_transaction(tx.clone()) {
                Ok(_) => {
                    included_user_transactions.push(raw_tx);
                    executed_decoded_txs.push(tx);
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

        let (gas_deltas, gas_used) = gas_deltas_and_used(&execution_result.receipts)?;
        let priority_fees =
            aggregate_priority_fees(&executed_decoded_txs, &gas_deltas, base_fee_per_gas)?;
        let claim_recipient = self.evm_config.fee_recipient();

        let state_root = {
            let mut canonical_db = self.state_db.write().unwrap();
            canonical_db.commit(&bundle).map_err(Into::into)?;
            if let Some(ref boundary_state_changes) = boundary_state_changes {
                apply_boundary_state_to_provider(&mut *canonical_db, boundary_state_changes)?;
            }
            credit_burned_fees(&mut *canonical_db, gas_used, base_fee_per_gas)?;
            credit_fee_pool_claim(&mut *canonical_db, claim_recipient, priority_fees)?;
            canonical_db.state_root().map_err(Into::into)?
        };

        let receipts_root =
            ordered_trie_root_with_encoder(&execution_result.receipts, |receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            });

        Ok(ProposedEvmPayload {
            included_user_transactions,
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
        let decoded_txs = decode_evm_transactions(raw_txs)?;

        let mut exec_state = {
            let db = self.state_db.read().unwrap();
            db.clone()
        };
        let boundary_state = load_epoch_boundary_state(&exec_state)?;
        let boundary_required = boundary_required_for_height(boundary_state, block.height);

        let parent_header = build_sealed_header(parent);
        let claim_recipient = validate_or_recover_fee_recipient(
            &self.evm_config,
            block.proposer_public_key,
            block.proposer_fee_recipient,
        )?;

        let env_attributes = NextBlockEnvAttributes {
            timestamp: block.timestamp,
            suggested_fee_recipient: FEE_POOL_PRECOMPILE_ADDRESS,
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

        let boundary_state_changes = execute_epoch_boundary_system_call_if_required(
            builder.evm_mut(),
            boundary_required,
            BoundaryCallFailureMode::Verify,
        )?;

        for (index, tx) in decoded_txs.iter().enumerate() {
            if tx_is_reserved_epoch_namespace(tx, tx.signer()) {
                return Err(EvmAppError::InvalidBlock(format!(
                    "reserved epoch boundary namespace transaction at index {index}"
                )));
            }
        }

        for tx in decoded_txs.iter().cloned() {
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
        let (gas_deltas, computed_gas_used) = gas_deltas_and_used(&execution_result.receipts)?;
        let priority_fees =
            aggregate_priority_fees(&decoded_txs, &gas_deltas, block.base_fee_per_gas)?;
        exec_state.commit(&bundle).map_err(Into::into)?;
        if let Some(ref boundary_state_changes) = boundary_state_changes {
            apply_boundary_state_to_provider(&mut exec_state, boundary_state_changes)?;
        }
        credit_burned_fees(&mut exec_state, computed_gas_used, block.base_fee_per_gas)?;
        credit_fee_pool_claim(&mut exec_state, claim_recipient, priority_fees)?;

        let computed_state_root = exec_state.state_root().map_err(Into::into)?;
        let computed_receipts_root =
            ordered_trie_root_with_encoder(&execution_result.receipts, |receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            });

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
            let payload = self.propose_evm_transactions(parent, &raw_pending, timestamp, height)?;

            let block_transactions = payload.included_user_transactions.clone();

            let transactions_root =
                ordered_trie_root_with_encoder(&block_transactions, |tx, out| {
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
                transactions: block_transactions,
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
    use crate::config::DEFAULT_PROPOSER_FEE_RECIPIENT;
    use alloy_consensus::{SignableTransaction, TxLegacy};
    use alloy_eips::eip2718::Encodable2718;
    use alloy_primitives::{Address, Signature, TxKind};
    use chainspec::{
        build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients,
        SAHARA_CHAIN_ID,
    };
    use evm_precompiles::{
        advance_epoch_calldata, claimable_balance_slot, current_epoch_slot, epoch_blocks_slot,
        epoch_system_tx_sender, next_epoch_block_slot, withdraw_calldata, COMMUNITY_POOL_ADDRESS,
        EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS, EPOCH_SYSTEM_TX_GAS_LIMIT,
        EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI, EPOCH_SYSTEM_TX_PRIVATE_KEY,
        FEE_POOL_PRECOMPILE_ADDRESS,
    };
    use reth_ethereum_primitives::TransactionSigned;
    use reth_primitives_traits::crypto::secp256k1::sign_message;
    use reth_primitives_traits::SignerRecoverable;
    use revm::state::Bytecode;
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
        let chain_spec = Arc::new(build_sahara_chain_spec());
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

    fn seed_epoch_boundary_state(db: &mut InMemoryStateDb, next_epoch_block: u64) {
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            current_epoch_slot(),
            U256::from(0_u64),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            epoch_blocks_slot(),
            U256::from(EPOCH_BLOCKS_DEFAULT),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(next_epoch_block),
        );
        db.insert_account(
            epoch_system_tx_sender(),
            revm::state::AccountInfo {
                balance: U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    fn sample_evm_tx_with_nonce(nonce: u64, receiver: Address) -> (Vec<u8>, Address) {
        let tx = TxLegacy {
            chain_id: Some(SAHARA_CHAIN_ID),
            nonce,
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

    fn sample_evm_tx() -> (Vec<u8>, Address) {
        sample_evm_tx_with_nonce(0, Address::with_last_byte(2))
    }

    fn sample_reserved_epoch_namespace_tx(nonce: u64, gas_price: u128) -> Vec<u8> {
        let tx = TxLegacy {
            chain_id: Some(SAHARA_CHAIN_ID),
            nonce,
            gas_price,
            gas_limit: EPOCH_SYSTEM_TX_GAS_LIMIT,
            to: TxKind::Call(EPOCH_PRECOMPILE_ADDRESS),
            value: U256::ZERO,
            input: advance_epoch_calldata(),
        };
        let signature = sign_message(EPOCH_SYSTEM_TX_PRIVATE_KEY, tx.signature_hash())
            .expect("epoch system tx signature");
        let signed: TransactionSigned = tx.into_signed(signature).into();

        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);
        encoded
    }

    fn precompile_proxy_runtime_bytecode() -> Bytes {
        let mut runtime = alloy_primitives::hex::decode("36600060003760006000366000600073")
            .expect("forwarder prefix");
        runtime.extend_from_slice(FEE_POOL_PRECOMPILE_ADDRESS.as_slice());
        runtime.extend_from_slice(
            &alloy_primitives::hex::decode("5af13d600060003e156034573d6000f35b3d6000fd")
                .expect("forwarder suffix"),
        );
        Bytes::from(runtime)
    }

    fn sample_proxy_precompile_withdraw_tx(proxy_address: Address) -> (Vec<u8>, Address) {
        let tx = TxLegacy {
            chain_id: Some(SAHARA_CHAIN_ID),
            nonce: 0,
            gas_price: 2_000_000_000,
            gas_limit: 200_000,
            to: TxKind::Call(proxy_address),
            value: U256::ZERO,
            input: withdraw_calldata(),
        };
        let signature = Signature::test_signature();
        let signed: TransactionSigned = tx.into_signed(signature).into();
        let recovered = signed.recover_signer().unwrap();

        let mut encoded = Vec::new();
        signed.encode_2718(&mut encoded);
        (encoded, recovered)
    }

    #[test]
    fn decode_evm_transaction_recovers_signer() {
        let (raw_tx, recovered) = sample_evm_tx();

        let decoded = decode_evm_transaction(&raw_tx).expect("tx should decode");

        assert_eq!(decoded.signer(), recovered);
    }

    #[test]
    fn decode_evm_transactions_reject_invalid_bytes() {
        let err = decode_evm_transactions(&[vec![0xff, 0x00, 0x01]])
            .expect_err("invalid bytes should fail decoding");

        assert!(matches!(err, EvmAppError::InvalidBlock(_)));
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
    async fn propose_routes_priority_fees_to_fee_pool_not_proposer() {
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
        let claim_slot = claimable_balance_slot(DEFAULT_PROPOSER_FEE_RECIPIENT);

        let db = db.read().unwrap();
        let community_pool_balance = db
            .get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance;
        let fee_pool_balance = db
            .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
            .unwrap_or_default()
            .balance;
        let fee_recipient_balance = db
            .get_account(DEFAULT_PROPOSER_FEE_RECIPIENT)
            .unwrap_or_default()
            .balance;
        let claimable = db.get_storage(FEE_POOL_PRECOMPILE_ADDRESS, claim_slot);

        assert_eq!(community_pool_balance, burned_amount);
        assert_eq!(fee_pool_balance, expected_priority_fees);
        assert_eq!(claimable, expected_priority_fees);
        assert_eq!(fee_recipient_balance, U256::ZERO);
        assert_eq!(
            block.proposer_fee_recipient,
            DEFAULT_PROPOSER_FEE_RECIPIENT.into_array()
        );
    }

    #[tokio::test]
    async fn propose_uses_final_cumulative_gas_used_for_block_gas_and_burned_fee_credit() {
        let (tx0, recovered0) = sample_evm_tx_with_nonce(0, Address::with_last_byte(2));
        let (tx1, recovered1) = (3u8..=u8::MAX)
            .map(|byte| sample_evm_tx_with_nonce(0, Address::with_last_byte(byte)))
            .find(|(_, recovered)| *recovered != recovered0)
            .expect("must find a second sender");
        let (app, db) = setup_app(vec![tx0, tx1]).await;

        {
            let mut db = db.write().unwrap();
            for recovered in [recovered0, recovered1] {
                let info = revm::state::AccountInfo {
                    balance: U256::from(1_000_000_000_000_000_000u64),
                    nonce: 0,
                    ..Default::default()
                };
                db.insert_account(recovered, info);
            }
        }

        let parent = app.genesis().await;
        let (block, _result) = app.propose(&parent, 1).await.unwrap();
        let receipts = app.pending_receipts();
        assert_eq!(receipts.len(), 2, "expected two successful tx receipts");

        let expected_gas_used = receipts.last().expect("has receipts").cumulative_gas_used;
        assert_eq!(
            block.gas_used, expected_gas_used,
            "block gas used should equal final cumulative gas"
        );

        let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);
        let expected_priority_fees =
            U256::from(block.gas_used) * U256::from(2_000_000_000u64 - block.base_fee_per_gas);
        let claim_slot = claimable_balance_slot(DEFAULT_PROPOSER_FEE_RECIPIENT);

        let db = db.read().unwrap();
        let community_pool_balance = db
            .get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance;
        let fee_pool_balance = db
            .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
            .unwrap_or_default()
            .balance;
        let claimable = db.get_storage(FEE_POOL_PRECOMPILE_ADDRESS, claim_slot);

        assert_eq!(
            community_pool_balance, burned_amount,
            "community pool burn credit should use corrected block gas used"
        );
        assert_eq!(
            fee_pool_balance, expected_priority_fees,
            "fee-pool sink should be credited exactly once by execution beneficiary"
        );
        assert_eq!(claimable, expected_priority_fees);
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

        let chain_spec = Arc::new(build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
        let pre_db = Arc::new(RwLock::new(pre_state));
        let source = Arc::new(MockTxSource { txs: vec![] });
        let verifier_app = EvmApplication::new(config, pre_db, source);

        assert!(verifier_app.verify(&parent, &block).await.is_ok());
    }

    #[tokio::test]
    async fn verify_accepts_block_with_precompile_proxy_transaction() {
        let proxy_address = Address::with_last_byte(0xaa);
        let (tx, recovered) = sample_proxy_precompile_withdraw_tx(proxy_address);
        let (app, db) = setup_app(vec![tx]).await;
        let claimable = U256::from(5_u64);

        {
            let mut db = db.write().unwrap();
            db.insert_account(
                recovered,
                revm::state::AccountInfo {
                    balance: U256::from(1_000_000_000_000_000_000u64),
                    nonce: 0,
                    ..Default::default()
                },
            );
            let mut proxy_info = revm::state::AccountInfo::default();
            proxy_info.set_code(Bytecode::new_raw(precompile_proxy_runtime_bytecode()));
            db.insert_account(proxy_address, proxy_info);
            let mut fee_pool_info = revm::state::AccountInfo::default();
            fee_pool_info.balance = claimable;
            db.insert_account(FEE_POOL_PRECOMPILE_ADDRESS, fee_pool_info);
            db.insert_storage(
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(proxy_address),
                claimable,
            );
        }

        let pre_state = db.read().unwrap().clone();
        let parent = app.genesis().await;
        let (block, _) = app.propose(&parent, 1).await.unwrap();
        let current_balance = db
            .read()
            .unwrap()
            .get_account(proxy_address)
            .unwrap_or_default()
            .balance;
        assert_eq!(current_balance, claimable);

        let chain_spec = Arc::new(build_sahara_chain_spec());
        let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
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
    async fn boundary_block_keeps_user_transactions_only() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx.clone()]).await;

        {
            let mut db = db.write().unwrap();
            seed_epoch_boundary_state(&mut db, 1);
            db.insert_account(
                recovered,
                revm::state::AccountInfo {
                    balance: U256::from(1_000_000_000_000_000_000u64),
                    nonce: 0,
                    ..Default::default()
                },
            );
        }

        let parent = app.genesis().await;
        let (block, result) = app
            .propose(&parent, 1)
            .await
            .expect("propose boundary block");

        assert_eq!(block.transactions, vec![tx]);
        assert_eq!(result.receipt_count, 1);
    }

    #[tokio::test]
    async fn propose_excludes_reserved_epoch_namespace_transaction() {
        let reserved_tx = sample_reserved_epoch_namespace_tx(0, 2_000_000_000);
        let (user_tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![reserved_tx, user_tx.clone()]).await;

        {
            let mut db = db.write().unwrap();
            seed_epoch_boundary_state(&mut db, 10);
            db.insert_account(
                recovered,
                revm::state::AccountInfo {
                    balance: U256::from(1_000_000_000_000_000_000u64),
                    nonce: 0,
                    ..Default::default()
                },
            );
        }

        let parent = app.genesis().await;
        let (block, result) = app
            .propose(&parent, 1)
            .await
            .expect("propose should skip reserved namespace transaction");

        assert_eq!(block.transactions, vec![user_tx]);
        assert_eq!(result.receipt_count, 1);
        assert_eq!(app.pending_receipts().len(), 1);
    }

    #[tokio::test]
    async fn boundary_block_system_call_advances_epoch_state_once() {
        let (app, db) = setup_app(vec![]).await;

        {
            let mut db = db.write().unwrap();
            seed_epoch_boundary_state(&mut db, 1);
        }
        {
            let db = db.read().unwrap();
            let boundary_state = crate::epoch_boundary::load_epoch_boundary_state(&*db)
                .expect("load boundary state");
            assert_eq!(boundary_state.next_epoch_block, 1);
        }

        let parent = app.genesis().await;
        let (boundary_block, _) = app
            .propose(&parent, 1)
            .await
            .expect("propose boundary block");
        {
            let db = db.read().unwrap();
            assert_eq!(
                db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
                U256::from(1_u64)
            );
        }
        let (_next_block, _) = app
            .propose(&boundary_block, 2)
            .await
            .expect("propose non-boundary block");

        let db = db.read().unwrap();
        assert_eq!(
            db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
            U256::from(1_u64)
        );
        assert_eq!(
            db.get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot()),
            U256::from(1_u64 + EPOCH_BLOCKS_DEFAULT)
        );
    }

    #[tokio::test]
    async fn boundary_block_receipts_and_gas_are_user_only() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx]).await;

        {
            let mut db = db.write().unwrap();
            seed_epoch_boundary_state(&mut db, 1);
            db.insert_account(
                recovered,
                revm::state::AccountInfo {
                    balance: U256::from(1_000_000_000_000_000_000u64),
                    nonce: 0,
                    ..Default::default()
                },
            );
        }

        let parent = app.genesis().await;
        let (block, result) = app
            .propose(&parent, 1)
            .await
            .expect("propose boundary block");
        let receipts = app.pending_receipts();

        assert_eq!(block.transactions.len(), 1);
        assert_eq!(receipts.len(), 1);
        assert_eq!(result.receipt_count, 1);
        assert_eq!(
            block.gas_used,
            receipts
                .last()
                .expect("must have receipt")
                .cumulative_gas_used
        );
    }

    #[tokio::test]
    async fn verify_accepts_boundary_block_with_user_only_transactions() {
        let (tx, recovered) = sample_evm_tx();
        let (app, db) = setup_app(vec![tx]).await;

        {
            let mut db = db.write().unwrap();
            seed_epoch_boundary_state(&mut db, 1);
            db.insert_account(
                recovered,
                revm::state::AccountInfo {
                    balance: U256::from(1_000_000_000_000_000_000u64),
                    nonce: 0,
                    ..Default::default()
                },
            );
        }

        let pre_state = db.read().unwrap().clone();
        let parent = app.genesis().await;
        let (block, _) = app
            .propose(&parent, 1)
            .await
            .expect("propose boundary block");

        let verifier = EvmApplication::new(
            WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
            Arc::new(RwLock::new(pre_state)),
            Arc::new(MockTxSource { txs: vec![] }),
        );

        assert!(verifier.verify(&parent, &block).await.is_ok());
    }

    #[tokio::test]
    async fn verify_rejects_reserved_epoch_namespace_transaction() {
        let reserved_tx = sample_reserved_epoch_namespace_tx(0, 2_000_000_000);
        let (app, db) = setup_app(vec![]).await;

        {
            let mut db = db.write().unwrap();
            seed_epoch_boundary_state(&mut db, 10);
        }

        let pre_state = db.read().unwrap().clone();
        let parent = app.genesis().await;
        let block = EvmBlock {
            height: 1,
            parent_id: parent.compute_id(),
            state_root: parent.state_root,
            transactions_root: ordered_trie_root_with_encoder(&[reserved_tx.clone()], |tx, out| {
                out.put_slice(tx)
            })
            .0,
            receipts_root: EMPTY_ROOT_HASH.0,
            proposer_public_key: parent.proposer_public_key,
            proposer_fee_recipient: parent.proposer_fee_recipient,
            gas_used: 0,
            base_fee_per_gas: parent.base_fee_per_gas,
            timestamp: parent.timestamp + 12,
            transactions: vec![reserved_tx],
        };

        let verifier = EvmApplication::new(
            WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
            Arc::new(RwLock::new(pre_state)),
            Arc::new(MockTxSource { txs: vec![] }),
        );

        let err = verifier
            .verify(&parent, &block)
            .await
            .expect_err("reserved epoch namespace tx must be invalid");
        assert!(matches!(err, EvmAppError::InvalidBlock(_)));
        assert!(
            err.to_string()
                .contains("reserved epoch boundary namespace transaction"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn propose_rejects_when_required_boundary_system_call_fails() {
        let (app, db) = setup_app(vec![]).await;

        {
            let mut db = db.write().unwrap();
            db.insert_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                current_epoch_slot(),
                U256::from(u64::MAX),
            );
            db.insert_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                epoch_blocks_slot(),
                U256::from(EPOCH_BLOCKS_DEFAULT),
            );
            db.insert_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                next_epoch_block_slot(),
                U256::from(1_u64),
            );
        }

        let parent = app.genesis().await;
        let err = app
            .propose(&parent, 1)
            .await
            .expect_err("boundary system call failure must fail proposal");
        assert!(matches!(err, EvmAppError::Execution(_)));
    }

    #[tokio::test]
    async fn verify_rejects_when_required_boundary_system_call_fails() {
        let (app, db) = setup_app(vec![]).await;

        {
            let mut db = db.write().unwrap();
            db.insert_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                current_epoch_slot(),
                U256::from(u64::MAX),
            );
            db.insert_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                epoch_blocks_slot(),
                U256::from(EPOCH_BLOCKS_DEFAULT),
            );
            db.insert_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                next_epoch_block_slot(),
                U256::from(1_u64),
            );
        }

        let parent = app.genesis().await;
        let boundary_block = EvmBlock {
            height: 1,
            parent_id: parent.compute_id(),
            state_root: parent.state_root,
            transactions_root: EMPTY_ROOT_HASH.0,
            receipts_root: EMPTY_ROOT_HASH.0,
            proposer_public_key: parent.proposer_public_key,
            proposer_fee_recipient: parent.proposer_fee_recipient,
            gas_used: 0,
            base_fee_per_gas: parent.base_fee_per_gas,
            timestamp: parent.timestamp + 12,
            transactions: vec![],
        };

        let err = app
            .verify(&parent, &boundary_block)
            .await
            .expect_err("boundary system call must fail verification");
        assert!(matches!(err, EvmAppError::InvalidBlock(_)));
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
