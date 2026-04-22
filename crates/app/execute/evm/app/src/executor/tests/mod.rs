use super::*;
use crate::config::DEFAULT_PROPOSER_FEE_RECIPIENT;
use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Signature, TxKind, U256};
use app::{encode_canonical_extra_data, CanonicalExtraDataV1, ExtraDataDecodeMode, FullDkgV1};
use chainspec::{
    build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients,
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config,
    CommunityPoolUnlockConfig, SAHARA_CHAIN_ID,
};
use evm_precompiles::{
    advance_epoch_calldata, claimable_balance_slot, community_pool_last_processed_epoch_slot,
    community_pool_locked_remaining_slot, community_pool_unlock_amount_per_cycle_slot,
    community_pool_unlock_every_epochs_slot, current_epoch_slot, epoch_blocks_slot,
    epoch_system_tx_sender, next_epoch_block_slot, withdraw_calldata, COMMUNITY_POOL_ADDRESS,
    EPOCH_BLOCKS_DEFAULT, EPOCH_PRECOMPILE_ADDRESS, EPOCH_SYSTEM_TX_GAS_LIMIT,
    EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI, EPOCH_SYSTEM_TX_PRIVATE_KEY, FEE_POOL_PRECOMPILE_ADDRESS,
};
use reth_ethereum_primitives::TransactionSigned;
use reth_evm::execute::BlockExecutionError;
use reth_evm::execute::BlockValidationError;
use reth_primitives_traits::crypto::secp256k1::sign_message;
use reth_primitives_traits::SignerRecoverable;
use revm::state::Bytecode;
use state_memory::InMemoryStateDb;
use std::collections::BTreeMap;

struct MockTxSource {
    txs: Vec<Vec<u8>>,
}

impl TxSource for MockTxSource {
    fn push(&self, _tx: Vec<u8>) {}

    fn pending(&self) -> Vec<Vec<u8>> {
        self.txs.clone()
    }
}

async fn setup_app(
    txs: Vec<Vec<u8>>,
) -> (
    EvmApplication<InMemoryStateDb>,
    Arc<RwLock<InMemoryStateDb>>,
) {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let source = Arc::new(MockTxSource { txs });

    let app = EvmApplication::new(config, db.clone(), source);
    (app, db)
}

async fn setup_app_with_config(
    txs: Vec<Vec<u8>>,
    config: WhirlpoolEvmConfig,
) -> (
    EvmApplication<InMemoryStateDb>,
    Arc<RwLock<InMemoryStateDb>>,
) {
    let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let source = Arc::new(MockTxSource { txs });
    let app = EvmApplication::new(config, db.clone(), source);
    (app, db)
}

fn setup_app_with_unlock_config(
    txs: Vec<Vec<u8>>,
    unlock_config: CommunityPoolUnlockConfig,
    simplex_validators: Vec<validators::ValidatorEntry>,
) -> (
    EvmApplication<InMemoryStateDb>,
    Arc<RwLock<InMemoryStateDb>>,
) {
    let chain_spec = Arc::new(
            build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
                BTreeMap::new(),
                BTreeMap::new(),
                simplex_validators,
                unlock_config,
            ),
        );
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let source = Arc::new(MockTxSource { txs });

    let app = EvmApplication::new(config, db.clone(), source);
    (app, db)
}

fn seed_epoch_boundary_state(db: &mut InMemoryStateDb, next_epoch_block: u64, epoch_blocks: u64) {
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        current_epoch_slot(),
        U256::from(0_u64),
    );
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        epoch_blocks_slot(),
        U256::from(epoch_blocks),
    );
    db.insert_storage(
        EPOCH_PRECOMPILE_ADDRESS,
        next_epoch_block_slot(),
        U256::from(next_epoch_block),
    );
    db.insert_account(
        epoch_system_tx_sender(),
        revm::state::AccountInfo {
            balance: U256::from(EPOCH_SYSTEM_TX_INITIAL_BALANCE_WEI),
            nonce: 0,
            ..Default::default()
        },
    );
}

