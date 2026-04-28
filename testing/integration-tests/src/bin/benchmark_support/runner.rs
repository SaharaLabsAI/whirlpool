use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::sync::Arc;
use std::time::{Duration, Instant};

use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_genesis::GenesisAccount;
use alloy_primitives::{Address, Bytes, TxKind, B256, U256};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use chainspec::{
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators, SAHARA_CHAIN_ID,
};
use commonware_cryptography::{ed25519, Signer as CwSigner};
use serde_json::json;
use tempfile::TempDir;
use validators_reader::ValidatorEntry;
use whirlpool_node::config::{
    ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig, RpcConfig as NodeRpcConfig,
    StorageConfig, DEFAULT_MAX_MESSAGE_SIZE,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

use super::cli::BenchArgs;

pub type BenchResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const TX_PRIORITY_FEE_WEI: u128 = 1_000_000_000;
const TX_MAX_FEE_WEI: u128 = 20_000_000_000;
const TRANSFER_VALUE_WEI: u128 = 1_000_000_000_000_000; // 0.001 ETH
const FUNDED_BALANCE_PER_SENDER_WEI: u128 = 10_000_000_000_000_000_000; // 10 ETH

struct SenderState {
    signer: PrivateKeySigner,
    next_nonce: u64,
}

struct SubmissionStats {
    submitted_transactions: u64,
    accepted_transactions: u64,
    rejected_submissions: u64,
}

struct BlockWindowStats {
    block_count: u64,
    transaction_count: u64,
    average_block_time_seconds: f64,
}

pub async fn run_benchmark(args: BenchArgs) -> BenchResult<()> {
    run_benchmark_impl(args).await
}

fn allocate_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind ephemeral port")
        .local_addr()
        .expect("failed to read local address")
        .port()
}

fn validator_public_key_bytes(public_key: &ed25519::PublicKey) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(public_key.as_ref());
    bytes
}

fn deterministic_signer(index: usize) -> BenchResult<PrivateKeySigner> {
    let ordinal = u64::try_from(index)
        .map_err(|err| format!("sender account index {index} does not fit in u64: {err}"))?
        .saturating_add(1);
    let mut raw = [0u8; 32];
    raw[24..].copy_from_slice(&ordinal.to_be_bytes());
    PrivateKeySigner::from_bytes(&B256::from(raw)).map_err(|err| {
        format!("failed to build deterministic signer for index {index}: {err}").into()
    })
}

fn deterministic_recipient(index: usize) -> Address {
    let ordinal = u64::try_from(index)
        .expect("recipient index should fit in u64")
        .saturating_add(1);
    let mut raw = [0u8; 20];
    raw[..4].copy_from_slice(&[0x52, 0x45, 0x43, 0x50]); // RECP
    raw[12..].copy_from_slice(&ordinal.to_be_bytes());
    Address::from(raw)
}

fn build_sender_alloc(senders: &[SenderState]) -> BTreeMap<Address, GenesisAccount> {
    let mut alloc = BTreeMap::new();
    let funded_balance = U256::from(FUNDED_BALANCE_PER_SENDER_WEI);
    for sender in senders {
        alloc.insert(
            sender.signer.address(),
            GenesisAccount {
                balance: funded_balance,
                ..GenesisAccount::default()
            },
        );
    }
    alloc
}

fn start_benchmark_node(
    alloc: BTreeMap<Address, GenesisAccount>,
    block_interval: Duration,
) -> BenchResult<(NodeHandle, TempDir)> {
    let seed = 0;
    let validator_private_key = ed25519::PrivateKey::from_seed(seed);
    let validator_public_key = validator_private_key.public_key();

    let chain_spec = build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
        alloc,
        BTreeMap::new(),
        vec![ValidatorEntry {
            consensus_pubkey: validator_public_key_bytes(&validator_public_key),
            ethereum_address: Address::ZERO,
        }],
    );

    let tempdir = TempDir::new()?;
    let p2p_addr: SocketAddr = format!("127.0.0.1:{}", allocate_port()).parse()?;
    let rpc_addr: SocketAddr = format!("127.0.0.1:{}", allocate_port()).parse()?;

    let config = NodeConfig {
        network: NetworkConfig {
            namespace: format!("benchmark-{}", std::process::id()).into_bytes(),
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
            namespace: format!("benchmark-{}", std::process::id()).into_bytes(),
            block_interval,
        },
        bootstrap_validators: Some(vec![validator_public_key]),
        bootstrap: Default::default(),
    };

    let handle = start_node_with_chain_spec(config, Some(Arc::new(chain_spec)))?;
    Ok((handle, tempdir))
}

