use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use serde_json::json;
use std::error::Error;
use std::net::{SocketAddr, TcpListener};
use std::time::Duration;
use whirlpool_node::config::{
    parse_bootstrap_peer, ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig,
    RpcConfig, StorageConfig,
};
use whirlpool_node::node::{start_node, NodeHandle};

fn allocate_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn test_four_node_consensus() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let num_nodes = 4usize;
    let validator_signers: Vec<_> = (0..num_nodes as u64)
        .map(ed25519::PrivateKey::from_seed)
        .collect();
    let validator_pubkeys: Vec<_> = validator_signers.iter().map(|s| s.public_key()).collect();

    let p2p_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
    let rpc_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
    let tempdirs: Vec<_> = (0..num_nodes)
        .map(|_| tempfile::tempdir().expect("tempdir"))
        .collect();

    let mut handles: Vec<NodeHandle> = Vec::new();

    for i in 0..num_nodes {
        let bootstrap_peers = (0..num_nodes)
            .filter(|&j| j != i)
            .map(|j| {
                let pk_hex = hex::encode(validator_pubkeys[j].as_ref());
                parse_bootstrap_peer(&format!("{pk_hex}@127.0.0.1:{}", p2p_ports[j]))
                    .expect("bootstrap peer")
            })
            .collect();

        let config = NodeConfig {
            network: NetworkConfig {
                namespace: b"whirlpool-multinode-test".to_vec(),
                listen_addr: format!("127.0.0.1:{}", p2p_ports[i]).parse().unwrap(),
                dialable_addr: format!("127.0.0.1:{}", p2p_ports[i]).parse().unwrap(),
                bootstrap_peers,
                max_message_size: whirlpool_node::config::DEFAULT_MAX_MESSAGE_SIZE,
            },
            identity: IdentityConfig { seed: i as u64 },
            rpc: RpcConfig {
                bind_addr: format!("127.0.0.1:{}", rpc_ports[i]).parse().unwrap(),
            },
            storage: StorageConfig {
                data_dir: tempdirs[i].path().to_path_buf(),
            },
            consensus: ConsensusStartupConfig {
                namespace: b"whirlpool-multinode-consensus".to_vec(),
                block_interval: Duration::from_secs(1),
            },
            validators: Some(validator_pubkeys.clone()),
        };

        let handle = start_node(config).expect("failed to start node");
        println!(
            "Started node {i}: rpc={}, p2p={}",
            handle.rpc_addr, handle.p2p_addr
        );
        handles.push(handle);
    }

    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(60);
    let target_height = 1u64;

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout: nodes did not reach consensus within 60s");
        }

        let mut heights = Vec::new();
        for (i, handle) in handles.iter().enumerate() {
            let height = rpc_get_block_number(handle.rpc_addr).await.unwrap_or(0);
            println!("Node {i}: height={height}");
            heights.push(height);
        }

        if heights.iter().all(|h| *h >= target_height) {
            let min = heights.iter().min().unwrap();
            let max = heights.iter().max().unwrap();
            if max - min <= 1 {
                println!("SUCCESS: All nodes synced. Heights: {:?}", heights);
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    drop(handles);
    drop(tempdirs);
}

async fn rpc_get_block_number(addr: SocketAddr) -> Result<u64, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{addr}"))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1,
        }))
        .send()
        .await?
        .error_for_status()?;

    let body: serde_json::Value = response.json().await?;
    let value = body
        .get("result")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("missing result in RPC response: {body}"))?;
    let height = value.strip_prefix("0x").unwrap_or(value);
    Ok(u64::from_str_radix(height, 16)?)
}