fn seed_community_pool_unlock_state(
    db: &mut InMemoryStateDb,
    unlock_every_epochs: u64,
    unlock_amount_per_cycle: U256,
    locked_remaining: U256,
    community_pool_balance: U256,
) {
    db.insert_account(
        COMMUNITY_POOL_ADDRESS,
        revm::state::AccountInfo {
            balance: community_pool_balance,
            nonce: 0,
            ..Default::default()
        },
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_unlock_every_epochs_slot(),
        U256::from(unlock_every_epochs),
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_unlock_amount_per_cycle_slot(),
        unlock_amount_per_cycle,
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
        locked_remaining,
    );
    db.insert_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
        U256::ZERO,
    );
}

fn sample_evm_tx_with_nonce(nonce: u64, receiver: Address) -> (Vec<u8>, Address) {
    sample_evm_tx_with_chain_id(Some(SAHARA_CHAIN_ID), nonce, receiver)
}

fn sample_evm_tx_with_chain_id(
    chain_id: Option<u64>,
    nonce: u64,
    receiver: Address,
) -> (Vec<u8>, Address) {
    let tx = TxLegacy {
        chain_id,
        nonce,
        gas_price: 2_000_000_000,
        gas_limit: 21_000,
        to: TxKind::Call(receiver),
        value: U256::from(1000),
        input: Bytes::default(),
    };
    let signature = Signature::test_signature();
    let signed: TransactionSigned = tx.into_signed(signature).into();
    let recovered = signed.recover_signer().unwrap();

    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    (encoded, recovered)
}

fn sample_evm_tx() -> (Vec<u8>, Address) {
    sample_evm_tx_with_nonce(0, Address::with_last_byte(2))
}

fn sample_reserved_epoch_namespace_tx(nonce: u64, gas_price: u128) -> Vec<u8> {
    sample_epoch_system_call_tx(nonce, gas_price, U256::ZERO)
}

fn sample_epoch_system_call_tx(nonce: u64, gas_price: u128, value: U256) -> Vec<u8> {
    let tx = TxLegacy {
        chain_id: Some(SAHARA_CHAIN_ID),
        nonce,
        gas_price,
        gas_limit: EPOCH_SYSTEM_TX_GAS_LIMIT,
        to: TxKind::Call(EPOCH_PRECOMPILE_ADDRESS),
        value,
        input: advance_epoch_calldata(),
    };
    let signature = sign_message(EPOCH_SYSTEM_TX_PRIVATE_KEY, tx.signature_hash())
        .expect("epoch system tx signature");
    let signed: TransactionSigned = tx.into_signed(signature).into();

    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    encoded
}

fn precompile_proxy_runtime_bytecode() -> Bytes {
    let mut runtime = alloy_primitives::hex::decode("36600060003760006000366000600073")
        .expect("forwarder prefix");
    runtime.extend_from_slice(FEE_POOL_PRECOMPILE_ADDRESS.as_slice());
    runtime.extend_from_slice(
        &alloy_primitives::hex::decode("5af13d600060003e156034573d6000f35b3d6000fd")
            .expect("forwarder suffix"),
    );
    Bytes::from(runtime)
}

fn sample_proxy_precompile_withdraw_tx(proxy_address: Address) -> (Vec<u8>, Address) {
    let tx = TxLegacy {
        chain_id: Some(SAHARA_CHAIN_ID),
        nonce: 0,
        gas_price: 2_000_000_000,
        gas_limit: 200_000,
        to: TxKind::Call(proxy_address),
        value: U256::ZERO,
        input: withdraw_calldata(),
    };
    let signature = Signature::test_signature();
    let signed: TransactionSigned = tx.into_signed(signature).into();
    let recovered = signed.recover_signer().unwrap();

    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    (encoded, recovered)
}

mod boundary_failures;
mod boundary_transactions;
mod boundary_unlock;
mod boundary_verification;
mod decoding;
mod fee_accounting;
mod full_dkg_activation;
mod full_dkg_core;
mod receipts;
mod verification;
