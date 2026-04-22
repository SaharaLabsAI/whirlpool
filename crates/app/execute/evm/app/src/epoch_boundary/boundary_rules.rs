use alloy_consensus::Transaction;
use alloy_primitives::{Address, TxKind};
use evm_precompiles::{
    boundary_required_for_height as epoch_boundary_required_for_height,
    reserved_advance_epoch_call_matches, EpochBoundaryEffect, EpochBoundaryStorageWrite,
    EPOCH_PRECOMPILE_ADDRESS,
};
use reth_ethereum_primitives::TransactionSigned;

use crate::{error::EvmAppError, traits::StateDb};

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

pub fn apply_epoch_boundary_effect<DB>(
    db: &mut DB,
    effect: &EpochBoundaryEffect,
) -> Result<(), EvmAppError>
where
    DB: StateDb,
    <DB as StateDb>::Error: Into<EvmAppError>,
{
    for EpochBoundaryStorageWrite { slot, value } in effect.writes {
        db.insert_storage(EPOCH_PRECOMPILE_ADDRESS, slot, value)
            .map_err(Into::into)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use evm_precompiles::{current_epoch_slot, next_epoch_block_slot};
    use state_memory::InMemoryStateDb;

    use super::*;

    #[test]
    fn applying_epoch_boundary_effect_updates_only_epoch_slots() {
        let mut db = InMemoryStateDb::new();
        db.insert_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot(), U256::ZERO);
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(10_u64),
        );

        let effect = EpochBoundaryEffect {
            writes: [
                EpochBoundaryStorageWrite {
                    slot: current_epoch_slot(),
                    value: U256::from(1_u64),
                },
                EpochBoundaryStorageWrite {
                    slot: next_epoch_block_slot(),
                    value: U256::from(20_u64),
                },
                EpochBoundaryStorageWrite {
                    slot: evm_precompiles::epoch::epoch_start_block_slot(1),
                    value: U256::from(11_u64),
                },
            ],
        };

        apply_epoch_boundary_effect(&mut db, &effect).expect("apply effect");

        assert_eq!(
            db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
            U256::from(1_u64)
        );
        assert_eq!(
            db.get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot()),
            U256::from(20_u64)
        );
        assert_eq!(
            db.get_storage(
                EPOCH_PRECOMPILE_ADDRESS,
                evm_precompiles::epoch::epoch_start_block_slot(1),
            ),
            U256::from(11_u64)
        );
    }
}
