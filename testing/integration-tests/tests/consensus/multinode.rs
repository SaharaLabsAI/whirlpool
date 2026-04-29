use alloy_primitives::Address;
use chainspec::build_sahara_chain_spec_with_alloc_and_validators;
use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use consensus_manager::{run_trusted_dealer_bootstrap, TrustedDealerBootstrapConfig};
use serde_json::json;
use std::collections::BTreeMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use validators_reader::ValidatorEntry;
use whirlpool_node::config::{
    parse_bootstrap_peer, ConsensusStartupConfig, IdentityConfig, NetworkConfig, NodeConfig,
    RpcConfig, StorageConfig,
};
use whirlpool_node::node::{start_node_with_chain_spec, NodeHandle};

use crate::common::ports::allocate_port;

#[tokio::test]
async fn test_four_node_consensus() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let num_nodes = 4usize;
    let validator_signers: Vec<_> = (0..num_nodes as u64)
        .map(ed25519::PrivateKey::from_seed)
        .collect();
    let validator_pubkeys: Vec<_> = validator_signers.iter().map(|s| s.public_key()).collect();
    let simplex_validators = validator_pubkeys
        .iter()
        .enumerate()
        .map(|(i, pubkey)| ValidatorEntry {
            consensus_pubkey: pubkey.as_ref().try_into().expect("ed25519 key length"),
            ethereum_address: Address::repeat_byte((i + 1) as u8),
        })
        .collect::<Vec<_>>();
    let chain_spec = Arc::new(build_sahara_chain_spec_with_alloc_and_validators(
        BTreeMap::new(),
        simplex_validators,
    ));

    let p2p_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
    let rpc_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
    let mem_rpc_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
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
                mem_bind_addr: format!("127.0.0.1:{}", mem_rpc_ports[i]).parse().unwrap(),
            },
            storage: StorageConfig {
                data_dir: tempdirs[i].path().to_path_buf(),
            },
            consensus: ConsensusStartupConfig {
                namespace: b"whirlpool-multinode-consensus".to_vec(),
                block_interval: Duration::from_secs(1),
            },
            bootstrap_validators: Some(validator_pubkeys.clone()),
            bootstrap: Default::default(),
        };

        let handle = start_node_with_chain_spec(config, Some(chain_spec.clone()))
            .expect("failed to start node");
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

#[tokio::test]
async fn test_single_node_consensus_with_bls_bootstrap_bundle() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let signer = ed25519::PrivateKey::from_seed(90);
    let pubkey = signer.public_key();
    let validator_entries = vec![ValidatorEntry {
        consensus_pubkey: pubkey.as_ref().try_into().expect("ed25519 key length"),
        ethereum_address: Address::repeat_byte(1),
    }];
    let chain_spec = Arc::new(build_sahara_chain_spec_with_alloc_and_validators(
        BTreeMap::new(),
        validator_entries,
    ));

    let p2p_port = allocate_port();
    let rpc_port = allocate_port();
    let mem_rpc_port = allocate_port();
    let data_dir = tempfile::tempdir().expect("tempdir");
    let bootstrap_dir = tempfile::tempdir().expect("bootstrap tempdir");
    let bootstrap_result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 0x501,
        output_dir: bootstrap_dir.path().to_path_buf(),
        participants: vec![pubkey.clone()],
    })
    .expect("bootstrap should succeed");

    let config = NodeConfig {
        network: NetworkConfig {
            namespace: b"whirlpool-single-bls-test".to_vec(),
            listen_addr: format!("127.0.0.1:{p2p_port}").parse().unwrap(),
            dialable_addr: format!("127.0.0.1:{p2p_port}").parse().unwrap(),
            bootstrap_peers: vec![],
            max_message_size: whirlpool_node::config::DEFAULT_MAX_MESSAGE_SIZE,
        },
        identity: IdentityConfig { seed: 90 },
        rpc: RpcConfig {
            bind_addr: format!("127.0.0.1:{rpc_port}").parse().unwrap(),
            mem_bind_addr: format!("127.0.0.1:{mem_rpc_port}").parse().unwrap(),
        },
        storage: StorageConfig {
            data_dir: data_dir.path().to_path_buf(),
        },
        consensus: ConsensusStartupConfig {
            namespace: b"whirlpool-single-bls-consensus".to_vec(),
            block_interval: Duration::from_secs(1),
        },
        bootstrap_validators: Some(vec![pubkey]),
        bootstrap: whirlpool_node::config::BootstrapConfig {
            genesis_dkg_session_dir: Some(bootstrap_result.session_dir),
            genesis_dkg_dealer_pubkey: Some(bootstrap_result.dealer_public_key),
            ..Default::default()
        },
    };

    let handle = start_node_with_chain_spec(config, Some(chain_spec)).expect("start node");
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);
    loop {
        if start.elapsed() > timeout {
            panic!("Timeout: BLS single-node did not finalize a block within 30s");
        }
        let height = rpc_get_block_number(handle.rpc_addr).await.unwrap_or(0);
        if height >= 1 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::test]
