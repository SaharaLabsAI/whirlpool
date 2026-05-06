use crate::epoch::*;
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
fn pure_core_signatures_remain_primitive_and_value_only() {
    let _predicate: fn(EpochBoundaryState, u64) -> bool = boundary_required_for_height;
    let _matcher: fn(Address, Address, U256, &[u8]) -> bool = reserved_advance_epoch_call_matches;
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