fn rpc_req(method: &str, params: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    })
}

fn parse_rpc_u64(value: &serde_json::Value, field: &str) -> BenchResult<u64> {
    let hex = value
        .as_str()
        .ok_or_else(|| format!("{field} should be a hex string, got {value}"))?;
    let digits = hex.trim_start_matches("0x");
    if digits.is_empty() {
        return Ok(0);
    }
    u64::from_str_radix(digits, 16)
        .map_err(|err| format!("failed to parse {field} value {hex}: {err}").into())
}

async fn post_json_to_addr(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    payload: serde_json::Value,
) -> BenchResult<serde_json::Value> {
    let response = client
        .post(format!("http://{rpc_addr}"))
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    let decoded = response.json::<serde_json::Value>().await?;
    Ok(decoded)
}

async fn query_block_number(client: &reqwest::Client, rpc_addr: SocketAddr) -> BenchResult<u64> {
    let response =
        post_json_to_addr(client, rpc_addr, rpc_req("eth_blockNumber", json!([]))).await?;
    if response["error"].is_object() {
        return Err(format!("eth_blockNumber returned error: {response}").into());
    }
    parse_rpc_u64(&response["result"], "eth_blockNumber result")
}

async fn wait_for_block(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    min_height: u64,
    timeout: Duration,
) -> BenchResult<u64> {
    let deadline = Instant::now() + timeout;

    loop {
        if let Ok(height) = query_block_number(client, rpc_addr).await {
            if height >= min_height {
                return Ok(height);
            }
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "timed out after {:?} waiting for eth_blockNumber >= {min_height}",
                timeout
            )
            .into());
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn sign_eip1559_tx(signer: &PrivateKeySigner, tx: TxEip1559) -> BenchResult<Vec<u8>> {
    let signature = signer.sign_hash(&tx.signature_hash()).await?;
    let signed = tx.into_signed(signature);
    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    Ok(encoded)
}

fn raw_tx_hex(raw_tx_bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(raw_tx_bytes))
}

async fn try_send_raw_tx(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    raw_tx_bytes: &[u8],
) -> BenchResult<B256> {
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_sendRawTransaction", json!([raw_tx_hex(raw_tx_bytes)])),
    )
    .await?;

    if response["error"].is_object() {
        return Err(format!(
            "eth_sendRawTransaction returned error: {}",
            response["error"]
        )
        .into());
    }

    let tx_hash = response["result"].as_str().ok_or_else(|| {
        format!("eth_sendRawTransaction result should be hash string, got {response}")
    })?;

    tx_hash
        .parse::<B256>()
        .map_err(|err| format!("failed to parse tx hash {tx_hash}: {err}").into())
}

async fn run_submission_phase(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    senders: &mut [SenderState],
    recipients: &[Address],
    duration: Duration,
) -> BenchResult<SubmissionStats> {
    let mut stats = SubmissionStats {
        submitted_transactions: 0,
        accepted_transactions: 0,
        rejected_submissions: 0,
    };
    let mut cursor = 0usize;
    let deadline = Instant::now() + duration;

    while Instant::now() < deadline {
        let sender_index = cursor % senders.len();
        let recipient = recipients[sender_index % recipients.len()];
        let nonce = senders[sender_index].next_nonce;
        cursor = cursor.wrapping_add(1);

        let tx = TxEip1559 {
            chain_id: SAHARA_CHAIN_ID,
            nonce,
            max_priority_fee_per_gas: TX_PRIORITY_FEE_WEI,
            max_fee_per_gas: TX_MAX_FEE_WEI,
            gas_limit: 21_000,
            to: TxKind::Call(recipient),
            value: U256::from(TRANSFER_VALUE_WEI),
            access_list: Default::default(),
            input: Bytes::default(),
        };

        let raw_tx = sign_eip1559_tx(&senders[sender_index].signer, tx).await?;

        stats.submitted_transactions = stats.submitted_transactions.saturating_add(1);
        match try_send_raw_tx(client, rpc_addr, &raw_tx).await {
            Ok(_tx_hash) => {
                stats.accepted_transactions = stats.accepted_transactions.saturating_add(1);
                senders[sender_index].next_nonce =
                    senders[sender_index].next_nonce.saturating_add(1);
            }
            Err(err) => {
                stats.rejected_submissions = stats.rejected_submissions.saturating_add(1);
                let message = err.to_string().to_lowercase();
                if message.contains("nonce too low") || message.contains("already known") {
                    senders[sender_index].next_nonce =
                        senders[sender_index].next_nonce.saturating_add(1);
                }
            }
        }
    }

    Ok(stats)
}

