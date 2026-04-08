//! Integration tests for the Ethereum JSON-RPC server.
//!
//! Legacy tests removed; see git history for the original suite.

use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use alloy_consensus::{SignableTransaction, TxEip1559, TxLegacy};
use alloy_eips::eip2718::Encodable2718;
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, Bytes, FixedBytes, Signature, TxKind, B256, U256};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use app::traits::TxSource;
use chainspec::{
    build_sahara_chain_spec, build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators,
    SAHARA_CHAIN_ID,
};
use commonware_cryptography::{ed25519, Signer as CwSigner};
use reqwest::Client;
use reth_chainspec::ChainSpec;
use rpc_eth::{start_rpc_server, RpcConfig};
use tempfile::TempDir;
use validators::ValidatorEntry;
use whirlpool_node::config::{
    ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig, RpcConfig as NodeRpcConfig,
    StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

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
    let state_db =
        Arc::new(state_reth::open_state_db(tmp_dir.path()).expect("failed to open reth state db"));
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
        body["result"].is_null() || body["result"].is_object() || body["error"].is_object(),
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

    assert!(
        legacy_body.get("error").is_none(),
        "legacy tx should be accepted: {legacy_body}"
    );
    assert_eq!(
        legacy_body["result"],
        serde_json::Value::String(legacy_hash)
    );
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
    assert!(
        blob_body["error"].is_object(),
        "blob tx should be rejected: {blob_body}"
    );
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
            || body["result"]
                .as_str()
                .is_some_and(|value| value.starts_with("0x")),
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
            || gas_price["result"]
                .as_str()
                .is_some_and(|value| value.starts_with("0x")),
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
            || accounts["result"]
                .as_array()
                .is_some_and(|value| value.is_empty()),
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
    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionCount",
            serde_json::json!([ZERO_ADDR, "latest"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getTransactionCount");

    // Valid: address only (optional block)
    let ok2 = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionCount", serde_json::json!([ZERO_ADDR])),
    )
    .await;
    assert_rpc_ok(&ok2, "eth_getTransactionCount (no block)");

    // Invalid: no params
    let err = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionCount", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getTransactionCount (no params)");

    // Invalid: bad address
    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionCount",
            serde_json::json!(["not_an_address", "latest"]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_getTransactionCount (bad address)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_code() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("eth_getCode", serde_json::json!([ZERO_ADDR, "latest"])),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getCode");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getCode", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getCode (no params)");

    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_getCode",
            serde_json::json!(["not_an_address", "latest"]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_getCode (bad address)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_storage_at() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getStorageAt",
            serde_json::json!([ZERO_ADDR, ZERO_HASH, "latest"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getStorageAt");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getStorageAt", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getStorageAt (no params)");

    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_getStorageAt",
            serde_json::json!(["not_an_address", ZERO_HASH, "latest"]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_getStorageAt (bad address)");
}

// ---- Hash-param methods ----------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_by_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // Valid: returns null for non-existent hash on empty DB
    let ok = post_json(
        client,
        &server,
        rpc_req("eth_getBlockByHash", serde_json::json!([ZERO_HASH, false])),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getBlockByHash");

    // Invalid: no params
    let err = post_json(
        client,
        &server,
        rpc_req("eth_getBlockByHash", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getBlockByHash (no params)");

    // Invalid: bad hash
    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_getBlockByHash",
            serde_json::json!(["0xbadhash", false]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_getBlockByHash (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_by_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionByHash", serde_json::json!([ZERO_HASH])),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getTransactionByHash");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionByHash", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getTransactionByHash (no params)");

    let err2 = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionByHash", serde_json::json!(["0xbadhash"])),
    )
    .await;
    assert_rpc_err(&err2, "eth_getTransactionByHash (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_receipt() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionReceipt", serde_json::json!([ZERO_HASH])),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getTransactionReceipt");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getTransactionReceipt", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getTransactionReceipt (no params)");

    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionReceipt",
            serde_json::json!(["0xbadhash"]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_getTransactionReceipt (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_transaction_count_by_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getBlockTransactionCountByHash",
            serde_json::json!([ZERO_HASH]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getBlockTransactionCountByHash");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getBlockTransactionCountByHash", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getBlockTransactionCountByHash (no params)");

    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_getBlockTransactionCountByHash",
            serde_json::json!(["0xbadhash"]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_getBlockTransactionCountByHash (bad hash)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_count_by_block_hash() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getUncleCountByBlockHash",
            serde_json::json!([ZERO_HASH]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getUncleCountByBlockHash");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getUncleCountByBlockHash", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getUncleCountByBlockHash (no params)");
}

// ---- Number-param methods --------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_transaction_count_by_number() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getBlockTransactionCountByNumber",
            serde_json::json!(["0x0"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getBlockTransactionCountByNumber");

    let err = post_json(
        client,
        &server,
        rpc_req(
            "eth_getBlockTransactionCountByNumber",
            serde_json::json!([]),
        ),
    )
    .await;
    assert_rpc_err(&err, "eth_getBlockTransactionCountByNumber (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_count_by_block_number() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("eth_getUncleCountByBlockNumber", serde_json::json!(["0x0"])),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getUncleCountByBlockNumber");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getUncleCountByBlockNumber", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getUncleCountByBlockNumber (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_block_receipts() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("eth_getBlockReceipts", serde_json::json!(["0x0"])),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getBlockReceipts");

    // No params — should error
    let err = post_json(
        client,
        &server,
        rpc_req("eth_getBlockReceipts", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getBlockReceipts (no params)");
}

// ---- Index methods (uncle/tx by block + index) -----------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_by_block_hash_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getUncleByBlockHashAndIndex",
            serde_json::json!([ZERO_HASH, "0x0"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getUncleByBlockHashAndIndex");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getUncleByBlockHashAndIndex", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getUncleByBlockHashAndIndex (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_uncle_by_block_number_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getUncleByBlockNumberAndIndex",
            serde_json::json!(["0x0", "0x0"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getUncleByBlockNumberAndIndex");

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getUncleByBlockNumberAndIndex", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getUncleByBlockNumberAndIndex (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_by_block_hash_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionByBlockHashAndIndex",
            serde_json::json!([ZERO_HASH, "0x0"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getTransactionByBlockHashAndIndex");

    let err = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionByBlockHashAndIndex",
            serde_json::json!([]),
        ),
    )
    .await;
    assert_rpc_err(&err, "eth_getTransactionByBlockHashAndIndex (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_get_transaction_by_block_number_and_index() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionByBlockNumberAndIndex",
            serde_json::json!(["0x0", "0x0"]),
        ),
    )
    .await;
    assert_rpc_ok(&ok, "eth_getTransactionByBlockNumberAndIndex");

    let err = post_json(
        client,
        &server,
        rpc_req(
            "eth_getTransactionByBlockNumberAndIndex",
            serde_json::json!([]),
        ),
    )
    .await;
    assert_rpc_err(&err, "eth_getTransactionByBlockNumberAndIndex (no params)");
}

// ---- Fee / estimate / call methods (expect error on empty DB) --------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_fee_history() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, feeHistory may error because no blocks exist
    let resp = post_json(
        client,
        &server,
        rpc_req(
            "eth_feeHistory",
            serde_json::json!(["0x1", "latest", [25, 75]]),
        ),
    )
    .await;
    // Accept either success or error — both are valid on empty DB
    assert!(
        resp.get("result").is_some() || resp["error"].is_object(),
        "eth_feeHistory: unexpected response: {resp}"
    );

    // Invalid: no params
    let err = post_json(
        client,
        &server,
        rpc_req("eth_feeHistory", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_feeHistory (no params)");

    // Invalid: bad block count type
    let err2 = post_json(
        client,
        &server,
        rpc_req(
            "eth_feeHistory",
            serde_json::json!(["not_a_number", "latest", []]),
        ),
    )
    .await;
    assert_rpc_err(&err2, "eth_feeHistory (bad block count)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_estimate_gas() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, estimateGas errors (no block)
    let err = post_json(
        client,
        &server,
        rpc_req("eth_estimateGas", serde_json::json!([{"to": ZERO_ADDR}])),
    )
    .await;
    assert_rpc_err(&err, "eth_estimateGas (empty DB)");

    // Invalid: no params
    let err2 = post_json(
        client,
        &server,
        rpc_req("eth_estimateGas", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err2, "eth_estimateGas (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_call() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, eth_call errors (no block)
    let err = post_json(
        client,
        &server,
        rpc_req("eth_call", serde_json::json!([{"to": ZERO_ADDR}, "latest"])),
    )
    .await;
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
    let err = post_json(
        client,
        &server,
        rpc_req(
            "eth_createAccessList",
            serde_json::json!([{"to": ZERO_ADDR}, "latest"]),
        ),
    )
    .await;
    assert_rpc_err(&err, "eth_createAccessList (empty DB)");

    // Invalid: no params
    let err2 = post_json(
        client,
        &server,
        rpc_req("eth_createAccessList", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err2, "eth_createAccessList (no params)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_max_priority_fee_per_gas() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    // On empty DB, maxPriorityFeePerGas errors (no block for fee oracle)
    let resp = post_json(
        client,
        &server,
        rpc_req("eth_maxPriorityFeePerGas", serde_json::json!([])),
    )
    .await;
    assert!(
        resp["error"].is_object() || resp["result"].as_str().is_some_and(|v| v.starts_with("0x")),
        "eth_maxPriorityFeePerGas: expected error or hex on empty DB: {resp}"
    );
}

// ---- net_ / web3_ methods --------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_net_version() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("net_version", serde_json::json!([])),
    )
    .await;
    assert_rpc_ok(&ok, "net_version");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_net_peer_count() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("net_peerCount", serde_json::json!([])),
    )
    .await;
    assert_rpc_ok(&ok, "net_peerCount");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_net_listening() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("net_listening", serde_json::json!([])),
    )
    .await;
    assert_rpc_ok(&ok, "net_listening");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_web3_client_version() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("web3_clientVersion", serde_json::json!([])),
    )
    .await;
    assert_rpc_ok(&ok, "web3_clientVersion");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_web3_sha3() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let ok = post_json(
        client,
        &server,
        rpc_req("web3_sha3", serde_json::json!(["0x68656c6c6f"])),
    )
    .await;
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
    let err2 = post_json(
        client,
        &server,
        rpc_req("web3_sha3", serde_json::json!(["not_hex"])),
    )
    .await;
    assert_rpc_err(&err2, "web3_sha3 (bad hex)");
}

// ---- Unimplemented methods (expect error) ----------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_coinbase() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let err = post_json(
        client,
        &server,
        rpc_req("eth_coinbase", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_coinbase (unimplemented)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_mining() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let resp = post_json(
        client,
        &server,
        rpc_req("eth_mining", serde_json::json!([])),
    )
    .await;
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

    let err = post_json(
        client,
        &server,
        rpc_req("eth_getWork", serde_json::json!([])),
    )
    .await;
    assert_rpc_err(&err, "eth_getWork (unimplemented)");
}

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_submit_work() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let err = post_json(
        client,
        &server,
        rpc_req(
            "eth_submitWork",
            serde_json::json!(["0x0", ZERO_HASH, ZERO_HASH]),
        ),
    )
    .await;
    assert_rpc_err(&err, "eth_submitWork (unimplemented)");
}

// ---- eth_protocolVersion ---------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn contract_eth_protocol_version() {
    let _guard = lock_rpc_tests();
    let server = start_test_rpc().await;
    let client = test_client();

    let resp = post_json(
        client,
        &server,
        rpc_req("eth_protocolVersion", serde_json::json!([])),
    )
    .await;
    // protocolVersion may be implemented or not — accept success or unimplemented error
    assert!(
        resp.get("result").is_some() || resp["error"].is_object(),
        "eth_protocolVersion: unexpected response: {resp}"
    );
}

fn allocate_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind ephemeral port")
        .local_addr()
        .expect("failed to get local addr")
        .port()
}

fn rpc_http_url(rpc_addr: SocketAddr) -> String {
    format!("http://{rpc_addr}")
}

async fn post_json_to_addr(
    client: &Client,
    rpc_addr: SocketAddr,
    body: serde_json::Value,
) -> serde_json::Value {
    let response = client
        .post(rpc_http_url(rpc_addr))
        .json(&body)
        .send()
        .await
        .unwrap_or_else(|err| panic!("failed to send RPC request to {rpc_addr}: {err}"));

    response
        .json()
        .await
        .unwrap_or_else(|err| panic!("failed to decode RPC response from {rpc_addr}: {err}"))
}

fn parse_rpc_u64(value: &serde_json::Value, field: &str) -> u64 {
    let hex = value
        .as_str()
        .unwrap_or_else(|| panic!("{field} should be a hex string, got {value}"));
    let digits = hex.trim_start_matches("0x");
    if digits.is_empty() {
        return 0;
    }
    u64::from_str_radix(digits, 16)
        .unwrap_or_else(|err| panic!("failed to parse {field} hex value {hex}: {err}"))
}

fn parse_rpc_u256(value: &serde_json::Value, field: &str) -> U256 {
    let hex = value
        .as_str()
        .unwrap_or_else(|| panic!("{field} should be a hex string, got {value}"));
    let digits = hex.trim_start_matches("0x");
    if digits.is_empty() {
        return U256::ZERO;
    }
    U256::from_str_radix(digits, 16)
        .unwrap_or_else(|err| panic!("failed to parse {field} hex value {hex}: {err}"))
}

fn parse_rpc_b256(value: &serde_json::Value, field: &str) -> B256 {
    let hex = value
        .as_str()
        .unwrap_or_else(|| panic!("{field} should be a 0x-prefixed hash string, got {value}"));
    let parsed: FixedBytes<32> = hex
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse {field} hash {hex}: {err}"));
    B256::from(parsed)
}

fn parse_rpc_address(value: &serde_json::Value, field: &str) -> Address {
    let hex = value
        .as_str()
        .unwrap_or_else(|| panic!("{field} should be a 0x-prefixed address string, got {value}"));
    hex.parse()
        .unwrap_or_else(|err| panic!("failed to parse {field} address {hex}: {err}"))
}

fn deploy_contract_bytecode() -> Bytes {
    // Init code (11 bytes):
    //   PUSH1 0x0a (10 = runtime size)
    //   DUP1
    //   PUSH1 0x0b (11 = offset of runtime in full bytecode)
    //   PUSH1 0x00
    //   CODECOPY   (copies 10 bytes from offset 11 to mem[0])
    //   PUSH1 0x00
    //   RETURN     (returns 10 bytes from mem[0])
    // Runtime code (10 bytes): 602a60005260206000f3
    //   PUSH1 42, PUSH1 0, MSTORE, PUSH1 32, PUSH1 0, RETURN
    Bytes::from(vec![
        0x60, 0x0a, 0x80, 0x60, 0x0b, 0x60, 0x00, 0x39, 0x60, 0x00, 0xf3, 0x60, 0x2a, 0x60, 0x00,
        0x52, 0x60, 0x20, 0x60, 0x00, 0xf3,
    ])
}

fn deployed_contract_runtime() -> &'static str {
    "0x602a60005260206000f3"
}

fn start_funded_node(seed: u64, funded_address: Address, balance: U256) -> (NodeHandle, TempDir) {
    let tempdir = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for funded node {seed}: {err}"));
    let validator_key = ed25519::PrivateKey::from_seed(seed);
    let public_key = validator_key.public_key();

    let mut alloc = BTreeMap::new();
    alloc.insert(
        funded_address,
        GenesisAccount {
            balance,
            ..GenesisAccount::default()
        },
    );

    let chain_spec: ChainSpec =
        build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
            alloc,
            BTreeMap::new(),
            vec![ValidatorEntry {
                consensus_pubkey: public_key.as_ref().try_into().expect("ed25519 key length"),
                ethereum_address: Address::ZERO,
            }],
        );
    let p2p_port = allocate_port();
    let rpc_port = allocate_port();
    let p2p_addr: SocketAddr = format!("127.0.0.1:{p2p_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse p2p socket address for seed {seed}: {err}"));
    let rpc_addr: SocketAddr = format!("127.0.0.1:{rpc_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse rpc socket address for seed {seed}: {err}"));

    let config = NodeConfig {
        network: NetworkConfig {
            namespace: format!("tx-test-{seed}").into_bytes(),
            listen_addr: p2p_addr,
            dialable_addr: p2p_addr,
            bootstrap_peers: vec![],
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
        },
        identity: IdentityConfig { seed },
        rpc: NodeRpcConfig {
            bind_addr: rpc_addr,
            mem_bind_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
        },
        storage: StorageConfig {
            data_dir: tempdir.path().to_path_buf(),
        },
        consensus: ConsensusStartupConfig {
            namespace: format!("tx-test-{seed}").into_bytes(),
            block_interval: Duration::from_secs(1),
        },
        bootstrap_validators: Some(vec![public_key.clone()]),
    };

    let handle = start_node_with_chain_spec(config, Some(Arc::new(chain_spec)))
        .unwrap_or_else(|err| panic!("failed to start funded node {seed}: {err}"));
    assert_eq!(
        handle.public_key, public_key,
        "started node should advertise the configured validator key"
    );

    (handle, tempdir)
}

async fn wait_for_block(rpc_addr: SocketAddr, min_height: u64, timeout: Duration) -> u64 {
    let client = test_client();
    let deadline = Instant::now() + timeout;

    loop {
        let response = post_json_to_addr(
            client,
            rpc_addr,
            rpc_req("eth_blockNumber", serde_json::json!([])),
        )
        .await;
        if response["error"].is_object() {
            panic!("eth_blockNumber returned an error while waiting for height {min_height}: {response}");
        }

        let height = parse_rpc_u64(&response["result"], "eth_blockNumber result");
        if height >= min_height {
            return height;
        }

        if Instant::now() >= deadline {
            panic!(
                "timed out after {:?} waiting for block height >= {min_height}; last response: {response}",
                timeout
            );
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn wait_for_receipt(
    rpc_addr: SocketAddr,
    tx_hash: B256,
    timeout: Duration,
) -> serde_json::Value {
    let client = test_client();
    let deadline = Instant::now() + timeout;
    let tx_hash_hex = format!("0x{tx_hash:x}");

    loop {
        let response = post_json_to_addr(
            client,
            rpc_addr,
            rpc_req(
                "eth_getTransactionReceipt",
                serde_json::json!([tx_hash_hex.clone()]),
            ),
        )
        .await;

        if response["error"].is_object() {
            panic!("eth_getTransactionReceipt returned an error for {tx_hash:#x}: {response}");
        }

        if !response["result"].is_null() {
            return response["result"].clone();
        }

        if Instant::now() >= deadline {
            panic!(
                "timed out after {:?} waiting for receipt {tx_hash:#x}; last response: {response}",
                timeout
            );
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn send_raw_tx(rpc_addr: SocketAddr, raw_tx_bytes: &[u8]) -> B256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_sendRawTransaction",
            serde_json::json!([raw_tx_hex(raw_tx_bytes)]),
        ),
    )
    .await;

    if response["error"].is_object() {
        panic!(
            "eth_sendRawTransaction failed for {}: {response}",
            raw_tx_hex(raw_tx_bytes)
        );
    }

    parse_rpc_b256(&response["result"], "eth_sendRawTransaction result")
}

async fn sign_eip1559_tx(signer: &PrivateKeySigner, tx: TxEip1559) -> Vec<u8> {
    let signature = signer
        .sign_hash(&tx.signature_hash())
        .await
        .unwrap_or_else(|err| panic!("failed to sign EIP-1559 transaction: {err}"));
    let signed = tx.into_signed(signature);
    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    encoded
}

async fn deploy_minimal_contract(
    rpc_addr: SocketAddr,
    signer: &PrivateKeySigner,
    nonce: u64,
) -> (B256, serde_json::Value, Address) {
    let deploy_tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce,
        max_priority_fee_per_gas: 1_000_000_000,
        max_fee_per_gas: 20_000_000_000,
        gas_limit: 100_000,
        to: TxKind::Create,
        value: U256::ZERO,
        access_list: Default::default(),
        input: deploy_contract_bytecode(),
    };

    let raw_tx = sign_eip1559_tx(signer, deploy_tx).await;
    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;

    assert_eq!(
        receipt["status"].as_str(),
        Some("0x1"),
        "contract deployment should succeed: {receipt}"
    );

    let contract_address = parse_rpc_address(&receipt["contractAddress"], "contractAddress");
    (tx_hash, receipt, contract_address)
}

#[tokio::test(flavor = "current_thread")]
async fn test_eth_transfer_full_node() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let one_eth = U256::from(1_000_000_000_000_000_000u128);
    let hundred_eth = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(100, sender, hundred_eth);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let recipient = PrivateKeySigner::random().address();
    let tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce: 0,
        max_priority_fee_per_gas: 1_000_000_000,
        max_fee_per_gas: 20_000_000_000,
        gas_limit: 21_000,
        to: TxKind::Call(recipient),
        value: one_eth,
        access_list: Default::default(),
        input: Bytes::default(),
    };

    let raw_tx = sign_eip1559_tx(&signer, tx).await;
    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;
    assert_eq!(
        receipt["status"].as_str(),
        Some("0x1"),
        "transfer receipt should indicate success: {receipt}"
    );

    let client = test_client();
    let recipient_balance = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getBalance", serde_json::json!([recipient, "latest"])),
    )
    .await;
    assert!(
        recipient_balance["error"].is_null() || recipient_balance.get("error").is_none(),
        "recipient balance query should succeed: {recipient_balance}"
    );
    assert_eq!(
        parse_rpc_u256(&recipient_balance["result"], "recipient balance"),
        one_eth,
        "recipient should receive exactly 1 ETH"
    );

    let sender_balance = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getBalance", serde_json::json!([sender, "latest"])),
    )
    .await;
    assert!(
        sender_balance["error"].is_null() || sender_balance.get("error").is_none(),
        "sender balance query should succeed: {sender_balance}"
    );
    let ninety_nine_eth = U256::from(99_000_000_000_000_000_000u128);
    assert!(
        parse_rpc_u256(&sender_balance["result"], "sender balance") < ninety_nine_eth,
        "sender balance should reflect transfer value plus gas cost: {sender_balance}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_contract_deploy_full_node() {
    let signer = PrivateKeySigner::random();
    let funded_address = signer.address();
    let (handle, _tempdir) = start_funded_node(
        101,
        funded_address,
        U256::from(100_000_000_000_000_000_000u128),
    );
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let (_tx_hash, receipt, contract_address) = deploy_minimal_contract(rpc_addr, &signer, 0).await;
    assert!(
        receipt["transactionHash"].as_str().is_some(),
        "deployment receipt should include transactionHash: {receipt}"
    );

    let client = test_client();
    let code_response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_getCode",
            serde_json::json!([contract_address, "latest"]),
        ),
    )
    .await;
    assert!(
        code_response["error"].is_null() || code_response.get("error").is_none(),
        "eth_getCode should succeed for deployed contract: {code_response}"
    );
    assert_eq!(
        code_response["result"].as_str(),
        Some(deployed_contract_runtime()),
        "deployed contract runtime bytecode should match"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_contract_call_full_node() {
    let signer = PrivateKeySigner::random();
    let funded_address = signer.address();
    let (handle, _tempdir) = start_funded_node(
        102,
        funded_address,
        U256::from(100_000_000_000_000_000_000u128),
    );
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let (_tx_hash, _receipt, contract_address) =
        deploy_minimal_contract(rpc_addr, &signer, 0).await;

    let client = test_client();
    let call_response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_call",
            serde_json::json!([
                {
                    "to": contract_address,
                    "data": "0x",
                },
                "latest"
            ]),
        ),
    )
    .await;
    assert!(
        call_response["error"].is_null() || call_response.get("error").is_none(),
        "eth_call should succeed for deployed contract: {call_response}"
    );
    assert_eq!(
        call_response["result"].as_str(),
        Some("0x000000000000000000000000000000000000000000000000000000000000002a"),
        "eth_call should return uint256(42)"
    );
}
