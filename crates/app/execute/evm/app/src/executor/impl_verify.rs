use super::*;

impl<DB> EvmApplication<DB>
where
    DB: std::fmt::Debug,
{
    pub fn verify_evm_transactions(
        &self,
        parent: &EvmBlock,
        block: &EvmBlock,
        raw_txs: &[Vec<u8>],
    ) -> Result<ExecutionResult, EvmAppError>
    where
        DB: StateDb + BlockStorage + Clone + revm::Database,
        <DB as StateDb>::Error: Into<EvmAppError>,
    {
        let decoded_txs = decode_evm_transactions(raw_txs)?;

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
        let decoded_extra_data = decode_extra_data(
            &block.extra_data,
            extra_data_decode_mode_for_height(&self.evm_config, block.height),
        )
        .map_err(|err| {
            EvmAppError::InvalidBlock(format!("failed to decode block extra_data: {err}"))
        })?;
        let decoded_proposer_public_key =
            proposer_public_key_from_raw_eth_section(&decoded_extra_data)?;
        if decoded_proposer_public_key != block.proposer_public_key {
            return Err(EvmAppError::InvalidBlock(format!(
                "block proposer key mismatch between block field and extra_data: field={:?}, extra_data={:?}",
                block.proposer_public_key, decoded_proposer_public_key
            )));
        }
        let claim_recipient = validate_or_recover_fee_recipient(
            &self.evm_config,
            decoded_proposer_public_key,
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
                        return Err(EvmAppError::InvalidBlock(
                            format!("Transaction execution failed validation: {message}"),
                        ))
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
            aggregate_priority_fees(&decoded_txs, &gas_deltas, block.base_fee_per_gas)?;
        exec_state.commit(&bundle).map_err(Into::into)?;
        if let Some(ref boundary_effect) = boundary_effect {
            apply_epoch_boundary_effect(&mut exec_state, boundary_effect).map_err(|err| {
                map_epoch_boundary_runtime_error(err, BoundaryCallFailureMode::Verify)
            })?;
        }
        maybe_apply_community_pool_unlock(
            &mut exec_state,
            boundary_required,
            self.evm_config.simplex_validators(),
        )?;
        let current_epoch = load_u64_storage_value(
            &exec_state,
            EPOCH_PRECOMPILE_ADDRESS,
            current_epoch_slot(),
            "epoch currentEpoch",
        )?;
        credit_burned_fees(&mut exec_state, computed_gas_used, block.base_fee_per_gas)?;
        credit_fee_pool_claim(&mut exec_state, claim_recipient, priority_fees)?;

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

        let boundary_epoch_context = if boundary_required {
            Some(BoundaryEpochContext::from_post_advance_epoch(
                current_epoch,
            )?)
        } else {
            None
        };
        let activation_resolver = ActivationSourceResolver::new(&self.evm_config);

        if !boundary_required && decoded_extra_data.reshare.is_some() {
            return Err(EvmAppError::InvalidBlock(
                "reshare section is forbidden on non-boundary blocks".into(),
            ));
        }

        if self.evm_config.full_dkg_feature_enabled() {
            let candidate_epoch = boundary_epoch_context
                .map(|ctx| ctx.full_dkg_epoch)
                .unwrap_or(current_epoch);
            let candidate_full_dkg = self.evm_config.current_full_dkg_payload(candidate_epoch);
            let latest_committed_full_dkg = {
                let db = self.state_db.read().unwrap();
                latest_committed_full_dkg(&*db, parent.height)?
            };

            match candidate_full_dkg {
                Some(candidate_full_dkg) => {
                    ensure_full_dkg_players_match_activation(
                        &activation_resolver,
                        &candidate_full_dkg,
                    )?;

                    if boundary_required {
                        let boundary_epoch_context =
                            boundary_epoch_context.expect("context exists for boundary");
                        let observed_full_dkg =
                            decoded_extra_data.full_dkg.as_ref().ok_or_else(|| {
                                EvmAppError::InvalidBlock(
                                    "full_dkg section must be present for boundary block".into(),
                                )
                            })?;
                        if observed_full_dkg.epoch != boundary_epoch_context.full_dkg_epoch {
                            return Err(EvmAppError::InvalidBlock(format!(
                                "full_dkg epoch mismatch on boundary: expected {}, found {}",
                                boundary_epoch_context.full_dkg_epoch, observed_full_dkg.epoch
                            )));
                        }
                        if observed_full_dkg != &candidate_full_dkg {
                            return Err(EvmAppError::InvalidBlock(
                                "full_dkg payload mismatch with configured candidate".into(),
                            ));
                        }

                        let observed_reshare =
                            decoded_extra_data.reshare.as_ref().ok_or_else(|| {
                                EvmAppError::InvalidBlock(
                                    "reshare section must be present for boundary block".into(),
                                )
                            })?;
                        if observed_reshare.target_epoch
                            != boundary_epoch_context.reshare_target_epoch
                        {
                            return Err(EvmAppError::InvalidBlock(format!(
                                "reshare target epoch mismatch on boundary: expected {}, found {}",
                                boundary_epoch_context.reshare_target_epoch,
                                observed_reshare.target_epoch
                            )));
                        }
                        let expected_reshare_players = activation_resolver
                            .resolve_players_for_epoch(
                                boundary_epoch_context.reshare_target_epoch,
                            )?;
                        if observed_reshare.players != expected_reshare_players {
                            return Err(EvmAppError::InvalidBlock(
                                "reshare players do not match activation-resolved player set"
                                    .into(),
                            ));
                        }
                    } else {
                        let should_include = full_dkg_should_be_included(
                            &self.evm_config,
                            latest_committed_full_dkg.as_ref(),
                            &candidate_full_dkg,
                        );
                        match (should_include, decoded_extra_data.full_dkg.as_ref()) {
                            (true, Some(observed)) => {
                                if observed != &candidate_full_dkg {
                                    return Err(EvmAppError::InvalidBlock(
                                        "full_dkg payload mismatch with configured candidate"
                                            .into(),
                                    ));
                                }
                            }
                            (true, None) => {
                                return Err(EvmAppError::InvalidBlock(
                                    "full_dkg section must be present for this block".into(),
                                ))
                            }
                            (false, Some(_)) => {
                                return Err(EvmAppError::InvalidBlock(
                                    "full_dkg section must be omitted for this block".into(),
                                ))
                            }
                            (false, None) => {}
                        }
                    }
                }
                None => {
                    if decoded_extra_data.full_dkg.is_some() {
                        return Err(EvmAppError::InvalidBlock(
                            "full_dkg section must be omitted when no full_dkg candidate is configured"
                                .into(),
                        ));
                    }
                    if decoded_extra_data.reshare.is_some() {
                        return Err(EvmAppError::InvalidBlock(
                            "reshare section must be omitted when no full_dkg candidate is configured"
                                .into(),
                        ));
                    }
                }
            }
        } else if decoded_extra_data.reshare.is_some() {
            return Err(EvmAppError::InvalidBlock(
                "reshare section must be omitted when full_dkg feature is disabled".into(),
            ));
        }

        let receipts: Vec<Receipt> = execution_result
            .receipts
            .iter()
            .map(|r: &reth_ethereum_primitives::Receipt| Receipt {
                status: r.status().into(),
                cumulative_gas_used: r.cumulative_gas_used(),
                logs: r.logs().to_vec(),
            })
            .collect();

        self.stage_receipts_for_block(block, receipts);

        Ok(ExecutionResult {
            state_root: block.state_root,
            receipts_root: block.receipts_root,
            gas_used: block.gas_used,
            receipt_count: execution_result.receipts.len(),
        })
    }
}
