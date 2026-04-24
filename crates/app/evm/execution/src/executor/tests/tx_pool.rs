use std::sync::{Arc, RwLock};

use super::*;
use app::{traits::Application, InMemoryTxPool};

#[tokio::test]
async fn test_propose_with_in_memory_pool() {
    let (encoded_tx, alice_addr) = sample_evm_tx();
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let config = WhirlpoolEvmConfig::new(chain_spec);
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));

    {
        let mut db = state_db.write().unwrap();
        let account_info = revm::state::AccountInfo {
            balance: U256::from(1_000_000_000_000_000_000u64),
            nonce: 0,
            ..Default::default()
        };
        db.insert_account(alice_addr, account_info);
    }

    let tx_pool = Arc::new(InMemoryTxPool::new());
    tx_pool.push(encoded_tx);

    let app = EvmApplication::new(config, state_db, tx_pool.clone());
    let genesis = app.genesis().await;
    let (block, execution_result) = app
        .propose(&genesis, 1)
        .await
        .expect("Propose should succeed");

    assert_eq!(block.height, 1);
    assert_eq!(block.transactions.len(), 1);
    assert!(block.gas_used > 0);
    assert_eq!(block.gas_used, execution_result.gas_used);
    assert!(
        tx_pool.pending().is_empty(),
        "Pool should be empty after propose"
    );
}
