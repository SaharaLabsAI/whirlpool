use std::fmt::Display;

use reth_evm::revm::{state::EvmState, DatabaseCommit};
use reth_evm::Evm;
use state::StateDb;

use crate::epoch::{
    advance_epoch_calldata, decode_u64_storage_value, epoch_system_tx_sender,
    extract_epoch_boundary_effect, next_epoch_block_slot, EpochBoundaryEffect, EpochBoundaryState,
    EPOCH_PRECOMPILE_ADDRESS,
};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EpochBoundaryRuntimeError {
    #[error("state access error: {0}")]
    StateAccess(String),
    #[error("{0}")]
    InvalidStoredValue(&'static str),
    #[error("required epoch boundary system call execution failed: {0}")]
    SystemCallExecution(String),
    #[error("required epoch boundary system call did not succeed")]
    SystemCallUnsuccessful,
    #[error("required epoch boundary effect extraction failed: {0}")]
    EffectExtraction(String),
}

pub fn load_epoch_boundary_state<DB>(
    db: &DB,
) -> Result<EpochBoundaryState, EpochBoundaryRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    let next_epoch_raw = db
        .get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
        .map_err(|err| EpochBoundaryRuntimeError::StateAccess(err.to_string()))?;
    let next_epoch_block = decode_u64_storage_value(next_epoch_raw).ok_or(
        EpochBoundaryRuntimeError::InvalidStoredValue(
            "epoch nextEpochBlock storage does not fit into u64",
        ),
    )?;

    Ok(EpochBoundaryState { next_epoch_block })
}

pub fn apply_epoch_boundary_effect<DB>(
    db: &mut DB,
    effect: &EpochBoundaryEffect,
) -> Result<(), EpochBoundaryRuntimeError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Display,
{
    for write in effect.writes {
        db.insert_storage(EPOCH_PRECOMPILE_ADDRESS, write.slot, write.value)
            .map_err(|err| EpochBoundaryRuntimeError::StateAccess(err.to_string()))?;
    }

    Ok(())
}

pub fn execute_epoch_boundary_system_call_if_required<EVM>(
    evm: &mut EVM,
    boundary_required: bool,
) -> Result<Option<EpochBoundaryEffect>, EpochBoundaryRuntimeError>
where
    EVM: Evm,
    EVM::DB: DatabaseCommit,
    EVM::Error: Display,
{
    if !boundary_required {
        return Ok(None);
    }

    let outcome = evm
        .transact_system_call(
            epoch_system_tx_sender(),
            EPOCH_PRECOMPILE_ADDRESS,
            advance_epoch_calldata(),
        )
        .map_err(|err| EpochBoundaryRuntimeError::SystemCallExecution(err.to_string()))?;

    if !outcome.result.is_success() {
        return Err(EpochBoundaryRuntimeError::SystemCallUnsuccessful);
    }

    commit_validated_epoch_boundary_effect(evm.db_mut(), outcome.state).map(Some)
}

fn commit_validated_epoch_boundary_effect<DB>(
    db: &mut DB,
    outcome_state: EvmState,
) -> Result<EpochBoundaryEffect, EpochBoundaryRuntimeError>
where
    DB: DatabaseCommit,
{
    let effect = extract_epoch_boundary_effect(&outcome_state)
        .map_err(|err| EpochBoundaryRuntimeError::EffectExtraction(err.to_string()))?;

    db.commit(outcome_state);

    Ok(effect)
}

#[cfg(test)]
#[path = "boundary_adapter_tests.rs"]
mod tests;
