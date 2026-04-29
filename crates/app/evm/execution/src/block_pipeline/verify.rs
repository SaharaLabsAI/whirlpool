use alloy_consensus::TxReceipt;
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Bytes, B256};
use alloy_trie::root::ordered_trie_root_with_encoder;
use app_primitives::{
    header_extra_data::{decode_strict_extra_data, proposer_public_key_from_raw_eth_section},
    ExecutionResult, Receipt,
};
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
use validators_dkg::{
    latest_committed_full_dkg, validate_dkg_extra_data, DkgHistory, DkgVerifyInput,
};

use crate::block_pipeline::accounting::{aggregate_priority_fees, gas_deltas_and_used};
use crate::block_pipeline::build_sealed_header;
use crate::block_pipeline::validators::load_active_validator_dkg_inputs;
use crate::block_pipeline::{
    classify_tx_execution_error, expected_next_block_base_fee, map_epoch_boundary_runtime_error,
    map_post_block_accounting_runtime_error, map_validators_runtime_error,
    tx_is_reserved_epoch_namespace, BoundaryCallFailureMode, EvmApplication,
    TxExecutionErrorDisposition, BLOCK_GAS_LIMIT,
};
use crate::error::EvmAppError;
use crate::traits::StateDb;

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn verify_evm_transactions(
        &self,
        parent: &app_primitives::EvmBlock,
        block: &app_primitives::EvmBlock,
        raw_txs: &[Vec<u8>],
    ) -> Result<ExecutionResult, EvmAppError>
    where
        DB: StateDb + DkgHistory + Clone + revm::Database,
        <DB as StateDb>::Error: Into<EvmAppError>,
        <DB as DkgHistory>::Error: std::fmt::Display,
    {
        let decoded_txs = crate::codec::decode_evm_transactions(raw_txs)?;

        let mut exec_state = {
            let db = self.state_db.read().unwrap();
            db.clone()
        };
        let boundary_state = load_epoch_boundary_state(&exec_state).map_err(|err| {
            map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Verify)
        })?;
        let boundary_required =
            evm_precompiles::boundary_required_for_height(boundary_state, block.height);

        let parent_header = build_sealed_header(parent);
        let expected_base_fee_per_gas = expected_next_block_base_fee(parent);
        if block.base_fee_per_gas != expected_base_fee_per_gas {
            return Err(EvmAppError::InvalidBlock(format!(
                "base fee mismatch: expected {expected_base_fee_per_gas}, found {}",
                block.base_fee_per_gas
            )));
        }
        let decoded_extra_data = decode_strict_extra_data(&block.extra_data).map_err(|err| {
            EvmAppError::InvalidBlock(format!("failed to decode block extra_data: {err}"))
        })?;
        let decoded_proposer_public_key =
            proposer_public_key_from_raw_eth_section(&decoded_extra_data).map_err(|err| {
                EvmAppError::InvalidBlock(format!(
                    "failed to decode proposer public key from block extra_data: {err}"
                ))
            })?;
        if decoded_proposer_public_key != block.proposer_public_key {
            return Err(EvmAppError::InvalidBlock(format!(
                "block proposer key mismatch between block field and extra_data: field={:?}, extra_data={:?}",
                block.proposer_public_key, decoded_proposer_public_key
            )));
        }
        let claim_recipient = evm_precompiles::validate_active_validator_fee_recipient(
            &exec_state,
            decoded_proposer_public_key,
            block.proposer_fee_recipient,
        )
        .map_err(map_validators_runtime_error)?;

        let env_attributes = NextBlockEnvAttributes {
            timestamp: block.timestamp,
            suggested_fee_recipient: FEE_POOL_PRECOMPILE_ADDRESS,
            prev_randao: B256::ZERO,
            gas_limit: BLOCK_GAS_LIMIT,
            parent_beacon_block_root: Some(B256::ZERO),
            withdrawals: None,
            extra_data: Bytes::default(),
        };

        let mut state = State::builder()
            .with_database(&mut exec_state)
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
                    map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Verify)
                })?;

        for (index, tx) in decoded_txs.iter().enumerate() {
            if tx_is_reserved_epoch_namespace(tx, tx.signer()) {
                return Err(EvmAppError::InvalidBlock(format!(
                    "reserved epoch boundary namespace transaction at index {index}"
                )));
            }
        }

        for tx in decoded_txs.iter().cloned() {
            if let Err(err) = builder.execute_transaction(tx) {
                match classify_tx_execution_error(err) {
                    TxExecutionErrorDisposition::InvalidTxValidation(message)
                    | TxExecutionErrorDisposition::OtherValidation(message) => {
                        return Err(EvmAppError::InvalidBlock(format!(
                            "Transaction execution failed validation: {message}"
                        )))
                    }
                    TxExecutionErrorDisposition::Other(message) => {
                        return Err(EvmAppError::Execution(format!(
                            "Transaction execution failed: {message}"
                        )))
                    }
                }
            }
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
            aggregate_priority_fees(&decoded_txs, &gas_deltas, expected_base_fee_per_gas)?;
        exec_state.commit(&bundle).map_err(Into::into)?;
        if let Some(ref boundary_effect) = boundary_effect {
            apply_epoch_boundary_effect(&mut exec_state, boundary_effect).map_err(|err| {
                map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Verify)
            })?;
        }
        let dkg_inputs = load_active_validator_dkg_inputs(&exec_state, &self.evm_config)?;
        let current_epoch = apply_post_block_accounting(
            &mut exec_state,
            &PostBlockAccountingInputs {
                boundary_required,
                gas_used: computed_gas_used,
                base_fee_per_gas: expected_base_fee_per_gas,
                priority_fees,
                claim_recipient,
                simplex_validators: dkg_inputs.entries.clone(),
            },
        )
        .map_err(map_post_block_accounting_runtime_error)?
        .current_epoch;

        let computed_state_root = exec_state.state_root().map_err(Into::into)?;
        let computed_receipts_root = ordered_trie_root_with_encoder(
            &execution_result.receipts,
            |receipt: &reth_ethereum_primitives::Receipt, out| {
                receipt.with_bloom_ref().encode_2718(out);
            },
        );

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

        let latest_committed_full_dkg = {
            let db = self.state_db.read().unwrap();
            latest_committed_full_dkg(&*db, parent.height)
                .map_err(crate::block_pipeline::map_dkg_metadata_error)?
        };
        validate_dkg_extra_data(
            &decoded_extra_data,
            DkgVerifyInput {
                feature_enabled: self.evm_config.full_dkg_feature_enabled(),
                activation_schedule: &dkg_inputs.activation_schedule,
                default_players: &dkg_inputs.default_players,
                previous_full_dkg: latest_committed_full_dkg.as_ref(),
                candidate_output: self.evm_config.current_full_dkg_output(),
                boundary_required,
                post_advance_epoch: current_epoch,
            },
        )
        .map_err(|err| EvmAppError::InvalidBlock(err.to_string()))?;

        let receipts: Vec<Receipt> = execution_result
            .receipts
            .iter()
            .map(|r: &reth_ethereum_primitives::Receipt| Receipt {
                status: r.status().into(),
                cumulative_gas_used: r.cumulative_gas_used(),
                logs: r.logs().to_vec(),
            })
            .collect();

        self.receipt_store.stage_for_block(block, receipts);

        Ok(ExecutionResult {
            state_root: block.state_root,
            receipts_root: block.receipts_root,
            gas_used: block.gas_used,
            receipt_count: execution_result.receipts.len(),
        })
    }
}
