use std::collections::BTreeMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::time::{Duration, Instant};

use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, U256};
use app_evm::build_sahara_chain_spec_with_alloc;
use app_mem::{PersonalityMarkdownTx, SignatureScheme, SUPPORTED_PERSONALITY_TX_VERSION};
use commonware_cryptography::{ed25519, Signer as CwSigner};
use reqwest::Client;
use reth_chainspec::ChainSpec;
use rpc_mem::{GetPersonalityRequest, SubmitPersonalityRequest};
use tempfile::TempDir;
use whirlpool_node::config::{
    ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig, RpcConfig as NodeRpcConfig,
    StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
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

fn assert_rpc_success(response: &serde_json::Value, method: &str) {
    assert!(
        response["error"].is_null() || response.get("error").is_none(),
        "{method} should succeed: {response}"
    );
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

async fn wait_for_mem_get_personality(
    client: &Client,
    rpc_addr: SocketAddr,
    personality_id: &str,
    timeout: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + timeout;

    loop {
        let response = post_json_to_addr(
            client,
            rpc_addr,
            rpc_req(
                "mem_getPersonality",
                serde_json::json!([GetPersonalityRequest {
                    personality_id: personality_id.to_string(),
                }]),
            ),
        )
        .await;

        if response["error"].is_object() {
            panic!(
                "mem_getPersonality returned an error while waiting for finalized data: {response}"
            );
        }

        if response["result"].is_object() {
            return response;
        }

        if Instant::now() >= deadline {
            panic!(
                "timed out after {:?} waiting for mem_getPersonality({personality_id}) to return data; last response: {response}",
                timeout
            );
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
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
    let mem_rpc_addr: SocketAddr =
        format!("127.0.0.1:{mem_rpc_port}")
            .parse()
            .unwrap_or_else(|err| {
                panic!("failed to parse mem rpc socket address for seed {seed}: {err}")
            });

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
        let response = post_json_to_addr(
            &client,
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

#[tokio::test(flavor = "current_thread")]
async fn test_mem_submit_personality_on_mem_rpc_only() {
    let signer = Address::repeat_byte(0x11);
    let (handle, _tempdir) =
        start_funded_node(103, signer, U256::from(100_000_000_000_000_000_000u128));

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
    assert_rpc_success(&mem_response, "mem_submitPersonality");
    assert!(
        mem_response["result"]["tx_hash"].as_str().is_some(),
        "mem_submitPersonality should return a tx hash: {mem_response}"
    );

    let wrong_server_response = post_json_to_addr(
        &client,
        handle.rpc_addr,
        rpc_req(
            "mem_submitPersonality",
            serde_json::json!([serde_json::json!({})]),
        ),
    )
    .await;
    assert!(
        wrong_server_response["error"].is_object(),
        "mem_submitPersonality should not be exposed on the Ethereum RPC server: {wrong_server_response}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_mem_get_personality_returns_null_when_missing() {
    let signer = Address::repeat_byte(0x12);
    let (handle, _tempdir) =
        start_funded_node(104, signer, U256::from(100_000_000_000_000_000_000u128));

    wait_for_block(handle.rpc_addr, 1, Duration::from_secs(30)).await;

    let client = test_client();
    let personality_id = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
    let response = post_json_to_addr(
        &client,
        handle.mem_rpc_addr,
        rpc_req(
            "mem_getPersonality",
            serde_json::json!([GetPersonalityRequest {
                personality_id: personality_id.clone(),
            }]),
        ),
    )
    .await;

    assert_rpc_success(&response, "mem_getPersonality");
    assert!(
        response["result"].is_null(),
        "mem_getPersonality should return null for a missing personality: {response}"
    );

    let wrong_server_response = post_json_to_addr(
        &client,
        handle.rpc_addr,
        rpc_req(
            "mem_getPersonality",
            serde_json::json!([GetPersonalityRequest { personality_id }]),
        ),
    )
    .await;
    assert!(
        wrong_server_response["error"].is_object(),
        "mem_getPersonality should not be exposed on the Ethereum RPC server: {wrong_server_response}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_mem_get_personality_returns_finalized_entry_after_submit() {
    let signer = Address::repeat_byte(0x22);
    let (handle, _tempdir) =
        start_funded_node(105, signer, U256::from(100_000_000_000_000_000_000u128));

    let initial_height = wait_for_block(handle.rpc_addr, 1, Duration::from_secs(30)).await;

    let client = test_client();
    let signer_hex = format!("{signer:#x}");
    let personality_id = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
    let markdown = "# hello finalized mem rpc".to_string();
    let signature = format!("0x{}", "22".repeat(65));
    let request = SubmitPersonalityRequest {
        version: SUPPORTED_PERSONALITY_TX_VERSION,
        signer: signer_hex.clone(),
        personality_id: personality_id.clone(),
        nonce: 7,
        markdown: markdown.clone(),
        signature_scheme: "raw_secp256k1".to_string(),
        signature: signature.clone(),
    };
    let expected_tx = PersonalityMarkdownTx::new(
        hex::decode(signer_hex.trim_start_matches("0x")).expect("signer hex should decode"),
        hex::decode(personality_id.trim_start_matches("0x"))
            .expect("personality_id hex should decode"),
        request.nonce,
        markdown.clone().into_bytes(),
        SignatureScheme::RawSecp256k1,
        hex::decode(signature.trim_start_matches("0x")).expect("signature hex should decode"),
    );
    let expected_tx_hash = format!(
        "0x{}",
        hex::encode(expected_tx.tx_hash().expect("tx hash should compute"))
    );
    let expected_markdown_hash = format!("0x{}", hex::encode(expected_tx.markdown_hash));

    let submit_response = post_json_to_addr(
        &client,
        handle.mem_rpc_addr,
        rpc_req("mem_submitPersonality", serde_json::json!([request])),
    )
    .await;
    assert_rpc_success(&submit_response, "mem_submitPersonality");
    assert_eq!(
        submit_response["result"]["tx_hash"].as_str(),
        Some(expected_tx_hash.as_str()),
        "mem_submitPersonality should return the deterministic tx hash"
    );

    let response = wait_for_mem_get_personality(
        &client,
        handle.mem_rpc_addr,
        &personality_id,
        Duration::from_secs(30),
    )
    .await;
    let result = &response["result"];

    assert_eq!(result["tx_hash"].as_str(), Some(expected_tx_hash.as_str()));
    assert_eq!(result["signer"].as_str(), Some(signer_hex.as_str()));
    assert_eq!(
        result["personality_id"].as_str(),
        Some(personality_id.as_str())
    );
    assert_eq!(result["nonce"].as_u64(), Some(7));
    assert_eq!(result["markdown"].as_str(), Some(markdown.as_str()));
    assert_eq!(
        result["markdown_hash"].as_str(),
        Some(expected_markdown_hash.as_str())
    );
    assert!(
        result["block_height"].as_u64().is_some_and(|height| height > initial_height),
        "mem_getPersonality should report a finalized block height above the initial height: {response}"
    );
}
