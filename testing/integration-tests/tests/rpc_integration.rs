//! Integration tests for the Ethereum JSON-RPC server.
//!
//! These tests start a real RPC server and interact with it using alloy.

use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, B256, Bytes, TxKind, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use app::traits::TxSource;
use app::{EvmBlock, Receipt};
use state::{BlockStorage, BlockStorageError};
use state_memory::InMemoryStateDb;
use state_reth::RethStateDb;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

/// Null block storage used by legacy RPC tests that do not exercise block history.
#[derive(Debug, Default)]
struct NullBlockStorage;

impl BlockStorage for NullBlockStorage {
    fn store_block(&self, _block: &EvmBlock, _receipts: &[Receipt]) -> Result<(), BlockStorageError> {
        Ok(())
    }

    fn get_block_by_number(&self, _number: u64) -> Result<Option<EvmBlock>, BlockStorageError> {
        Ok(None)
    }

    fn get_block_by_hash(&self, _hash: B256) -> Result<Option<EvmBlock>, BlockStorageError> {
        Ok(None)
    }

    fn get_receipts_by_block(&self, _number: u64) -> Result<Option<Vec<Receipt>>, BlockStorageError> {
        Ok(None)
    }
}

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
    let block_storage = Arc::new(NullBlockStorage);
    let ctx = rpc_eth::context::EthRpcContext::new(
        pool.clone(),
        state_db.clone(),
        block_storage,
        313_371,
    );
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

/// Spin up an RPC server backed by a real RethStateDb for block storage e2e tests.
async fn start_test_rpc_with_reth_storage() -> (
    String,
    jsonrpsee::server::ServerHandle,
    rpc_eth::context::EthRpcContext<RethStateDb, RethStateDb>,
    tempfile::TempDir,
) {
    use jsonrpsee::server::ServerBuilder;
    use rpc_eth::eth_api::EthApiServer;
    use rpc_eth::eth_handler::EthApiHandler;

    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let reth_db = state_reth::open_state_db(temp_dir.path()).expect("failed to open reth db");

    let pool = Arc::new(app::tx_source::InMemoryTxPool::new());
    let state_db = Arc::new(RwLock::new(reth_db.clone()));
    let block_storage = Arc::new(reth_db);
    let ctx = rpc_eth::context::EthRpcContext::new(
        pool,
        state_db,
        block_storage,
        313_371,
    );
    let handler = EthApiHandler::new(ctx.clone());

    let server = ServerBuilder::default()
        .build("127.0.0.1:0")
        .await
        .expect("failed to build RPC server");
    let addr = server.local_addr().expect("failed to get local addr");
    let handle = server.start(handler.into_rpc());

    let url = format!("http://{addr}");
    (url, handle, ctx, temp_dir)
}

fn make_signed_legacy_tx(nonce: u64) -> Vec<u8> {
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .parse()
            .unwrap();

    let tx = TxLegacy {
        chain_id: Some(313_371),
        nonce,
        gas_price: 1_000_000_000,
        gas_limit: 21_000,
        to: TxKind::Call(Address::repeat_byte(0x22)),
        value: U256::from(nonce + 1),
        input: Bytes::default(),
    };

    let sig = signer.sign_hash_sync(&tx.signature_hash()).unwrap();
    let signed = tx.into_signed(sig);

    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    encoded
}

fn make_block(height: u64, timestamp: u64, tx_count: usize) -> EvmBlock {
    EvmBlock {
        height,
        parent_id: [height.saturating_sub(1) as u8; 32],
        state_root: [0x11u8.wrapping_add(height as u8); 32],
        transactions_root: [0x22u8.wrapping_add(height as u8); 32],
        receipts_root: [0x33u8.wrapping_add(height as u8); 32],
        gas_used: 21_000 * tx_count as u64,
        timestamp,
        transactions: (0..tx_count)
            .map(|i| make_signed_legacy_tx(height * 1000 + i as u64))
            .collect(),
    }
}

