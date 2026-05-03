use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::{Duration, Instant};

use crate::common::encoding::raw_tx_hex;
use crate::common::http::{post_json_to_addr, rpc_req, test_client};
use crate::common::ports::allocate_port;
use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, Bytes, FixedBytes, B256, U256};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use chainspec::genesis::{build_sahara_chain_spec_from, SaharaGenesisConfig};
use chainspec::SAHARA_CHAIN_ID;
use commonware_cryptography::{ed25519, Signer as CwSigner};
use evm_precompiles::{
    claimable_balance_calldata, community_pool_balance_calldata, fee_pool_balance_calldata,
    withdraw_calldata, COMMUNITY_POOL_ADDRESS, FEE_POOL_PRECOMPILE_ADDRESS,
};
use reth_chainspec::ChainSpec;
use tempfile::TempDir;
use validators_reader::ValidatorEntry;
use whirlpool_node::config::{
    parse_bootstrap_peer, ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig,
    RpcConfig as NodeRpcConfig, StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

const TEST_PROPOSER_FEE_RECIPIENT: Address = Address::new([
    0x70, 0x72, 0x6f, 0x70, 0x6f, 0x73, 0x65, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x73, 0x65, 0x61,
    0x6d, 0x2d, 0x30, 0x31,
]);

const MAX_PRIORITY_FEE_PER_GAS: u128 = 1_000_000_000;
const MAX_FEE_PER_GAS: u128 = 20_000_000_000;
const TRANSFER_GAS_LIMIT: u64 = 21_000;

fn chain_spec_from_alloc_and_validators(
    alloc: BTreeMap<Address, GenesisAccount>,
    simplex_validators: Vec<ValidatorEntry>,
) -> ChainSpec {
    build_sahara_chain_spec_from(SaharaGenesisConfig {
        alloc,
        simplex_validators,
        ..SaharaGenesisConfig::default()
    })
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

fn validator_public_key_bytes(public_key: &ed25519::PublicKey) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(public_key.as_ref());
    bytes
}

fn block_reward_recipient(block: &serde_json::Value) -> Address {
    block
        .get("miner")
        .or_else(|| block.get("beneficiary"))
        .map(|value| parse_rpc_address(value, "block reward recipient"))
        .unwrap_or_else(|| panic!("block missing miner/beneficiary field: {block}"))
}

fn start_funded_node(
    seed: u64,
    funded_address: Address,
    balance: U256,
    validator_fee_recipient: Address,
) -> (NodeHandle, TempDir) {
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

    let chain_spec: ChainSpec = chain_spec_from_alloc_and_validators(
        alloc,
        vec![ValidatorEntry {
            consensus_pubkey: validator_public_key_bytes(&public_key),
            ethereum_address: validator_fee_recipient,
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
        bootstrap_validators: Some(vec![public_key.clone()]),
        bootstrap: Default::default(),
    };

    let handle = start_node_with_chain_spec(config, Some(std::sync::Arc::new(chain_spec)))
        .unwrap_or_else(|err| panic!("failed to start funded node {seed}: {err}"));
    assert_eq!(handle.public_key, public_key);

    (handle, tempdir)
}

struct MultiNodeFeeNetwork {
    handles: Vec<NodeHandle>,
    _tempdirs: Vec<TempDir>,
    fee_recipients: Vec<Address>,
}

fn start_multinode_fee_network(
    seeds_and_fee_recipients: &[(u64, Address)],
    funded_address: Address,
    balance: U256,
) -> MultiNodeFeeNetwork {
    let validator_keys: Vec<_> = seeds_and_fee_recipients
        .iter()
        .map(|(seed, _)| ed25519::PrivateKey::from_seed(*seed))
        .collect();
    let validator_pubkeys: Vec<_> = validator_keys.iter().map(|key| key.public_key()).collect();
    let fee_recipients: Vec<_> = seeds_and_fee_recipients
        .iter()
        .map(|(_, fee_recipient)| *fee_recipient)
        .collect();

    let mut alloc = BTreeMap::new();
    alloc.insert(
        funded_address,
        GenesisAccount {
            balance,
            ..GenesisAccount::default()
        },
    );

    let validator_registry_entries = validator_pubkeys
        .iter()
        .zip(fee_recipients.iter().copied())
        .map(|(public_key, fee_recipient)| ValidatorEntry {
            consensus_pubkey: validator_public_key_bytes(public_key),
            ethereum_address: fee_recipient,
        })
        .collect();
    let chain_spec = std::sync::Arc::new(chain_spec_from_alloc_and_validators(
        alloc,
        validator_registry_entries,
    ));

    let p2p_ports: Vec<u16> = (0..seeds_and_fee_recipients.len())
        .map(|_| allocate_port())
        .collect();
    let rpc_ports: Vec<u16> = (0..seeds_and_fee_recipients.len())
        .map(|_| allocate_port())
        .collect();
    let tempdirs: Vec<_> = (0..seeds_and_fee_recipients.len())
        .map(|_| TempDir::new().expect("failed to create multi-node temp dir"))
        .collect();

    let mut handles = Vec::with_capacity(seeds_and_fee_recipients.len());
    for (i, (seed, _fee_recipient)) in seeds_and_fee_recipients.iter().enumerate() {
        let bootstrap_peers = (0..seeds_and_fee_recipients.len())
            .filter(|&j| j != i)
            .map(|j| {
                let pk_hex = hex::encode(validator_pubkeys[j].as_ref());
                parse_bootstrap_peer(&format!("{pk_hex}@127.0.0.1:{}", p2p_ports[j]))
                    .expect("bootstrap peer")
            })
            .collect();

        let config = NodeConfig {
            network: NetworkConfig {
                namespace: b"community-pool-multinode".to_vec(),
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
                namespace: b"community-pool-multinode-consensus".to_vec(),
                block_interval: Duration::from_secs(1),
            },
            bootstrap_validators: Some(validator_pubkeys.clone()),
            bootstrap: Default::default(),
        };

        let handle = start_node_with_chain_spec(config, Some(chain_spec.clone()))
            .unwrap_or_else(|err| panic!("failed to start multi-node validator {seed}: {err}"));
        handles.push(handle);
    }

    MultiNodeFeeNetwork {
        handles,
        _tempdirs: tempdirs,
        fee_recipients,
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

async fn query_community_pool_balance_via_precompile(rpc_addr: SocketAddr) -> U256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_call",
            serde_json::json!([
                {
                    "to": COMMUNITY_POOL_ADDRESS,
                    "data": raw_tx_hex(community_pool_balance_calldata().as_ref()),
                },
                "latest"
            ]),
        ),
    )
    .await;
    assert!(
        response["error"].is_null() || response.get("error").is_none(),
        "community-pool precompile eth_call should succeed: {response}"
    );
    parse_rpc_u256(
        &response["result"],
        "community-pool precompile eth_call result",
    )
}

async fn query_fee_pool_balance_via_precompile(rpc_addr: SocketAddr) -> U256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_call",
            serde_json::json!([
                {
                    "to": FEE_POOL_PRECOMPILE_ADDRESS,
                    "data": raw_tx_hex(fee_pool_balance_calldata().as_ref()),
                },
                "latest"
            ]),
        ),
    )
    .await;
    assert!(
        response["error"].is_null() || response.get("error").is_none(),
        "fee-pool precompile eth_call should succeed: {response}"
    );
    parse_rpc_u256(&response["result"], "fee-pool precompile eth_call result")
}

