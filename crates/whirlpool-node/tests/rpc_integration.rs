//! Integration tests for the Ethereum JSON-RPC server.
//!
//! These tests start a real RPC server and interact with it using alloy.

use alloy_primitives::{Address, U256};
use alloy_provider::{Provider, ProviderBuilder};
use state_memory::InMemoryStateDb;
use std::sync::{Arc, RwLock};
use whirlpool_node::rpc;

/// Spin up an RPC server with a fresh in-memory state and return the provider URL.
async fn start_test_rpc() -> (
    String,
    jsonrpsee::server::ServerHandle,
    Arc<RwLock<InMemoryStateDb>>,
    Arc<app::tx_source::InMemoryTxPool>,
) {
    use jsonrpsee::server::ServerBuilder;
    use whirlpool_node::rpc::eth_api::EthApiServer;
    use whirlpool_node::rpc::eth_handler::EthApiHandler;

    let pool = Arc::new(app::tx_source::InMemoryTxPool::new());
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let ctx = rpc::context::EthRpcContext::new(pool.clone(), state_db.clone(), 313_371);
    let handler = EthApiHandler::new(ctx);

    let server = ServerBuilder::default()
        .build("127.0.0.1:0")
        .await
        .expect("failed to build RPC server");
    let addr = server.local_addr().expect("failed to get local addr");
    let handle = server.start(handler.into_rpc());

    let url = format!("http://{addr}");
    (url, handle, state_db, pool)
}

#[tokio::test]
async fn test_eth_chain_id() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());
    let chain_id = provider.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 313_371);
}

#[tokio::test]
async fn test_eth_gas_price() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());
    let gas_price = provider.get_gas_price().await.unwrap();
    assert_eq!(gas_price, 1_000_000_000); // 1 gwei
}

#[tokio::test]
async fn test_eth_get_balance_zero_for_unknown() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());
    let balance = provider.get_balance(Address::ZERO).await.unwrap();
    assert_eq!(balance, U256::ZERO);
}

#[tokio::test]
async fn test_eth_get_balance_with_funded_account() {
    let (url, _handle, state, _pool) = start_test_rpc().await;

    let addr = Address::repeat_byte(0x42);
    {
        let mut db = state.write().unwrap();
        db.insert_account(
            addr,
            revm::state::AccountInfo {
                balance: U256::from(5_000_000_000_000_000_000u128), // 5 ETH
                nonce: 0,
                code_hash: alloy_primitives::B256::ZERO,
                code: None,
                ..Default::default()
            },
        );
    }

    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());
    let balance = provider.get_balance(addr).await.unwrap();
    assert_eq!(balance, U256::from(5_000_000_000_000_000_000u128));
}

#[tokio::test]
async fn test_eth_get_transaction_count() {
    let (url, _handle, state, _pool) = start_test_rpc().await;

    let addr = Address::repeat_byte(0x11);
    {
        let mut db = state.write().unwrap();
        db.insert_account(
            addr,
            revm::state::AccountInfo {
                balance: U256::ZERO,
                nonce: 7,
                code_hash: alloy_primitives::B256::ZERO,
                code: None,
                ..Default::default()
            },
        );
    }

    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());
    let nonce = provider.get_transaction_count(addr).await.unwrap();
    assert_eq!(nonce, 7);
}

#[tokio::test]
async fn test_eth_estimate_gas() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());

    use alloy_rpc_types::TransactionRequest;
    let tx = TransactionRequest::default()
        .to(Address::repeat_byte(0x01))
        .value(U256::from(1000u64));

    let gas = provider.estimate_gas(tx).await.unwrap();
    assert_eq!(gas, 21_000);
}

#[tokio::test]
async fn test_eth_get_transaction_receipt_not_found() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new()
        .connect_http(url.parse().unwrap());

    let receipt = provider
        .get_transaction_receipt(alloy_primitives::B256::ZERO)
        .await
        .unwrap();
    assert!(receipt.is_none());
}
