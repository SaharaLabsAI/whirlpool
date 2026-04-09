use std::fmt::Display;

use alloy_consensus::Transaction;
use alloy_primitives::{Address, TxKind, U256};
use evm_precompiles::{
    advance_epoch_calldata, epoch_system_tx_sender, is_advance_epoch_calldata,
    next_epoch_block_slot, EPOCH_PRECOMPILE_ADDRESS,
};
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::Evm;
use revm::state::EvmState;
use revm::DatabaseCommit;

use crate::{error::EvmAppError, traits::StateProvider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochBoundaryState {
    pub next_epoch_block: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryCallFailureMode {
    Propose,
    Verify,
}

pub fn load_epoch_boundary_state<DB>(db: &DB) -> Result<EpochBoundaryState, EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    let next_epoch_raw = db
        .get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot())
        .map_err(Into::into)?;
    let next_epoch_block = u64::try_from(next_epoch_raw).map_err(|_| {
        EvmAppError::InvalidBlock("epoch nextEpochBlock storage does not fit into u64".into())
    })?;

    Ok(EpochBoundaryState { next_epoch_block })
}

pub fn boundary_required_for_height(state: EpochBoundaryState, block_height: u64) -> bool {
    block_height == state.next_epoch_block
}

pub fn tx_is_reserved_epoch_namespace(tx: &TransactionSigned, signer: Address) -> bool {
    signer == epoch_system_tx_sender()
        && tx.kind() == TxKind::Call(EPOCH_PRECOMPILE_ADDRESS)
        && tx.value() == U256::ZERO
        && is_advance_epoch_calldata(tx.input())
}

pub fn execute_epoch_boundary_system_call_if_required<EVM>(
    evm: &mut EVM,
    boundary_required: bool,
    failure_mode: BoundaryCallFailureMode,
) -> Result<Option<EvmState>, EvmAppError>
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

    Ok(Some(outcome.state))
}

pub fn apply_boundary_state_to_provider<DB>(
    db: &mut DB,
    boundary_state: &EvmState,
) -> Result<(), EvmAppError>
where
    DB: StateProvider,
    <DB as StateProvider>::Error: Into<EvmAppError>,
{
    for (address, account) in boundary_state {
        db.insert_account(*address, account.info.clone())
            .map_err(Into::into)?;

        for (slot, slot_value) in account.changed_storage_slots() {
            db.insert_storage(*address, *slot, slot_value.present_value())
                .map_err(Into::into)?;
        }
    }

    Ok(())
}

fn boundary_call_failure(mode: BoundaryCallFailureMode, message: String) -> EvmAppError {
    match mode {
        BoundaryCallFailureMode::Propose => EvmAppError::Execution(message),
        BoundaryCallFailureMode::Verify => EvmAppError::InvalidBlock(message),
    }
}
