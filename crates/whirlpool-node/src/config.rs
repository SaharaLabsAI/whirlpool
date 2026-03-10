use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use commonware_codec::Read;
use commonware_cryptography::ed25519;
use commonware_p2p::Ingress;
use commonware_utils::from_hex;

pub type BootstrapPeer = (ed25519::PublicKey, Ingress);

pub const APPLICATION_NAMESPACE: &[u8] = b"whirlpool-dev";
pub const NAMESPACE: &[u8] = b"sahara-chain-v0";
pub const BLOCK_INTERVAL: Duration = Duration::from_secs(5);
pub const BIND_ADDR: &str = "127.0.0.1:0";
pub const VALIDATOR_SEED: u64 = 0;
pub const RPC_BIND_ADDR: &str = "127.0.0.1:8545";
pub const DEFAULT_DATA_DIR: &str = "data";
pub const DEFAULT_MAX_MESSAGE_SIZE: u32 = 1024 * 1024;

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(name = "whirlpool-node", about = "Whirlpool consensus node")]
pub struct NodeArgs {
    #[arg(long, default_value = "127.0.0.1:0")]
    pub listen_addr: SocketAddr,
    #[arg(long, default_value = "127.0.0.1:0")]
    pub dialable_addr: SocketAddr,
    #[arg(long)]
    pub bootstrap_peer: Vec<String>,
    #[arg(long)]
    pub dial_peer: Vec<String>,
    #[arg(long, default_value_t = 0)]
    pub validator_seed: u64,
    #[arg(long, default_value = "127.0.0.1:8545")]
    pub rpc_addr: SocketAddr,
    #[arg(long, default_value = "data")]
    pub data_dir: PathBuf,
    #[arg(long, default_value_t = 1048576)]
    pub max_message_size: u32,
    #[arg(long)]
    pub network_namespace: Option<String>,
    #[arg(long)]
    pub consensus_namespace: Option<String>,
    #[arg(long)]
    pub block_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub identity: IdentityConfig,
    pub rpc: RpcConfig,
    pub storage: StorageConfig,
    pub consensus: ConsensusStartupConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkConfig {
    pub namespace: Vec<u8>,
    pub listen_addr: SocketAddr,
    pub dialable_addr: SocketAddr,
    pub bootstrap_peers: Vec<BootstrapPeer>,
    pub max_message_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityConfig {
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcConfig {
    pub bind_addr: SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusStartupConfig {
    pub namespace: Vec<u8>,
    pub block_interval: Duration,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            identity: IdentityConfig::default(),
            rpc: RpcConfig::default(),
            storage: StorageConfig::default(),
            consensus: ConsensusStartupConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            namespace: APPLICATION_NAMESPACE.to_vec(),
            listen_addr: BIND_ADDR.parse().expect("default listen address must be valid"),
            dialable_addr: BIND_ADDR
                .parse()
                .expect("default dialable address must be valid"),
            bootstrap_peers: Vec::new(),
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
        }
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            seed: VALIDATOR_SEED,
        }
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            bind_addr: RPC_BIND_ADDR
                .parse()
                .expect("default RPC bind address must be valid"),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
        }
    }
}

impl StorageConfig {
    pub fn runtime_dir(&self) -> PathBuf {
        self.data_dir.join("runtime")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.data_dir.join("state")
    }

    pub fn mempool_dir(&self) -> PathBuf {
        self.data_dir.join("mempool")
    }
}

impl Default for ConsensusStartupConfig {
    fn default() -> Self {
        Self {
            namespace: NAMESPACE.to_vec(),
            block_interval: BLOCK_INTERVAL,
        }
    }
}

pub fn parse_bootstrap_peer(s: &str) -> Result<BootstrapPeer, String> {
    let (public_key_hex, addr) = s
        .split_once('@')
        .ok_or_else(|| "bootstrap peer must be formatted as <pubkey>@<socket_addr>".to_string())?;

    let public_key_bytes = from_hex(public_key_hex)
        .ok_or_else(|| format!("invalid bootstrap peer public key hex: {public_key_hex}"))?;
    let mut reader = public_key_bytes.as_slice();
    let public_key = ed25519::PublicKey::read_cfg(&mut reader, &())
        .map_err(|err| format!("invalid bootstrap peer public key: {err}"))?;
    if !reader.is_empty() {
        return Err("invalid bootstrap peer public key length".to_string());
    }

    let addr = addr
        .parse()
        .map(Ingress::Socket)
        .map_err(|err| format!("invalid bootstrap peer socket address: {err}"))?;

    Ok((public_key, addr))
}

