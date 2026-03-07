use std::sync::{Arc, RwLock};

use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, Signature, TxKind, U256};
use app::{
    traits::{Application, TxSource},
    InMemoryTxPool,
};
use app_evm::executor::EvmApplication;
use app_evm::WhirlpoolEvmConfig;
use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::SignerRecoverable;
use state_memory::InMemoryStateDb;

/// Simple MockTxSource for integration tests
struct MockTxSource {
    txs: Vec<Vec<u8>>,
}

impl TxSource for MockTxSource {
    fn push(&self, _tx: Vec<u8>) {
        // MockTxSource is pre-loaded, ignores push.
    }

    fn pending(&self) -> Vec<Vec<u8>> {
        self.txs.clone()
    }
}

#[tokio::test]
async fn test_full_propose_verify_cycle() {
    // 1. Setup Environment
    let chain_spec = Arc::new(app_evm::build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec.clone());

    // Initial state DB
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));

    // 2. Create a valid transaction (Alice -> Bob)
    let alice_sk = Signature::test_signature(); // Using test signature/signer
    let bob_addr = Address::with_last_byte(2);

    let tx = TxLegacy {
        chain_id: Some(app_evm::config::SAHARA_CHAIN_ID),
        nonce: 0,
        gas_price: 10,
        gas_limit: 21_000,
        to: TxKind::Call(bob_addr),
        value: U256::from(1000),
        input: Bytes::default(),
    };

    let signed: TransactionSigned = tx.into_signed(alice_sk).into();
    let alice_addr = signed.recover_signer().expect("Should recover signer");

    let mut encoded_tx = Vec::new();
    signed.encode_2718(&mut encoded_tx);

    // 3. Fund Alice in the state
    {
        let mut db = state_db.write().unwrap();
        let account_info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(alice_addr, account_info);
    }

    // Snapshot pre-state for verification later
    let pre_state_snapshot = state_db.read().unwrap().clone();

    // 4. Initialize Proposer App
    let tx_source = Arc::new(MockTxSource {
        txs: vec![encoded_tx],
    });
    let proposer_app = EvmApplication::new(config.clone(), state_db.clone(), tx_source);

    // 5. Propose Block
    let genesis = proposer_app.genesis().await;
    let (block, execution_result) = proposer_app
        .propose(&genesis, 1)
        .await
        .expect("Propose should succeed");

    // Assertions on Block
    assert_eq!(block.height, 1);
    assert_eq!(block.transactions.len(), 1);
    assert!(block.gas_used > 0);
    assert_eq!(block.gas_used, execution_result.gas_used);

    // 6. Verify Block using a FRESH app instance with the SAME pre-state
    // This simulates a validator receiving the block and verifying it against their local state (which matches parent)
    let validator_db = Arc::new(RwLock::new(pre_state_snapshot));
    // TxSource is irrelevant for verification as txs are in the block
    let empty_source = Arc::new(MockTxSource { txs: vec![] });

    let validator_app = EvmApplication::new(config, validator_db.clone(), empty_source);

    let verify_result = validator_app.verify(&genesis, &block).await;
    assert!(
        verify_result.is_ok(),
        "Verification failed: {:?}",
        verify_result.err()
    );

    // 7. Check Post-Verification State (optional, if verify applies changes? usually verify is stateless or updates canonical)
    // In this implementation, `verify` updates the state_db provided to it if successful?
    // Let's check the source code of `verify`.
    // Wait, `verify` in `executor.rs` DOES NOT commit to the underlying DB by default unless `Application` trait mandates it?
    // Looking at `executor.rs`:
    // It clones the state: `let mut exec_state = self.state_db.read().unwrap().clone();`
    // It commits to the *cloned* state: `exec_state.commit(&bundle);`
    // It does NOT update `self.state_db`.
    // So `validator_db` should remain unchanged in this specific implementation of `verify`.
    // Ideally, `verify` is for checking validity. Applying the block is a separate step (e.g. `commit_block`).
    // But `EvmApplication` struct doesn't have a `commit_block` method in the snippets I saw.
    // The `Application` trait might have `commit` or similar?
    // Re-reading `verify` logic: it returns `Result<ExecutionResult, ...>`.

    // For this test, ensuring `verify` returns Ok is sufficient to prove the round-trip works.
}

#[tokio::test]
async fn test_propose_with_in_memory_pool() {
    // 1. Setup Environment
    let chain_spec = Arc::new(app_evm::build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec.clone());
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));

    // 2. Create a valid transaction (Alice -> Bob)
    let alice_sk = Signature::test_signature();
    let bob_addr = Address::with_last_byte(2);

    let tx = TxLegacy {
        chain_id: Some(app_evm::config::SAHARA_CHAIN_ID),
        nonce: 0,
        gas_price: 10,
        gas_limit: 21_000,
        to: TxKind::Call(bob_addr),
        value: U256::from(1000),
        input: Bytes::default(),
    };

    let signed: TransactionSigned = tx.into_signed(alice_sk).into();
    let alice_addr = signed.recover_signer().expect("Should recover signer");

    let mut encoded_tx = Vec::new();
    signed.encode_2718(&mut encoded_tx);

    // 3. Fund Alice in the state
    {
        let mut db = state_db.write().unwrap();
        let account_info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(alice_addr, account_info);
    }

    // 4. Push tx into InMemoryTxPool (the real implementation)
    let tx_pool = Arc::new(InMemoryTxPool::new());
    tx_pool.push(encoded_tx);

    // 5. Create app with InMemoryTxPool and propose
    let app = EvmApplication::new(config, state_db.clone(), tx_pool.clone());
    let genesis = app.genesis().await;
    let (block, execution_result) = app
        .propose(&genesis, 1)
        .await
        .expect("Propose should succeed");

    // 6. Assertions
    assert_eq!(block.height, 1);
    assert_eq!(block.transactions.len(), 1);
    assert!(block.gas_used > 0);
    assert_eq!(block.gas_used, execution_result.gas_used);

    // 7. Pool should be drained after propose
    assert!(
        tx_pool.pending().is_empty(),
        "Pool should be empty after propose"
    );
}
