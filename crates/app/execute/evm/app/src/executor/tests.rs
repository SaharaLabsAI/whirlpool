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
    let tx = TxLegacy {
        chain_id: Some(SAHARA_CHAIN_ID),
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
    let tx = TxLegacy {
        chain_id: Some(SAHARA_CHAIN_ID),
        nonce,
        gas_price,
        gas_limit: EPOCH_SYSTEM_TX_GAS_LIMIT,
        to: TxKind::Call(EPOCH_PRECOMPILE_ADDRESS),
        value: U256::ZERO,
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

#[test]
fn decode_evm_transaction_recovers_signer() {
    let (raw_tx, recovered) = sample_evm_tx();

    let decoded = decode_evm_transaction(&raw_tx).expect("tx should decode");

    assert_eq!(decoded.signer(), recovered);
}

#[test]
fn decode_evm_transactions_reject_invalid_bytes() {
    let err = decode_evm_transactions(&[vec![0xff, 0x00, 0x01]])
        .expect_err("invalid bytes should fail decoding");

    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[tokio::test]
async fn propose_executes_transfer_transaction() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, result) = app.propose(&parent, 1).await.unwrap();

    assert_eq!(block.transactions.len(), 1);
    assert!(result.gas_used > 0);
}

#[tokio::test]
async fn propose_routes_priority_fees_to_fee_pool_not_proposer() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, _result) = app.propose(&parent, 1).await.unwrap();
    let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);
    let expected_priority_fees =
        U256::from(block.gas_used) * U256::from(2_000_000_000u64 - block.base_fee_per_gas);
    let claim_slot = claimable_balance_slot(DEFAULT_PROPOSER_FEE_RECIPIENT);

    let db = db.read().unwrap();
    let community_pool_balance = db
        .get_account(COMMUNITY_POOL_ADDRESS)
        .unwrap_or_default()
        .balance;
    let fee_pool_balance = db
        .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .unwrap_or_default()
        .balance;
    let fee_recipient_balance = db
        .get_account(DEFAULT_PROPOSER_FEE_RECIPIENT)
        .unwrap_or_default()
        .balance;
    let claimable = db.get_storage(FEE_POOL_PRECOMPILE_ADDRESS, claim_slot);

    assert_eq!(community_pool_balance, burned_amount);
    assert_eq!(fee_pool_balance, expected_priority_fees);
    assert_eq!(claimable, expected_priority_fees);
    assert_eq!(fee_recipient_balance, U256::ZERO);
    assert_eq!(
        block.proposer_fee_recipient,
        DEFAULT_PROPOSER_FEE_RECIPIENT.into_array()
    );
}

#[tokio::test]
async fn propose_uses_final_cumulative_gas_used_for_block_gas_and_burned_fee_credit() {
    let (tx0, recovered0) = sample_evm_tx_with_nonce(0, Address::with_last_byte(2));
    let (tx1, recovered1) = (3u8..=u8::MAX)
        .map(|byte| sample_evm_tx_with_nonce(0, Address::with_last_byte(byte)))
        .find(|(_, recovered)| *recovered != recovered0)
        .expect("must find a second sender");
    let (app, db) = setup_app(vec![tx0, tx1]).await;

    {
        let mut db = db.write().unwrap();
        for recovered in [recovered0, recovered1] {
            let info = revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            };
            db.insert_account(recovered, info);
        }
    }

    let parent = app.genesis().await;
    let (block, _result) = app.propose(&parent, 1).await.unwrap();
    let receipts = app.pending_receipts();
    assert_eq!(receipts.len(), 2, "expected two successful tx receipts");

    let expected_gas_used = receipts.last().expect("has receipts").cumulative_gas_used;
    assert_eq!(
        block.gas_used, expected_gas_used,
        "block gas used should equal final cumulative gas"
    );

    let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);
    let expected_priority_fees =
        U256::from(block.gas_used) * U256::from(2_000_000_000u64 - block.base_fee_per_gas);
    let claim_slot = claimable_balance_slot(DEFAULT_PROPOSER_FEE_RECIPIENT);

    let db = db.read().unwrap();
    let community_pool_balance = db
        .get_account(COMMUNITY_POOL_ADDRESS)
        .unwrap_or_default()
        .balance;
    let fee_pool_balance = db
        .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .unwrap_or_default()
        .balance;
    let claimable = db.get_storage(FEE_POOL_PRECOMPILE_ADDRESS, claim_slot);

    assert_eq!(
        community_pool_balance, burned_amount,
        "community pool burn credit should use corrected block gas used"
    );
    assert_eq!(
        fee_pool_balance, expected_priority_fees,
        "fee-pool sink should be credited exactly once by execution beneficiary"
    );
    assert_eq!(claimable, expected_priority_fees);
}

