mod community_pool_unlock;
mod native_token;
mod simplex_validator_reader;
mod spec_builders_alloc;
mod spec_builders_base;
mod spec_builders_core;
mod spec_builders_try;

pub use community_pool_unlock::CommunityPoolUnlockConfig;
pub use native_token::{
    sahara_hard_cap_base_units, total_allocated_supply, validate_genesis_alloc, NativeTokenError,
    SAHARA_DECIMALS, SAHARA_HARD_CAP_BASE_UNITS_U128, SAHARA_HARD_CAP_TOKENS,
};
pub use simplex_validator_reader::try_simplex_validators_from_chain_spec;
pub use spec_builders_alloc::{
    build_sahara_chain_spec_with_alloc_and_validators, try_build_sahara_chain_spec_with_alloc,
};
pub use spec_builders_base::{
    build_sahara_chain_spec, build_sahara_chain_spec_with_alloc, try_build_sahara_chain_spec,
};
pub use spec_builders_core::try_build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config;
pub use spec_builders_try::{
    build_sahara_chain_spec_with_alloc_and_validators_and_community_pool_unlock_config,
    try_build_sahara_chain_spec_with_alloc_and_validators,
};

pub const SAHARA_CHAIN_ID: u64 = 313_371;

#[cfg(test)]
mod native_token_tests;

#[cfg(test)]
mod tests;