impl From<NodeArgs> for NodeConfig {
    fn from(args: NodeArgs) -> Self {
        let defaults = Self::default();
        let bootstrap_peers = args
            .bootstrap_peer
            .into_iter()
            .chain(args.dial_peer)
            .map(|peer| match parse_bootstrap_peer(&peer) {
                Ok(parsed) => parsed,
                Err(err) => panic!("failed to parse peer '{peer}': {err}"),
            })
            .collect();

        Self {
            network: NetworkConfig {
                namespace: args
                    .network_namespace
                    .map(String::into_bytes)
                    .unwrap_or(defaults.network.namespace),
                listen_addr: args.listen_addr,
                dialable_addr: args.dialable_addr,
                bootstrap_peers,
                max_message_size: args.max_message_size,
            },
            identity: IdentityConfig {
                seed: args.validator_seed,
            },
            rpc: RpcConfig {
                bind_addr: args.rpc_addr,
            },
            storage: StorageConfig {
                data_dir: args.data_dir,
            },
            consensus: ConsensusStartupConfig {
                namespace: args
                    .consensus_namespace
                    .map(String::into_bytes)
                    .unwrap_or(defaults.consensus.namespace),
                block_interval: args
                    .block_interval_ms
                    .map(Duration::from_millis)
                    .unwrap_or(defaults.consensus.block_interval),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::Signer;
    use commonware_p2p::Ingress;
    use commonware_utils::hex;

    #[test]
    fn test_node_config_default_matches_hardcoded() {
        let config = NodeConfig::default();

        assert_eq!(config.network.namespace, b"whirlpool-dev");
        assert_eq!(config.network.listen_addr, BIND_ADDR.parse().unwrap());
        assert_eq!(config.network.dialable_addr, BIND_ADDR.parse().unwrap());
        assert!(config.network.bootstrap_peers.is_empty());
        assert_eq!(config.network.max_message_size, 1_048_576);
        assert_eq!(config.identity.seed, 0);
        assert_eq!(config.rpc.bind_addr, RPC_BIND_ADDR.parse().unwrap());
        assert_eq!(config.storage.data_dir, PathBuf::from("data"));
        assert_eq!(config.consensus.namespace, b"sahara-chain-v0");
        assert_eq!(config.consensus.block_interval, Duration::from_secs(5));
    }

    #[test]
    fn test_storage_config_path_helpers() {
        let config = StorageConfig {
            data_dir: PathBuf::from("custom-data"),
        };

        assert_eq!(config.runtime_dir(), PathBuf::from("custom-data/runtime"));
        assert_eq!(config.state_dir(), PathBuf::from("custom-data/state"));
        assert_eq!(config.mempool_dir(), PathBuf::from("custom-data/mempool"));
    }

    #[test]
    fn test_parse_bootstrap_peer_valid() {
        let public_key = ed25519::PrivateKey::from_seed(7).public_key();
        let encoded = format!("{}@127.0.0.1:3000", hex(public_key.as_ref()));

        let parsed = parse_bootstrap_peer(&encoded).expect("bootstrap peer should parse");

        assert_eq!(parsed.0, public_key);
        assert_eq!(parsed.1, Ingress::Socket("127.0.0.1:3000".parse().unwrap()));
    }

    #[test]
    fn test_parse_bootstrap_peer_invalid_format() {
        let err = parse_bootstrap_peer("not-a-bootstrap-peer").expect_err("format should fail");

        assert!(err.contains("formatted as <pubkey>@<socket_addr>"));
    }

    /// TST-REQ4-003: Malformed bootstrap peer input table.
    #[test]
    fn test_parse_bootstrap_peer_malformed_variants() {
        // Missing '@' separator
        assert!(parse_bootstrap_peer("deadbeef127.0.0.1:3000").is_err());
        // Empty pubkey segment
        assert!(parse_bootstrap_peer("@127.0.0.1:3000").is_err());
        // Empty address segment
        let pk = ed25519::PrivateKey::from_seed(1).public_key();
        let pk_hex = hex(pk.as_ref());
        assert!(parse_bootstrap_peer(&format!("{pk_hex}@")).is_err());
        // Invalid hex in pubkey
        assert!(parse_bootstrap_peer("ZZZZ@127.0.0.1:3000").is_err());
        // Wrong key length (too short)
        assert!(parse_bootstrap_peer("aabb@127.0.0.1:3000").is_err());
        // Invalid socket address
        assert!(parse_bootstrap_peer(&format!("{pk_hex}@not-an-addr")).is_err());
    }

    /// TST-REQ4-004: NodeArgs normalizes all explicit flags into NodeConfig,
    /// merges bootstrap and dial peers, and converts block_interval_ms.
    #[test]
    fn test_node_args_to_node_config_full_custom() {
        let pk1 = ed25519::PrivateKey::from_seed(10).public_key();
        let pk2 = ed25519::PrivateKey::from_seed(20).public_key();
        let peer1 = format!("{}@10.0.0.1:5000", hex(pk1.as_ref()));
        let peer2 = format!("{}@10.0.0.2:6000", hex(pk2.as_ref()));

        let args = NodeArgs {
            listen_addr: "0.0.0.0:9000".parse().unwrap(),
            dialable_addr: "1.2.3.4:9000".parse().unwrap(),
            bootstrap_peer: vec![peer1],
            dial_peer: vec![peer2],
            validator_seed: 42,
            rpc_addr: "0.0.0.0:8080".parse().unwrap(),
            data_dir: PathBuf::from("/tmp/whirlpool"),
            max_message_size: 2_000_000,
            network_namespace: Some("custom-net".to_string()),
            consensus_namespace: Some("custom-cons".to_string()),
            block_interval_ms: Some(2000),
        };

        let config = NodeConfig::from(args);

        assert_eq!(config.network.listen_addr, "0.0.0.0:9000".parse().unwrap());
        assert_eq!(config.network.dialable_addr, "1.2.3.4:9000".parse().unwrap());
        assert_eq!(config.network.bootstrap_peers.len(), 2);
        assert_eq!(config.network.bootstrap_peers[0].0, pk1);
        assert_eq!(config.network.bootstrap_peers[1].0, pk2);
        assert_eq!(config.network.max_message_size, 2_000_000);
        assert_eq!(config.network.namespace, b"custom-net");
        assert_eq!(config.identity.seed, 42);
        assert_eq!(config.rpc.bind_addr, "0.0.0.0:8080".parse().unwrap());
        assert_eq!(config.storage.data_dir, PathBuf::from("/tmp/whirlpool"));
        assert_eq!(config.consensus.namespace, b"custom-cons");
        assert_eq!(config.consensus.block_interval, Duration::from_millis(2000));
    }

    /// Verify accumulation: bootstrap_peer and dial_peer merge in order.
    #[test]
    fn test_node_args_peers_accumulate() {
        let pk1 = ed25519::PrivateKey::from_seed(30).public_key();
        let pk2 = ed25519::PrivateKey::from_seed(31).public_key();
        let pk3 = ed25519::PrivateKey::from_seed(32).public_key();
        let p1 = format!("{}@10.0.0.1:1111", hex(pk1.as_ref()));
        let p2 = format!("{}@10.0.0.2:2222", hex(pk2.as_ref()));
        let p3 = format!("{}@10.0.0.3:3333", hex(pk3.as_ref()));

        let args = NodeArgs::parse_from([
            "whirlpool-node",
            "--bootstrap-peer",
            &p1,
            "--bootstrap-peer",
            &p2,
            "--dial-peer",
            &p3,
        ]);

        let config = NodeConfig::from(args);

        // All three peers accumulated, bootstrap first then dial
        assert_eq!(config.network.bootstrap_peers.len(), 3);
        assert_eq!(config.network.bootstrap_peers[0].0, pk1);
        assert_eq!(config.network.bootstrap_peers[1].0, pk2);
        assert_eq!(config.network.bootstrap_peers[2].0, pk3);
    }

    /// Default NodeArgs (no CLI flags) produces identical NodeConfig to NodeConfig::default().
    #[test]
    fn test_node_args_default_roundtrip() {
        let args = NodeArgs::parse_from(["whirlpool-node"]);
        let from_args = NodeConfig::from(args);
        let from_default = NodeConfig::default();

        assert_eq!(from_args, from_default);
    }
}
