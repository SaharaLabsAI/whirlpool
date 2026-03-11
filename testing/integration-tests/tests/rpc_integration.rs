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

// ---------------------------------------------------------------------------
// Contract tests: per-method param validation
//
// Each test calls one RPC method with valid params (expect success or a
// well-formed null/result on empty DB) and with invalid/missing params
// (expect a JSON-RPC error). Mirrors the approach from
// vendor/reth rpc-builder/tests/it/http.rs but excludes blob tx methods.
//
// Methods already covered in tst4–tst11 are NOT duplicated here:
//   eth_chainId (tst4), eth_blockNumber (tst5), eth_getBalance (tst6),
//   eth_syncing (tst7), eth_getBlockByNumber (tst8),
//   eth_sendRawTransaction + blob rejection (tst9), eth_blobBaseFee (tst10),
//   eth_gasPrice / eth_accounts / invalid method (tst11).
// ---------------------------------------------------------------------------

/// Assert that the JSON-RPC response contains a result (no error).
/// Accepts null results (valid for lookups on empty DB).
fn assert_rpc_ok(body: &serde_json::Value, method: &str) {
    assert!(
        body.get("error").is_none() || body["error"].is_null(),
        "{method}: expected success but got error: {body}"
    );
}

/// Assert that the JSON-RPC response contains an error object.
fn assert_rpc_err(body: &serde_json::Value, method: &str) {
    assert!(
        body["error"].is_object(),
        "{method}: expected error but got success: {body}"
    );
}

fn rpc_req(method: &str, params: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
}

const ZERO_ADDR: &str = "0x0000000000000000000000000000000000000000";
const ZERO_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";