#[tokio::test]
async fn burned_fee_credit_preserves_community_pool_unlock_storage() {
    let validators = vec![validators::ValidatorEntry {
        consensus_pubkey: [0x11; 32],
        ethereum_address: Address::repeat_byte(0x11),
    }];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(50_u64),
        unlock_every_epochs: 2,
        unlock_amount_per_cycle: U256::from(7_u64),
    };
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app_with_unlock_config(vec![tx], unlock_config, validators);
    let initial_community_pool_balance = U256::from(50_u64);
    let expected_locked_remaining = U256::from(42_u64);
    let expected_last_processed_epoch = U256::from(7_u64);

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            expected_locked_remaining,
            initial_community_pool_balance,
        );
        db.insert_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
            expected_last_processed_epoch,
        );
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let parent = app.genesis().await;
    let (block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose non-boundary block with burned fee credit");
    let burned_amount = U256::from(block.gas_used) * U256::from(block.base_fee_per_gas);

    let db = db.read().unwrap();
    let community_pool_balance = db
        .get_account(COMMUNITY_POOL_ADDRESS)
        .unwrap_or_default()
        .balance;
    assert_eq!(
        db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
        U256::ZERO,
        "non-boundary block should not advance epoch state"
    );
    assert_eq!(
        community_pool_balance,
        initial_community_pool_balance + burned_amount,
        "burned fee credit should only increase community pool balance"
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_every_epochs_slot()
        ),
        U256::from(unlock_config.unlock_every_epochs)
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_unlock_amount_per_cycle_slot()
        ),
        unlock_config.unlock_amount_per_cycle
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        expected_locked_remaining
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot()
        ),
        expected_last_processed_epoch
    );
}

#[tokio::test]
async fn verify_accepts_valid_block() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.unwrap();

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
    let pre_db = Arc::new(RwLock::new(pre_state));
    let source = Arc::new(MockTxSource { txs: vec![] });
    let verifier_app = EvmApplication::new(config, pre_db, source);

    assert!(verifier_app.verify(&parent, &block).await.is_ok());
}

#[tokio::test]
async fn verify_accepts_legacy_extra_data_before_strict_height() {
    let strict_height = 2;
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(strict_height);
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;
    let pre_state = db.read().unwrap().clone();

    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();
    block.extra_data = legacy_proposer_extra_data_bytes(block.proposer_public_key);

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    assert!(
        verifier.verify(&parent, &block).await.is_ok(),
        "legacy extra_data must remain accepted before strict-height boundary"
    );
}

#[tokio::test]
async fn verify_rejects_legacy_extra_data_at_or_after_strict_height() {
    let strict_height = 2;
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(strict_height);
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    let genesis = app.genesis().await;
    let (parent, _) = app.propose(&genesis, 1).await.unwrap();
    let pre_state = db.read().unwrap().clone();
    let (mut block, _) = app.propose(&parent, strict_height).await.unwrap();
    block.extra_data = legacy_proposer_extra_data_bytes(block.proposer_public_key);

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("legacy extra_data must be rejected at/after strict height");

    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("failed to decode block extra_data")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_accepts_block_with_precompile_proxy_transaction() {
    let proxy_address = Address::with_last_byte(0xaa);
    let (tx, recovered) = sample_proxy_precompile_withdraw_tx(proxy_address);
    let (app, db) = setup_app(vec![tx]).await;
    let claimable = U256::from(5_u64);

    {
        let mut db = db.write().unwrap();
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
        let mut proxy_info = revm::state::AccountInfo::default();
        proxy_info.set_code(Bytecode::new_raw(precompile_proxy_runtime_bytecode()));
        db.insert_account(proxy_address, proxy_info);
        let fee_pool_info = revm::state::AccountInfo {
            balance: claimable,
            ..Default::default()
        };
        db.insert_account(FEE_POOL_PRECOMPILE_ADDRESS, fee_pool_info);
        db.insert_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(proxy_address),
            claimable,
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.unwrap();
    let current_balance = db
        .read()
        .unwrap()
        .get_account(proxy_address)
        .unwrap_or_default()
        .balance;
    assert_eq!(current_balance, claimable);

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x77; 32]);
    let pre_db = Arc::new(RwLock::new(pre_state));
    let source = Arc::new(MockTxSource { txs: vec![] });
    let verifier_app = EvmApplication::new(config, pre_db, source);

    assert!(verifier_app.verify(&parent, &block).await.is_ok());
}

#[tokio::test]
async fn verify_rejects_fee_recipient_that_conflicts_with_genesis_mapping() {
    let proposer_public_key = [0x11; 32];
    let expected_fee_recipient = Address::repeat_byte(0x22);
    let mut validator_fee_recipients = BTreeMap::new();
    validator_fee_recipients.insert(proposer_public_key, expected_fee_recipient);

    let chain_spec = Arc::new(build_sahara_chain_spec_with_alloc_and_fee_recipients(
        BTreeMap::new(),
        validator_fee_recipients,
    ));
    let proposer_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key(proposer_public_key);

    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app_with_config(vec![tx], proposer_config).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();
    block.proposer_fee_recipient = Address::repeat_byte(0x77).into_array();

    let verifier_config =
        WhirlpoolEvmConfig::new(chain_spec).with_local_proposer_public_key([0x55; 32]);
    let verifier_app = EvmApplication::new(
        verifier_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    let err = verifier_app
        .verify(&parent, &block)
        .await
        .expect_err("genesis mapping should reject mismatched fee recipient");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(_)),
        "expected invalid block error, got {err:?}"
    );
}

#[tokio::test]
async fn boundary_block_keeps_user_transactions_only() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx.clone()]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let parent = app.genesis().await;
    let (block, result) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");

    assert_eq!(block.transactions, vec![tx]);
    assert_eq!(result.receipt_count, 1);
}

