//! Integration tests for the Ethereum JSON-RPC server.
//!
//! Legacy tests removed; see git history for the original suite.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use alloy_primitives::{Address, U256};
use alloy_provider::{Provider, ProviderBuilder};
use app::traits::TxSource;
use app_evm::{build_sahara_chain_spec, SAHARA_CHAIN_ID};
use reqwest::Client;
use rpc_eth::{start_rpc_server, RpcConfig};
use tempfile::TempDir;

struct MockTxSource;

impl TxSource for MockTxSource {
    fn push(&self, _tx: Vec<u8>) {}

    fn pending(&self) -> Vec<Vec<u8>> {
        vec![]
    }
}

struct TestRpcServer {
    _handle: reth_rpc_builder::RpcServerHandle,
    addr: SocketAddr,
    _tmp_dir: TempDir,
}

async fn start_test_rpc() -> TestRpcServer {
    let tmp_dir = TempDir::new().expect("failed to create temp dir");
    let state_db = Arc::new(
        state_reth::open_state_db(tmp_dir.path()).expect("failed to open reth state db"),
    );
    let chain_spec = Arc::new(build_sahara_chain_spec());

    let config = RpcConfig {
        state_db,
        chain_spec,
        tx_source: Arc::new(MockTxSource),
        addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
    };

    let (handle, addr) = start_rpc_server(config)
        .await
        .expect("failed to start RPC server");

    TestRpcServer {
        _handle: handle,
        addr,
        _tmp_dir: tmp_dir,
    }
}

#[tokio::test]
async fn tst4_server_returns_chain_id() {
    let server = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(format!("http://{}", server.addr).parse().unwrap());

    let chain_id = provider.get_chain_id().await.unwrap();

    assert_eq!(chain_id, SAHARA_CHAIN_ID);
}

#[tokio::test]
async fn tst5_latest_block_number() {
    let server = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(format!("http://{}", server.addr).parse().unwrap());

    let block_number = provider.get_block_number().await.unwrap();

    assert_eq!(block_number, 0);
}

#[tokio::test]
async fn tst6_balance_query_returns_zero_for_empty_db() {
    let server = start_test_rpc().await;
    let provider = ProviderBuilder::new().connect_http(format!("http://{}", server.addr).parse().unwrap());

    let balance = provider.get_balance(Address::ZERO).await.unwrap();

    assert_eq!(balance, U256::ZERO);
}

#[tokio::test]
async fn tst7_eth_syncing_returns_false() {
    let server = start_test_rpc().await;
    let client = Client::new();

    let response = client
        .post(format!("http://{}", server.addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_syncing",
            "params": [],
            "id": 1,
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = response.json().await.unwrap();

    assert_eq!(body["result"], serde_json::Value::Bool(false));
}
