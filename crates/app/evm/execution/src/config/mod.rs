//! Static EVM configuration and adapter composition.
//!
//! This module keeps `WhirlpoolEvmConfig` and the static Reth/EVM adapter
//! surface. Node-local/runtime inputs live under `crate::context`; runtime
//! validator state is loaded by the block pipeline at proposal/verification
//! timing.

use std::sync::Arc;

use reth_chainspec::ChainSpec;

mod chain_spec_access;
pub mod dkg;
mod evm_adapter;
pub mod proposer;
mod proposer_facade;

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

    pub fn dkg_transition(&self) -> &DkgTransitionConfig {
        &self.dkg_transition
    }
}

#[cfg(test)]
mod tests;