async fn get_block_by_number(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    block_number: u64,
) -> BenchResult<serde_json::Value> {
    let block_tag = format!("0x{block_number:x}");
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getBlockByNumber", json!([block_tag, false])),
    )
    .await?;

    if response["error"].is_object() {
        return Err(format!("eth_getBlockByNumber returned error: {response}").into());
    }

    if response["result"].is_null() {
        return Err(format!("eth_getBlockByNumber returned null for block {block_number}").into());
    }

    Ok(response["result"].clone())
}

async fn collect_block_window_stats(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    start_block: u64,
    end_block: u64,
    measurement_window_seconds: u64,
) -> BenchResult<BlockWindowStats> {
    if end_block <= start_block {
        return Ok(BlockWindowStats {
            block_count: 0,
            transaction_count: 0,
            average_block_time_seconds: 0.0,
        });
    }

    let mut block_count = 0u64;
    let mut transaction_count = 0u64;

    for block_number in (start_block + 1)..=end_block {
        let block = get_block_by_number(client, rpc_addr, block_number).await?;
        let tx_count_in_block = block["transactions"]
            .as_array()
            .map_or(0usize, |txs| txs.len());

        block_count = block_count.saturating_add(1);
        transaction_count =
            transaction_count.saturating_add(u64::try_from(tx_count_in_block).unwrap_or(u64::MAX));
    }

    let average_block_time_seconds = if block_count > 0 {
        measurement_window_seconds as f64 / block_count as f64
    } else {
        0.0
    };

    Ok(BlockWindowStats {
        block_count,
        transaction_count,
        average_block_time_seconds,
    })
}

async fn run_benchmark_impl(args: BenchArgs) -> BenchResult<()> {
    eprintln!(
        "starting single-node transfer benchmark: duration={}s, sender_accounts={}, recipient_accounts={}, block_interval_ms={}",
        args.duration_seconds, args.sender_accounts, args.recipient_accounts, args.block_interval_ms
    );

    let mut senders = Vec::with_capacity(args.sender_accounts);
    for index in 0..args.sender_accounts {
        senders.push(SenderState {
            signer: deterministic_signer(index)?,
            next_nonce: 0,
        });
    }

    let recipients = (0..args.recipient_accounts)
        .map(deterministic_recipient)
        .collect::<Vec<_>>();

    let alloc = build_sender_alloc(&senders);
    let (handle, _tempdir) =
        start_benchmark_node(alloc, Duration::from_millis(args.block_interval_ms))?;
    let rpc_addr = handle.rpc_addr;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    wait_for_block(&client, rpc_addr, 1, Duration::from_secs(90)).await?;

    let start_block = query_block_number(&client, rpc_addr).await?;
    let submission_stats = run_submission_phase(
        &client,
        rpc_addr,
        &mut senders,
        &recipients,
        Duration::from_secs(args.duration_seconds),
    )
    .await?;

    let end_block = query_block_number(&client, rpc_addr).await?;
    let block_stats = collect_block_window_stats(
        &client,
        rpc_addr,
        start_block,
        end_block,
        args.duration_seconds,
    )
    .await?;

    let tps = block_stats.transaction_count as f64 / args.duration_seconds as f64;

    let payload = json!({
        "measurement_window_seconds": args.duration_seconds,
        "sender_accounts": args.sender_accounts,
        "recipient_accounts": args.recipient_accounts,
        "start_block": start_block,
        "end_block": end_block,
        "block_count": block_stats.block_count,
        "average_block_time_seconds": block_stats.average_block_time_seconds,
        "transaction_count": block_stats.transaction_count,
        "submitted_transactions": submission_stats.submitted_transactions,
        "accepted_transactions": submission_stats.accepted_transactions,
        "rejected_submissions": submission_stats.rejected_submissions,
        "tps": tps,
    });

    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}
