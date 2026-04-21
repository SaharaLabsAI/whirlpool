use alloy_primitives::{address, Address, Bytes, U256};
use reth_evm::revm::precompile::PrecompileResult;

use crate::RegisteredPrecompile;

mod decode_outputs;
mod dispatch;
pub mod gas;
mod r#impl;
pub mod storage;

pub use decode_outputs::{
    decode_claimable_balance_output, decode_fee_pool_balance_output, decode_withdraw_output,
};
pub use dispatch::{claimable_balance_calldata, fee_pool_balance_calldata, withdraw_calldata};
pub use storage::claimable_balance_slot;

pub const FEE_POOL_PRECOMPILE_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000102");

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FeePoolPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported fee-pool selector")]
    UnsupportedSelector,
    #[error("invalid feePoolBalance calldata")]
    InvalidFeePoolBalanceCalldata,
    #[error("invalid claimableBalance calldata")]
    InvalidClaimableBalanceCalldata,
    #[error("invalid withdraw calldata")]
    InvalidWithdrawCalldata,
    #[error("withdraw cannot run in a static context")]
    StaticCallWithdraw,
    #[error("invalid fee-pool return payload")]
    InvalidReturnPayload,
}

pub fn register() -> RegisteredPrecompile {
    RegisteredPrecompile::new_stateful(
        "whirlpool_fee_pool",
        FEE_POOL_PRECOMPILE_ADDRESS,
        r#impl::execute,
    )
}

fn encode_u256_word(value: U256) -> Bytes {
    Bytes::copy_from_slice(&value.to_be_bytes::<32>())
}

fn encode_revert_reason(reason: &str) -> Bytes {
    let reason_bytes = reason.as_bytes();
    let padded_len = reason_bytes.len().div_ceil(32) * 32;
    let mut payload = Vec::with_capacity(4 + 32 * 3 + padded_len);
    payload.extend_from_slice(&[0x08, 0xc3, 0x79, 0xa0]);
    payload.extend_from_slice(&U256::from(32_u64).to_be_bytes::<32>());
    payload.extend_from_slice(&U256::from(reason_bytes.len()).to_be_bytes::<32>());
    payload.extend_from_slice(reason_bytes);
    payload.resize(4 + 32 * 2 + padded_len, 0);
    Bytes::from(payload)
}

fn revert_result(gas_used: u64, error: FeePoolPrecompileError) -> PrecompileResult {
    Ok(reth_evm::revm::precompile::PrecompileOutput::new_reverted(
        gas_used,
        encode_revert_reason(&error.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reth_evm::revm::{
        context::{BlockEnv, TxEnv},
        database::EmptyDB,
        Context,
    };
    use reth_evm::{
        eth::EthEvmContext,
        precompiles::{Precompile, PrecompileInput},
        traits::EvmInternals,
    };

    fn call_precompile(
        context: &mut Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB>,
        caller: Address,
        data: Bytes,
        gas: u64,
        is_static: bool,
    ) -> reth_evm::revm::precompile::PrecompileOutput {
        register()
            .precompile()
            .call(PrecompileInput {
                data: data.as_ref(),
                gas,
                caller,
                value: U256::ZERO,
                target_address: FEE_POOL_PRECOMPILE_ADDRESS,
                bytecode_address: FEE_POOL_PRECOMPILE_ADDRESS,
                is_static,
                internals: EvmInternals::from_context(context),
            })
            .expect("fee-pool precompile call should succeed")
    }

    #[test]
    fn calldata_helpers_decode_outputs() {
        let expected = U256::from(42_u64);
        let encoded = encode_u256_word(expected);
        assert_eq!(
            decode_fee_pool_balance_output(&encoded).expect("decode balance"),
            expected
        );
        assert_eq!(
            decode_claimable_balance_output(&encoded).expect("decode claimable"),
            expected
        );
        assert_eq!(
            decode_withdraw_output(&encoded).expect("decode withdraw"),
            expected
        );
    }

    #[test]
    fn claimable_balance_slot_is_deterministic() {
        let recipient = Address::repeat_byte(0x11);
        let slot = claimable_balance_slot(recipient);
        assert_eq!(slot, claimable_balance_slot(recipient));
        assert_ne!(slot, claimable_balance_slot(Address::repeat_byte(0x12)));
    }

    #[test]
    fn claimable_balance_defaults_to_zero() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        let caller = Address::repeat_byte(0xaa);

        let output = call_precompile(
            &mut context,
            caller,
            claimable_balance_calldata(caller),
            gas::CLAIMABLE_BALANCE_GAS,
            true,
        );

        assert!(!output.reverted);
        assert_eq!(
            decode_claimable_balance_output(&output.bytes).expect("decode output"),
            U256::ZERO
        );
    }

    #[test]
    fn withdraw_transfers_claim_and_clears_ledger() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        let caller = Address::repeat_byte(0x33);
        let claimable = U256::from(21_000_u64);
        let slot = claimable_balance_slot(caller);

        {
            let mut internals = EvmInternals::from_context(&mut context);
            internals
                .balance_incr(FEE_POOL_PRECOMPILE_ADDRESS, claimable)
                .expect("credit pool");
            internals
                .sstore(FEE_POOL_PRECOMPILE_ADDRESS, slot, claimable)
                .expect("seed claim slot");
        }

        let output = call_precompile(
            &mut context,
            caller,
            withdraw_calldata(),
            gas::WITHDRAW_GAS,
            false,
        );
        assert!(!output.reverted);
        assert_eq!(
            decode_withdraw_output(&output.bytes).expect("decode withdraw"),
            claimable
        );

        let mut internals = EvmInternals::from_context(&mut context);
        let caller_balance = internals
            .load_account(caller)
            .expect("load caller")
            .data
            .info
            .balance;
        assert_eq!(caller_balance, claimable);

        let stored_claim = internals
            .sload(FEE_POOL_PRECOMPILE_ADDRESS, slot)
            .expect("load claim slot")
            .data;
        assert_eq!(stored_claim, U256::ZERO);
    }

    #[test]
    fn remapped_recipient_cannot_withdraw_historical_claim() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        let original = Address::repeat_byte(0x44);
        let remapped = Address::repeat_byte(0x55);
        let claimable = U256::from(10_000_u64);
        let slot = claimable_balance_slot(original);

        {
            let mut internals = EvmInternals::from_context(&mut context);
            internals
                .balance_incr(FEE_POOL_PRECOMPILE_ADDRESS, claimable)
                .expect("credit pool");
            internals
                .sstore(FEE_POOL_PRECOMPILE_ADDRESS, slot, claimable)
                .expect("seed claim slot");
        }

        let remapped_withdraw = call_precompile(
            &mut context,
            remapped,
            withdraw_calldata(),
            gas::WITHDRAW_GAS,
            false,
        );
        assert!(!remapped_withdraw.reverted);
        assert_eq!(
            decode_withdraw_output(&remapped_withdraw.bytes).expect("decode remapped withdraw"),
            U256::ZERO
        );

        let original_withdraw = call_precompile(
            &mut context,
            original,
            withdraw_calldata(),
            gas::WITHDRAW_GAS,
            false,
        );
        assert!(!original_withdraw.reverted);
        assert_eq!(
            decode_withdraw_output(&original_withdraw.bytes).expect("decode original withdraw"),
            claimable
        );
    }
}