#[tokio::test]
async fn propose_excludes_reserved_epoch_namespace_transaction() {
    let reserved_tx = sample_reserved_epoch_namespace_tx(0, 2_000_000_000);
    let (user_tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![reserved_tx, user_tx.clone()]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let parent = app.genesis().await;
    let (block, result) = app
        .propose(&parent, 1)
        .await
        .expect("propose should skip reserved namespace transaction");

    assert_eq!(block.transactions, vec![user_tx]);
    assert_eq!(result.receipt_count, 1);
    assert_eq!(app.pending_receipts().len(), 1);
}

#[tokio::test]
async fn boundary_block_system_call_advances_epoch_state_once() {
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }
    {
        let db = db.read().unwrap();
        let boundary_state =
            crate::epoch_boundary::load_epoch_boundary_state(&*db).expect("load boundary state");
        assert_eq!(boundary_state.next_epoch_block, 1);
    }

    let parent = app.genesis().await;
    let (boundary_block, _) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");
    {
        let db = db.read().unwrap();
        assert_eq!(
            db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
            U256::from(1_u64)
        );
    }
    let (_next_block, _) = app
        .propose(&boundary_block, 2)
        .await
        .expect("propose non-boundary block");

    let db = db.read().unwrap();
    assert_eq!(
        db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot()),
        U256::from(1_u64)
    );
    assert_eq!(
        db.get_storage(EPOCH_PRECOMPILE_ADDRESS, next_epoch_block_slot()),
        U256::from(1_u64 + EPOCH_BLOCKS_DEFAULT)
    );
}

#[tokio::test]
async fn boundary_unlock_credits_simplex_validator_addresses_and_conserves_balance() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x33; 32],
            ethereum_address: Address::repeat_byte(0x33),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(25_u64),
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, 1);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            unlock_config.genesis_prefund_amount,
            unlock_config.genesis_prefund_amount,
        );
    }

    let parent = app.genesis().await;
    let (_block, _result) = app.propose(&parent, 1).await.expect("boundary propose");

    let db = db.read().unwrap();
    let community_pool_balance = db
        .get_account(COMMUNITY_POOL_ADDRESS)
        .unwrap_or_default()
        .balance;
    let fee_pool_balance = db
        .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .unwrap_or_default()
        .balance;
    let remaining_locked = db.get_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
    );
    let last_processed = db.get_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
    );
    let current_epoch = db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot());

    assert_eq!(current_epoch, U256::from(1_u64));
    assert_eq!(community_pool_balance, U256::from(15_u64));
    assert_eq!(fee_pool_balance, U256::from(10_u64));
    assert_eq!(remaining_locked, U256::from(15_u64));
    assert_eq!(last_processed, U256::from(1_u64));

    let claim0 = db.get_storage(
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[0].ethereum_address),
    );
    let claim1 = db.get_storage(
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[1].ethereum_address),
    );
    let claim2 = db.get_storage(
        FEE_POOL_PRECOMPILE_ADDRESS,
        claimable_balance_slot(validators[2].ethereum_address),
    );
    assert_eq!(claim0, U256::from(4_u64));
    assert_eq!(claim1, U256::from(3_u64));
    assert_eq!(claim2, U256::from(3_u64));
    assert_eq!(claim0 + claim1 + claim2, U256::from(10_u64));
}

#[tokio::test]
async fn boundary_unlock_final_tranche_distributes_top_k_remainder() {
    let validators: Vec<_> = (1_u8..=5_u8)
        .map(|idx| validators::ValidatorEntry {
            consensus_pubkey: [idx; 32],
            ethereum_address: Address::repeat_byte(idx),
        })
        .collect();
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(4_u64),
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, 1);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            unlock_config.genesis_prefund_amount,
            unlock_config.genesis_prefund_amount,
        );
    }

    let parent = app.genesis().await;
    let (_block, _result) = app.propose(&parent, 1).await.expect("boundary propose");

    let db = db.read().unwrap();
    assert_eq!(
        db.get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance,
        U256::ZERO
    );
    assert_eq!(
        db.get_account(FEE_POOL_PRECOMPILE_ADDRESS)
            .unwrap_or_default()
            .balance,
        U256::from(4_u64)
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        U256::ZERO
    );

    for (index, validator) in validators.iter().enumerate() {
        let claim = db.get_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validator.ethereum_address),
        );
        let expected = if index < 4 {
            U256::from(1_u64)
        } else {
            U256::ZERO
        };
        assert_eq!(claim, expected, "validator index {index}");
    }
}

#[tokio::test]
async fn boundary_unlock_skips_non_multiple_epoch() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(25_u64),
        unlock_every_epochs: 2,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, 1);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            unlock_config.genesis_prefund_amount,
            unlock_config.genesis_prefund_amount,
        );
    }

    let parent = app.genesis().await;
    let (_block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose first boundary block");

    let db = db.read().unwrap();
    let community_pool_balance = db
        .get_account(COMMUNITY_POOL_ADDRESS)
        .unwrap_or_default()
        .balance;
    let fee_pool_balance = db
        .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
        .unwrap_or_default()
        .balance;
    let locked_remaining = db.get_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_locked_remaining_slot(),
    );
    let last_processed = db.get_storage(
        COMMUNITY_POOL_ADDRESS,
        community_pool_last_processed_epoch_slot(),
    );
    let current_epoch = db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot());

    assert_eq!(current_epoch, U256::from(1_u64));
    assert_eq!(community_pool_balance, unlock_config.genesis_prefund_amount);
    assert_eq!(fee_pool_balance, U256::ZERO);
    assert_eq!(locked_remaining, unlock_config.genesis_prefund_amount);
    assert_eq!(last_processed, U256::ZERO);

    for validator in &validators {
        assert_eq!(
            db.get_storage(
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(validator.ethereum_address),
            ),
            U256::ZERO
        );
    }
}