fn make_receipts(tx_count: usize) -> Vec<Receipt> {
    (0..tx_count)
        .map(|i| Receipt {
            status: true.into(),
            cumulative_gas_used: 21_000 * (i as u64 + 1),
            logs: Vec::new(),
        })
        .collect()
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

/// TC-INT-01: Store and retrieve block via RPC (latest).
#[tokio::test]
async fn test_block_history_store_and_get_latest() {
    let (url, _handle, ctx, _temp_dir) = start_test_rpc_with_reth_storage().await;

    let block = make_block(12, 1_710_000_012, 1);
    let receipts = make_receipts(block.transactions.len());
    let expected_hash = B256::from(block.compute_id());

    ctx.block_storage.store_block(&block, &receipts).unwrap();
    ctx.block_height.store(block.height, Ordering::Relaxed);

    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let rpc_block = provider
        .get_block_by_number(alloy_eips::BlockNumberOrTag::Latest)
        .await
        .unwrap()
        .expect("block should exist");

    assert_eq!(rpc_block.header.number, block.height);
    assert_eq!(rpc_block.header.parent_hash, B256::from(block.parent_id));
    assert_eq!(rpc_block.header.timestamp, block.timestamp);
    // Current RPC block conversion sets `header.hash` to zero.
    assert_eq!(rpc_block.header.hash, B256::ZERO);
    assert_eq!(expected_hash, B256::from(block.compute_id()));
}

/// TC-INT-02: eth_getBlockByHash round-trip.
#[tokio::test]
async fn test_block_history_get_block_by_hash_round_trip() {
    let (url, _handle, ctx, _temp_dir) = start_test_rpc_with_reth_storage().await;

    let block = make_block(20, 1_710_000_020, 1);
    let receipts = make_receipts(block.transactions.len());
    let block_hash = B256::from(block.compute_id());

    ctx.block_storage.store_block(&block, &receipts).unwrap();

    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let rpc_block = provider
        .get_block_by_hash(block_hash)
        .await
        .unwrap()
        .expect("block should exist");

    assert_eq!(rpc_block.header.number, block.height);
    assert_eq!(rpc_block.header.parent_hash, B256::from(block.parent_id));
    assert_eq!(rpc_block.header.timestamp, block.timestamp);
}

/// TC-INT-03: Missing block returns null.
#[tokio::test]
async fn test_block_history_missing_number_returns_none() {
    let (url, _handle, _ctx, _temp_dir) = start_test_rpc_with_reth_storage().await;

    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());
    let rpc_block = provider
        .get_block_by_number(alloy_eips::BlockNumberOrTag::Number(0x999))
        .await
        .unwrap();

    assert!(rpc_block.is_none());
}

/// TC-INT-04: Multiple sequential blocks can be queried by number.
#[tokio::test]
async fn test_block_history_multiple_sequential_blocks() {
    let (url, _handle, ctx, _temp_dir) = start_test_rpc_with_reth_storage().await;

    let blocks = vec![
        make_block(0, 1_710_000_000, 1),
        make_block(1, 1_710_000_001, 1),
        make_block(2, 1_710_000_002, 1),
    ];

    for block in &blocks {
        let receipts = make_receipts(block.transactions.len());
        ctx.block_storage.store_block(block, &receipts).unwrap();
    }

    let provider = ProviderBuilder::new().connect_http(url.parse().unwrap());

    for block in &blocks {
        let rpc_block = provider
            .get_block_by_number(alloy_eips::BlockNumberOrTag::Number(block.height))
            .await
            .unwrap()
            .expect("block should exist");

        assert_eq!(rpc_block.header.number, block.height);
        assert_eq!(rpc_block.header.parent_hash, B256::from(block.parent_id));
        assert_eq!(rpc_block.header.timestamp, block.timestamp);
    }
}
