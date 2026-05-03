use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::common::encoding::raw_tx_hex;
use crate::common::http::{post_json_to_addr, rpc_req, test_client};
use crate::common::ports::allocate_port;
use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{Address, Bytes, FixedBytes, TxKind, B256, U256};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use chainspec::genesis::{build_sahara_chain_spec_from, SaharaGenesisConfig};
use chainspec::native_token::sahara_hard_cap_base_units;
use chainspec::SAHARA_CHAIN_ID;
use commonware_cryptography::{ed25519, Signer as CwSigner};
use evm_precompiles::{
    community_pool_balance_calldata, fee_pool_balance_calldata, COMMUNITY_POOL_ADDRESS,
    FEE_POOL_PRECOMPILE_ADDRESS,
};
use reth_chainspec::{Chain, ChainSpec, ChainSpecBuilder};
use tempfile::TempDir;
use validators_reader::{
    encode_validator_registry_storage, ValidatorEntry, SIMPLEX_VALIDATORS_REGISTRY,
};
use whirlpool_node::config::{
    ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig, RpcConfig as NodeRpcConfig,
    StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

const TEST_PROPOSER_FEE_RECIPIENT: Address = Address::new([
    0x70, 0x72, 0x6f, 0x70, 0x6f, 0x73, 0x65, 0x72, 0x2d, 0x66, 0x65, 0x65, 0x2d, 0x73, 0x65, 0x61,
    0x6d, 0x2d, 0x30, 0x31,
]);

const MAX_PRIORITY_FEE_PER_GAS: u128 = 1_000_000_000;
const MAX_FEE_PER_GAS: u128 = 20_000_000_000;
const TRANSFER_GAS_LIMIT: u64 = 21_000;

fn chain_spec_from_alloc(alloc: BTreeMap<Address, GenesisAccount>) -> ChainSpec {
    build_sahara_chain_spec_from(SaharaGenesisConfig {
        alloc,
        ..SaharaGenesisConfig::default()
    })
}

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

fn validator_public_key_bytes(public_key: &ed25519::PublicKey) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(public_key.as_ref());
    bytes
}

fn manual_chain_spec_with_alloc(alloc: BTreeMap<Address, GenesisAccount>) -> ChainSpec {
    ChainSpecBuilder::default()
        .chain(Chain::from_id(SAHARA_CHAIN_ID))
        .genesis(Genesis {
            gas_limit: 30_000_000,
            difficulty: U256::ZERO,
            alloc,
            ..Default::default()
        })
        .cancun_activated()
        .build()
}

fn start_node_for_chain_spec(
    seed: u64,
    mut chain_spec: ChainSpec,
) -> Result<(NodeHandle, TempDir), String> {
    let tempdir = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for funded node {seed}: {err}"));
    let validator_key = ed25519::PrivateKey::from_seed(seed);
    let public_key = validator_key.public_key();
    chain_spec
        .genesis
        .alloc
        .entry(SIMPLEX_VALIDATORS_REGISTRY)
        .or_insert_with(|| GenesisAccount {
            balance: U256::ZERO,
            storage: Some(encode_validator_registry_storage(&[ValidatorEntry {
                consensus_pubkey: validator_public_key_bytes(&public_key),
                ethereum_address: Address::repeat_byte(1),
            }])),
            ..GenesisAccount::default()
        });
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
            namespace: format!("native-token-supply-{seed}").into_bytes(),
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
            namespace: format!("native-token-supply-{seed}").into_bytes(),
            block_interval: Duration::from_secs(1),
        },
        bootstrap_validators: Some(vec![public_key]),
        bootstrap: Default::default(),
    };

    start_node_with_chain_spec(config, Some(Arc::new(chain_spec)))
        .map(|handle| (handle, tempdir))
        .map_err(|err| err.to_string())
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

async fn get_block_by_number(rpc_addr: SocketAddr, block_number: u64) -> serde_json::Value {
    let client = test_client();
    let block_tag = format!("0x{block_number:x}");
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req(
            "eth_getBlockByNumber",
            serde_json::json!([block_tag, false]),
        ),
    )
    .await;
    assert!(
        response["error"].is_null() || response.get("error").is_none(),
        "eth_getBlockByNumber should succeed: {response}"
    );
    response["result"].clone()
}

fn sum_balances(balances: &[U256]) -> U256 {
    balances
        .iter()
        .copied()
        .fold(U256::ZERO, |total, balance| total + balance)
}

