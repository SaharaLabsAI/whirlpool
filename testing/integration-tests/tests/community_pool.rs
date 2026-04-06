use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use app_evm::{
    build_sahara_chain_spec_with_alloc, DEFAULT_PROPOSER_FEE_RECIPIENT, SAHARA_CHAIN_ID,
};
use commonware_cryptography::{ed25519, Signer as CwSigner};
use community_pool::COMMUNITY_POOL_ADDRESS;
use reqwest::Client;
use reth_chainspec::ChainSpec;
use tempfile::TempDir;
use whirlpool_node::config::{
    ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig, RpcConfig as NodeRpcConfig,
    StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

const MAX_PRIORITY_FEE_PER_GAS: u128 = 1_000_000_000;
const MAX_FEE_PER_GAS: u128 = 20_000_000_000;
const TRANSFER_GAS_LIMIT: u64 = 21_000;

fn test_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}

fn rpc_req(method: &str, params: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    })
}

fn raw_tx_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
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
    let p2p_addr: SocketAddr = format!("127.0.0.1:{p2p_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse p2p socket address for seed {seed}: {err}"));
    let rpc_addr: SocketAddr = format!("127.0.0.1:{rpc_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse rpc socket address for seed {seed}: {err}"));

    let config = NodeConfig {
        network: NetworkConfig {
            namespace: format!("community-pool-{seed}").into_bytes(),
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
            namespace: format!("community-pool-{seed}").into_bytes(),
            block_interval: Duration::from_secs(1),
        },
        validators: Some(vec![public_key.clone()]),
    };

    let handle = start_node_with_chain_spec(config, Some(std::sync::Arc::new(chain_spec)))
        .unwrap_or_else(|err| panic!("failed to start funded node {seed}: {err}"));
    assert_eq!(handle.public_key, public_key);

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

async fn query_balance(rpc_addr: SocketAddr, address: Address) -> U256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getBalance", serde_json::json!([address, "latest"])),
    )
    .await;
    assert!(
        response["error"].is_null() || response.get("error").is_none(),
        "eth_getBalance should succeed for {address}: {response}"
    );
    parse_rpc_u256(&response["result"], "eth_getBalance result")
}

async fn submit_fee_only_transfer(
    rpc_addr: SocketAddr,
    signer: &PrivateKeySigner,
) -> (B256, serde_json::Value, serde_json::Value) {
    let tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce: 0,
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
        max_fee_per_gas: MAX_FEE_PER_GAS,
        gas_limit: TRANSFER_GAS_LIMIT,
        to: alloy_primitives::TxKind::Call(PrivateKeySigner::random().address()),
        value: U256::ZERO,
        access_list: Default::default(),
        input: Bytes::default(),
    };

    let raw_tx = sign_eip1559_tx(signer, tx).await;
    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;
    assert_eq!(
        receipt["status"].as_str(),
        Some("0x1"),
        "fee-only transfer should succeed: {receipt}"
    );

    let block_number = receipt["blockNumber"]
        .as_str()
        .unwrap_or_else(|| panic!("receipt missing blockNumber: {receipt}"));
    let client = test_client();
    let block_response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_getBlockByNumber",
            serde_json::json!([block_number, false]),
        ),
    )
    .await;
    assert!(
        block_response["error"].is_null() || block_response.get("error").is_none(),
        "eth_getBlockByNumber should succeed for {block_number}: {block_response}"
    );

    let block = block_response["result"].clone();
    let transactions = block["transactions"]
        .as_array()
        .unwrap_or_else(|| panic!("block transactions should be an array: {block}"));
    let tx_hash_hex = format!("0x{tx_hash:x}");
    assert!(
        transactions
            .iter()
            .any(|value| value.as_str() == Some(tx_hash_hex.as_str())),
        "expected transaction {tx_hash_hex} to appear in block {block}"
    );

    (tx_hash, receipt, block)
}

#[tokio::test(flavor = "current_thread")]
async fn test_community_pool_accrues_burned_amount_from_fee_only_transfer() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(300, sender, funded_balance);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let initial_community_pool_balance = query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await;
    let (_tx_hash, receipt, block) = submit_fee_only_transfer(rpc_addr, &signer).await;
    let final_community_pool_balance = query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await;

    let block_gas_used = parse_rpc_u256(&block["gasUsed"], "block gasUsed");
    let receipt_gas_used = parse_rpc_u256(&receipt["gasUsed"], "receipt gasUsed");
    assert_eq!(
        block_gas_used, receipt_gas_used,
        "expected a single-tx block"
    );

    let base_fee_per_gas = parse_rpc_u256(&block["baseFeePerGas"], "block baseFeePerGas");
    let burned_amount = block_gas_used * base_fee_per_gas;

    assert_eq!(
        final_community_pool_balance - initial_community_pool_balance,
        burned_amount,
        "community pool should accrue the block burned amount"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_proposer_fee_recipient_accrues_priority_fee_from_fee_only_transfer() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(301, sender, funded_balance);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let initial_fee_recipient_balance =
        query_balance(rpc_addr, DEFAULT_PROPOSER_FEE_RECIPIENT).await;
    let (_tx_hash, receipt, block) = submit_fee_only_transfer(rpc_addr, &signer).await;
    let final_fee_recipient_balance = query_balance(rpc_addr, DEFAULT_PROPOSER_FEE_RECIPIENT).await;

    let block_gas_used = parse_rpc_u256(&block["gasUsed"], "block gasUsed");
    let receipt_gas_used = parse_rpc_u256(&receipt["gasUsed"], "receipt gasUsed");
    assert_eq!(
        block_gas_used, receipt_gas_used,
        "expected a single-tx block"
    );

    let expected_priority_fees = block_gas_used * U256::from(MAX_PRIORITY_FEE_PER_GAS);
    assert_eq!(
        final_fee_recipient_balance - initial_fee_recipient_balance,
        expected_priority_fees,
        "fee recipient should accrue the priority-fee portion"
    );
}
