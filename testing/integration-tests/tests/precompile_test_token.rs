use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use chainspec::{
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators, SAHARA_CHAIN_ID,
};
use commonware_cryptography::{ed25519, Signer as CwSigner};
use evm_precompiles::{balance_of_calldata, mint_calldata, TEST_TOKEN_PRECOMPILE_ADDRESS};
use reqwest::Client;
use reth_chainspec::ChainSpec;
use tempfile::TempDir;
use validators::ValidatorEntry;
use whirlpool_node::config::{
    parse_bootstrap_peer, ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig,
    RpcConfig as NodeRpcConfig, StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

const MAX_PRIORITY_FEE_PER_GAS: u128 = 1_000_000_000;
const MAX_FEE_PER_GAS: u128 = 20_000_000_000;
const PRECOMPILE_CALL_GAS_LIMIT: u64 = 100_000;
const PROXY_DEPLOY_GAS_LIMIT: u64 = 150_000;

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
        .unwrap_or_else(|| panic!("expected hex string for {field}, got {value}"));
    u64::from_str_radix(hex.trim_start_matches("0x"), 16)
        .unwrap_or_else(|err| panic!("failed to decode {field} value {hex}: {err}"))
}

fn parse_rpc_b256(value: &serde_json::Value, field: &str) -> B256 {
    let hex = value
        .as_str()
        .unwrap_or_else(|| panic!("expected hex string for {field}, got {value}"));
    hex.parse()
        .unwrap_or_else(|err| panic!("failed to parse {field} as B256 from {hex}: {err}"))
}

fn parse_rpc_u256_hex(value: &serde_json::Value, field: &str) -> U256 {
    let hex = value
        .as_str()
        .unwrap_or_else(|| panic!("expected hex string for {field}, got {value}"));
    U256::from_str_radix(hex.trim_start_matches("0x"), 16)
        .unwrap_or_else(|err| panic!("failed to parse {field} as U256 from {hex}: {err}"))
}

fn parse_rpc_address(value: &serde_json::Value, field: &str) -> Address {
    let text = value
        .as_str()
        .unwrap_or_else(|| panic!("expected hex string for {field}, got {value}"));
    text.parse()
        .unwrap_or_else(|err| panic!("failed to parse {field} address {text}: {err}"))
}

fn validator_entries(pubkeys: &[ed25519::PublicKey]) -> Vec<ValidatorEntry> {
    pubkeys
        .iter()
        .enumerate()
        .map(|(i, pubkey)| ValidatorEntry {
            consensus_pubkey: pubkey.as_ref().try_into().expect("ed25519 key length"),
            ethereum_address: Address::repeat_byte((i + 1) as u8),
        })
        .collect()
}

fn build_funded_chain_spec(
    funded_address: Address,
    balance: U256,
    simplex_pubkeys: &[ed25519::PublicKey],
) -> ChainSpec {
    let mut alloc = BTreeMap::new();
    alloc.insert(
        funded_address,
        GenesisAccount {
            balance,
            ..GenesisAccount::default()
        },
    );
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        alloc,
        BTreeMap::new(),
        validator_entries(simplex_pubkeys),
    )
}

fn start_funded_node(seed: u64, funded_address: Address, balance: U256) -> (NodeHandle, TempDir) {
    let tempdir = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for funded node {seed}: {err}"));
    let public_key = ed25519::PrivateKey::from_seed(seed).public_key();
    let chain_spec = build_funded_chain_spec(funded_address, balance, &[public_key.clone()]);
    let p2p_port = allocate_port();
    let rpc_port = allocate_port();
    let p2p_addr: SocketAddr = format!("127.0.0.1:{p2p_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse p2p addr for seed {seed}: {err}"));
    let rpc_addr: SocketAddr = format!("127.0.0.1:{rpc_port}")
        .parse()
        .unwrap_or_else(|err| panic!("failed to parse rpc addr for seed {seed}: {err}"));

    let config = NodeConfig {
        network: NetworkConfig {
            namespace: format!("precompile-test-{seed}").into_bytes(),
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
            namespace: format!("precompile-test-consensus-{seed}").into_bytes(),
            block_interval: Duration::from_secs(1),
        },
        bootstrap_validators: Some(vec![public_key.clone()]),
    };

    let handle = start_node_with_chain_spec(config, Some(std::sync::Arc::new(chain_spec)))
        .unwrap_or_else(|err| panic!("failed to start funded node {seed}: {err}"));
    assert_eq!(handle.public_key, public_key);

    (handle, tempdir)
}

