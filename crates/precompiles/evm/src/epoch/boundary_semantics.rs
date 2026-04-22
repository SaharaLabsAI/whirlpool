use alloy_primitives::{Address, U256};

use super::{epoch_system_tx_sender, is_advance_epoch_calldata, EPOCH_PRECOMPILE_ADDRESS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochBoundaryState {
    pub next_epoch_block: u64,
}

pub fn boundary_required_for_height(state: EpochBoundaryState, block_height: u64) -> bool {
    block_height == state.next_epoch_block
}

pub fn reserved_advance_epoch_call_matches(
    caller: Address,
    target_address: Address,
    value: U256,
    calldata: &[u8],
) -> bool {
    caller == epoch_system_tx_sender()
        && target_address == EPOCH_PRECOMPILE_ADDRESS
        && value == U256::ZERO
        && is_advance_epoch_calldata(calldata)
}