#[tokio::test]
async fn boundary_unlock_applies_once_on_matching_epoch() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x11; 32],
            ethereum_address: Address::repeat_byte(0x11),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x22; 32],
            ethereum_address: Address::repeat_byte(0x22),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(25_u64),
        unlock_every_epochs: 2,
        unlock_amount_per_cycle: U256::from(10_u64),
    };
    let (app, db) = setup_app_with_unlock_config(vec![], unlock_config, validators.clone());

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, 1);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            unlock_config.genesis_prefund_amount,
            unlock_config.genesis_prefund_amount,
        );
    }

    let parent = app.genesis().await;
    let (first_block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose first boundary block");
    let (_second_block, _result) = app
        .propose(&first_block, 2)
        .await
        .expect("propose second boundary block");

    {
        let db = db.read().unwrap();
        let current_epoch = db.get_storage(EPOCH_PRECOMPILE_ADDRESS, current_epoch_slot());
        let community_pool_balance = db
            .get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance;
        let fee_pool_balance = db
            .get_account(FEE_POOL_PRECOMPILE_ADDRESS)
            .unwrap_or_default()
            .balance;
        let locked_remaining = db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot(),
        );
        let last_processed = db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot(),
        );
        let claim0 = db.get_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[0].ethereum_address),
        );
        let claim1 = db.get_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[1].ethereum_address),
        );

        assert_eq!(current_epoch, U256::from(2_u64));
        assert_eq!(community_pool_balance, U256::from(15_u64));
        assert_eq!(fee_pool_balance, U256::from(10_u64));
        assert_eq!(locked_remaining, U256::from(15_u64));
        assert_eq!(last_processed, U256::from(2_u64));
        assert_eq!(claim0, U256::from(5_u64));
        assert_eq!(claim1, U256::from(5_u64));
        assert_eq!(claim0 + claim1, U256::from(10_u64));
    }

    let (
        community_pool_before_repeat,
        fee_pool_before_repeat,
        remaining_before_repeat,
        last_processed_before_repeat,
        claim0_before_repeat,
        claim1_before_repeat,
    ) = {
        let db = db.read().unwrap();
        (
            db.get_account(COMMUNITY_POOL_ADDRESS)
                .unwrap_or_default()
                .balance,
            db.get_account(FEE_POOL_PRECOMPILE_ADDRESS)
                .unwrap_or_default()
                .balance,
            db.get_storage(
                COMMUNITY_POOL_ADDRESS,
                community_pool_locked_remaining_slot(),
            ),
            db.get_storage(
                COMMUNITY_POOL_ADDRESS,
                community_pool_last_processed_epoch_slot(),
            ),
            db.get_storage(
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(validators[0].ethereum_address),
            ),
            db.get_storage(
                FEE_POOL_PRECOMPILE_ADDRESS,
                claimable_balance_slot(validators[1].ethereum_address),
            ),
        )
    };

    {
        let mut db = db.write().unwrap();
        maybe_apply_community_pool_unlock(&mut *db, true, &validators)
            .expect("same-epoch unlock invocation must no-op");
    }

    let db = db.read().unwrap();
    assert_eq!(
        db.get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance,
        community_pool_before_repeat
    );
    assert_eq!(
        db.get_account(FEE_POOL_PRECOMPILE_ADDRESS)
            .unwrap_or_default()
            .balance,
        fee_pool_before_repeat
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        remaining_before_repeat
    );
    assert_eq!(
        db.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot()
        ),
        last_processed_before_repeat
    );
    assert_eq!(
        db.get_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[0].ethereum_address),
        ),
        claim0_before_repeat
    );
    assert_eq!(
        db.get_storage(
            FEE_POOL_PRECOMPILE_ADDRESS,
            claimable_balance_slot(validators[1].ethereum_address),
        ),
        claim1_before_repeat
    );
}

#[tokio::test]
async fn verify_boundary_unlock_matches_propose_state() {
    let validators = vec![
        validators::ValidatorEntry {
            consensus_pubkey: [0x01; 32],
            ethereum_address: Address::repeat_byte(0x01),
        },
        validators::ValidatorEntry {
            consensus_pubkey: [0x02; 32],
            ethereum_address: Address::repeat_byte(0x02),
        },
    ];
    let unlock_config = CommunityPoolUnlockConfig {
        genesis_prefund_amount: U256::from(11_u64),
        unlock_every_epochs: 1,
        unlock_amount_per_cycle: U256::from(5_u64),
    };
    let chain_spec = Arc::new(
            build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators_and_community_pool_unlock_config(
                BTreeMap::new(),
                BTreeMap::new(),
                validators.clone(),
                unlock_config,
            ),
        );
    let (app, db) =
        setup_app_with_config(vec![], WhirlpoolEvmConfig::new(chain_spec.clone())).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, 1);
        seed_community_pool_unlock_state(
            &mut db,
            unlock_config.unlock_every_epochs,
            unlock_config.unlock_amount_per_cycle,
            unlock_config.genesis_prefund_amount,
            unlock_config.genesis_prefund_amount,
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _result) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");

    let proposer_state = db.read().unwrap().clone();
    let proposer_state_root = proposer_state.state_root().0;
    let verifier_db = Arc::new(RwLock::new(pre_state));
    let verifier_app = EvmApplication::new(
        WhirlpoolEvmConfig::new(chain_spec),
        verifier_db.clone(),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let verify_result = verifier_app
        .verify(&parent, &block)
        .await
        .expect("verify boundary block with unlock");

    // verify() computes against an ephemeral clone and checks the computed state root
    // against the block; it does not mutate the application's backing DB.
    assert_eq!(verify_result.state_root, block.state_root);
    assert_eq!(verify_result.receipts_root, block.receipts_root);
    assert_eq!(proposer_state_root, block.state_root);

    let verifier_state = verifier_db.read().unwrap();
    assert_eq!(
        verifier_state
            .get_account(COMMUNITY_POOL_ADDRESS)
            .unwrap_or_default()
            .balance,
        unlock_config.genesis_prefund_amount
    );
    assert_eq!(
        verifier_state.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_locked_remaining_slot()
        ),
        unlock_config.genesis_prefund_amount
    );
    assert_eq!(
        verifier_state.get_storage(
            COMMUNITY_POOL_ADDRESS,
            community_pool_last_processed_epoch_slot()
        ),
        U256::ZERO
    );
}