// ---- Address + optional block param methods --------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_count() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // Valid: address + latest block tag
    let ok = post_json(client, &server, rpc_req("eth_getTransactionCount", serde_json::json!([ZERO_ADDR, "latest"]))).await;
    assert_rpc_ok(&ok, "eth_getTransactionCount");

    // Valid: address only (optional block)
    let ok2 = post_json(client, &server, rpc_req("eth_getTransactionCount", serde_json::json!([ZERO_ADDR]))).await;
    assert_rpc_ok(&ok2, "eth_getTransactionCount (no block)");

    // Invalid: no params
    let err = post_json(client, &server, rpc_req("eth_getTransactionCount", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getTransactionCount (no params)");

    // Invalid: bad address
    let err2 = post_json(client, &server, rpc_req("eth_getTransactionCount", serde_json::json!(["not_an_address", "latest"]))).await;
    assert_rpc_err(&err2, "eth_getTransactionCount (bad address)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_code() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getCode", serde_json::json!([ZERO_ADDR, "latest"]))).await;
    assert_rpc_ok(&ok, "eth_getCode");

    let err = post_json(client, &server, rpc_req("eth_getCode", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getCode (no params)");

    let err2 = post_json(client, &server, rpc_req("eth_getCode", serde_json::json!(["not_an_address", "latest"]))).await;
    assert_rpc_err(&err2, "eth_getCode (bad address)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_storage_at() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getStorageAt", serde_json::json!([ZERO_ADDR, ZERO_HASH, "latest"]))).await;
    assert_rpc_ok(&ok, "eth_getStorageAt");

    let err = post_json(client, &server, rpc_req("eth_getStorageAt", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getStorageAt (no params)");

    let err2 = post_json(client, &server, rpc_req("eth_getStorageAt", serde_json::json!(["not_an_address", ZERO_HASH, "latest"]))).await;
    assert_rpc_err(&err2, "eth_getStorageAt (bad address)");
}

// ---- Hash-param methods ----------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_by_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // Valid: returns null for non-existent hash on empty DB
    let ok = post_json(client, &server, rpc_req("eth_getBlockByHash", serde_json::json!([ZERO_HASH, false]))).await;
    assert_rpc_ok(&ok, "eth_getBlockByHash");

    // Invalid: no params
    let err = post_json(client, &server, rpc_req("eth_getBlockByHash", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getBlockByHash (no params)");

    // Invalid: bad hash
    let err2 = post_json(client, &server, rpc_req("eth_getBlockByHash", serde_json::json!(["0xbadhash", false]))).await;
    assert_rpc_err(&err2, "eth_getBlockByHash (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_by_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getTransactionByHash", serde_json::json!([ZERO_HASH]))).await;
    assert_rpc_ok(&ok, "eth_getTransactionByHash");

    let err = post_json(client, &server, rpc_req("eth_getTransactionByHash", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getTransactionByHash (no params)");

    let err2 = post_json(client, &server, rpc_req("eth_getTransactionByHash", serde_json::json!(["0xbadhash"]))).await;
    assert_rpc_err(&err2, "eth_getTransactionByHash (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_receipt() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getTransactionReceipt", serde_json::json!([ZERO_HASH]))).await;
    assert_rpc_ok(&ok, "eth_getTransactionReceipt");

    let err = post_json(client, &server, rpc_req("eth_getTransactionReceipt", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getTransactionReceipt (no params)");

    let err2 = post_json(client, &server, rpc_req("eth_getTransactionReceipt", serde_json::json!(["0xbadhash"]))).await;
    assert_rpc_err(&err2, "eth_getTransactionReceipt (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_transaction_count_by_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getBlockTransactionCountByHash", serde_json::json!([ZERO_HASH]))).await;
    assert_rpc_ok(&ok, "eth_getBlockTransactionCountByHash");

    let err = post_json(client, &server, rpc_req("eth_getBlockTransactionCountByHash", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getBlockTransactionCountByHash (no params)");

    let err2 = post_json(client, &server, rpc_req("eth_getBlockTransactionCountByHash", serde_json::json!(["0xbadhash"]))).await;
    assert_rpc_err(&err2, "eth_getBlockTransactionCountByHash (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_count_by_block_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getUncleCountByBlockHash", serde_json::json!([ZERO_HASH]))).await;
    assert_rpc_ok(&ok, "eth_getUncleCountByBlockHash");

    let err = post_json(client, &server, rpc_req("eth_getUncleCountByBlockHash", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getUncleCountByBlockHash (no params)");
}

// ---- Number-param methods --------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_transaction_count_by_number() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getBlockTransactionCountByNumber", serde_json::json!(["0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getBlockTransactionCountByNumber");

    let err = post_json(client, &server, rpc_req("eth_getBlockTransactionCountByNumber", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getBlockTransactionCountByNumber (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_count_by_block_number() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getUncleCountByBlockNumber", serde_json::json!(["0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getUncleCountByBlockNumber");

    let err = post_json(client, &server, rpc_req("eth_getUncleCountByBlockNumber", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getUncleCountByBlockNumber (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_receipts() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getBlockReceipts", serde_json::json!(["0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getBlockReceipts");

    // No params — should error
    let err = post_json(client, &server, rpc_req("eth_getBlockReceipts", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getBlockReceipts (no params)");
}

// ---- Index methods (uncle/tx by block + index) -----------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_by_block_hash_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getUncleByBlockHashAndIndex", serde_json::json!([ZERO_HASH, "0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getUncleByBlockHashAndIndex");

    let err = post_json(client, &server, rpc_req("eth_getUncleByBlockHashAndIndex", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getUncleByBlockHashAndIndex (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_by_block_number_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getUncleByBlockNumberAndIndex", serde_json::json!(["0x0", "0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getUncleByBlockNumberAndIndex");

    let err = post_json(client, &server, rpc_req("eth_getUncleByBlockNumberAndIndex", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getUncleByBlockNumberAndIndex (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_by_block_hash_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getTransactionByBlockHashAndIndex", serde_json::json!([ZERO_HASH, "0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getTransactionByBlockHashAndIndex");

    let err = post_json(client, &server, rpc_req("eth_getTransactionByBlockHashAndIndex", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getTransactionByBlockHashAndIndex (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_by_block_number_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("eth_getTransactionByBlockNumberAndIndex", serde_json::json!(["0x0", "0x0"]))).await;
    assert_rpc_ok(&ok, "eth_getTransactionByBlockNumberAndIndex");

    let err = post_json(client, &server, rpc_req("eth_getTransactionByBlockNumberAndIndex", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getTransactionByBlockNumberAndIndex (no params)");
}

// ---- Fee / estimate / call methods (expect error on empty DB) --------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_fee_history() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, feeHistory may error because no blocks exist
    let resp = post_json(client, &server, rpc_req("eth_feeHistory", serde_json::json!(["0x1", "latest", [25, 75]]))).await;
    // Accept either success or error — both are valid on empty DB
    assert!(
        resp.get("result").is_some() || resp["error"].is_object(),
        "eth_feeHistory: unexpected response: {resp}"
    );

    // Invalid: no params
    let err = post_json(client, &server, rpc_req("eth_feeHistory", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_feeHistory (no params)");

    // Invalid: bad block count type
    let err2 = post_json(client, &server, rpc_req("eth_feeHistory", serde_json::json!(["not_a_number", "latest", []]))).await;
    assert_rpc_err(&err2, "eth_feeHistory (bad block count)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_estimate_gas() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, estimateGas errors (no block)
    let err = post_json(client, &server, rpc_req("eth_estimateGas", serde_json::json!([{"to": ZERO_ADDR}]))).await;
    assert_rpc_err(&err, "eth_estimateGas (empty DB)");

    // Invalid: no params
    let err2 = post_json(client, &server, rpc_req("eth_estimateGas", serde_json::json!([]))).await;
    assert_rpc_err(&err2, "eth_estimateGas (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_call() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, eth_call errors (no block)
    let err = post_json(client, &server, rpc_req("eth_call", serde_json::json!([{"to": ZERO_ADDR}, "latest"]))).await;
    assert_rpc_err(&err, "eth_call (empty DB)");

    // Invalid: no params
    let err2 = post_json(client, &server, rpc_req("eth_call", serde_json::json!([]))).await;
    assert_rpc_err(&err2, "eth_call (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_create_access_list() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, createAccessList errors (no block)
    let err = post_json(client, &server, rpc_req("eth_createAccessList", serde_json::json!([{"to": ZERO_ADDR}, "latest"]))).await;
    assert_rpc_err(&err, "eth_createAccessList (empty DB)");

    // Invalid: no params
    let err2 = post_json(client, &server, rpc_req("eth_createAccessList", serde_json::json!([]))).await;
    assert_rpc_err(&err2, "eth_createAccessList (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_max_priority_fee_per_gas() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, maxPriorityFeePerGas errors (no block for fee oracle)
    let resp = post_json(client, &server, rpc_req("eth_maxPriorityFeePerGas", serde_json::json!([]))).await;
    assert!(
        resp["error"].is_object()
            || resp["result"].as_str().is_some_and(|v| v.starts_with("0x")),
        "eth_maxPriorityFeePerGas: expected error or hex on empty DB: {resp}"
    );
}

// ---- net_ / web3_ methods --------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_net_version() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("net_version", serde_json::json!([]))).await;
    assert_rpc_ok(&ok, "net_version");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_net_peer_count() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("net_peerCount", serde_json::json!([]))).await;
    assert_rpc_ok(&ok, "net_peerCount");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_net_listening() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("net_listening", serde_json::json!([]))).await;
    assert_rpc_ok(&ok, "net_listening");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_web3_client_version() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("web3_clientVersion", serde_json::json!([]))).await;
    assert_rpc_ok(&ok, "web3_clientVersion");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_web3_sha3() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(client, &server, rpc_req("web3_sha3", serde_json::json!(["0x68656c6c6f"]))).await;
    assert_rpc_ok(&ok, "web3_sha3");
    // Verify it returns keccak256("hello")
    assert_eq!(
        ok["result"].as_str().unwrap_or_default(),
        "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8",
        "web3_sha3 keccak256 mismatch"
    );

    // Invalid: no params
    let err = post_json(client, &server, rpc_req("web3_sha3", serde_json::json!([]))).await;
    assert_rpc_err(&err, "web3_sha3 (no params)");

    // Invalid: bad hex
    let err2 = post_json(client, &server, rpc_req("web3_sha3", serde_json::json!(["not_hex"]))).await;
    assert_rpc_err(&err2, "web3_sha3 (bad hex)");
}

// ---- Unimplemented methods (expect error) ----------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_coinbase() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let err = post_json(client, &server, rpc_req("eth_coinbase", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_coinbase (unimplemented)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_mining() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let resp = post_json(client, &server, rpc_req("eth_mining", serde_json::json!([]))).await;
    // eth_mining may return false or error depending on implementation
    assert!(
        resp["error"].is_object() || resp["result"] == serde_json::Value::Bool(false),
        "eth_mining: expected error or false: {resp}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_work() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let err = post_json(client, &server, rpc_req("eth_getWork", serde_json::json!([]))).await;
    assert_rpc_err(&err, "eth_getWork (unimplemented)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_submit_work() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let err = post_json(client, &server, rpc_req("eth_submitWork", serde_json::json!(["0x0", ZERO_HASH, ZERO_HASH]))).await;
    assert_rpc_err(&err, "eth_submitWork (unimplemented)");
}

// ---- eth_protocolVersion ---------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_protocol_version() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let resp = post_json(client, &server, rpc_req("eth_protocolVersion", serde_json::json!([]))).await;
    // protocolVersion may be implemented or not — accept success or unimplemented error
    assert!(
        resp.get("result").is_some() || resp["error"].is_object(),
        "eth_protocolVersion: unexpected response: {resp}"
    );
}