struct MultiNodeTestNetwork {
    handles: Vec<NodeHandle>,
    _tempdirs: Vec<TempDir>,
}

fn start_multinode_test_network(
    seeds: &[u64],
    funded_address: Address,
    balance: U256,
) -> MultiNodeTestNetwork {
    let validator_keys: Vec<_> = seeds
        .iter()
        .map(|seed| ed25519::PrivateKey::from_seed(*seed))
        .collect();
    let validator_pubkeys: Vec<_> = validator_keys.iter().map(|key| key.public_key()).collect();
    let chain_spec = std::sync::Arc::new(build_funded_chain_spec(
        funded_address,
        balance,
        &validator_pubkeys,
    ));
    let p2p_ports: Vec<u16> = (0..seeds.len()).map(|_| allocate_port()).collect();
    let rpc_ports: Vec<u16> = (0..seeds.len()).map(|_| allocate_port()).collect();
    let tempdirs: Vec<_> = (0..seeds.len())
        .map(|_| TempDir::new().expect("failed to create multi-node temp dir"))
        .collect();

    let mut handles = Vec::with_capacity(seeds.len());
    for (i, seed) in seeds.iter().enumerate() {
        let bootstrap_peers = (0..seeds.len())
            .filter(|&j| j != i)
            .map(|j| {
                let pk_hex = hex::encode(validator_pubkeys[j].as_ref());
                parse_bootstrap_peer(&format!("{pk_hex}@127.0.0.1:{}", p2p_ports[j]))
                    .expect("bootstrap peer")
            })
            .collect();

        let config = NodeConfig {
            network: NetworkConfig {
                namespace: b"precompile-test-multinode".to_vec(),
                listen_addr: format!("127.0.0.1:{}", p2p_ports[i]).parse().unwrap(),
                dialable_addr: format!("127.0.0.1:{}", p2p_ports[i]).parse().unwrap(),
                bootstrap_peers,
                max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            },
            identity: IdentityConfig { seed: *seed },
            rpc: NodeRpcConfig {
                bind_addr: format!("127.0.0.1:{}", rpc_ports[i]).parse().unwrap(),
                mem_bind_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)),
            },
            storage: StorageConfig {
                data_dir: tempdirs[i].path().to_path_buf(),
            },
            consensus: ConsensusStartupConfig {
                namespace: b"precompile-test-multinode-consensus".to_vec(),
                block_interval: Duration::from_secs(1),
            },
            bootstrap_validators: Some(validator_pubkeys.clone()),
        };

        handles.push(
            start_node_with_chain_spec(config, Some(chain_spec.clone()))
                .unwrap_or_else(|err| panic!("failed to start multi-node validator {seed}: {err}")),
        );
    }

    MultiNodeTestNetwork {
        handles,
        _tempdirs: tempdirs,
    }
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