async fn test_four_node_consensus_with_bls_bootstrap_bundle() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();

    let num_nodes = 4usize;
    let validator_signers: Vec<_> = (0..num_nodes as u64)
        .map(|seed| ed25519::PrivateKey::from_seed(seed + 100))
        .collect();
    let validator_pubkeys: Vec<_> = validator_signers.iter().map(|s| s.public_key()).collect();
    let bootstrap_dir = tempfile::tempdir().expect("bootstrap tempdir");
    let bootstrap_result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: 0x777,
        output_dir: bootstrap_dir.path().to_path_buf(),
        participants: validator_pubkeys.clone(),
    })
    .expect("bootstrap should succeed");
    let ordered_simplex_keys: Vec<_> = bootstrap_result
        .bundle_paths
        .iter()
        .map(|(public_key, _)| public_key.clone())
        .collect();
    let simplex_validators = ordered_simplex_keys
        .iter()
        .enumerate()
        .map(|(i, pubkey)| ValidatorEntry {
            consensus_pubkey: pubkey.as_ref().try_into().expect("ed25519 key length"),
            ethereum_address: Address::repeat_byte((i + 1) as u8),
        })
        .collect::<Vec<_>>();
    let chain_spec = Arc::new(build_sahara_chain_spec_with_alloc_and_validators(
        BTreeMap::new(),
        simplex_validators,
    ));

    let p2p_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
    let rpc_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
    let mem_rpc_ports: Vec<u16> = (0..num_nodes).map(|_| allocate_port()).collect();
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
                namespace: b"whirlpool-multinode-bls-test".to_vec(),
                listen_addr: format!("127.0.0.1:{}", p2p_ports[i]).parse().unwrap(),
                dialable_addr: format!("127.0.0.1:{}", p2p_ports[i]).parse().unwrap(),
                bootstrap_peers,
                max_message_size: whirlpool_node::config::DEFAULT_MAX_MESSAGE_SIZE,
            },
            identity: IdentityConfig {
                seed: i as u64 + 100,
            },
            rpc: RpcConfig {
                bind_addr: format!("127.0.0.1:{}", rpc_ports[i]).parse().unwrap(),
                mem_bind_addr: format!("127.0.0.1:{}", mem_rpc_ports[i]).parse().unwrap(),
            },
            storage: StorageConfig {
                data_dir: tempdirs[i].path().to_path_buf(),
            },
            consensus: ConsensusStartupConfig {
                namespace: b"whirlpool-multinode-bls-consensus".to_vec(),
                block_interval: Duration::from_secs(1),
            },
            bootstrap_validators: Some(validator_pubkeys.clone()),
            bootstrap: whirlpool_node::config::BootstrapConfig {
                genesis_dkg_session_dir: Some(bootstrap_result.session_dir.clone()),
                genesis_dkg_dealer_pubkey: Some(bootstrap_result.dealer_public_key.clone()),
                ..Default::default()
            },
        };

        let handle = start_node_with_chain_spec(config, Some(chain_spec.clone()))
            .expect("failed to start node");
        handles.push(handle);
    }

    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(60);
    let target_height = 1u64;
    loop {
        if start.elapsed() > timeout {
            panic!("Timeout: BLS nodes did not reach consensus within 60s");
        }

        let mut heights = Vec::new();
        for handle in &handles {
            heights.push(rpc_get_block_number(handle.rpc_addr).await.unwrap_or(0));
        }
        if heights.iter().all(|h| *h >= target_height) {
            let min = heights.iter().min().unwrap();
            let max = heights.iter().max().unwrap();
            if max - min <= 1 {
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
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
