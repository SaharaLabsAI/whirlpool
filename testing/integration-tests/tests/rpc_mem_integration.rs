use std::collections::BTreeMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::time::{Duration, Instant};

use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, U256};
use app_evm::build_sahara_chain_spec_with_alloc;
use app_mem::SUPPORTED_PERSONALITY_TX_VERSION;
use commonware_cryptography::{ed25519, Signer as CwSigner};
use reqwest::Client;
use reth_chainspec::ChainSpec;
use rpc_mem::SubmitPersonalityRequest;
use tempfile::TempDir;
use whirlpool_node::config::{
    ConsensusStartupConfig, DEFAULT_MAX_MESSAGE_SIZE, IdentityConfig, NetworkConfig, NodeConfig,
    RpcConfig as NodeRpcConfig, StorageConfig,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

fn test_client() -> Client {
    Client::new()
}

fn rpc_req(method: &str, params: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
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

    let chain_spec: ChainSpec = build_sahara_chain_spec_with_alloc(alloc);
    let p2p_port = allocate_port();
    let rpc_port = allocate_port();
    let mem_rpc_port = allocate_port();
    let p2p_addr: SocketAddr = format!("127.0.0.1:{p2p_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse p2p socket address for seed {seed}: {err}"));
    let rpc_addr: SocketAddr = format!("127.0.0.1:{rpc_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse rpc socket address for seed {seed}: {err}"));
    let mem_rpc_addr: SocketAddr = format!("127.0.0.1:{mem_rpc_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse mem rpc socket address for seed {seed}: {err}"));

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
            mem_bind_addr: mem_rpc_addr,
        },
        storage: StorageConfig {
            data_dir: tempdir.path().to_path_buf(),
        },
        consensus: ConsensusStartupConfig {
            namespace: format!("tx-test-{seed}").into_bytes(),
            block_interval: Duration::from_secs(1),
        },
        validators: Some(vec![public_key.clone()]),
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
        let response = post_json_to_addr(&client, rpc_addr, rpc_req("eth_blockNumber", serde_json::json!([]))).await;
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

#[tokio::test(flavor = "current_thread")]
async fn test_mem_submit_personality_on_mem_rpc_only() {
    let signer = Address::repeat_byte(0x11);
    let (handle, _tempdir) = start_funded_node(
        103,
        signer,
        U256::from(100_000_000_000_000_000_000u128),
    );

    wait_for_block(handle.rpc_addr, 1, Duration::from_secs(30)).await;

    let client = test_client();
    let request = SubmitPersonalityRequest {
        version: SUPPORTED_PERSONALITY_TX_VERSION,
        signer: format!("{signer:#x}"),
        personality_id: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        nonce: 1,
        markdown: "# hello mem rpc".to_string(),
        signature_scheme: "raw_secp256k1".to_string(),
        signature: format!("0x{}", "11".repeat(65)),
    };

    let mem_response = post_json_to_addr(
        &client,
        handle.mem_rpc_addr,
        rpc_req("mem_submitPersonality", serde_json::json!([request])),
    )
    .await;
    assert!(
        mem_response["error"].is_null() || mem_response.get("error").is_none(),
        "mem_submitPersonality should succeed: {mem_response}"
    );
    assert!(
        mem_response["result"]["tx_hash"].as_str().is_some(),
        "mem_submitPersonality should return a tx hash: {mem_response}"
    );

    let wrong_server_response = post_json_to_addr(
        &client,
        handle.rpc_addr,
        rpc_req("mem_submitPersonality", serde_json::json!([serde_json::json!({})])),
    )
    .await;
    assert!(
        wrong_server_response["error"].is_object(),
        "mem_submitPersonality should not be exposed on the Ethereum RPC server: {wrong_server_response}"
    );
}
