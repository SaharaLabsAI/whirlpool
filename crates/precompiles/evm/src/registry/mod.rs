use alloy_primitives::Address;

mod build;
mod direct_call_guard;
mod entry;
mod installed;
mod runtime;

#[cfg(test)]
pub use build::build_precompiles;
pub use build::{build_whirlpool_precompiles, build_whirlpool_precompiles_with_validators};
#[cfg(test)]
pub use direct_call_guard::non_direct_call_revert_bytes;
pub use direct_call_guard::non_direct_call_revert_result;
pub use direct_call_guard::NonDirectCall;
pub use entry::{RegisteredPrecompile, WhirlpoolStatefulPrecompile};
pub use runtime::{whirlpool_precompiles, whirlpool_precompiles_with_validators};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("custom precompile address {0} collides with an existing built-in precompile")]
    BuiltinAddressCollision(Address),
    #[error("custom precompile address {0} is registered more than once")]
    DuplicateCustomAddress(Address),
}
