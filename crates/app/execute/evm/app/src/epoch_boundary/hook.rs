use std::fmt::Display;

use alloy_primitives::Address;
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::Evm;
use revm::state::EvmState;
use revm::DatabaseCommit;

use crate::{error::EvmAppError, traits::StateProvider};

use super::{
    apply_boundary_state_to_provider, boundary_required_for_height,
    execute_epoch_boundary_system_call_if_required, load_epoch_boundary_state,
    tx_is_reserved_epoch_namespace, BoundaryCallFailureMode, EpochBoundaryState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EpochBoundaryHook {
    #[default]
    PrecompileSemanticsV1,
}

impl EpochBoundaryHook {
    pub fn load_boundary_state<DB>(&self, db: &DB) -> Result<EpochBoundaryState, EvmAppError>
    where
        DB: StateProvider,
        <DB as StateProvider>::Error: Into<EvmAppError>,
    {
        match self {
            Self::PrecompileSemanticsV1 => load_epoch_boundary_state(db),
        }
    }

    pub fn boundary_required_for_height(
        &self,
        state: EpochBoundaryState,
        block_height: u64,
    ) -> bool {
        match self {
            Self::PrecompileSemanticsV1 => boundary_required_for_height(state, block_height),
        }
    }

    pub fn execute_system_call_if_required<EVM>(
        &self,
        evm: &mut EVM,
        boundary_required: bool,
        failure_mode: BoundaryCallFailureMode,
    ) -> Result<Option<EvmState>, EvmAppError>
    where
        EVM: Evm,
        EVM::DB: DatabaseCommit,
        EVM::Error: Display,
    {
        match self {
            Self::PrecompileSemanticsV1 => {
                execute_epoch_boundary_system_call_if_required(evm, boundary_required, failure_mode)
            }
        }
    }

    pub fn tx_is_reserved_namespace(&self, tx: &TransactionSigned, signer: Address) -> bool {
        match self {
            Self::PrecompileSemanticsV1 => tx_is_reserved_epoch_namespace(tx, signer),
        }
    }

    pub fn apply_boundary_state_to_provider<DB>(
        &self,
        db: &mut DB,
        boundary_state: &EvmState,
    ) -> Result<(), EvmAppError>
    where
        DB: StateProvider,
        <DB as StateProvider>::Error: Into<EvmAppError>,
    {
        match self {
            Self::PrecompileSemanticsV1 => apply_boundary_state_to_provider(db, boundary_state),
        }
    }
}
