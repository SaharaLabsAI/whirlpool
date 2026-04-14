use std::env;
use std::fs::OpenOptions;
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use alloy_consensus::{SignableTransaction, TxEip1559};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, TxKind, B256, U256};
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use chainspec::SAHARA_CHAIN_ID;
use evm_precompiles::EPOCH_SYSTEM_TX_PRIVATE_KEY;
use serde_json::json;
use tempfile::TempDir;

type BenchResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const TX_PRIORITY_FEE_WEI: u128 = 1_000_000_000;
const TX_MAX_FEE_WEI: u128 = 20_000_000_000;
const TRANSFER_VALUE_WEI: u128 = 100_000_000_000_000_000; // 0.1 ETH

fn usage(program: &str) {
    eprintln!(
        "Usage: {program} [--whirlpool-node-bin <path>]\n\
         Env fallback: WHIRLPOOL_NODE_BINARY (default: target/release/whirlpool-node)"
    );
}

fn parse_node_binary_arg() -> PathBuf {
    let mut binary_path = env::var_os("WHIRLPOOL_NODE_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release/whirlpool-node"));

    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "single_node_transfer_benchmark".into());
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--whirlpool-node-bin" => {
                let Some(path) = iter.next() else {
                    usage(&program);
                    panic!("--whirlpool-node-bin requires a path");
                };
                binary_path = PathBuf::from(path);
            }
            "--help" | "-h" => {
                usage(&program);
                std::process::exit(0);
            }
            _ => {
                usage(&program);
                panic!("unknown argument: {arg}");
            }
        }
    }

    binary_path
}

fn allocate_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind ephemeral port")
        .local_addr()
        .expect("failed to read local address")
        .port()
}

struct NodeProcess {
    child: Child,
    log_path: PathBuf,
}

impl NodeProcess {
    fn spawn(
        binary: &Path,
        data_dir: &Path,
        rpc_addr: SocketAddr,
        p2p_addr: SocketAddr,
    ) -> BenchResult<Self> {
        let namespace = format!("bench-{}", std::process::id());
        let node_rust_log =
            env::var("WHIRLPOOL_BENCH_NODE_RUST_LOG").unwrap_or_else(|_| "error".to_string());
        let log_path = data_dir.join("whirlpool-node.log");
        let stdout_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        let stderr_log = stdout_log.try_clone()?;
        let mut command = Command::new(binary);
        command
            .arg("--listen-addr")
            .arg(p2p_addr.to_string())
            .arg("--dialable-addr")
            .arg(p2p_addr.to_string())
            .arg("--rpc-addr")
            .arg(rpc_addr.to_string())
            .arg("--mem-rpc-addr")
            .arg("127.0.0.1:0")
            .arg("--data-dir")
            .arg(data_dir)
            .arg("--validator-seed")
            .arg("0")
            .arg("--network-namespace")
            .arg(&namespace)
            .arg("--consensus-namespace")
            .arg(&namespace)
            .arg("--block-interval-ms")
            .arg("1000")
            .env("RUST_LOG", node_rust_log)
            .stdout(Stdio::from(stdout_log))
            .stderr(Stdio::from(stderr_log));

        let child = command.spawn().map_err(|err| {
            format!(
                "failed to spawn whirlpool-node binary at {}: {err}",
                binary.display()
            )
        })?;

        Ok(Self { child, log_path })
    }

    fn ensure_running(&mut self) -> BenchResult<()> {
        if let Some(status) = self.child.try_wait()? {
            return Err(format!(
                "whirlpool-node exited unexpectedly with status {status}; inspect {}",
                self.log_path.display()
            )
            .into());
        }
        Ok(())
    }
}

impl Drop for NodeProcess {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
            Err(_) => {}
        }
    }
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

fn parse_rpc_u256(value: &serde_json::Value, field: &str) -> BenchResult<U256> {
    let hex = value
        .as_str()
        .ok_or_else(|| format!("{field} should be a hex string, got {value}"))?;
    let digits = hex.trim_start_matches("0x");
    if digits.is_empty() {
        return Ok(U256::ZERO);
    }
    U256::from_str_radix(digits, 16)
        .map_err(|err| format!("failed to parse {field} value {hex}: {err}").into())
}

fn parse_rpc_b256(value: &serde_json::Value, field: &str) -> BenchResult<B256> {
    let hex = value
        .as_str()
        .ok_or_else(|| format!("{field} should be a hash string, got {value}"))?;
    hex.parse()
        .map_err(|err| format!("failed to parse {field} hash {hex}: {err}").into())
}

fn raw_tx_hex(raw_tx_bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(raw_tx_bytes))
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

