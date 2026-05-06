use crate::{community_pool, epoch, fee_pool, validators, RegisteredPrecompile};

pub fn installed_precompiles() -> [RegisteredPrecompile; 4] {
    [
        community_pool::register(),
        epoch::register(),
        fee_pool::register(),
        validators::register(Vec::new()),
    ]
}
