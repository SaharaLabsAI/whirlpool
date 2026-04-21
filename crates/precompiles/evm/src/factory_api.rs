use ::validators::ValidatorEntry as RegistryValidatorEntry;

use super::WhirlpoolEvmFactory;

impl WhirlpoolEvmFactory {
    pub fn with_validators(simplex_validators: Vec<RegistryValidatorEntry>) -> Self {
        Self { simplex_validators }
    }
}