#[tokio::test(flavor = "current_thread")]
async fn test_rejects_over_cap_genesis_allocation() {
    let signer = PrivateKeySigner::random();
    let mut alloc = BTreeMap::new();
    alloc.insert(
        signer.address(),
        GenesisAccount {
            balance: sahara_hard_cap_base_units() + U256::from(1u64),
            ..GenesisAccount::default()
        },
    );

    let err = match start_node_for_chain_spec(400, manual_chain_spec_with_alloc(alloc)) {
        Ok(_) => panic!("over-cap startup should fail"),
        Err(err) => err,
    };
    assert!(
        err.contains("hard cap exceeded"),
        "startup failure should mention the supply violation: {err}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_accepts_exact_cap_genesis_allocation() {
    let signer = PrivateKeySigner::random();
    let secondary = PrivateKeySigner::random().address();
    let primary_balance = sahara_hard_cap_base_units() - U256::from(1u64);

    let mut alloc = BTreeMap::new();
    alloc.insert(
        signer.address(),
        GenesisAccount {
            balance: primary_balance,
            ..GenesisAccount::default()
        },
    );
    alloc.insert(
        secondary,
        GenesisAccount {
            balance: U256::from(1u64),
            ..GenesisAccount::default()
        },
    );

    let chain_spec = chain_spec_from_alloc(alloc);
    let (handle, _tempdir) =
        start_node_for_chain_spec(401, chain_spec).expect("exact-cap startup should succeed");

    wait_for_block(handle.rpc_addr, 1, Duration::from_secs(30)).await;

    let primary = query_balance(handle.rpc_addr, signer.address()).await;
    let secondary_balance = query_balance(handle.rpc_addr, secondary).await;
    assert_eq!(primary, primary_balance);
    assert_eq!(secondary_balance, U256::from(1u64));
    assert_eq!(
        sum_balances(&[primary, secondary_balance]),
        sahara_hard_cap_base_units()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_direct_chain_spec_bypass_still_rejected() {
    let signer = PrivateKeySigner::random();
    let mut alloc = BTreeMap::new();
    alloc.insert(
        signer.address(),
        GenesisAccount {
            balance: sahara_hard_cap_base_units(),
            ..GenesisAccount::default()
        },
    );

    let mut chain_spec = chain_spec_from_alloc(alloc);
    chain_spec.genesis.alloc.insert(
        Address::repeat_byte(0xfe),
        GenesisAccount {
            balance: U256::from(1u64),
            ..GenesisAccount::default()
        },
    );

    let err = match start_node_for_chain_spec(402, chain_spec) {
        Ok(_) => panic!("bypassing the helper should still be rejected at startup"),
        Err(err) => err,
    };
    assert!(
        err.contains("hard cap exceeded"),
        "startup failure should mention the supply violation: {err}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_post_genesis_transfer_conserves_supply() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let recipient = PrivateKeySigner::random().address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);

    let mut alloc = BTreeMap::new();
    alloc.insert(
        sender,
        GenesisAccount {
            balance: funded_balance,
            ..GenesisAccount::default()
        },
    );

    let chain_spec = chain_spec_from_alloc(alloc);
    let (handle, _tempdir) =
        start_node_for_chain_spec(403, chain_spec).expect("funded node should start");
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let tracked_addresses = [
        sender,
        recipient,
        TEST_PROPOSER_FEE_RECIPIENT,
        FEE_POOL_PRECOMPILE_ADDRESS,
        COMMUNITY_POOL_ADDRESS,
    ];
    let before = [
        query_balance(rpc_addr, sender).await,
        query_balance(rpc_addr, recipient).await,
        query_balance(rpc_addr, TEST_PROPOSER_FEE_RECIPIENT).await,
        query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await,
        query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await,
    ];

    let tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce: 0,
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
        max_fee_per_gas: MAX_FEE_PER_GAS,
        gas_limit: TRANSFER_GAS_LIMIT,
        to: TxKind::Call(recipient),
        value: U256::from(1_000_000_000_000_000_000u128),
        access_list: Default::default(),
        input: Bytes::default(),
    };

    let raw_tx = sign_eip1559_tx(&signer, tx).await;
    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;
    assert_eq!(receipt["status"].as_str(), Some("0x1"));

    let mut after = [U256::ZERO; 5];
    for (slot, address) in after.iter_mut().zip(tracked_addresses) {
        *slot = query_balance(rpc_addr, address).await;
    }

    assert_eq!(
        sum_balances(&before),
        sum_balances(&after),
        "tracked balances should remain supply-conserving after a transfer"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn test_community_pool_credit_is_supply_conserving() {
    let signer = PrivateKeySigner::random();
    let sender = signer.address();
    let fee_recipient = Address::repeat_byte(0x77);
    let recipient = PrivateKeySigner::random().address();
    let funded_balance = U256::from(100_000_000_000_000_000_000u128);

    let validator_key = ed25519::PrivateKey::from_seed(404);
    let public_key = validator_key.public_key();
    let mut alloc = BTreeMap::new();
    alloc.insert(
        sender,
        GenesisAccount {
            balance: funded_balance,
            ..GenesisAccount::default()
        },
    );
    let chain_spec = chain_spec_from_alloc_and_validators(
        alloc,
        vec![ValidatorEntry {
            consensus_pubkey: validator_public_key_bytes(&public_key),
            ethereum_address: fee_recipient,
        }],
    );
    let (handle, _tempdir) =
        start_node_for_chain_spec(404, chain_spec).expect("funded node should start");
    let rpc_addr = handle.rpc_addr;

    wait_for_block(rpc_addr, 1, Duration::from_secs(30)).await;

    let before_zero_balance = query_balance(rpc_addr, Address::ZERO).await;
    let before = [
        query_balance(rpc_addr, sender).await,
        query_balance(rpc_addr, recipient).await,
        query_balance(rpc_addr, fee_recipient).await,
        query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await,
        query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await,
    ];
    let before_precompile_balance = query_community_pool_balance_via_precompile(rpc_addr).await;
    let before_fee_pool_precompile_balance = query_fee_pool_balance_via_precompile(rpc_addr).await;
    assert_eq!(
        before_precompile_balance, before[4],
        "precompile getter should match community pool account before tx"
    );
    assert_eq!(
        before_fee_pool_precompile_balance, before[3],
        "precompile getter should match fee-pool account before tx"
    );

    let tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce: 0,
        max_priority_fee_per_gas: MAX_PRIORITY_FEE_PER_GAS,
        max_fee_per_gas: MAX_FEE_PER_GAS,
        gas_limit: TRANSFER_GAS_LIMIT,
        to: TxKind::Call(recipient),
        value: U256::ZERO,
        access_list: Default::default(),
        input: Bytes::default(),
    };

    let raw_tx = sign_eip1559_tx(&signer, tx).await;
    let tx_hash = send_raw_tx(rpc_addr, &raw_tx).await;
    let receipt = wait_for_receipt(rpc_addr, tx_hash, Duration::from_secs(30)).await;
    let block_number = parse_rpc_u64(&receipt["blockNumber"], "receipt blockNumber");
    let block = get_block_by_number(rpc_addr, block_number).await;
    let gas_used = parse_rpc_u256(&block["gasUsed"], "block gasUsed");
    let base_fee = parse_rpc_u256(&block["baseFeePerGas"], "block baseFeePerGas");
    let burned_amount = gas_used * base_fee;
    let expected_priority_fees = gas_used * U256::from(MAX_PRIORITY_FEE_PER_GAS);

    let after = [
        query_balance(rpc_addr, sender).await,
        query_balance(rpc_addr, recipient).await,
        query_balance(rpc_addr, fee_recipient).await,
        query_balance(rpc_addr, FEE_POOL_PRECOMPILE_ADDRESS).await,
        query_balance(rpc_addr, COMMUNITY_POOL_ADDRESS).await,
    ];
    let after_zero_balance = query_balance(rpc_addr, Address::ZERO).await;
    let after_precompile_balance = query_community_pool_balance_via_precompile(rpc_addr).await;
    let after_fee_pool_precompile_balance = query_fee_pool_balance_via_precompile(rpc_addr).await;

    assert_eq!(
        after[4] - before[4],
        burned_amount,
        "community pool should receive the burned-fee amount"
    );
    assert_eq!(
        after[3] - before[3],
        expected_priority_fees,
        "fee pool should receive the priority-fee amount"
    );
    assert_eq!(
        after[2] - before[2],
        U256::ZERO,
        "configured fee recipient account should not receive direct priority-fee credit"
    );
    assert_eq!(
        sum_balances(&before),
        sum_balances(&after),
        "community-pool credit should be supply-conserving"
    );
    assert_eq!(
        after_zero_balance, before_zero_balance,
        "controlled fee-only transfer should not change Address::ZERO balance"
    );
    assert_eq!(
        after_precompile_balance, after[4],
        "precompile getter should match community pool account after tx"
    );
    assert_eq!(
        after_fee_pool_precompile_balance, after[3],
        "precompile getter should match fee-pool account after tx"
    );
}
