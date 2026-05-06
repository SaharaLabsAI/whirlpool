use crate::fee_pool::{fee_pool_balance_calldata, withdraw_calldata, FEE_POOL_PRECOMPILE_ADDRESS};
use crate::*;
use alloy_primitives::{address, Bytes, U256};
use reth_evm::revm::Context;
use reth_evm::revm::{database::EmptyDB, precompile::PrecompileOutput as RevmPrecompileOutput};
use reth_evm::{
    precompiles::{Precompile, PrecompileInput},
    traits::EvmInternals,
};

#[allow(clippy::too_many_arguments)]
fn call_registered_precompile_with_context(
    precompile: DynPrecompile,
    context: &mut Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB>,
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