#[tokio::test]
async fn boundary_block_receipts_and_gas_are_user_only() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let parent = app.genesis().await;
    let (block, result) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");
    let receipts = app.pending_receipts();

    assert_eq!(block.transactions.len(), 1);
    assert_eq!(receipts.len(), 1);
    assert_eq!(result.receipt_count, 1);
    assert_eq!(
        block.gas_used,
        receipts
            .last()
            .expect("must have receipt")
            .cumulative_gas_used
    );
}

#[tokio::test]
async fn verify_accepts_boundary_block_with_user_only_transactions() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("propose boundary block");

    let verifier = EvmApplication::new(
        WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    assert!(verifier.verify(&parent, &block).await.is_ok());
}

#[tokio::test]
async fn verify_rejects_reserved_epoch_namespace_transaction() {
    let reserved_tx = sample_reserved_epoch_namespace_tx(0, 2_000_000_000);
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 10, EPOCH_BLOCKS_DEFAULT);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let block = EvmBlock {
        height: 1,
        parent_id: parent.compute_id(),
        state_root: parent.state_root,
        transactions_root: ordered_trie_root_with_encoder(
            std::slice::from_ref(&reserved_tx),
            |tx, out| out.put_slice(tx),
        )
        .0,
        receipts_root: EMPTY_ROOT_HASH.0,
        proposer_public_key: parent.proposer_public_key,
        proposer_fee_recipient: parent.proposer_fee_recipient,
        extra_data: parent.extra_data.clone(),
        gas_used: 0,
        base_fee_per_gas: parent.base_fee_per_gas,
        timestamp: parent.timestamp + 12,
        transactions: vec![reserved_tx],
    };

    let verifier = EvmApplication::new(
        WhirlpoolEvmConfig::new(Arc::new(build_sahara_chain_spec())),
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );

    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("reserved epoch namespace tx must be invalid");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
    assert!(
        err.to_string()
            .contains("reserved epoch boundary namespace transaction"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn propose_rejects_when_required_boundary_system_call_fails() {
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            current_epoch_slot(),
            U256::from(u64::MAX),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            epoch_blocks_slot(),
            U256::from(EPOCH_BLOCKS_DEFAULT),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(1_u64),
        );
    }

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("boundary system call failure must fail proposal");
    assert!(matches!(err, EvmAppError::Execution(_)));
}

#[tokio::test]
async fn verify_rejects_when_required_boundary_system_call_fails() {
    let (app, db) = setup_app(vec![]).await;

    {
        let mut db = db.write().unwrap();
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            current_epoch_slot(),
            U256::from(u64::MAX),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            epoch_blocks_slot(),
            U256::from(EPOCH_BLOCKS_DEFAULT),
        );
        db.insert_storage(
            EPOCH_PRECOMPILE_ADDRESS,
            next_epoch_block_slot(),
            U256::from(1_u64),
        );
    }

    let parent = app.genesis().await;
    let boundary_block = EvmBlock {
        height: 1,
        parent_id: parent.compute_id(),
        state_root: parent.state_root,
        transactions_root: EMPTY_ROOT_HASH.0,
        receipts_root: EMPTY_ROOT_HASH.0,
        proposer_public_key: parent.proposer_public_key,
        proposer_fee_recipient: parent.proposer_fee_recipient,
        extra_data: parent.extra_data.clone(),
        gas_used: 0,
        base_fee_per_gas: parent.base_fee_per_gas,
        timestamp: parent.timestamp + 12,
        transactions: vec![],
    };

    let err = app
        .verify(&parent, &boundary_block)
        .await
        .expect_err("boundary system call must fail verification");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[tokio::test]
async fn propose_cache_isolated_for_same_height_different_parent() {
    let (app, _db) = setup_app(vec![]).await;
    let parent = app.genesis().await;
    let (first_block, _) = app.propose(&parent, 1).await.expect("first propose");

    let mut alternate_parent = parent.clone();
    alternate_parent.state_root[0] ^= 0x01;
    let (second_block, _) = app
        .propose(&alternate_parent, 1)
        .await
        .expect("second propose with alternate parent");

    assert_eq!(first_block.parent_id, parent.compute_id());
    assert_eq!(second_block.parent_id, alternate_parent.compute_id());
    assert_ne!(first_block.parent_id, second_block.parent_id);
}

