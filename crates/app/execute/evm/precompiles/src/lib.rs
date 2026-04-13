use ::validators::ValidatorEntry as RegistryValidatorEntry;
use alloy_primitives::{Address, Bytes};
use alloy_sol_types::{sol, SolError};
use reth_evm::{
    eth::{EthEvm, EthEvmBuilder, EthEvmContext},
    precompiles::{DynPrecompile, PrecompileInput, PrecompilesMap},
    EvmEnv, EvmFactory,
};
use revm::{
    context::{BlockEnv, TxEnv},
    inspector::{Inspector, NoOpInspector},
    precompile::{PrecompileId, PrecompileOutput, PrecompileResult, PrecompileSpecId, Precompiles},
    primitives::hardfork::SpecId,
};
use std::collections::HashSet;

pub mod community_pool;
pub mod epoch;
pub mod fee_pool;
pub mod validators;

pub use community_pool::{
    community_pool_balance_calldata, community_pool_last_processed_epoch_slot,
    community_pool_last_processed_epoch_storage_slot, community_pool_locked_remaining_slot,
    community_pool_locked_remaining_storage_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_amount_per_cycle_storage_slot, community_pool_unlock_every_epochs_slot,
    community_pool_unlock_every_epochs_storage_slot, decode_community_pool_balance_output,
    encode_u256_storage_value, COMMUNITY_POOL_ADDRESS,
};
pub use epoch::{
    advance_epoch_calldata, current_epoch_calldata, current_epoch_slot, current_epoch_storage_slot,
    decode_current_epoch_output, decode_epoch_blocks_output, decode_epoch_start_block_output,
    decode_next_epoch_block_output, encode_epoch_start_block_storage_value,
    encode_u64_storage_value, epoch_blocks_calldata, epoch_blocks_slot, epoch_blocks_storage_slot,
    epoch_start_block_calldata, epoch_start_block_storage_slot, epoch_system_tx_sender,
    is_advance_epoch_calldata, next_epoch_block_calldata, next_epoch_block_slot,
    next_epoch_block_storage_slot, EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS,
    EPOCH_SYSTEM_TX_GAS_LIMIT, EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI, EPOCH_SYSTEM_TX_PRIVATE_KEY,
};
pub use fee_pool::{
    claimable_balance_calldata, claimable_balance_slot, decode_claimable_balance_output,
    decode_fee_pool_balance_output, decode_withdraw_output, fee_pool_balance_calldata,
    withdraw_calldata, FEE_POOL_PRECOMPILE_ADDRESS,
};
pub use validators::{
    decode_validators_output, validators_calldata, VALIDATORS_PRECOMPILE_ADDRESS,
};

sol! {
    /// Shared framework-level error used when a Whirlpool-owned stateful precompile
    /// is invoked through a non-direct path such as DELEGATECALL or CALLCODE.
    #[derive(Debug, PartialEq, Eq)]
    error NonDirectCall();
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("custom precompile address {0} collides with an existing built-in precompile")]
    BuiltinAddressCollision(Address),
    #[error("custom precompile address {0} is registered more than once")]
    DuplicateCustomAddress(Address),
}

#[derive(Clone)]
pub struct RegisteredPrecompile {
    address: Address,
    precompile: DynPrecompile,
}