async fn wait_for_receipt_on_any(
    rpc_addrs: &[SocketAddr],
    tx_hash: B256,
    timeout: Duration,
) -> (SocketAddr, serde_json::Value) {
    let client = test_client();
    let deadline = Instant::now() + timeout;
    let tx_hash_hex = format!("0x{tx_hash:x}");

    loop {
        for rpc_addr in rpc_addrs {
            let response = post_json_to_addr(
                client,
                *rpc_addr,
                rpc_req(
                    "eth_getTransactionReceipt",
                    serde_json::json!([tx_hash_hex.clone()]),
                ),
            )
            .await;

            if response["error"].is_object() {
                panic!(
                    "eth_getTransactionReceipt returned an error for {tx_hash:#x} on {rpc_addr}: {response}"
                );
            }

            if !response["result"].is_null() {
                return (*rpc_addr, response["result"].clone());
            }
        }

        if Instant::now() >= deadline {
            panic!(
                "timed out after {:?} waiting for receipt {tx_hash:#x} across rpc nodes {:?}",
                timeout, rpc_addrs
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

fn precompile_proxy_deployment_bytecode() -> Bytes {
    let mut runtime = hex::decode("36600060003760006000366000600073")
        .expect("forwarder runtime prefix should decode");
    runtime.extend_from_slice(TEST_TOKEN_PRECOMPILE_ADDRESS.as_slice());
    runtime.extend_from_slice(
        &hex::decode("5af13d600060003e156034573d6000f35b3d6000fd")
            .expect("forwarder runtime suffix should decode"),
    );

    let mut init =
        hex::decode("6039600c60003960396000f3").expect("forwarder init prefix should decode");
    init.extend_from_slice(&runtime);
    Bytes::from(init)
}

async fn deploy_precompile_proxy(
    rpc_addr: SocketAddr,
    signer: &PrivateKeySigner,
    nonce: u64,
) -> (B256, serde_json::Value, Address) {
    let deploy_tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce,
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
        max_fee_per_gas: MAX_FEE_PER_GAS,
        gas_limit: PROXY_DEPLOY_GAS_LIMIT,
        to: alloy_primitives::TxKind::Create,
        value: U256::ZERO,
        access_list: Default::default(),
        input: precompile_proxy_deployment_bytecode(),
    };

    let raw_tx = sign_eip1559_tx(signer, deploy_tx).await;
    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;
    assert_eq!(
        receipt["status"].as_str(),
        Some("0x1"),
        "proxy deployment should succeed: {receipt}"
    );
    let contract_address = parse_rpc_address(&receipt["contractAddress"], "contractAddress");
    (tx_hash, receipt, contract_address)
}

async fn submit_raw_tx_to_network(
    rpc_addrs: &[SocketAddr],
    raw_tx: &[u8],
) -> (B256, serde_json::Value) {
    let tx_hash = send_raw_tx(rpc_addrs[0], raw_tx).await;
    let client = test_client();
    for rpc_addr in &rpc_addrs[1..] {
        let response = post_json_to_addr(
            client,
            *rpc_addr,
            rpc_req(
                "eth_sendRawTransaction",
                serde_json::json!([raw_tx_hex(raw_tx)]),
            ),
        )
        .await;

        if response["error"].is_object() {
            let message = response["error"]["message"].as_str().unwrap_or_default();
            if !message.contains("already")
                && !message.contains("known")
                && !message.contains("nonce")
            {
                panic!(
                    "eth_sendRawTransaction failed for {} on {rpc_addr}: {response}",
                    raw_tx_hex(raw_tx)
                );
            }
        } else {
            let echoed_hash = parse_rpc_b256(&response["result"], "eth_sendRawTransaction result");
            assert_eq!(
                echoed_hash, tx_hash,
                "all nodes should return the same tx hash"
            );
        }
    }

    let (_receipt_addr, receipt) =
        wait_for_receipt_on_any(rpc_addrs, tx_hash, Duration::from_secs(60)).await;
    (tx_hash, receipt)
}

async fn precompile_balance_of(
    rpc_addr: SocketAddr,
    proxy_address: Address,
    owner: Address,
) -> U256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_call",
            serde_json::json!([
                {
                    "to": proxy_address,
                    "data": raw_tx_hex(balance_of_calldata(owner).as_ref()),
                },
                "latest"
            ]),
        ),
    )
    .await;

    if response["error"].is_object() {
        panic!("eth_call balanceOf failed: {response}");
    }

    parse_rpc_u256_hex(&response["result"], "eth_call result")
}

async fn account_balance(rpc_addr: SocketAddr, owner: Address) -> U256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getBalance", serde_json::json!([owner, "latest"])),
    )
    .await;

    if response["error"].is_object() {
        panic!("eth_getBalance failed: {response}");
    }

    parse_rpc_u256_hex(&response["result"], "eth_getBalance result")
}

