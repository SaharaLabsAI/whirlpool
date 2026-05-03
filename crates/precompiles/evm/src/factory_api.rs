use validators_reader::ValidatorEntry as RegistryValidatorEntry;

use crate::WhirlpoolEvmFactory;

impl WhirlpoolEvmFactory {
    /// Compatibility constructor retained for callers that already pass genesis
    /// validator entries. The validators precompile reads runtime state.
    pub fn with_validators(_simplex_validators: Vec<RegistryValidatorEntry>) -> Self {
        Self
    }
}
