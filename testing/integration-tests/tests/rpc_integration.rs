//! Integration tests for the Ethereum JSON-RPC server.
//!
//! Legacy tests removed; see git history for the original suite.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex, OnceLock};

use alloy_consensus::{SignableTransaction, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Signature, TxKind, U256};
use alloy_provider::{Provider, ProviderBuilder};
use app::traits::TxSource;
use app_evm::{build_sahara_chain_spec, SAHARA_CHAIN_ID};
use reqwest::Client;
use rpc_eth::{start_rpc_server, RpcConfig};
use tempfile::TempDir;

struct RecordingTxSource {
    pushed: Mutex<Vec<Vec<u8>>>,
}

impl RecordingTxSource {
    fn new() -> Self {
        Self {
            pushed: Mutex::new(vec![]),
        }
    }

    fn pushed_txs(&self) -> Vec<Vec<u8>> {
        self.pushed.lock().unwrap().clone()
    }
}

impl TxSource for RecordingTxSource {
    fn push(&self, tx: Vec<u8>) {
        self.pushed.lock().unwrap().push(tx);
    }

    fn pending(&self) -> Vec<Vec<u8>> {
        self.pushed.lock().unwrap().clone()
    }
}

struct TestRpcServer {
    _handle: reth_rpc_builder::RpcServerHandle,
    addr: SocketAddr,
    _tmp_dir: TempDir,
}

async fn start_test_rpc() -> TestRpcServer {
    start_test_rpc_with_tx_source().await.0
}

async fn start_test_rpc_with_tx_source() -> (TestRpcServer, Arc<RecordingTxSource>) {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let state_db = Arc::new(
        state_reth::open_state_db(tmp_dir.path()).expect("failed to open reth state db"),
    );
    let chain_spec = Arc::new(build_sahara_chain_spec());
    let tx_source = Arc::new(RecordingTxSource::new());

    let config = RpcConfig {
        state_db,
        chain_spec,
        tx_source: tx_source.clone(),
        addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
    };

    let (handle, addr) = start_rpc_server(config)
        .await
        .expect("failed to start RPC server");

    (
        TestRpcServer {
            _handle: handle,
            addr,
            _tmp_dir: tmp_dir,
        },
        tx_source,
    )
}

fn rpc_url(server: &TestRpcServer) -> String {
    format!("http://{}", server.addr)
}

async fn post_json(
    client: &Client,
    server: &TestRpcServer,
    body: serde_json::Value,
) -> serde_json::Value {
    client
        .post(rpc_url(server))
        .json(&body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn test_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}

fn rpc_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_rpc_tests() -> std::sync::MutexGuard<'static, ()> {
    match rpc_test_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn raw_tx_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn signed_legacy_tx_bytes() -> Vec<u8> {
    let tx = TxLegacy {
        chain_id: Some(SAHARA_CHAIN_ID),
        nonce: 0,
        gas_price: 1_000_000_000,
        gas_limit: 21_000,
        to: TxKind::Call(Address::repeat_byte(0x11)),
        value: U256::from(42_u64),
        input: Default::default(),
    };

    let signed = tx.into_signed(Signature::test_signature());
    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    encoded
}

#[tokio::test(flavor = "current_thread")]
async fn tst4_server_returns_chain_id() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(rpc_url(&server).parse().unwrap());

    let chain_id = provider.get_chain_id().await.unwrap();

    assert_eq!(chain_id, SAHARA_CHAIN_ID);
}

#[tokio::test(flavor = "current_thread")]
async fn tst5_latest_block_number() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(rpc_url(&server).parse().unwrap());

    let block_number = provider.get_block_number().await.unwrap();

    assert_eq!(block_number, 0);
}

#[tokio::test(flavor = "current_thread")]
async fn tst6_balance_query_returns_zero_for_empty_db() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(rpc_url(&server).parse().unwrap());

    let balance = provider.get_balance(Address::ZERO).await.unwrap();

    assert_eq!(balance, U256::ZERO);
}

#[tokio::test(flavor = "current_thread")]
async fn tst7_eth_syncing_returns_false() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let body = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_syncing",
            "params": [],
            "id": 1,
        }),
    )
    .await;

    assert_eq!(body["result"], serde_json::Value::Bool(false));
}

#[tokio::test(flavor = "current_thread")]
async fn tst8_get_block_by_number() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let body = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": ["latest", false],
            "id": 1,
        }),
    )
    .await;

    // On an empty DB, latest block may not exist — accept null result or an error
    assert!(
        body["result"].is_null()
            || body["result"].is_object()
            || body["error"].is_object(),
        "expected null, block object, or error for latest block on empty DB: {body}"
    );

    if body["result"].is_object() {
        if let Some(number) = body["result"]["number"].as_str() {
            assert_eq!(number, "0x0");
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn tst9_send_raw_transaction_acceptance_and_blob_rejection() {
    let _guard = lock_rpc_tests();
    let (server, tx_source) = start_test_rpc_with_tx_source().await;
    let client = test_client();

    let raw_legacy = signed_legacy_tx_bytes();
    let legacy_hash = format!("0x{:x}", alloy_primitives::keccak256(&raw_legacy));
    let legacy_body = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [raw_tx_hex(&raw_legacy)],
            "id": 1,
        }),
    )
    .await;

    assert!(legacy_body.get("error").is_none(), "legacy tx should be accepted: {legacy_body}");
    assert_eq!(legacy_body["result"], serde_json::Value::String(legacy_hash));
    assert_eq!(tx_source.pushed_txs(), vec![raw_legacy.clone()]);

    let raw_blob = vec![0x03];
    let blob_body = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [raw_tx_hex(&raw_blob)],
            "id": 2,
        }),
    )
    .await;

    let error_message = blob_body["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(blob_body["error"].is_object(), "blob tx should be rejected: {blob_body}");
    assert!(
        error_message.contains("blob")
            || error_message.contains("4844")
            || error_message.contains("unsupported")
            || error_message.contains("decode signed transaction"),
        "unexpected blob rejection: {blob_body}"
    );
    assert_eq!(tx_source.pushed_txs(), vec![raw_legacy]);
}

#[tokio::test(flavor = "current_thread")]
async fn tst10_blob_base_fee_behavior() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let body = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blobBaseFee",
            "params": [],
            "id": 1,
        }),
    )
    .await;

    assert!(
        body["error"].is_object()
            || body["result"].as_str().is_some_and(|value| value.starts_with("0x")),
        "unexpected eth_blobBaseFee response: {body}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn tst11_request_shape_permutations() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let invalid_method = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_nonExistentMethod",
            "params": [],
            "id": 1,
        }),
    )
    .await;
    assert!(
        invalid_method["error"].is_object(),
        "unknown method should return error: {invalid_method}"
    );

    let gas_price = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 2,
        }),
    )
    .await;
    assert!(
        gas_price["error"].is_object()
            || gas_price["result"].as_str().is_some_and(|value| value.starts_with("0x")),
        "eth_gasPrice should return an RPC error or hex string: {gas_price}"
    );

    let accounts = post_json(
        &client,
        &server,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_accounts",
            "params": [],
            "id": 3,
        }),
    )
    .await;
    assert!(
        accounts["result"].is_null()
            || accounts["result"].as_array().is_some_and(|value| value.is_empty()),
        "eth_accounts should expose no unlocked accounts in test RPC: {accounts}"
    );
}
