use std::fmt::Display;

use reth_evm::revm::DatabaseCommit;
use reth_evm::Evm;
use state::StateDb;

use super::{
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

    evm.db_mut().commit(outcome.state.clone());

    let effect = extract_epoch_boundary_effect(&outcome.state)
        .map_err(|err| EpochBoundaryRuntimeError::EffectExtraction(err.to_string()))?;

    Ok(Some(effect))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use reth_evm::revm::database::{CacheDB, EmptyDB};
    use reth_evm::{EvmEnv, EvmFactory};
    use state_reth::InMemoryStateDb;

    use super::super::{current_epoch_slot, epoch_start_block_slot, EpochBoundaryStorageWrite};
    use super::*;
    use crate::WhirlpoolEvmFactory;

    fn storage_value(db: &InMemoryStateDb, slot: U256) -> U256 {
        db.get_storage(EPOCH_PRECOMPILE_ADDRESS, slot)
    }

    #[test]
    fn load_epoch_boundary_state_from_statedb() {
        let mut db = InMemoryStateDb::new();
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(42_u64),
        );

        let state = load_epoch_boundary_state(&db).expect("load boundary state");

        assert_eq!(state.next_epoch_block, 42);
    }

    #[test]
    fn apply_epoch_boundary_effect_via_statedb() {
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
                    slot: epoch_start_block_slot(1),
                    value: U256::from(11_u64),
                },
            ],
        };

        apply_epoch_boundary_effect(&mut db, &effect).expect("apply effect");

        assert_eq!(storage_value(&db, current_epoch_slot()), U256::from(1_u64));
        assert_eq!(storage_value(&db, next_epoch_block_slot()), U256::from(20_u64));
        assert_eq!(storage_value(&db, epoch_start_block_slot(1)), U256::from(11_u64));
    }

    #[test]
    fn execute_epoch_boundary_system_call_if_required_commits_immediately() {
        let mut db = CacheDB::<EmptyDB>::default();
        db.insert_account_info(EPOCH_PRECOMPILE_ADDRESS, Default::default());
        db.insert_account_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot(), U256::ZERO)
            .expect("seed current epoch");
        db.insert_account_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            super::super::epoch_blocks_slot(),
            U256::from(10_u64),
        )
        .expect("seed epoch blocks");
        db.insert_account_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(5_u64),
        )
        .expect("seed next epoch block");
        db.insert_account_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            epoch_start_block_slot(0),
            U256::from(1_u64),
        )
        .expect("seed epoch zero start");

        let evm_env = EvmEnv::default().with_block_number(U256::from(5_u64));
        let mut evm = WhirlpoolEvmFactory::default().create_evm(db, evm_env);

        let effect = execute_epoch_boundary_system_call_if_required(&mut evm, true)
            .expect("first boundary system call should succeed")
            .expect("boundary effect should exist");
        assert_eq!(effect.writes[0].slot, current_epoch_slot());
        assert_eq!(effect.writes[0].value, U256::from(1_u64));
        assert_eq!(effect.writes[1].slot, next_epoch_block_slot());
        assert_eq!(effect.writes[1].value, U256::from(15_u64));

        let second = execute_epoch_boundary_system_call_if_required(&mut evm, true)
            .expect_err("second call should observe committed in-memory state");
        assert_eq!(second, EpochBoundaryRuntimeError::SystemCallUnsuccessful);
    }
}
