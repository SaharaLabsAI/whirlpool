use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol, SolCall};
use reth_evm::precompiles::PrecompileInput;
use reth_evm::revm::precompile::{PrecompileError, PrecompileOutput, PrecompileResult};

use crate::RegisteredPrecompile;

mod slot_storage_primary;
mod slot_storage_tail;
mod slot_value_primary;
mod slot_value_tail;

pub const COMMUNITY_POOL_ADDRESS: Address = Address::new([
    0x63, 0x6f, 0x6d, 0x6d, 0x75, 0x6e, 0x69, 0x74, 0x79, 0x2d, 0x70, 0x6f, 0x6f, 0x6c, 0x2d, 0x61,
    0x63, 0x63, 0x6f, 0x75,
]);

pub const COMMUNITY_POOL_UNLOCK_EVERY_EPOCHS_SLOT: U256 = U256::ZERO;
pub const COMMUNITY_POOL_UNLOCK_AMOUNT_PER_CYCLE_SLOT: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const COMMUNITY_POOL_LOCKED_REMAINING_SLOT: U256 = U256::from_limbs([2, 0, 0, 0]);
pub const COMMUNITY_POOL_LAST_PROCESSED_EPOCH_SLOT: U256 = U256::from_limbs([3, 0, 0, 0]);

pub use slot_storage_primary::{
    community_pool_locked_remaining_storage_slot,
    community_pool_unlock_amount_per_cycle_storage_slot,
    community_pool_unlock_every_epochs_storage_slot,
};
pub use slot_storage_tail::{
    community_pool_last_processed_epoch_storage_slot, encode_u256_storage_value,
};
pub use slot_value_primary::{
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot,
};
pub use slot_value_tail::community_pool_last_processed_epoch_slot;

sol! {
    function communityPoolBalance() external view returns (uint256);
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CommunityPoolPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported community-pool selector")]
    UnsupportedSelector,
    #[error("invalid community-pool calldata")]
    InvalidCalldata,
    #[error("invalid community-pool return payload")]
    InvalidReturnPayload,
}

pub fn community_pool_balance_calldata() -> Bytes {
    Bytes::from(communityPoolBalanceCall {}.abi_encode())
}

pub fn decode_community_pool_balance_output(
    payload: &Bytes,
) -> Result<U256, CommunityPoolPrecompileError> {
    if payload.len() != 32 {
        return Err(CommunityPoolPrecompileError::InvalidReturnPayload);
    }

    let mut word = [0u8; 32];
    word.copy_from_slice(payload.as_ref());
    Ok(U256::from_be_bytes(word))
}

pub fn register() -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_community_pool_balance",
        COMMUNITY_POOL_ADDRESS,
        execute,
    )
}

fn execute(mut input: PrecompileInput<'_>) -> PrecompileResult {
    let gas_limit = input.gas();
    if gas_limit < gas::COMMUNITY_POOL_BALANCE_GAS {
        return Err(PrecompileError::OutOfGas);
    }

    decode_call(input.data())?;

    let balance = input
        .internals_mut()
        .load_account(COMMUNITY_POOL_ADDRESS)
        .map(|account| account.data.info.balance)
        .map_err(|err| PrecompileError::other(err.to_string()))?;

    Ok(PrecompileOutput::new(
        gas::COMMUNITY_POOL_BALANCE_GAS,
        encode_u256_word(balance),
    ))
}

fn decode_call(data: &[u8]) -> Result<(), PrecompileError> {
    if data.len() < 4 {
        return Err(PrecompileError::other(
            CommunityPoolPrecompileError::CalldataTooShort.to_string(),
        ));
    }

    if !data.starts_with(&communityPoolBalanceCall::SELECTOR) {
        return Err(PrecompileError::other(
            CommunityPoolPrecompileError::UnsupportedSelector.to_string(),
        ));
    }

    communityPoolBalanceCall::abi_decode_validate(data)
        .map(|_| ())
        .map_err(|_| {
            PrecompileError::other(CommunityPoolPrecompileError::InvalidCalldata.to_string())
        })
}

fn encode_u256_word(value: U256) -> Bytes {
    Bytes::copy_from_slice(&value.to_be_bytes::<32>())
}

pub mod gas {
    pub const COMMUNITY_POOL_BALANCE_GAS: u64 = 750;
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(calldata.starts_with(&communityPoolBalanceCall::SELECTOR));
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
        let decoded = decode_community_pool_balance_output(&output.bytes)
            .expect("return payload should decode");
        assert_eq!(decoded, U256::ZERO);
    }

    #[test]
    fn unlock_storage_slots_are_stable() {
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
}
