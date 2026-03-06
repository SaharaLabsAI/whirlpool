//! Integration tests for the Ethereum JSON-RPC server.
//!
//! These tests start a real RPC server and interact with it using alloy.

use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use app::traits::TxSource;
use state_memory::InMemoryStateDb;
use std::sync::{Arc, RwLock};
/// Spin up an RPC server with a fresh in-memory state and return the provider URL.
async fn start_test_rpc() -> (
    String,
    jsonrpsee::server::ServerHandle,
    Arc<RwLock<InMemoryStateDb>>,
    Arc<app::tx_source::InMemoryTxPool>,
) {
    use jsonrpsee::server::ServerBuilder;
    use rpc_eth::eth_api::EthApiServer;
    use rpc_eth::eth_handler::EthApiHandler;

    let pool = Arc::new(app::tx_source::InMemoryTxPool::new());
    let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
    let ctx = rpc_eth::context::EthRpcContext::new(pool.clone(), state_db.clone(), 313_371);
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
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let chain_id = provider.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 313_371);
}

#[tokio::test]
async fn test_eth_gas_price() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let gas_price = provider.get_gas_price().await.unwrap();
    assert_eq!(gas_price, 1_000_000_000); // 1 gwei
}

#[tokio::test]
async fn test_eth_get_balance_zero_for_unknown() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
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

    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
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

    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let nonce = provider.get_transaction_count(addr).await.unwrap();
    assert_eq!(nonce, 7);
}

#[tokio::test]
async fn test_eth_estimate_gas() {
    let (url, _handle, _state, _pool) = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());

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
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());

    let receipt = provider
        .get_transaction_receipt(alloy_primitives::B256::ZERO)
        .await
        .unwrap();
    assert!(receipt.is_none());
}

#[tokio::test]
async fn test_eth_send_raw_transaction_transfer() {
    let (url, _handle, state, pool) = start_test_rpc().await;

    // Set up a funded sender account.
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse()
            .unwrap();
    let sender = signer.address();
    let recipient = Address::repeat_byte(0xBB);
    let send_value = U256::from(1_000_000_000_000_000_000u128); // 1 ETH

    // Fund the sender with enough balance to cover value + gas.
    {
        let mut db = state.write().unwrap();
        db.insert_account(
            sender,
            revm::state::AccountInfo {
                balance: U256::from(10_000_000_000_000_000_000u128), // 10 ETH
                nonce: 0,
                code_hash: alloy_primitives::B256::ZERO,
                code: None,
                ..Default::default()
            },
        );
    }

    // Build a legacy transfer transaction.
    let tx = TxLegacy {
        chain_id: Some(313_371),
        nonce: 0,
        gas_price: 1_000_000_000, // 1 gwei (matches gas_price RPC)
        gas_limit: 21_000,
        to: TxKind::Call(recipient),
        value: send_value,
        input: Bytes::default(),
    };

    // Sign the transaction.
    let sig = signer.sign_hash_sync(&tx.signature_hash()).unwrap();
    let signed = tx.into_signed(sig);

    // RLP-encode the signed transaction (EIP-2718 envelope).
    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);

    // Compute the expected tx hash (keccak256 of the raw bytes).
    let expected_hash = alloy_primitives::keccak256(&encoded);

    // Send via RPC.
    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let tx_hash = provider.send_raw_transaction(&encoded).await.unwrap();
    assert_eq!(*tx_hash.tx_hash(), expected_hash);

    // Verify the transaction was added to the pool.
    let pending = pool.pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0], encoded);
}