async fn query_fee_pool_claimable_via_precompile(rpc_addr: SocketAddr, recipient: Address) -> U256 {
    let client = test_client();
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_call",
            serde_json::json!([
                {
                    "to": FEE_POOL_PRECOMPILE_ADDRESS,
                    "data": raw_tx_hex(claimable_balance_calldata(recipient).as_ref()),
                },
                "latest"
            ]),
        ),
    )
    .await;
    assert!(
        response["error"].is_null() || response.get("error").is_none(),
        "fee-pool claimable precompile eth_call should succeed: {response}"
    );
    parse_rpc_u256(&response["result"], "fee-pool claimable eth_call result")
}

async fn build_fee_only_transfer_raw_tx(signer: &PrivateKeySigner) -> Vec<u8> {
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

    sign_eip1559_tx(signer, tx).await
}

async fn build_fee_pool_withdraw_raw_tx(signer: &PrivateKeySigner, nonce: u64) -> Vec<u8> {
    let tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce,
        max_priority_fee_per_gas: 0,
        max_fee_per_gas: MAX_FEE_PER_GAS,
        gas_limit: 100_000,
        to: alloy_primitives::TxKind::Call(FEE_POOL_PRECOMPILE_ADDRESS),
        value: U256::ZERO,
        access_list: Default::default(),
        input: withdraw_calldata(),
    };

    sign_eip1559_tx(signer, tx).await
}

