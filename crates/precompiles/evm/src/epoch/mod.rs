use alloy_primitives::{address, Address, Bytes, B256, U256};
use reth_evm::revm::precompile::PrecompileResult;

mod boundary_effect;
mod boundary_semantics;
mod decode_primary;
mod decode_tail;
mod dispatch;
pub mod gas;
mod r#impl;
mod registration;
pub mod storage;

pub use boundary_effect::{
    extract_epoch_boundary_effect, EpochBoundaryEffect, EpochBoundaryEffectError,
    EpochBoundaryStorageWrite,
};
pub use boundary_semantics::{
    boundary_required_for_height, reserved_advance_epoch_call_matches, EpochBoundaryState,
};
pub use decode_primary::{
    decode_current_epoch_output, decode_next_epoch_block_output, decode_u64_output,
};
pub use decode_tail::{decode_epoch_blocks_output, decode_epoch_start_block_output};
pub use dispatch::{
    advance_epoch_calldata, current_epoch_calldata, epoch_blocks_calldata,
    epoch_start_block_calldata, is_advance_epoch_calldata, next_epoch_block_calldata,
};
pub use registration::{epoch_system_tx_sender, register};
pub use storage::{
    current_epoch_slot, current_epoch_storage_slot, decode_epoch_start_block_storage_value,
    decode_u64_storage_value, encode_epoch_start_block_storage_value, encode_u64_storage_value,
    epoch_blocks_slot, epoch_blocks_storage_slot, epoch_start_block_slot,
    epoch_start_block_storage_slot, next_epoch_block_slot, next_epoch_block_storage_slot,
};

pub const EPOCH_PRECOMPILE_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000103");

pub const EPOCH_BLOCKS_DEFAULT: u64 = 403_200;
pub const EPOCH_SYSTEM_TX_GAS_LIMIT: u64 = 120_000;
pub const EPOCH_SYSTEM_TX_PRIVATE_KEY: B256 = B256::repeat_byte(0x42);
pub const EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI: u128 = 1_000_000_000_000_000_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EpochPrecompileError {
    #[error("calldata is too short")]
    CalldataTooShort,
    #[error("unsupported epoch selector")]
    UnsupportedSelector,
    #[error("invalid currentEpoch calldata")]
    InvalidCurrentEpochCalldata,
    #[error("invalid nextEpochBlock calldata")]
    InvalidNextEpochBlockCalldata,
    #[error("invalid epochBlocks calldata")]
    InvalidEpochBlocksCalldata,
    #[error("invalid epochStartBlock calldata")]
    InvalidEpochStartBlockCalldata,
    #[error("invalid advanceEpoch calldata")]
    InvalidAdvanceEpochCalldata,
    #[error("advanceEpoch cannot run in a static context")]
    StaticCallAdvanceEpoch,
    #[error("advanceEpoch caller is not authorized")]
    UnauthorizedAdvanceEpochCaller,
    #[error("advanceEpoch called at block {got} but expected boundary block {expected}")]
    InvalidBoundaryBlock { expected: u64, got: u64 },
    #[error("epoch {0} is not initialized")]
    EpochNotInitialized(u64),
    #[error("epoch start for epoch {0} is already initialized")]
    EpochStartAlreadyInitialized(u64),
    #[error("epoch storage value does not fit into uint64")]
    ValueOutOfRange,
    #[error("epoch arithmetic overflow")]
    ArithmeticOverflow,
    #[error("invalid epoch return payload")]
    InvalidReturnPayload,
}

