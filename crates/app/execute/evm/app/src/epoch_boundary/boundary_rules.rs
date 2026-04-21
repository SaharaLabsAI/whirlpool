use alloy_consensus::Transaction;
use alloy_primitives::{Address, TxKind, U256};
use evm_precompiles::{
    epoch_system_tx_sender, is_advance_epoch_calldata, EPOCH_PRECOMPILE_ADDRESS,
};
use reth_ethereum_primitives::TransactionSigned;
use revm::state::EvmState;

use crate::{error::EvmAppError, traits::StateProvider};

use super::boundary_state::EpochBoundaryState;

pub fn boundary_required_for_height(state: EpochBoundaryState, block_height: u64) -> bool {
    block_height == state.next_epoch_block
}

pub fn tx_is_reserved_epoch_namespace(tx: &TransactionSigned, signer: Address) -> bool {
    signer == epoch_system_tx_sender()
        && tx.kind() == TxKind::Call(EPOCH_PRECOMPILE_ADDRESS)
        && tx.value() == U256::ZERO
        && is_advance_epoch_calldata(tx.input())
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
