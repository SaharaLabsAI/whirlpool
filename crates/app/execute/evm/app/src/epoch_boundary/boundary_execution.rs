use std::fmt::Display;

use evm_precompiles::{
    advance_epoch_calldata, epoch_system_tx_sender, extract_epoch_boundary_effect,
    EpochBoundaryEffect, EPOCH_PRECOMPILE_ADDRESS,
};
use reth_evm::Evm;
use revm::DatabaseCommit;

use crate::error::EvmAppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryCallFailureMode {
    Propose,
    Verify,
}

pub fn execute_epoch_boundary_system_call_if_required<EVM>(
    evm: &mut EVM,
    boundary_required: bool,
    failure_mode: BoundaryCallFailureMode,
) -> Result<Option<EpochBoundaryEffect>, EvmAppError>
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
        .map_err(|err| {
            boundary_call_failure(
                failure_mode,
                format!("required epoch boundary system call execution failed: {err}"),
            )
        })?;

    if !outcome.result.is_success() {
        return Err(boundary_call_failure(
            failure_mode,
            "required epoch boundary system call did not succeed".into(),
        ));
    }

    evm.db_mut().commit(outcome.state.clone());

    let effect = extract_epoch_boundary_effect(&outcome.state).map_err(|err| {
        boundary_call_failure(
            failure_mode,
            format!("required epoch boundary effect extraction failed: {err}"),
        )
    })?;

    Ok(Some(effect))
}

fn boundary_call_failure(mode: BoundaryCallFailureMode, message: String) -> EvmAppError {
    match mode {
        BoundaryCallFailureMode::Propose => EvmAppError::Execution(message),
        BoundaryCallFailureMode::Verify => EvmAppError::InvalidBlock(message),
    }
}