async fn submit_fee_only_transfer(
    rpc_addr: SocketAddr,
    signer: &PrivateKeySigner,
) -> (B256, serde_json::Value, serde_json::Value) {
    let raw_tx = build_fee_only_transfer_raw_tx(signer).await;
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

async fn submit_fee_only_transfer_to_network(
    rpc_addrs: &[SocketAddr],
    signer: &PrivateKeySigner,
) -> (B256, serde_json::Value, serde_json::Value) {
    let raw_tx = build_fee_only_transfer_raw_tx(signer).await;
    let tx_hash = send_raw_tx(rpc_addrs[0], &raw_tx).await;

    let client = test_client();
    for rpc_addr in &rpc_addrs[1..] {
        let response = post_json_to_addr(
            client,
            *rpc_addr,
            rpc_req(
                "eth_sendRawTransaction",
                serde_json::json!([raw_tx_hex(&raw_tx)]),
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
                    raw_tx_hex(&raw_tx)
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

    let (receipt_rpc_addr, receipt) =
        wait_for_receipt_on_any(rpc_addrs, tx_hash, Duration::from_secs(60)).await;
    let block_number = receipt["blockNumber"]
        .as_str()
        .unwrap_or_else(|| panic!("receipt missing blockNumber: {receipt}"));
    let block_response = post_json_to_addr(
        client,
        receipt_rpc_addr,
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

    (tx_hash, receipt, block_response["result"].clone())
}

#[tokio::test(flavor = "current_thread")]
async fn test_community_pool_accrues_burned_amount_from_fee_only_transfer() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let configured_fee_recipient = Address::repeat_byte(0x31);
    let (handle, _tempdir) =
        start_funded_node(300, sender, funded_balance, configured_fee_recipient);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let initial_zero_balance = query_balance(rpc_addr, Address::ZERO).await;
    let initial_community_pool_balance = query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await;
    let initial_precompile_balance = query_community_pool_balance_via_precompile(rpc_addr).await;
    assert_eq!(
        initial_precompile_balance, initial_community_pool_balance,
        "precompile getter should match community pool account balance before tx"
    );
    let (_tx_hash, receipt, block) = submit_fee_only_transfer(rpc_addr, &signer).await;
    let final_zero_balance = query_balance(rpc_addr, Address::ZERO).await;
    let final_community_pool_balance = query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await;
    let final_precompile_balance = query_community_pool_balance_via_precompile(rpc_addr).await;

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
    assert_eq!(
        final_zero_balance, initial_zero_balance,
        "controlled fee-only transfer should not change Address::ZERO balance"
    );
    assert_eq!(
        final_precompile_balance, final_community_pool_balance,
        "precompile getter should match community pool account balance after tx"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_proposer_fee_recipient_metadata_survives_while_priority_fees_accrue_to_fee_pool() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let configured_fee_recipient = Address::repeat_byte(0x42);
    let (handle, _tempdir) =
        start_funded_node(301, sender, funded_balance, configured_fee_recipient);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let initial_fee_recipient_balance = query_balance(rpc_addr, configured_fee_recipient).await;
    let initial_legacy_fee_recipient_balance =
        query_balance(rpc_addr, TEST_PROPOSER_FEE_RECIPIENT).await;
    let initial_fee_pool_balance = query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await;
    let initial_fee_pool_precompile_balance = query_fee_pool_balance_via_precompile(rpc_addr).await;
    let initial_claimable =
        query_fee_pool_claimable_via_precompile(rpc_addr, configured_fee_recipient).await;
    let (_tx_hash, receipt, block) = submit_fee_only_transfer(rpc_addr, &signer).await;
    let final_fee_recipient_balance = query_balance(rpc_addr, configured_fee_recipient).await;
    let final_legacy_fee_recipient_balance =
        query_balance(rpc_addr, TEST_PROPOSER_FEE_RECIPIENT).await;
    let final_fee_pool_balance = query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await;
    let final_fee_pool_precompile_balance = query_fee_pool_balance_via_precompile(rpc_addr).await;
    let final_claimable =
        query_fee_pool_claimable_via_precompile(rpc_addr, configured_fee_recipient).await;

    let block_gas_used = parse_rpc_u256(&block["gasUsed"], "block gasUsed");
    let receipt_gas_used = parse_rpc_u256(&receipt["gasUsed"], "receipt gasUsed");
    assert_eq!(
        block_gas_used, receipt_gas_used,
        "expected a single-tx block"
    );

    let expected_priority_fees = block_gas_used * U256::from(MAX_PRIORITY_FEE_PER_GAS);
    assert_eq!(
        block_reward_recipient(&block),
        configured_fee_recipient,
        "block should expose the configured proposer reward recipient"
    );
    assert_eq!(
        final_fee_recipient_balance - initial_fee_recipient_balance,
        U256::ZERO,
        "configured proposer recipient account should not be directly credited"
    );
    assert_eq!(
        final_legacy_fee_recipient_balance - initial_legacy_fee_recipient_balance,
        U256::ZERO,
        "legacy hardcoded fee recipient should not receive the priority-fee portion"
    );
    assert_eq!(
        final_fee_pool_balance - initial_fee_pool_balance,
        expected_priority_fees,
        "fee-pool sink account should accrue the priority-fee portion exactly once"
    );
    assert_eq!(
        final_claimable - initial_claimable,
        expected_priority_fees,
        "configured proposer recipient should accrue claimable fee-pool balance"
    );
    assert_eq!(
        initial_fee_pool_precompile_balance, initial_fee_pool_balance,
        "fee-pool precompile getter should match fee-pool account balance before tx"
    );
    assert_eq!(
        final_fee_pool_precompile_balance, final_fee_pool_balance,
        "fee-pool precompile getter should match fee-pool account balance after tx"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_fee_pool_withdraw_transfers_claimable_balance_and_clears_slot() {
    let signer = PrivateKeySigner::random();
    let signer_addr = signer.address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let (handle, _tempdir) = start_funded_node(302, signer_addr, funded_balance, signer_addr);
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let initial_fee_pool_balance = query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await;
    let initial_claimable = query_fee_pool_claimable_via_precompile(rpc_addr, signer_addr).await;
    let (_tx_hash, receipt, block) = submit_fee_only_transfer(rpc_addr, &signer).await;
    let block_gas_used = parse_rpc_u256(&block["gasUsed"], "block gasUsed");
    let receipt_gas_used = parse_rpc_u256(&receipt["gasUsed"], "receipt gasUsed");
    assert_eq!(
        block_gas_used, receipt_gas_used,
        "expected a single-tx block"
    );

    let expected_priority_fees = block_gas_used * U256::from(MAX_PRIORITY_FEE_PER_GAS);
    let accrued_claimable = query_fee_pool_claimable_via_precompile(rpc_addr, signer_addr).await;
    let fee_pool_before_withdraw = query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await;
    assert_eq!(
        accrued_claimable - initial_claimable,
        expected_priority_fees,
        "fee-only transfer should accrue claimable balance for proposer recipient"
    );
    assert_eq!(
        fee_pool_before_withdraw - initial_fee_pool_balance,
        expected_priority_fees,
        "fee-only transfer should credit fee-pool sink by priority-fee amount"
    );

    let withdraw_raw_tx = build_fee_pool_withdraw_raw_tx(&signer, 1).await;
    let withdraw_hash = send_raw_tx(rpc_addr, &withdraw_raw_tx).await;
    let withdraw_receipt = wait_for_receipt(rpc_addr, withdraw_hash, Duration::from_secs(30)).await;
    assert_eq!(
        withdraw_receipt["status"].as_str(),
        Some("0x1"),
        "withdraw transaction should succeed: {withdraw_receipt}"
    );

    let claimable_after = query_fee_pool_claimable_via_precompile(rpc_addr, signer_addr).await;
    let fee_pool_after_withdraw = query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await;
    let fee_pool_precompile_after = query_fee_pool_balance_via_precompile(rpc_addr).await;
    assert_eq!(
        claimable_after,
        U256::ZERO,
        "withdraw should clear claimable slot when tx tip is zero"
    );
    assert_eq!(
        fee_pool_before_withdraw - fee_pool_after_withdraw,
        accrued_claimable,
        "withdraw should transfer exactly accrued claimable amount out of fee-pool sink"
    );
    assert_eq!(
        fee_pool_precompile_after, fee_pool_after_withdraw,
        "fee-pool precompile getter should match account balance after withdraw"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_multivalidator_priority_fee_follows_actual_proposer() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);
    let network = start_multinode_fee_network(
        &[
            (400, Address::repeat_byte(0x91)),
            (401, Address::repeat_byte(0x92)),
            (402, Address::repeat_byte(0x93)),
        ],
        sender,
        funded_balance,
    );

    for handle in &network.handles {
        wait_for_block(handle.rpc_addr, 1, Duration::from_secs(60)).await;
    }

    let mut initial_community_pool_balances = Vec::with_capacity(network.handles.len());
    let mut initial_fee_pool_balances = Vec::with_capacity(network.handles.len());
    let mut initial_zero_balances = Vec::with_capacity(network.handles.len());
    let mut initial_legacy_fee_recipient_balances = Vec::with_capacity(network.handles.len());
    let mut initial_fee_recipient_balances_by_node = Vec::with_capacity(network.handles.len());
    let mut initial_claimable_balances_by_node = Vec::with_capacity(network.handles.len());
    for handle in &network.handles {
        initial_zero_balances.push(query_balance(handle.rpc_addr, Address::ZERO).await);
        initial_community_pool_balances
            .push(query_balance(handle.rpc_addr, COMMUNITY_POOL_ADDRESS).await);
        initial_fee_pool_balances
            .push(query_balance(handle.rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await);
        initial_legacy_fee_recipient_balances
            .push(query_balance(handle.rpc_addr, TEST_PROPOSER_FEE_RECIPIENT).await);

        let mut balances = Vec::with_capacity(network.fee_recipients.len());
        for fee_recipient in &network.fee_recipients {
            balances.push(query_balance(handle.rpc_addr, *fee_recipient).await);
        }
        initial_fee_recipient_balances_by_node.push(balances);

        let mut claimables = Vec::with_capacity(network.fee_recipients.len());
        for fee_recipient in &network.fee_recipients {
            claimables.push(
                query_fee_pool_claimable_via_precompile(handle.rpc_addr, *fee_recipient).await,
            );
        }
        initial_claimable_balances_by_node.push(claimables);
    }

    let rpc_addrs: Vec<_> = network
        .handles
        .iter()
        .map(|handle| handle.rpc_addr)
        .collect();
    let (_tx_hash, receipt, block) = submit_fee_only_transfer_to_network(&rpc_addrs, &signer).await;

    let block_gas_used = parse_rpc_u256(&block["gasUsed"], "block gasUsed");
    let receipt_gas_used = parse_rpc_u256(&receipt["gasUsed"], "receipt gasUsed");
    assert_eq!(
        block_gas_used, receipt_gas_used,
        "expected a single-tx block"
    );

    let expected_priority_fees = block_gas_used * U256::from(MAX_PRIORITY_FEE_PER_GAS);
    let expected_burned_amount =
        block_gas_used * parse_rpc_u256(&block["baseFeePerGas"], "block baseFeePerGas");
    let rewarded_recipient = block_reward_recipient(&block);
    assert!(
        network.fee_recipients.contains(&rewarded_recipient),
        "rewarded recipient {rewarded_recipient} should be one of the configured validator recipients"
    );

    let rewarded_index = network
        .fee_recipients
        .iter()
        .position(|fee_recipient| *fee_recipient == rewarded_recipient)
        .expect("rewarded recipient must be configured");
    let proposer_rpc_addr = network.handles[rewarded_index].rpc_addr;

    let final_community_pool_balance =
        query_balance(proposer_rpc_addr, COMMUNITY_POOL_ADDRESS).await;
    let final_fee_pool_balance =
        query_balance(proposer_rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await;
    let final_fee_pool_precompile_balance =
        query_fee_pool_balance_via_precompile(proposer_rpc_addr).await;
    let final_zero_balance = query_balance(proposer_rpc_addr, Address::ZERO).await;
    let final_precompile_balance =
        query_community_pool_balance_via_precompile(proposer_rpc_addr).await;
    let final_legacy_fee_recipient_balance =
        query_balance(proposer_rpc_addr, TEST_PROPOSER_FEE_RECIPIENT).await;
    let mut final_fee_recipient_balances = Vec::with_capacity(network.fee_recipients.len());
    for fee_recipient in &network.fee_recipients {
        final_fee_recipient_balances.push(query_balance(proposer_rpc_addr, *fee_recipient).await);
    }
    let balance_deltas: Vec<_> = final_fee_recipient_balances
        .iter()
        .zip(initial_fee_recipient_balances_by_node[rewarded_index].iter())
        .map(|(final_balance, initial_balance)| *final_balance - *initial_balance)
        .collect();
    let mut final_claimable_by_recipient = Vec::with_capacity(network.fee_recipients.len());
    for recipient in &network.fee_recipients {
        final_claimable_by_recipient
            .push(query_fee_pool_claimable_via_precompile(proposer_rpc_addr, *recipient).await);
    }
    let claimable_deltas: Vec<_> = final_claimable_by_recipient
        .iter()
        .zip(initial_claimable_balances_by_node[rewarded_index].iter())
        .map(|(final_claim, initial_claim)| *final_claim - *initial_claim)
        .collect();

    assert_eq!(
        balance_deltas[rewarded_index],
        U256::ZERO,
        "actual proposer's account should not receive direct priority-fee credit"
    );
    for (index, balance_delta) in balance_deltas.iter().enumerate() {
        assert_eq!(
            *balance_delta,
            U256::ZERO,
            "validator recipient account at index {index} should not receive direct priority-fee credit"
        );
    }
    assert_eq!(
        claimable_deltas[rewarded_index], expected_priority_fees,
        "actual proposer's configured recipient should accrue claimable priority fees"
    );
    for (index, claimable_delta) in claimable_deltas.iter().enumerate() {
        if index != rewarded_index {
            assert_eq!(
                *claimable_delta,
                U256::ZERO,
                "non-proposer validator recipient at index {index} should not accrue this block's claimable priority fees"
            );
        }
    }
    assert_eq!(
        final_legacy_fee_recipient_balance - initial_legacy_fee_recipient_balances[rewarded_index],
        U256::ZERO,
        "legacy hardcoded fee recipient should not receive the rewarded block's priority fee"
    );
    assert_eq!(
        final_community_pool_balance - initial_community_pool_balances[rewarded_index],
        expected_burned_amount,
        "community pool should still accrue the burned amount in multi-validator mode"
    );
    assert_eq!(
        final_fee_pool_balance - initial_fee_pool_balances[rewarded_index],
        expected_priority_fees,
        "fee-pool sink should accrue the rewarded block's priority-fee amount"
    );
    assert_eq!(
        final_fee_pool_precompile_balance, final_fee_pool_balance,
        "fee-pool precompile getter should match fee-pool account balance in multi-validator mode"
    );
    assert_eq!(
        final_zero_balance, initial_zero_balances[rewarded_index],
        "controlled fee-only transfer should not change Address::ZERO balance in multi-validator mode"
    );
    assert_eq!(
        final_precompile_balance, final_community_pool_balance,
        "precompile getter should match community pool account balance in multi-validator mode"
    );
}