impl RegisteredPrecompile {
    /// Registers a Whirlpool-owned stateful precompile using the safe default path.
    ///
    /// Precompiles registered here are direct-call-only: the final hop into the
    /// precompile must have `target_address == bytecode_address`, which allows
    /// ordinary `CALL` and `STATICCALL` while rejecting delegate-style execution.
    pub fn new_stateful<F>(name: &'static str, address: Address, handler: F) -> Self
    where
        F: Fn(PrecompileInput<'_>) -> PrecompileResult + Send + Sync + 'static,
    {
        Self {
            address,
            precompile: DynPrecompile::new_stateful(PrecompileId::custom(name), move |input| {
                if !input.is_direct_call() {
                    // This guard rejects delegate-style entry before the target precompile's
                    // business logic begins. Returning a reverted output with `gas_used = 0`
                    // keeps the precompile-local charge at zero because the handler never ran;
                    // surrounding EVM call overhead is still accounted for by the caller frame.
                    return non_direct_call_revert_result();
                }
                handler(input)
            }),
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn precompile(&self) -> DynPrecompile {
        self.precompile.clone()
    }
}

pub trait WhirlpoolStatefulPrecompile {
    fn register() -> RegisteredPrecompile;
}

fn non_direct_call_revert_bytes() -> Bytes {
    Bytes::from(NonDirectCall {}.abi_encode())
}

fn non_direct_call_revert_result() -> PrecompileResult {
    // `REVERT` does not imply zero gas in general, but this framework-level rejection happens
    // before the precompile executes any opcode-equivalent work or applies its own gas policy.
    // We therefore report zero precompile gas here and let the enclosing EVM machinery account
    // for any call/setup cost outside the precompile itself.
    Ok(PrecompileOutput::new_reverted(
        0,
        non_direct_call_revert_bytes(),
    ))
}

fn build_precompiles<I>(
    spec: SpecId,
    custom_precompiles: I,
) -> Result<PrecompilesMap, RegistryError>
where
    I: IntoIterator<Item = RegisteredPrecompile>,
{
    let mut precompiles =
        PrecompilesMap::from_static(Precompiles::new(PrecompileSpecId::from_spec_id(spec)));
    let mut seen = HashSet::new();

    for registered in custom_precompiles {
        let address = registered.address();
        if !seen.insert(address) {
            return Err(RegistryError::DuplicateCustomAddress(address));
        }
        if precompiles.get(&address).is_some() {
            return Err(RegistryError::BuiltinAddressCollision(address));
        }
        precompiles.apply_precompile(&address, |_| Some(registered.precompile()));
    }

    Ok(precompiles)
}

pub fn build_whirlpool_precompiles(spec: SpecId) -> Result<PrecompilesMap, RegistryError> {
    build_whirlpool_precompiles_with_validators(spec, Vec::new())
}

pub fn build_whirlpool_precompiles_with_validators(
    spec: SpecId,
    simplex_validators: Vec<RegistryValidatorEntry>,
) -> Result<PrecompilesMap, RegistryError> {
    build_precompiles(
        spec,
        [
            community_pool::register(),
            epoch::register(),
            fee_pool::register(),
            validators::register(simplex_validators),
        ],
    )
}

pub fn whirlpool_precompiles(spec: SpecId) -> PrecompilesMap {
    whirlpool_precompiles_with_validators(spec, Vec::new())
}

pub fn whirlpool_precompiles_with_validators(
    spec: SpecId,
    simplex_validators: Vec<RegistryValidatorEntry>,
) -> PrecompilesMap {
    build_whirlpool_precompiles_with_validators(spec, simplex_validators)
        .expect("Whirlpool custom precompile registry must be valid")
}

#[derive(Debug, Default, Clone)]
pub struct WhirlpoolEvmFactory {
    simplex_validators: Vec<RegistryValidatorEntry>,
}

impl WhirlpoolEvmFactory {
    pub fn with_validators(simplex_validators: Vec<RegistryValidatorEntry>) -> Self {
        Self { simplex_validators }
    }
}

impl EvmFactory for WhirlpoolEvmFactory {
    type Evm<DB: reth_evm::Database, I: Inspector<Self::Context<DB>>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Context<DB: reth_evm::Database> = EthEvmContext<DB>;
    type Tx = TxEnv;
    type Error<DBError: std::error::Error + Send + Sync + 'static> =
        revm::context_interface::result::EVMError<DBError>;
    type HaltReason = revm::context_interface::result::HaltReason;
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
            .precompiles(whirlpool_precompiles_with_validators(
                spec,
                self.simplex_validators.clone(),
            ))
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
            .precompiles(whirlpool_precompiles_with_validators(
                spec,
                self.simplex_validators.clone(),
            ))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fee_pool::{
        fee_pool_balance_calldata, withdraw_calldata, FEE_POOL_PRECOMPILE_ADDRESS,
    };
    use alloy_primitives::{address, Bytes, U256};
    use reth_evm::{precompiles::Precompile, traits::EvmInternals};
    use revm::Context;
    use revm::{database::EmptyDB, precompile::PrecompileOutput as RevmPrecompileOutput};

    fn call_registered_precompile_with_context(
        precompile: DynPrecompile,
        context: &mut Context<BlockEnv, TxEnv, revm::context::CfgEnv, EmptyDB>,
        caller: Address,
        data: Bytes,
        gas: u64,
        is_static: bool,
        target_address: Address,
        bytecode_address: Address,
    ) -> RevmPrecompileOutput {
        precompile
            .call(PrecompileInput {
                data: data.as_ref(),
                gas,
                caller,
                value: U256::ZERO,
                target_address,
                bytecode_address,
                is_static,
                internals: EvmInternals::from_context(context),
            })
            .expect("precompile call should succeed")
    }

    fn decode_word(bytes: &Bytes) -> U256 {
        let mut word = [0u8; 32];
        word.copy_from_slice(bytes.as_ref());
        U256::from_be_bytes(word)
    }

    #[test]
    fn proxy_style_caller_is_still_treated_as_direct_at_precompile_boundary() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = fee_pool::register().precompile();
        let proxy_caller = address!("0x0000000000000000000000000000000000000abc");

        let balance_result = call_registered_precompile_with_context(
            precompile,
            &mut ctx,
            proxy_caller,
            fee_pool_balance_calldata(),
            fee_pool::gas::FEE_POOL_BALANCE_GAS,
            true,
            FEE_POOL_PRECOMPILE_ADDRESS,
            FEE_POOL_PRECOMPILE_ADDRESS,
        );
        assert!(
            !balance_result.reverted,
            "proxy-style caller should still be direct"
        );
        assert_eq!(decode_word(&balance_result.bytes), U256::ZERO);
    }

    #[test]
    fn registry_builds_expected_addresses() {
        let registry = build_whirlpool_precompiles(SpecId::CANCUN).expect("registry");
        assert!(registry.get(&COMMUNITY_POOL_ADDRESS).is_some());
        assert!(registry.get(&FEE_POOL_PRECOMPILE_ADDRESS).is_some());
        assert!(registry.get(&VALIDATORS_PRECOMPILE_ADDRESS).is_some());

        let duplicate = build_precompiles(
            SpecId::CANCUN,
            [
                fee_pool::register(),
                RegisteredPrecompile::new_stateful(
                    "duplicate_fee_pool",
                    FEE_POOL_PRECOMPILE_ADDRESS,
                    |_input| Ok(RevmPrecompileOutput::new(1, Bytes::new())),
                ),
            ],
        );
        assert_eq!(
            duplicate.expect_err("duplicate address must fail"),
            RegistryError::DuplicateCustomAddress(FEE_POOL_PRECOMPILE_ADDRESS)
        );
    }

    #[test]
    fn fee_pool_rejects_non_direct_state_changing_calls() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = fee_pool::register().precompile();
        let proxy_target = address!("0x0000000000000000000000000000000000000def");

        let revert_result = call_registered_precompile_with_context(
            precompile.clone(),
            &mut ctx,
            proxy_target,
            withdraw_calldata(),
            fee_pool::gas::WITHDRAW_GAS,
            false,
            proxy_target,
            FEE_POOL_PRECOMPILE_ADDRESS,
        );

        assert!(revert_result.reverted);
        assert_eq!(revert_result.bytes, non_direct_call_revert_bytes());
        assert_eq!(revert_result.gas_used, 0);
    }

    #[test]
    fn fee_pool_rejects_non_direct_read_calls() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = fee_pool::register().precompile();
        let proxy_target = address!("0x0000000000000000000000000000000000000fed");

        let revert_result = call_registered_precompile_with_context(
            precompile,
            &mut ctx,
            proxy_target,
            fee_pool_balance_calldata(),
            fee_pool::gas::FEE_POOL_BALANCE_GAS,
            true,
            proxy_target,
            FEE_POOL_PRECOMPILE_ADDRESS,
        );

        assert!(revert_result.reverted);
        assert_eq!(revert_result.bytes, non_direct_call_revert_bytes());
        assert_eq!(revert_result.gas_used, 0);
    }
}
