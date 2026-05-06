use alloy_primitives::U256;

use crate::epoch::transition::advance::{
    finalize_advance_epoch, plan_advance_epoch, AdvanceEpochInput, AdvanceEpochState,
};
use crate::epoch::{storage, EpochPrecompileError};

#[test]
fn plans_epoch_advance_storage_writes() {
    let plan = plan_advance_epoch(
        AdvanceEpochInput { block_number: 5 },
        AdvanceEpochState {
            current_epoch: 0,
            next_epoch_block: 5,
            epoch_blocks: 10,
        },
    )
    .expect("advance should plan");

    assert_eq!(plan.next_epoch, 1);
    assert_eq!(plan.epoch_start_slot, storage::epoch_start_block_slot(1));

    let outcome = finalize_advance_epoch(plan, U256::ZERO).expect("finalize should pass");
    assert_eq!(outcome.effect.writes[0].value, U256::from(1_u64));
    assert_eq!(outcome.effect.writes[1].value, U256::from(15_u64));
    assert_eq!(outcome.effect.writes[2].value, U256::from(6_u64));
}

#[test]
fn rejects_non_boundary_block() {
    let err = plan_advance_epoch(
        AdvanceEpochInput { block_number: 4 },
        AdvanceEpochState {
            current_epoch: 0,
            next_epoch_block: 5,
            epoch_blocks: 10,
        },
    )
    .expect_err("non-boundary block should fail");

    assert_eq!(
        err,
        EpochPrecompileError::InvalidBoundaryBlock {
            expected: 5,
            got: 4,
        }
    );
}

#[test]
fn rejects_existing_next_epoch_start_slot() {
    let plan = plan_advance_epoch(
        AdvanceEpochInput { block_number: 5 },
        AdvanceEpochState {
            current_epoch: 0,
            next_epoch_block: 5,
            epoch_blocks: 10,
        },
    )
    .expect("advance should plan");

    let err =
        finalize_advance_epoch(plan, U256::from(1_u64)).expect_err("existing start should fail");

    assert_eq!(err, EpochPrecompileError::EpochStartAlreadyInitialized(1));
}
