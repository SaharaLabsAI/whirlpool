pub mod community_pool;
pub mod epoch;
pub mod fee_pool;
pub mod validators;

mod factory;
mod registry;

pub use crate::validators::{
    decode_validators_output, load_active_validator_registry,
    resolve_active_validator_fee_recipient, validate_active_validator_fee_recipient,
    validators_calldata, ValidatorsRuntimeError, VALIDATORS_PRECOMPILE_ADDRESS,
};
pub use community_pool::{
    apply_post_block_accounting, build_post_block_accounting_effect,
    community_pool_balance_calldata, community_pool_last_processed_epoch_slot,
    community_pool_last_processed_epoch_storage_slot, community_pool_locked_remaining_slot,
    community_pool_locked_remaining_storage_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_amount_per_cycle_storage_slot, community_pool_unlock_every_epochs_slot,
    community_pool_unlock_every_epochs_storage_slot, decode_community_pool_balance_output,
    encode_u256_storage_value, CommunityPoolUnlockEffect, CommunityPoolUnlockState,
    PostBlockAccountingEffect, PostBlockAccountingEffectError, PostBlockAccountingInputs,
    PostBlockAccountingOutcome, PostBlockAccountingRuntimeError, COMMUNITY_POOL_ADDRESS,
};
pub use epoch::{
    advance_epoch_calldata, apply_epoch_boundary_effect, boundary_required_for_height,
    current_epoch_calldata, current_epoch_slot, current_epoch_storage_slot,
    decode_current_epoch_output, decode_epoch_blocks_output, decode_epoch_start_block_output,
    decode_epoch_start_block_storage_value, decode_next_epoch_block_output,
    decode_u64_storage_value, encode_epoch_start_block_storage_value, encode_u64_storage_value,
    epoch_blocks_calldata, epoch_blocks_slot, epoch_blocks_storage_slot,
    epoch_start_block_calldata, epoch_start_block_storage_slot, epoch_system_tx_sender,
    execute_epoch_boundary_system_call_if_required, extract_epoch_boundary_effect,
    is_advance_epoch_calldata, load_epoch_boundary_state, next_epoch_block_calldata,
    next_epoch_block_slot, next_epoch_block_storage_slot, reserved_advance_epoch_call_matches,
    EpochBoundaryEffect, EpochBoundaryEffectError, EpochBoundaryRuntimeError, EpochBoundaryState,
    EpochBoundaryStorageWrite, EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS,
    EPOCH_SYSTEM_TX_GAS_LIMIT, EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI, EPOCH_SYSTEM_TX_PRIVATE_KEY,
};
pub use factory::WhirlpoolEvmFactory;
pub use fee_pool::{
    claimable_balance_calldata, claimable_balance_slot, decode_claimable_balance_output,
    decode_fee_pool_balance_output, decode_withdraw_output, fee_pool_balance_calldata,
    withdraw_calldata, ClaimCredit, FEE_POOL_PRECOMPILE_ADDRESS,
};
pub use registry::{
    build_whirlpool_precompiles, build_whirlpool_precompiles_with_validators,
    whirlpool_precompiles, whirlpool_precompiles_with_validators, NonDirectCall,
    RegisteredPrecompile, RegistryError, WhirlpoolStatefulPrecompile,
};

#[cfg(test)]
use registry::{build_precompiles, non_direct_call_revert_bytes};

#[cfg(test)]
mod tests;