async fn wait_for_precompile_balance(
    rpc_addr: SocketAddr,
    proxy_address: Address,
    owner: Address,
    expected: U256,
    timeout: Duration,
) -> U256 {
    let deadline = Instant::now() + timeout;
    loop {
        let balance = precompile_balance_of(rpc_addr, proxy_address, owner).await;
        if balance == expected {
            return balance;
        }

        if Instant::now() >= deadline {
            panic!(
                "timed out after {:?} waiting for precompile balance {} on {rpc_addr}; last balance was {}",
                timeout, expected, balance
            );
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_test_token_state_changing_tx_full_node() {
    let signer = PrivateKeySigner::from_slice(&[0x11; 32]).expect("signer");
    let recipient = Address::repeat_byte(0x44);
    let hundred_eth = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(400, signer.address(), hundred_eth);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;
    let (_deploy_hash, _deploy_receipt, proxy_address) =
        deploy_precompile_proxy(rpc_addr, &signer, 0).await;
    assert_eq!(
        precompile_balance_of(rpc_addr, proxy_address, recipient).await,
        U256::ZERO
    );

    let raw_tx = sign_eip1559_tx(
        &signer,
        TxEip1559 {
            chain_id: SAHARA_CHAIN_ID,
            nonce: 1,
            max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
            max_fee_per_gas: MAX_FEE_PER_GAS,
            gas_limit: PRECOMPILE_CALL_GAS_LIMIT,
            to: proxy_address.into(),
            value: U256::ZERO,
            access_list: Default::default(),
            input: mint_calldata(recipient, U256::from(7_u64)),
        },
    )
    .await;

    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;

    assert_eq!(
        receipt["status"].as_str(),
        Some("0x1"),
        "precompile mint tx should succeed: {receipt}"
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let minted_balance = account_balance(rpc_addr, recipient).await;
        if minted_balance == U256::from(7_u64) {
            break;
        }
        if Instant::now() >= deadline {
            panic!(
                "timed out after 30s waiting for minted balance 7 on {rpc_addr}; last balance was {minted_balance}"
            );
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert_eq!(
        account_balance(rpc_addr, recipient).await,
        U256::from(7_u64)
    );
    assert!(
        parse_rpc_u64(&receipt["gasUsed"], "receipt.gasUsed") >= 21_000,
        "gas used should include execution costs: {receipt}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_test_token_revert_surface_full_node() {
    let signer = PrivateKeySigner::from_slice(&[0x22; 32]).expect("signer");
    let recipient = Address::repeat_byte(0x55);
    let hundred_eth = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(401, signer.address(), hundred_eth);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;
    let (_deploy_hash, _deploy_receipt, proxy_address) =
        deploy_precompile_proxy(rpc_addr, &signer, 0).await;

    let raw_tx = sign_eip1559_tx(
        &signer,
        TxEip1559 {
            chain_id: SAHARA_CHAIN_ID,
            nonce: 1,
            max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
            max_fee_per_gas: MAX_FEE_PER_GAS,
            gas_limit: PRECOMPILE_CALL_GAS_LIMIT,
            to: proxy_address.into(),
            value: U256::ZERO,
            access_list: Default::default(),
            input: mint_calldata(recipient, U256::ZERO),
        },
    )
    .await;

    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;

    assert_eq!(
        receipt["status"].as_str(),
        Some("0x0"),
        "zero-amount mint should revert: {receipt}"
    );
    assert_eq!(
        precompile_balance_of(rpc_addr, proxy_address, recipient).await,
        U256::ZERO
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_test_token_eth_call_read_path() {
    let signer = PrivateKeySigner::from_slice(&[0x33; 32]).expect("signer");
    let recipient = Address::repeat_byte(0x66);
    let hundred_eth = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(402, signer.address(), hundred_eth);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;
    let (_deploy_hash, _deploy_receipt, proxy_address) =
        deploy_precompile_proxy(rpc_addr, &signer, 0).await;

    let raw_tx = sign_eip1559_tx(
        &signer,
        TxEip1559 {
            chain_id: SAHARA_CHAIN_ID,
            nonce: 1,
            max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
            max_fee_per_gas: MAX_FEE_PER_GAS,
            gas_limit: PRECOMPILE_CALL_GAS_LIMIT,
            to: proxy_address.into(),
            value: U256::ZERO,
            access_list: Default::default(),
            input: mint_calldata(recipient, U256::from(9_u64)),
        },
    )
    .await;

    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let _receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;

    let balance = wait_for_precompile_balance(
        rpc_addr,
        proxy_address,
        recipient,
        U256::from(9_u64),
        Duration::from_secs(30),
    )
    .await;
    assert_eq!(balance, U256::from(9_u64));
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "multi-node teardown is currently unstable; verification-path symmetry is covered in app-evm unit tests"]
async fn test_precompile_framework_is_available_in_verification_path() {
    let signer = PrivateKeySigner::from_slice(&[0x44; 32]).expect("signer");
    let recipient = Address::repeat_byte(0x77);
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let network = start_multinode_test_network(&[500, 501], signer.address(), funded_balance);
    let rpc_addrs: Vec<_> = network
        .handles
        .iter()
        .map(|handle| handle.rpc_addr)
        .collect();

    for rpc_addr in &rpc_addrs {
        wait_for_block(*rpc_addr, 1, Duration::from_secs(60)).await;
    }

    let deploy_raw_tx = sign_eip1559_tx(
        &signer,
        TxEip1559 {
            chain_id: SAHARA_CHAIN_ID,
            nonce: 0,
            max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
            max_fee_per_gas: MAX_FEE_PER_GAS,
            gas_limit: PROXY_DEPLOY_GAS_LIMIT,
            to: alloy_primitives::TxKind::Create,
            value: U256::ZERO,
            access_list: Default::default(),
            input: precompile_proxy_deployment_bytecode(),
        },
    )
    .await;
    let (_deploy_hash, deploy_receipt) = submit_raw_tx_to_network(&rpc_addrs, &deploy_raw_tx).await;
    let proxy_address = parse_rpc_address(&deploy_receipt["contractAddress"], "contractAddress");
    let deploy_block = parse_rpc_u64(&deploy_receipt["blockNumber"], "deploy receipt.blockNumber");
    for rpc_addr in &rpc_addrs {
        wait_for_block(*rpc_addr, deploy_block, Duration::from_secs(60)).await;
    }

    let mint_raw_tx = sign_eip1559_tx(
        &signer,
        TxEip1559 {
            chain_id: SAHARA_CHAIN_ID,
            nonce: 1,
            max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
            max_fee_per_gas: MAX_FEE_PER_GAS,
            gas_limit: PRECOMPILE_CALL_GAS_LIMIT,
            to: proxy_address.into(),
            value: U256::ZERO,
            access_list: Default::default(),
            input: mint_calldata(recipient, U256::from(11_u64)),
        },
    )
    .await;
    let (_tx_hash, receipt) = submit_raw_tx_to_network(&rpc_addrs, &mint_raw_tx).await;
    assert_eq!(
        receipt["status"].as_str(),
        Some("0x1"),
        "multi-node precompile tx should finalize successfully: {receipt}"
    );

    let receipt_block = parse_rpc_u64(&receipt["blockNumber"], "receipt.blockNumber");
    for rpc_addr in &rpc_addrs {
        wait_for_block(*rpc_addr, receipt_block, Duration::from_secs(60)).await;
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let minted_balance = account_balance(*rpc_addr, recipient).await;
            if minted_balance == U256::from(11_u64) {
                break;
            }
            if Instant::now() >= deadline {
                panic!(
                    "timed out after 30s waiting for minted balance 11 on {rpc_addr}; last balance was {minted_balance}"
                );
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert_eq!(
            account_balance(*rpc_addr, recipient).await,
            U256::from(11_u64),
            "all validators should persist the same minted balance"
        );
    }
}