async fn wait_for_block(
    client: &reqwest::Client,
    node: &mut NodeProcess,
    rpc_addr: SocketAddr,
    min_height: u64,
    timeout: Duration,
) -> BenchResult<u64> {
    let deadline = Instant::now() + timeout;

    loop {
        node.ensure_running()?;

        if let Ok(response) =
            post_json_to_addr(client, rpc_addr, rpc_req("eth_blockNumber", json!([]))).await
        {
            if response["error"].is_object() {
                return Err(format!("eth_blockNumber returned error: {response}").into());
            }

            let height = parse_rpc_u64(&response["result"], "eth_blockNumber result")?;
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

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

async fn wait_for_receipt(
    client: &reqwest::Client,
    node: &mut NodeProcess,
    rpc_addr: SocketAddr,
    tx_hash: B256,
    timeout: Duration,
) -> BenchResult<serde_json::Value> {
    let deadline = Instant::now() + timeout;
    let tx_hash_hex = format!("0x{tx_hash:x}");

    loop {
        node.ensure_running()?;
        let response = post_json_to_addr(
            client,
            rpc_addr,
            rpc_req("eth_getTransactionReceipt", json!([tx_hash_hex])),
        )
        .await?;

        if response["error"].is_object() {
            return Err(format!("eth_getTransactionReceipt returned error: {response}").into());
        }

        if !response["result"].is_null() {
            return Ok(response["result"].clone());
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "timed out after {:?} waiting for transaction receipt {tx_hash:#x}",
                timeout
            )
            .into());
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn query_nonce(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    sender: Address,
) -> BenchResult<u64> {
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getTransactionCount", json!([sender, "latest"])),
    )
    .await?;

    if response["error"].is_object() {
        return Err(format!("eth_getTransactionCount returned error: {response}").into());
    }

    parse_rpc_u64(&response["result"], "eth_getTransactionCount result")
}

async fn send_raw_tx(
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
        return Err(format!("eth_sendRawTransaction returned error: {response}").into());
    }

    parse_rpc_b256(&response["result"], "eth_sendRawTransaction result")
}

async fn sign_eip1559_tx(signer: &PrivateKeySigner, tx: TxEip1559) -> BenchResult<Vec<u8>> {
    let signature = signer.sign_hash(&tx.signature_hash()).await?;
    let signed = tx.into_signed(signature);
    let mut encoded = Vec::new();
    signed.encode_2718(&mut encoded);
    Ok(encoded)
}

async fn verify_recipient_balance(
    client: &reqwest::Client,
    rpc_addr: SocketAddr,
    recipient: Address,
    expected_balance: U256,
) -> BenchResult<()> {
    let response = post_json_to_addr(
        client,
        rpc_addr,
        rpc_req("eth_getBalance", json!([recipient, "latest"])),
    )
    .await?;

    if response["error"].is_object() {
        return Err(format!("eth_getBalance returned error: {response}").into());
    }

    let balance = parse_rpc_u256(&response["result"], "eth_getBalance result")?;
    if balance != expected_balance {
        return Err(format!(
            "unexpected recipient balance: expected {expected_balance}, got {balance}"
        )
        .into());
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> BenchResult<()> {
    let node_binary = parse_node_binary_arg();
    if !node_binary.exists() {
        return Err(format!(
            "whirlpool-node binary not found at {}; build it with `cargo build --release -p whirlpool-node --bin whirlpool-node`",
            node_binary.display()
        )
        .into());
    }

    let tempdir = TempDir::new()?;
    let rpc_addr: SocketAddr = format!("127.0.0.1:{}", allocate_port()).parse()?;
    let p2p_addr: SocketAddr = format!("127.0.0.1:{}", allocate_port()).parse()?;
    let mut node = NodeProcess::spawn(&node_binary, tempdir.path(), rpc_addr, p2p_addr)?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    wait_for_block(&client, &mut node, rpc_addr, 1, Duration::from_secs(90)).await?;

    let signer = PrivateKeySigner::from_bytes(&EPOCH_SYSTEM_TX_PRIVATE_KEY)?;
    let sender = signer.address();
    let recipient = Address::repeat_byte(0x11);
    let nonce = query_nonce(&client, rpc_addr, sender).await?;
    let transfer_value = U256::from(TRANSFER_VALUE_WEI);

    let tx = TxEip1559 {
        chain_id: SAHARA_CHAIN_ID,
        nonce,
        max_priority_fee_per_gas: TX_PRIORITY_FEE_WEI,
        max_fee_per_gas: TX_MAX_FEE_WEI,
        gas_limit: 21_000,
        to: TxKind::Call(recipient),
        value: transfer_value,
        access_list: Default::default(),
        input: Bytes::default(),
    };

    let raw_tx = sign_eip1559_tx(&signer, tx).await?;
    let tx_hash = send_raw_tx(&client, rpc_addr, &raw_tx).await?;
    let receipt = wait_for_receipt(
        &client,
        &mut node,
        rpc_addr,
        tx_hash,
        Duration::from_secs(30),
    )
    .await?;

    let status = receipt["status"].as_str().unwrap_or_default();
    if status != "0x1" {
        return Err(format!("transfer receipt status was not success: {receipt}").into());
    }

    verify_recipient_balance(&client, rpc_addr, recipient, transfer_value).await?;

    println!(
        "single_node_transfer_benchmark completed successfully: tx_hash=0x{tx_hash:x}, rpc_addr={rpc_addr}"
    );

    Ok(())
}