#[tokio::test]
async fn verify_rejects_parent_id_mismatch() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("propose block");
    let mut wrong_parent = parent.clone();
    wrong_parent.state_root[0] ^= 0x01;

    let err = app
        .verify(&wrong_parent, &block)
        .await
        .expect_err("verify must reject mismatched parent id");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
}

#[tokio::test]
async fn store_finalized_block_retains_receipts_when_store_fails() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("propose block");

    struct FailingBlockStorage;
    impl BlockStorage for FailingBlockStorage {
        fn store_block(
            &self,
            _block: &EvmBlock,
            _receipts: &[Receipt],
        ) -> Result<(), state::BlockStorageError> {
            Err(state::BlockStorageError::Database(
                "injected persistence failure".into(),
            ))
        }

        fn get_block_by_number(
            &self,
            _number: u64,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(
            &self,
            _hash: B256,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
            Ok(None)
        }
    }

    let err = app
        .store_finalized_block(&block, &FailingBlockStorage)
        .expect_err("finalize persistence failure should return error");
    assert!(matches!(err, EvmAppError::State(_)));
    assert_eq!(app.pending_receipts().len(), 1);
    assert!(app
        .staged_receipts
        .lock()
        .unwrap()
        .contains_key(&block.compute_id()));
}

#[tokio::test]
async fn store_finalized_block_rejects_receipts_for_mismatched_cached_block() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("propose block");
    let staged_block_id = block.compute_id();
    let mut mismatched_block = block.clone();
    mismatched_block.parent_id[0] ^= 0x01;

    #[derive(Default)]
    struct CountingStorage {
        calls: Mutex<usize>,
    }

    impl BlockStorage for CountingStorage {
        fn store_block(
            &self,
            _block: &EvmBlock,
            _receipts: &[Receipt],
        ) -> Result<(), state::BlockStorageError> {
            let mut calls = self.calls.lock().unwrap();
            *calls += 1;
            Ok(())
        }

        fn get_block_by_number(
            &self,
            _number: u64,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(
            &self,
            _hash: B256,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
            Ok(None)
        }
    }

    let storage = CountingStorage::default();
    let err = app
        .store_finalized_block(&mismatched_block, &storage)
        .expect_err("mismatched staged receipts must be rejected");
    assert!(matches!(err, EvmAppError::InvalidBlock(_)));
    assert_eq!(*storage.calls.lock().unwrap(), 0);
    assert!(app
        .staged_receipts
        .lock()
        .unwrap()
        .contains_key(&staged_block_id));
}

#[tokio::test]
async fn store_finalized_block_stores_and_clears_receipts() {
    let (tx, recovered) = sample_evm_tx();
    let (app, db) = setup_app(vec![tx]).await;

    {
        let mut db = db.write().unwrap();
        let info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(recovered, info);
    }

    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.unwrap();

    #[derive(Default)]
    struct MockBlockStorage {
        stored: Mutex<Vec<(EvmBlock, Vec<Receipt>)>>,
    }

    impl BlockStorage for MockBlockStorage {
        fn store_block(
            &self,
            block: &EvmBlock,
            receipts: &[Receipt],
        ) -> Result<(), state::BlockStorageError> {
            self.stored
                .lock()
                .unwrap()
                .push((block.clone(), receipts.to_vec()));
            Ok(())
        }

        fn get_block_by_number(
            &self,
            _number: u64,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_block_by_hash(
            &self,
            _hash: B256,
        ) -> Result<Option<EvmBlock>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, state::BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, state::BlockStorageError> {
            Ok(None)
        }
    }

    let storage = MockBlockStorage::default();
    app.store_finalized_block(&block, &storage).unwrap();

    let stored = storage.stored.lock().unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].0.height, 1);
    assert_eq!(stored[0].1.len(), 1);
    assert!(app.pending_receipts().is_empty());
    assert!(app.staged_receipts.lock().unwrap().is_empty());
}

#[test]
fn latest_committed_full_dkg_scans_backwards_past_raw_eth_only_blocks() {
    use state::BlockStorageError;

    #[derive(Default)]
    struct MockStorage {
        blocks: BTreeMap<u64, EvmBlock>,
    }

    impl BlockStorage for MockStorage {
        fn store_block(
            &self,
            _block: &EvmBlock,
            _receipts: &[Receipt],
        ) -> Result<(), BlockStorageError> {
            Ok(())
        }

        fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, BlockStorageError> {
            Ok(self.blocks.get(&number).cloned())
        }

        fn get_block_by_hash(&self, _hash: B256) -> Result<Option<EvmBlock>, BlockStorageError> {
            Ok(None)
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError> {
            Ok(self.blocks.keys().next_back().cloned())
        }
    }

    fn block_with_extra_data(height: u64, extra_data: Vec<u8>) -> EvmBlock {
        EvmBlock {
            height,
            parent_id: [0u8; 32],
            state_root: [0u8; 32],
            transactions_root: [0u8; 32],
            receipts_root: [0u8; 32],
            proposer_public_key: [0x11; 32],
            proposer_fee_recipient: [0x22; 20],
            extra_data,
            gas_used: 0,
            base_fee_per_gas: 1_000_000_000,
            timestamp: height * 12,
            transactions: vec![],
        }
    }

    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let players = config.simplex_consensus_public_keys();
    let full_dkg = FullDkgV1 {
        epoch: 1,
        output: app::FullDkgOutputV1 {
            dealers: players.clone(),
            players: players.clone(),
            public_polynomial: vec![1, 2, 3, 4],
        },
    };

    let extra_with_full_dkg = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: Some(full_dkg.clone()),
        reshare: None,
    })
    .expect("encode full_dkg");
    let extra_raw_only = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(vec![0x11; 32]),
        full_dkg: None,
        reshare: None,
    })
    .expect("encode raw only");

    let mut storage = MockStorage::default();
    storage
        .blocks
        .insert(0, block_with_extra_data(0, extra_with_full_dkg));
    storage
        .blocks
        .insert(1, block_with_extra_data(1, extra_raw_only.clone()));
    storage
        .blocks
        .insert(2, block_with_extra_data(2, extra_raw_only));

    let resolved = latest_committed_full_dkg(&storage, 2)
        .expect("scan should succeed")
        .expect("full_dkg should resolve from earlier block");
    assert_eq!(resolved, full_dkg);

    assert!(
        !full_dkg_should_be_included(&config, Some(&resolved), &full_dkg),
        "unchanged baseline must not force redundant FullDkg inclusion"
    );
}

