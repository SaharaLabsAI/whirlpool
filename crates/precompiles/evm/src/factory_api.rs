use crate::validators::ValidatorEntry as RegistryValidatorEntry;

use super::WhirlpoolEvmFactory;

impl WhirlpoolEvmFactory {
    /// Creates the canonical validator-aware Whirlpool EVM factory.
    ///
    /// Prefer this over [`Default::default`] for runtime wiring so the
    /// validators precompile exposes the ordered simplex-validator list.
    pub fn with_validators(simplex_validators: Vec<RegistryValidatorEntry>) -> Self {
        Self { simplex_validators }
    }
}
