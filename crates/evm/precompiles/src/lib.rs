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

pub mod test_token;
pub mod validators;

pub use test_token::{balance_of_calldata, mint_calldata, TEST_TOKEN_PRECOMPILE_ADDRESS};
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
            test_token::TestTokenPrecompile::register(),
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
    use crate::test_token::{balance_of_calldata, gas, mint_calldata, TestTokenPrecompile};
    use alloy_primitives::{address, Bytes, U256};
    use reth_evm::{precompiles::Precompile, traits::EvmInternals};
    use revm::Context;
    use revm::{database::EmptyDB, precompile::PrecompileOutput as RevmPrecompileOutput};

    fn call_registered_precompile(
        precompile: DynPrecompile,
        context: &mut Context<BlockEnv, TxEnv, revm::context::CfgEnv, EmptyDB>,
        data: Bytes,
        gas: u64,
        is_static: bool,
    ) -> RevmPrecompileOutput {
        call_registered_precompile_with_context(
            precompile,
            context,
            Address::ZERO,
            data,
            gas,
            is_static,
            TEST_TOKEN_PRECOMPILE_ADDRESS,
            TEST_TOKEN_PRECOMPILE_ADDRESS,
        )
    }

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
        let precompile = TestTokenPrecompile::register().precompile();
        let proxy_caller = address!("0x0000000000000000000000000000000000000abc");
        let account = address!("0x00000000000000000000000000000000000000ad");

        let mint_result = call_registered_precompile_with_context(
            precompile.clone(),
            &mut ctx,
            proxy_caller,
            mint_calldata(account, U256::from(3_u64)),
            gas::MINT_GAS,
            false,
            TEST_TOKEN_PRECOMPILE_ADDRESS,
            TEST_TOKEN_PRECOMPILE_ADDRESS,
        );

        assert!(
            !mint_result.reverted,
            "proxy-style caller should still be direct"
        );

        let balance_result = call_registered_precompile(
            precompile,
            &mut ctx,
            balance_of_calldata(account),
            gas::BALANCE_OF_GAS,
            true,
        );
        assert_eq!(decode_word(&balance_result.bytes), U256::from(3_u64));
    }

    #[test]
    fn registry_builds_expected_addresses() {
        let registry = build_whirlpool_precompiles(SpecId::CANCUN).expect("registry");
        assert!(registry.get(&TEST_TOKEN_PRECOMPILE_ADDRESS).is_some());
        assert!(registry.get(&VALIDATORS_PRECOMPILE_ADDRESS).is_some());

        let duplicate = build_precompiles(
            SpecId::CANCUN,
            [
                TestTokenPrecompile::register(),
                RegisteredPrecompile::new_stateful(
                    "duplicate_test_token",
                    TEST_TOKEN_PRECOMPILE_ADDRESS,
                    |_input| Ok(RevmPrecompileOutput::new(1, Bytes::new())),
                ),
            ],
        );
        assert_eq!(
            duplicate.expect_err("duplicate address must fail"),
            RegistryError::DuplicateCustomAddress(TEST_TOKEN_PRECOMPILE_ADDRESS)
        );
    }

    #[test]
    fn test_token_dispatch_routes_supported_methods() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = TestTokenPrecompile::register().precompile();
        let account = address!("0x00000000000000000000000000000000000000aa");

        let mint_result = call_registered_precompile(
            precompile.clone(),
            &mut ctx,
            mint_calldata(account, U256::from(7_u64)),
            gas::MINT_GAS,
            false,
        );
        assert!(!mint_result.reverted);
        assert_eq!(mint_result.gas_used, gas::MINT_GAS);

        let balance_result = call_registered_precompile(
            precompile,
            &mut ctx,
            balance_of_calldata(account),
            gas::BALANCE_OF_GAS,
            false,
        );
        assert_eq!(decode_word(&balance_result.bytes), U256::from(7_u64));
    }

    #[test]
    fn test_token_gas_policy_matches_declared_behavior() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = TestTokenPrecompile::register().precompile();
        let account = address!("0x00000000000000000000000000000000000000bb");

        let mint_result = call_registered_precompile(
            precompile.clone(),
            &mut ctx,
            mint_calldata(account, U256::from(1_u64)),
            gas::MINT_GAS,
            false,
        );
        assert_eq!(mint_result.gas_used, gas::MINT_GAS);

        let read_result = call_registered_precompile(
            precompile,
            &mut ctx,
            balance_of_calldata(account),
            gas::BALANCE_OF_GAS,
            true,
        );
        assert_eq!(read_result.gas_used, gas::BALANCE_OF_GAS);
    }

    #[test]
    fn test_token_rejects_non_direct_state_changing_calls() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = TestTokenPrecompile::register().precompile();
        let account = address!("0x00000000000000000000000000000000000000dd");
        let proxy_target = address!("0x0000000000000000000000000000000000000def");

        let revert_result = call_registered_precompile_with_context(
            precompile.clone(),
            &mut ctx,
            proxy_target,
            mint_calldata(account, U256::from(1_u64)),
            gas::MINT_GAS,
            false,
            proxy_target,
            TEST_TOKEN_PRECOMPILE_ADDRESS,
        );

        assert!(revert_result.reverted);
        assert_eq!(revert_result.bytes, non_direct_call_revert_bytes());
        assert_eq!(revert_result.gas_used, 0);

        let balance_result = call_registered_precompile(
            precompile,
            &mut ctx,
            balance_of_calldata(account),
            gas::BALANCE_OF_GAS,
            true,
        );
        assert_eq!(decode_word(&balance_result.bytes), U256::ZERO);
    }

    #[test]
    fn test_token_rejects_non_direct_read_calls() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = TestTokenPrecompile::register().precompile();
        let proxy_target = address!("0x0000000000000000000000000000000000000fed");
        let account = address!("0x00000000000000000000000000000000000000ee");

        let revert_result = call_registered_precompile_with_context(
            precompile,
            &mut ctx,
            proxy_target,
            balance_of_calldata(account),
            gas::BALANCE_OF_GAS,
            true,
            proxy_target,
            TEST_TOKEN_PRECOMPILE_ADDRESS,
        );

        assert!(revert_result.reverted);
        assert_eq!(revert_result.bytes, non_direct_call_revert_bytes());
        assert_eq!(revert_result.gas_used, 0);
    }

    #[test]
    fn test_token_error_maps_to_revert() {
        let mut ctx = EthEvmContext::new(EmptyDB::default(), Default::default());
        let precompile = TestTokenPrecompile::register().precompile();
        let account = address!("0x00000000000000000000000000000000000000cc");

        let revert_result = call_registered_precompile(
            precompile.clone(),
            &mut ctx,
            mint_calldata(account, U256::ZERO),
            gas::MINT_GAS,
            false,
        );
        assert!(revert_result.reverted, "zero-amount mint should revert");
        assert!(
            revert_result
                .bytes
                .as_ref()
                .starts_with(&[0x08, 0xc3, 0x79, 0xa0]),
            "revert payload should use Error(string) encoding"
        );

        let balance_result = call_registered_precompile(
            precompile,
            &mut ctx,
            balance_of_calldata(account),
            gas::BALANCE_OF_GAS,
            false,
        );
        assert_eq!(decode_word(&balance_result.bytes), U256::ZERO);
    }
}