#[test]
fn full_dkg_trigger_includes_when_only_dealers_change() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let players = config.simplex_consensus_public_keys();

    let previous = FullDkgV1 {
        epoch: 3,
        output: app::FullDkgOutputV1 {
            dealers: vec![[0x11; 32]],
            players: players.clone(),
            public_polynomial: vec![0xaa, 0xbb],
        },
    };
    let candidate = FullDkgV1 {
        epoch: 3,
        output: app::FullDkgOutputV1 {
            dealers: vec![[0x22; 32]],
            players,
            public_polynomial: vec![0xaa, 0xbb],
        },
    };

    assert!(
        full_dkg_should_be_included(&config, Some(&previous), &candidate),
        "dealer-only changes must trigger FullDkg inclusion"
    );
}

#[tokio::test]
async fn verify_rejects_full_dkg_payload_mismatch_against_candidate() {
    let (tx, recovered) = sample_evm_tx();
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone());
    let players = base_config.simplex_consensus_public_keys();
    let candidate_output = app::FullDkgOutputV1 {
        dealers: players.clone(),
        players: players.clone(),
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    };
    let proposer_config = base_config
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0)
        .with_current_full_dkg_output(candidate_output.clone());
    let (app, db) = setup_app_with_config(vec![tx], proposer_config.clone()).await;

    {
        let mut db = db.write().unwrap();
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();

    let mut decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data must decode");
    decoded
        .full_dkg
        .as_mut()
        .expect("proposed block should include full_dkg")
        .output
        .public_polynomial
        .push(0xff);
    block.extra_data =
        encode_canonical_extra_data(&decoded).expect("mutated canonical extra_data encodes");

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("mismatched full_dkg payload must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg payload mismatch")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_rejects_full_dkg_when_candidate_is_not_configured() {
    let (tx, recovered) = sample_evm_tx();
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let (app, db) = setup_app_with_config(vec![tx], config.clone()).await;

    {
        let mut db = db.write().unwrap();
        db.insert_account(
            recovered,
            revm::state::AccountInfo {
                balance: U256::from(1_000_000_000_000_000_000u64),
                nonce: 0,
                ..Default::default()
            },
        );
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app.propose(&parent, 1).await.unwrap();

    let players = config.simplex_consensus_public_keys();
    block.extra_data = encode_canonical_extra_data(&CanonicalExtraDataV1 {
        raw_eth: Some(block.proposer_public_key.to_vec()),
        full_dkg: Some(FullDkgV1 {
            epoch: 0,
            output: app::FullDkgOutputV1 {
                dealers: players.clone(),
                players,
                public_polynomial: vec![0x01],
            },
        }),
        reshare: None,
    })
    .expect("canonical extra_data with full_dkg");

    let verifier = EvmApplication::new(
        config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("unexpected full_dkg without configured candidate must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("must be omitted when no full_dkg candidate is configured")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn propose_rejects_non_boundary_full_dkg_players_mismatch_with_activation_schedule() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let candidate_players = base_config.simplex_consensus_public_keys();
    let proposer_config = base_config
        .with_current_full_dkg_output(app::FullDkgOutputV1 {
            dealers: candidate_players.clone(),
            players: candidate_players,
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(0, vec![[0x41; 32], [0x42; 32]]);
    let (app, _db) = setup_app_with_config(vec![], proposer_config).await;

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("non-boundary propose must fail-closed when activation players mismatch");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg output.players does not match activation-resolved player set")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_rejects_non_boundary_full_dkg_players_mismatch_with_activation_schedule() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let candidate_players = base_config.simplex_consensus_public_keys();
    let candidate_output = app::FullDkgOutputV1 {
        dealers: candidate_players.clone(),
        players: candidate_players,
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    };
    let proposer_config = base_config
        .clone()
        .with_current_full_dkg_output(candidate_output.clone());
    let (app, db) = setup_app_with_config(vec![], proposer_config).await;

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app.propose(&parent, 1).await.expect("non-boundary propose");

    let verifier_config = base_config
        .with_current_full_dkg_output(candidate_output)
        .with_activation_players_for_epoch(0, vec![[0x41; 32], [0x42; 32]]);
    let verifier = EvmApplication::new(
        verifier_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("non-boundary verify must fail-closed when activation players mismatch");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("full_dkg output.players does not match activation-resolved player set")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn propose_boundary_block_emits_forward_full_dkg_and_reshare_sections_when_candidate_configured(
) {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
    let proposer_config = base_config.with_current_full_dkg_output(app::FullDkgOutputV1 {
        dealers: players.clone(),
        players: players.clone(),
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    });
    let (app, db) = setup_app_with_config(vec![], proposer_config).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let parent = app.genesis().await;
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("boundary block should propose");
    let decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data should decode");

    let full_dkg = decoded
        .full_dkg
        .as_ref()
        .expect("boundary block should include full_dkg");
    assert_eq!(full_dkg.epoch, 2, "boundary full_dkg must target epoch E+1");
    assert_eq!(full_dkg.output.players, players);

    let reshare = decoded
        .reshare
        .as_ref()
        .expect("boundary block should include reshare");
    assert_eq!(
        reshare.target_epoch, 3,
        "boundary reshare must target epoch E+2"
    );
    assert_eq!(reshare.players, full_dkg.output.players);
}

#[tokio::test]
async fn verify_rejects_missing_reshare_section_on_boundary_when_candidate_configured() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
    let proposer_config = base_config.with_current_full_dkg_output(app::FullDkgOutputV1 {
        dealers: players.clone(),
        players: players.clone(),
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    });
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app
        .propose(&parent, 1)
        .await
        .expect("boundary block should propose");

    let mut decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data must decode");
    decoded.reshare = None;
    block.extra_data =
        encode_canonical_extra_data(&decoded).expect("mutated canonical extra_data encodes");

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("missing boundary reshare must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("reshare section must be present for boundary block")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn verify_rejects_reshare_section_on_non_boundary_block() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let players = base_config.simplex_consensus_public_keys();
    let proposer_config = base_config.with_current_full_dkg_output(app::FullDkgOutputV1 {
        dealers: players.clone(),
        players: players.clone(),
        public_polynomial: vec![0xaa, 0xbb, 0xcc],
    });
    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (mut block, _) = app
        .propose(&parent, 1)
        .await
        .expect("non-boundary block should propose");

    let mut decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data must decode");
    decoded.reshare = Some(app::ReshareV1 {
        target_epoch: 1,
        players: players.clone(),
    });
    block.extra_data =
        encode_canonical_extra_data(&decoded).expect("mutated canonical extra_data encodes");

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    let err = verifier
        .verify(&parent, &block)
        .await
        .expect_err("reshare on non-boundary block must be rejected");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("reshare section is forbidden on non-boundary blocks")),
        "unexpected error: {err:?}"
    );
}

#[tokio::test]
async fn boundary_reshare_can_follow_epoch_pipeline_lag_from_activation_schedule() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);

    let next_epoch_players = base_config.simplex_consensus_public_keys();
    let next_next_epoch_players = vec![[0x41; 32], [0x42; 32]];
    let proposer_config = base_config
        .with_current_full_dkg_output(app::FullDkgOutputV1 {
            dealers: next_epoch_players.clone(),
            players: next_epoch_players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(2, next_epoch_players.clone())
        .with_activation_players_for_epoch(3, next_next_epoch_players.clone());

    let (app, db) = setup_app_with_config(vec![], proposer_config.clone()).await;

    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let pre_state = db.read().unwrap().clone();
    let parent = app.genesis().await;
    let (block, _) = app
        .propose(&parent, 1)
        .await
        .expect("boundary block should propose");
    let decoded = decode_extra_data(&block.extra_data, ExtraDataDecodeMode::Strict)
        .expect("canonical extra_data should decode");
    let full_dkg = decoded
        .full_dkg
        .as_ref()
        .expect("boundary block should include full_dkg");
    let reshare = decoded
        .reshare
        .as_ref()
        .expect("boundary block should include reshare");

    assert_eq!(
        full_dkg.output.players, next_epoch_players,
        "full_dkg players should match next-epoch activation set"
    );
    assert_eq!(
        reshare.players, next_next_epoch_players,
        "reshare players should match next-next-epoch activation set"
    );

    let verifier = EvmApplication::new(
        proposer_config,
        Arc::new(RwLock::new(pre_state)),
        Arc::new(MockTxSource { txs: vec![] }),
    );
    verifier
        .verify(&parent, &block)
        .await
        .expect("boundary verify should accept activation pipeline-lag schedule");
}

#[tokio::test]
async fn propose_rejects_boundary_when_activation_schedule_missing_reshare_epoch() {
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let base_config = WhirlpoolEvmConfig::new(chain_spec.clone())
        .with_local_proposer_public_key([0x77; 32])
        .with_full_dkg_strict_height(0);
    let next_epoch_players = base_config.simplex_consensus_public_keys();
    let proposer_config = base_config
        .with_current_full_dkg_output(app::FullDkgOutputV1 {
            dealers: next_epoch_players.clone(),
            players: next_epoch_players.clone(),
            public_polynomial: vec![0xaa, 0xbb, 0xcc],
        })
        .with_activation_players_for_epoch(2, next_epoch_players);

    let (app, db) = setup_app_with_config(vec![], proposer_config).await;
    {
        let mut db = db.write().unwrap();
        seed_epoch_boundary_state(&mut db, 1, EPOCH_BLOCKS_DEFAULT);
    }

    let parent = app.genesis().await;
    let err = app
        .propose(&parent, 1)
        .await
        .expect_err("boundary propose should fail-closed when reshare epoch data is missing");
    assert!(
        matches!(err, EvmAppError::InvalidBlock(ref msg) if msg.contains("activation resolver missing player set for epoch 3")),
        "unexpected error: {err:?}"
    );
}
