use alloy_consensus::Transaction;
use alloy_primitives::{Address, TxKind};
use evm_precompiles::{
    boundary_required_for_height as epoch_boundary_required_for_height,
    reserved_advance_epoch_call_matches,
};
use reth_ethereum_primitives::TransactionSigned;
use revm::state::EvmState;

use crate::{error::EvmAppError, traits::StateProvider};

use super::boundary_state::EpochBoundaryState;

pub fn boundary_required_for_height(state: EpochBoundaryState, block_height: u64) -> bool {
    epoch_boundary_required_for_height(state, block_height)
}

pub fn tx_is_reserved_epoch_namespace(tx: &TransactionSigned, signer: Address) -> bool {
    match tx.kind() {
        TxKind::Call(target_address) => {
            reserved_advance_epoch_call_matches(signer, target_address, tx.value(), tx.input())
        }
        _ => false,
    }
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
