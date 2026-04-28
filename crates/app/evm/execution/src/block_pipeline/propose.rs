use alloy_consensus::TxReceipt;
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Bytes, B256};
use alloy_trie::root::ordered_trie_root_with_encoder;
use app::{ExecutionResult, Receipt};
use evm_precompiles::{
    apply_epoch_boundary_effect, apply_post_block_accounting,
    execute_epoch_boundary_system_call_if_required, load_epoch_boundary_state,
    PostBlockAccountingInputs, FEE_POOL_PRECOMPILE_ADDRESS,
};
use reth_evm::{
    execute::{BlockBuilder, BlockExecutor},
    ConfigureEvm, NextBlockEnvAttributes,
};
use revm::database::states::bundle_state::BundleRetention;
use revm::database::State;

use crate::block_pipeline::build_sealed_header;
use crate::block_pipeline::state_helpers::fee_accounting::aggregate_priority_fees;
use crate::block_pipeline::state_helpers::receipt_accounting::gas_deltas_and_used;
use crate::block_pipeline::{
    classify_tx_execution_error, expected_next_block_base_fee, map_epoch_boundary_runtime_error,
    map_post_block_accounting_runtime_error, tx_is_reserved_epoch_namespace,
    BoundaryCallFailureMode, EvmApplication, ProposedEvmPayload, TxExecutionErrorDisposition,
    BLOCK_GAS_LIMIT,
};
use crate::error::EvmAppError;
use crate::traits::StateDb;
use validators_dkg::{
    build_canonical_dkg_extra_data, latest_committed_full_dkg, DkgHistory, DkgProposalInput,
};

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn propose_evm_transactions(
        &self,
        parent: &app::EvmBlock,
        raw_txs: &[Vec<u8>],
        timestamp: u64,
        block_height: u64,
    ) -> Result<ProposedEvmPayload, EvmAppError>
    where
        DB: StateDb + DkgHistory + Clone + revm::Database,
        <DB as StateDb>::Error: Into<EvmAppError>,
        <DB as DkgHistory>::Error: std::fmt::Display,
    {
        let parent_header = build_sealed_header(parent);

        let mut state_snapshot = {
            let db = self.state_db.read().unwrap();
            db.clone()
        };
        let boundary_state = load_epoch_boundary_state(&state_snapshot).map_err(|err| {
            map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Propose)
        })?;
        let base_fee_per_gas = expected_next_block_base_fee(parent);
        let boundary_required =
            evm_precompiles::boundary_required_for_height(boundary_state, block_height);
        let decoded_txs = crate::codec::decode_evm_transactions(raw_txs)?;

        let env_attributes = NextBlockEnvAttributes {
            timestamp,
            suggested_fee_recipient: FEE_POOL_PRECOMPILE_ADDRESS,
            prev_randao: B256::ZERO,
            gas_limit: BLOCK_GAS_LIMIT,
            parent_beacon_block_root: Some(B256::ZERO),
            withdrawals: None,
            extra_data: Bytes::default(),
        };

        let mut state = State::builder()
            .with_database(&mut state_snapshot)
            .with_bundle_update()
            .build();

        let mut builder = self
            .evm_config
            .builder_for_next_block(&mut state, &parent_header, env_attributes)
            .map_err(|err| EvmAppError::Execution(err.to_string()))?;

        builder
            .apply_pre_execution_changes()
            .map_err(|err| EvmAppError::Execution(err.to_string()))?;

        let boundary_effect =
            execute_epoch_boundary_system_call_if_required(builder.evm_mut(), boundary_required)
                .map_err(|err| {
                    map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Propose)
                })?;

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
                Err(err) => match classify_tx_execution_error(err) {
                    TxExecutionErrorDisposition::InvalidTxValidation(_) => {
                        inclusion_outcomes.push(false);
                    }
                    TxExecutionErrorDisposition::OtherValidation(message)
                    | TxExecutionErrorDisposition::Other(message) => {
                        return Err(EvmAppError::Execution(message));
                    }
                },
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
            .map(|r: &reth_ethereum_primitives::Receipt| Receipt {
                status: r.status().into(),
                cumulative_gas_used: r.cumulative_gas_used(),
                logs: r.logs().to_vec(),
            })
            .collect();

        let (gas_deltas, gas_used) = gas_deltas_and_used(&execution_result.receipts)?;
        let priority_fees =
            aggregate_priority_fees(&executed_decoded_txs, &gas_deltas, base_fee_per_gas)?;
        let claim_recipient = self.evm_config.fee_recipient();

        let (state_root, current_epoch) = {
            let mut canonical_db = self.state_db.write().unwrap();
            canonical_db.commit(&bundle).map_err(Into::into)?;
            if let Some(ref boundary_effect) = boundary_effect {
                apply_epoch_boundary_effect(&mut *canonical_db, boundary_effect).map_err(
                    |err| map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Propose),
                )?;
            }
            let accounting_outcome = apply_post_block_accounting(
                &mut *canonical_db,
                &PostBlockAccountingInputs {
                    boundary_required,
                    gas_used,
                    base_fee_per_gas,
                    priority_fees,
                    claim_recipient,
                    simplex_validators: self.evm_config.simplex_validators().to_vec(),
                },
            )
            .map_err(map_post_block_accounting_runtime_error)?;
            (
                canonical_db.state_root().map_err(Into::into)?,
                accounting_outcome.current_epoch,
            )
        };

        let receipts_root = ordered_trie_root_with_encoder(
            &execution_result.receipts,
            |receipt: &reth_ethereum_primitives::Receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            },
        );

        let latest_committed_full_dkg = {
            let db = self.state_db.read().unwrap();
            latest_committed_full_dkg(&*db, parent.height)
                .map_err(crate::block_pipeline::map_dkg_metadata_error)?
        };
        let extra_data = build_canonical_dkg_extra_data(DkgProposalInput {
            feature_enabled: self.evm_config.full_dkg_feature_enabled(),
            activation_schedule: &self.evm_config.validator_activation_schedule(),
            default_players: &self.evm_config.simplex_consensus_public_keys(),
            previous_full_dkg: latest_committed_full_dkg.as_ref(),
            candidate_output: self.evm_config.current_full_dkg_output(),
            proposer_public_key: self.evm_config.local_proposer_public_key(),
            boundary_required,
            post_advance_epoch: current_epoch,
        })
        .map_err(|err| EvmAppError::InvalidBlock(format!("invalid canonical extra_data: {err}")))?;

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
            extra_data,
            receipts,
        })
    }
}
