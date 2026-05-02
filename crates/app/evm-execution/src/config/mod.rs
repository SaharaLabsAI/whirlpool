//! Static EVM configuration and adapter composition.
//!
//! This module keeps `WhirlpoolEvmConfig` and the static Reth/EVM adapter
//! surface. Node-local/runtime inputs live under `crate::context`; runtime
//! validator state is loaded by the block pipeline at proposal/verification
//! timing.

use std::sync::Arc;

use reth_chainspec::ChainSpec;
use validators_dkg::{FullDkgOutputV1, ValidatorActivationSchedule};

mod chain_spec_access;
mod evm_adapter;

use crate::context::dkg::DkgTransitionConfig;
use crate::context::proposer::ProposerRuntimeContext;
use evm_adapter::WhirlpoolRethEvmAdapter;

#[derive(Debug, Clone)]
pub struct WhirlpoolEvmConfig {
    evm_adapter: WhirlpoolRethEvmAdapter,
    proposer_context: ProposerRuntimeContext,
    dkg_transition: DkgTransitionConfig,
}

impl WhirlpoolEvmConfig {
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self {
            evm_adapter: WhirlpoolRethEvmAdapter::new(chain_spec),
            proposer_context: ProposerRuntimeContext::default(),
            dkg_transition: DkgTransitionConfig::default(),
        }
    }

    pub fn proposer_context(&self) -> &ProposerRuntimeContext {
        &self.proposer_context
    }

    pub fn with_local_proposer_public_key(mut self, local_proposer_public_key: [u8; 32]) -> Self {
        self.proposer_context = self
            .proposer_context
            .with_local_public_key(local_proposer_public_key);
        self
    }

    pub fn local_proposer_public_key(&self) -> [u8; 32] {
        self.proposer_context.local_public_key()
    }

    pub fn dkg_transition(&self) -> &DkgTransitionConfig {
        &self.dkg_transition
    }

    pub fn with_activation_players_for_epoch(mut self, epoch: u64, players: Vec<[u8; 32]>) -> Self {
        self.dkg_transition = self
            .dkg_transition
            .with_activation_players_for_epoch(epoch, players);
        self
    }

    pub fn validator_activation_schedule_for_default_players(
        &self,
        default_players: Vec<[u8; 32]>,
    ) -> ValidatorActivationSchedule {
        self.dkg_transition
            .activation_schedule_for_default_players(default_players)
    }

    pub fn with_full_dkg_feature_enabled(mut self, enabled: bool) -> Self {
        self.dkg_transition = self.dkg_transition.with_full_dkg_feature_enabled(enabled);
        self
    }

    pub fn full_dkg_feature_enabled(&self) -> bool {
        self.dkg_transition.feature_gate().enabled()
    }

    pub fn with_current_full_dkg_output(mut self, output: FullDkgOutputV1) -> Self {
        self.dkg_transition = self.dkg_transition.with_current_full_dkg_output(output);
        self
    }

    pub fn current_full_dkg_output(&self) -> Option<&FullDkgOutputV1> {
        self.dkg_transition.current_candidate().output()
    }
}

#[cfg(test)]
mod tests;