fn encode_u64_word(value: u64) -> Bytes {
    Bytes::copy_from_slice(&U256::from(value).to_be_bytes::<32>())
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

fn revert_result(gas_used: u64, error: EpochPrecompileError) -> PrecompileResult {
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
                target_address: EPOCH_PRECOMPILE_ADDRESS,
                bytecode_address: EPOCH_PRECOMPILE_ADDRESS,
                is_static,
                internals: EvmInternals::from_context(context),
            })
            .expect("epoch precompile call should succeed")
    }

    fn seed_epoch_state(
        context: &mut Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB>,
        current_epoch: u64,
        epoch_blocks: u64,
        next_epoch_block: u64,
    ) {
        let mut internals = EvmInternals::from_context(context);
        internals
            .sstore(
                EPOCH_PRECOMPILE_ADDRESS,
                storage::current_epoch_slot(),
                U256::from(current_epoch),
            )
            .expect("seed current epoch");
        internals
            .sstore(
                EPOCH_PRECOMPILE_ADDRESS,
                storage::epoch_blocks_slot(),
                U256::from(epoch_blocks),
            )
            .expect("seed epoch blocks");
        internals
            .sstore(
                EPOCH_PRECOMPILE_ADDRESS,
                storage::next_epoch_block_slot(),
                U256::from(next_epoch_block),
            )
            .expect("seed next epoch block");
        internals
            .sstore(
                EPOCH_PRECOMPILE_ADDRESS,
                storage::epoch_start_block_slot(0),
                U256::from(1_u64),
            )
            .expect("seed epoch zero start");
    }

    #[test]
    fn decode_helpers_roundtrip_u64_word() {
        let encoded = encode_u64_word(42);
        assert_eq!(
            decode_current_epoch_output(&encoded).expect("decode current epoch"),
            42
        );
        assert_eq!(
            decode_next_epoch_block_output(&encoded).expect("decode next epoch"),
            42
        );
        assert_eq!(
            decode_epoch_blocks_output(&encoded).expect("decode epoch blocks"),
            42
        );
        assert_eq!(
            decode_epoch_start_block_output(&encoded).expect("decode start block"),
            42
        );
    }

    #[test]
    fn epoch_system_sender_is_stable() {
        let sender = epoch_system_tx_sender();
        assert_eq!(sender, epoch_system_tx_sender());
    }

    #[test]
    fn boundary_required_for_height_matches_next_epoch_block() {
        let state = EpochBoundaryState {
            next_epoch_block: 5,
        };

        assert!(boundary_required_for_height(state, 5));
        assert!(!boundary_required_for_height(state, 4));
        assert!(!boundary_required_for_height(state, 6));
    }

    #[test]
    fn reserved_advance_epoch_call_matches_requires_zero_value() {
        assert!(reserved_advance_epoch_call_matches(
            epoch_system_tx_sender(),
            EPOCH_PRECOMPILE_ADDRESS,
            U256::ZERO,
            &advance_epoch_calldata(),
        ));
    }

    #[test]
    fn reserved_advance_epoch_call_matches_rejects_non_zero_value_near_miss() {
        assert!(!reserved_advance_epoch_call_matches(
            epoch_system_tx_sender(),
            EPOCH_PRECOMPILE_ADDRESS,
            U256::from(1_u64),
            &advance_epoch_calldata(),
        ));
    }

    #[test]
    fn semantic_core_signatures_do_not_expose_app_types() {
        let _predicate: fn(EpochBoundaryState, u64) -> bool = boundary_required_for_height;
        let _matcher: fn(Address, Address, U256, &[u8]) -> bool =
            reserved_advance_epoch_call_matches;
    }

    #[test]
    fn advance_epoch_updates_epoch_state() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        context.block.number = U256::from(5_u64);
        seed_epoch_state(&mut context, 0, 10, 5);

        let output = call_precompile(
            &mut context,
            epoch_system_tx_sender(),
            advance_epoch_calldata(),
            gas::ADVANCE_EPOCH_GAS,
            false,
        );
        assert!(!output.reverted);

        let current_epoch = call_precompile(
            &mut context,
            Address::ZERO,
            current_epoch_calldata(),
            gas::CURRENT_EPOCH_GAS,
            true,
        );
        assert_eq!(
            decode_current_epoch_output(&current_epoch.bytes).expect("decode current"),
            1
        );
        let next_epoch = call_precompile(
            &mut context,
            Address::ZERO,
            next_epoch_block_calldata(),
            gas::NEXT_EPOCH_BLOCK_GAS,
            true,
        );
        assert_eq!(
            decode_next_epoch_block_output(&next_epoch.bytes).expect("decode next"),
            15
        );
        let start_epoch_one = call_precompile(
            &mut context,
            Address::ZERO,
            epoch_start_block_calldata(1),
            gas::EPOCH_START_BLOCK_GAS,
            true,
        );
        assert_eq!(
            decode_epoch_start_block_output(&start_epoch_one.bytes).expect("decode epoch start"),
            5
        );
    }

    #[test]
    fn advance_epoch_reverts_for_unauthorized_caller() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        context.block.number = U256::from(5_u64);
        seed_epoch_state(&mut context, 0, 10, 5);

        let output = call_precompile(
            &mut context,
            Address::with_last_byte(0x99),
            advance_epoch_calldata(),
            gas::ADVANCE_EPOCH_GAS,
            false,
        );
        assert!(output.reverted);
    }

    #[test]
    fn advance_epoch_reverts_for_static_call() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        context.block.number = U256::from(5_u64);
        seed_epoch_state(&mut context, 0, 10, 5);

        let output = call_precompile(
            &mut context,
            epoch_system_tx_sender(),
            advance_epoch_calldata(),
            gas::ADVANCE_EPOCH_GAS,
            true,
        );
        assert!(output.reverted);
    }

    #[test]
    fn advance_epoch_reverts_when_next_epoch_start_is_already_initialized() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        context.block.number = U256::from(5_u64);
        seed_epoch_state(&mut context, 0, 10, 5);
        {
            let mut internals = EvmInternals::from_context(&mut context);
            internals
                .sstore(
                    EPOCH_PRECOMPILE_ADDRESS,
                    storage::epoch_start_block_slot(1),
                    U256::from(123_u64),
                )
                .expect("seed epoch one start");
        }

        let output = call_precompile(
            &mut context,
            epoch_system_tx_sender(),
            advance_epoch_calldata(),
            gas::ADVANCE_EPOCH_GAS,
            false,
        );
        assert!(output.reverted);
    }

    #[test]
    fn historical_epoch_starts_remain_immutable_after_epoch_blocks_change() {
        let mut context: Context<BlockEnv, TxEnv, reth_evm::revm::context::CfgEnv, EmptyDB> =
            EthEvmContext::new(EmptyDB::default(), Default::default());
        context.block.number = U256::from(5_u64);
        seed_epoch_state(&mut context, 0, 10, 5);

        let first_advance = call_precompile(
            &mut context,
            epoch_system_tx_sender(),
            advance_epoch_calldata(),
            gas::ADVANCE_EPOCH_GAS,
            false,
        );
        assert!(!first_advance.reverted);

        {
            let mut internals = EvmInternals::from_context(&mut context);
            internals
                .sstore(
                    EPOCH_PRECOMPILE_ADDRESS,
                    storage::epoch_blocks_slot(),
                    U256::from(20_u64),
                )
                .expect("update epoch blocks");
        }

        context.block.number = U256::from(15_u64);
        let second_advance = call_precompile(
            &mut context,
            epoch_system_tx_sender(),
            advance_epoch_calldata(),
            gas::ADVANCE_EPOCH_GAS,
            false,
        );
        assert!(!second_advance.reverted);

        let epoch_one_start = call_precompile(
            &mut context,
            Address::ZERO,
            epoch_start_block_calldata(1),
            gas::EPOCH_START_BLOCK_GAS,
            true,
        );
        assert_eq!(
            decode_epoch_start_block_output(&epoch_one_start.bytes).expect("decode epoch one"),
            5
        );

        let epoch_two_start = call_precompile(
            &mut context,
            Address::ZERO,
            epoch_start_block_calldata(2),
            gas::EPOCH_START_BLOCK_GAS,
            true,
        );
        assert_eq!(
            decode_epoch_start_block_output(&epoch_two_start.bytes).expect("decode epoch two"),
            15
        );

        let next_epoch = call_precompile(
            &mut context,
            Address::ZERO,
            next_epoch_block_calldata(),
            gas::NEXT_EPOCH_BLOCK_GAS,
            true,
        );
        assert_eq!(
            decode_next_epoch_block_output(&next_epoch.bytes).expect("decode next epoch block"),
            35
        );
    }
}
