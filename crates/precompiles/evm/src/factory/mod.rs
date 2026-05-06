use reth_evm::revm::{
    context::{BlockEnv, TxEnv},
    inspector::{Inspector, NoOpInspector},
    primitives::hardfork::SpecId,
};
use reth_evm::{
    eth::{EthEvm, EthEvmBuilder, EthEvmContext},
    precompiles::PrecompilesMap,
    EvmEnv, EvmFactory,
};
use validators_reader::ValidatorEntry as RegistryValidatorEntry;

use crate::whirlpool_precompiles_with_validators;

/// Whirlpool EVM factory that injects the workspace precompile registry.
///
/// `Default::default()` and [`WhirlpoolEvmFactory::with_validators`] are both
/// retained for compatibility. Validator reads are runtime-state-backed, so the
/// factory no longer carries a validator snapshot.
#[derive(Debug, Default, Clone)]
pub struct WhirlpoolEvmFactory;

impl WhirlpoolEvmFactory {
    /// Compatibility constructor retained for callers that already pass genesis
    /// validator entries. The validators precompile reads runtime state.
    pub fn with_validators(_simplex_validators: Vec<RegistryValidatorEntry>) -> Self {
        Self
    }
}

impl EvmFactory for WhirlpoolEvmFactory {
    type Evm<DB: reth_evm::Database, I: Inspector<Self::Context<DB>>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Context<DB: reth_evm::Database> = EthEvmContext<DB>;
    type Tx = TxEnv;
    type Error<DBError: std::error::Error + Send + Sync + 'static> =
        reth_evm::revm::context_interface::result::EVMError<DBError>;
    type HaltReason = reth_evm::revm::context_interface::result::HaltReason;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: reth_evm::Database>(
        &self,
        db: DB,
        evm_env: EvmEnv<Self::Spec, Self::BlockEnv>,
    ) -> Self::Evm<DB, NoOpInspector> {
        let spec = evm_env.cfg_env.spec;
        EthEvmBuilder::new(db, evm_env)
            .precompiles(whirlpool_precompiles_with_validators(spec, Vec::new()))
            .build()
    }

    fn create_evm_with_inspector<DB: reth_evm::Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        evm_env: EvmEnv<Self::Spec, Self::BlockEnv>,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        let spec = evm_env.cfg_env.spec;
        EthEvmBuilder::new(db, evm_env)
            .activate_inspector(inspector)
            .precompiles(whirlpool_precompiles_with_validators(spec, Vec::new()))
            .build()
    }
}
