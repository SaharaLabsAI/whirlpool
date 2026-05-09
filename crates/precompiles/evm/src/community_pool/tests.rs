use crate::community_pool::codec::{decode_call, encode_u256_word};
use crate::community_pool::*;
use alloy_primitives::B256;
use reth_evm::revm::{
    context::{BlockEnv, TxEnv},
    database::EmptyDB,
    Context,
};
use reth_evm::{eth::EthEvmContext, precompiles::Precompile, traits::EvmInternals};

#[test]
fn calldata_roundtrip_decodes_balance_word() {
    let expected = U256::from(42_u64);
    let encoded = encode_u256_word(expected);
    let decoded =
        decode_community_pool_balance_output(&encoded).expect("must decode U256 return word");
    assert_eq!(decoded, expected);
}

#[test]
fn calldata_helper_uses_community_pool_balance_selector() {
    let calldata = community_pool_balance_calldata();
    decode_call(calldata.as_ref()).expect("calldata helper must encode the supported selector");
}

#[test]
fn rejects_unsupported_selector() {
    let precompile = register().precompile();
    let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
        EthEvmContext::new(EmptyDB::default(), Default::default());

    let result = precompile
        .call(PrecompileInput {
            data: &[0xde, 0xad, 0xbe, 0xef],
            gas: gas::COMMUNITY_POOL_BALANCE_GAS,
            caller: Address::ZERO,
            value: U256::ZERO,
            target_address: COMMUNITY_POOL_ADDRESS,
            bytecode_address: COMMUNITY_POOL_ADDRESS,
            is_static: true,
            internals: EvmInternals::from_context(&mut context),
        })
        .expect_err("unsupported selector should error");

    assert!(result
        .to_string()
        .contains("unsupported community-pool selector"));
}

#[test]
fn empty_state_balance_defaults_to_zero() {
    let precompile = register().precompile();
    let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
        EthEvmContext::new(EmptyDB::default(), Default::default());

    let output = precompile
        .call(PrecompileInput {
            data: community_pool_balance_calldata().as_ref(),
            gas: gas::COMMUNITY_POOL_BALANCE_GAS,
            caller: Address::ZERO,
            value: U256::ZERO,
            target_address: COMMUNITY_POOL_ADDRESS,
            bytecode_address: COMMUNITY_POOL_ADDRESS,
            is_static: true,
            internals: EvmInternals::from_context(&mut context),
        })
        .expect("call should succeed");

    assert!(!output.reverted);
    let decoded =
        decode_community_pool_balance_output(&output.bytes).expect("return payload should decode");
    assert_eq!(decoded, U256::ZERO);
}

#[test]
fn unlock_schedule_storage_slots_are_stable() {
    assert_eq!(community_pool_unlock_every_epochs_slot(), U256::from(0_u64));
    assert_eq!(
        community_pool_unlock_amount_per_cycle_slot(),
        U256::from(1_u64)
    );
    assert_eq!(community_pool_locked_remaining_slot(), U256::from(2_u64));
    assert_eq!(
        community_pool_last_processed_epoch_slot(),
        U256::from(3_u64)
    );
    assert_eq!(
        community_pool_unlock_every_epochs_storage_slot(),
        B256::from(U256::from(0_u64).to_be_bytes::<32>())
    );
}
