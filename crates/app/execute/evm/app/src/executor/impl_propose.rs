use super::*;

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn propose_evm_transactions(
        &self,
        parent: &EvmBlock,
        raw_txs: &[Vec<u8>],
        timestamp: u64,
        block_height: u64,
    ) -> Result<ProposedEvmPayload, EvmAppError>
    where
        DB: StateProvider + BlockStorage + Clone + revm::Database,
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
            if let Some(ref boundary_state_changes) = boundary_state_changes {
                apply_boundary_state_to_provider(&mut *canonical_db, boundary_state_changes)?;
            }
            maybe_apply_community_pool_unlock(
                &mut *canonical_db,
                boundary_required,
                self.evm_config.simplex_validators(),
            )?;
            let current_epoch = load_u64_storage_value(
                &*canonical_db,
                EPOCH_PRECOMPILE_ADDRESS,
                current_epoch_slot(),
                "epoch currentEpoch",
            )?;
            credit_burned_fees(&mut *canonical_db, gas_used, base_fee_per_gas)?;
            credit_fee_pool_claim(&mut *canonical_db, claim_recipient, priority_fees)?;
            (
                canonical_db.state_root().map_err(Into::into)?,
                current_epoch,
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
            latest_committed_full_dkg(&*db, parent.height)?
        };
        let extra_data = build_canonical_extra_data(
            &self.evm_config,
            latest_committed_full_dkg.as_ref(),
            self.evm_config.local_proposer_public_key(),
            boundary_required,
            current_epoch,
        )?;

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
