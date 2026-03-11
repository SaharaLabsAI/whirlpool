use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use commonware_codec::Read;
use commonware_cryptography::ed25519;
use commonware_p2p::Ingress;
use commonware_utils::from_hex;
use serde::Deserialize;

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
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub listen_addr: Option<SocketAddr>,
    #[arg(long)]
    pub dialable_addr: Option<SocketAddr>,
    #[arg(long)]
    pub bootstrap_peer: Vec<String>,
    #[arg(long)]
    pub dial_peer: Vec<String>,
    #[arg(long)]
    pub validator_seed: Option<u64>,
    #[arg(long)]
    pub validator: Vec<String>,
    #[arg(long)]
    pub rpc_addr: Option<SocketAddr>,
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
    #[arg(long)]
    pub max_message_size: Option<u32>,
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
    pub validators: Option<Vec<ed25519::PublicKey>>,
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TomlConfig {
    pub listen_addr: Option<String>,
    pub dialable_addr: Option<String>,
    pub bootstrap_peers: Option<Vec<String>>,
    pub validator_seed: Option<u64>,
    pub rpc_addr: Option<String>,
    pub data_dir: Option<String>,
    pub max_message_size: Option<u32>,
    pub network_namespace: Option<String>,
    pub consensus_namespace: Option<String>,
    pub block_interval_ms: Option<u64>,
    pub validators: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum ConfigError {
    ReadConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseToml {
        path: PathBuf,
        source: toml::de::Error,
    },
    InvalidSocketAddr {
        field: &'static str,
        value: String,
        source: std::net::AddrParseError,
    },
    InvalidBootstrapPeer {
        value: String,
        reason: String,
    },
    InvalidValidator {
        value: String,
        reason: String,
    },
    EmptyValidators,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadConfig { path, source } => {
                write!(f, "failed to read config file '{}': {source}", path.display())
            }
            Self::ParseToml { path, source } => {
                write!(f, "failed to parse TOML config '{}': {source}", path.display())
            }
            Self::InvalidSocketAddr {
                field,
                value,
                source,
            } => write!(f, "invalid {field} '{value}': {source}"),
            Self::InvalidBootstrapPeer { value, reason } => {
                write!(f, "failed to parse bootstrap peer '{value}': {reason}")
            }
            Self::InvalidValidator { value, reason } => {
                write!(f, "failed to parse validator '{value}': {reason}")
            }
            Self::EmptyValidators => {
                write!(f, "validators must not be empty when explicitly configured")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadConfig { source, .. } => Some(source),
            Self::ParseToml { source, .. } => Some(source),
            Self::InvalidSocketAddr { source, .. } => Some(source),
            Self::InvalidBootstrapPeer { .. }
            | Self::InvalidValidator { .. }
            | Self::EmptyValidators => None,
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            identity: IdentityConfig::default(),
            rpc: RpcConfig::default(),
            storage: StorageConfig::default(),
            consensus: ConsensusStartupConfig::default(),
            validators: None,
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

fn parse_socket_addr(field: &'static str, value: String) -> Result<SocketAddr, ConfigError> {
    value
        .parse()
        .map_err(|source| ConfigError::InvalidSocketAddr {
            field,
            value,
            source,
        })
}

fn parse_validator_hex(value: &str) -> Result<ed25519::PublicKey, String> {
    let bytes = from_hex(value).ok_or_else(|| format!("invalid validator public key hex: {value}"))?;
    let mut reader = bytes.as_slice();
    let public_key = ed25519::PublicKey::read_cfg(&mut reader, &())
        .map_err(|err| format!("invalid validator public key: {err}"))?;
    if !reader.is_empty() {
        return Err("invalid validator public key length".to_string());
    }
    Ok(public_key)
}

fn parse_validator_list(values: Vec<String>) -> Result<Vec<ed25519::PublicKey>, ConfigError> {
    if values.is_empty() {
        return Err(ConfigError::EmptyValidators);
    }

    values
        .into_iter()
        .map(|value| {
            parse_validator_hex(&value).map_err(|reason| ConfigError::InvalidValidator { value, reason })
        })
        .collect()
}

fn load_toml_config(path: &Path) -> Result<TomlConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::ReadConfig {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&contents).map_err(|source| ConfigError::ParseToml {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load_config(args: NodeArgs) -> Result<NodeConfig, ConfigError> {
    let defaults = NodeConfig::default();
    let file_config = match args.config.as_deref() {
        Some(path) => Some(load_toml_config(path)?),
        None => None,
    };

    let network_namespace = args
        .network_namespace
        .clone()
        .or_else(|| file_config.as_ref().and_then(|cfg| cfg.network_namespace.clone()));
    let consensus_namespace = args
        .consensus_namespace
        .clone()
        .or_else(|| file_config.as_ref().and_then(|cfg| cfg.consensus_namespace.clone()));
    let listen_addr = match args.listen_addr {
        Some(addr) => addr,
        None => match file_config.as_ref().and_then(|cfg| cfg.listen_addr.clone()) {
            Some(value) => parse_socket_addr("listen_addr", value)?,
            None => defaults.network.listen_addr,
        },
    };
    let dialable_addr = match args.dialable_addr {
        Some(addr) => addr,
        None => match file_config.as_ref().and_then(|cfg| cfg.dialable_addr.clone()) {
            Some(value) => parse_socket_addr("dialable_addr", value)?,
            None => defaults.network.dialable_addr,
        },
    };
    let rpc_addr = match args.rpc_addr {
        Some(addr) => addr,
        None => match file_config.as_ref().and_then(|cfg| cfg.rpc_addr.clone()) {
            Some(value) => parse_socket_addr("rpc_addr", value)?,
            None => defaults.rpc.bind_addr,
        },
    };
    let validator_seed = args
        .validator_seed
        .or_else(|| file_config.as_ref().and_then(|cfg| cfg.validator_seed))
        .unwrap_or(defaults.identity.seed);
    let data_dir = args
        .data_dir
        .clone()
        .or_else(|| {
            file_config
                .as_ref()
                .and_then(|cfg| cfg.data_dir.as_ref().map(PathBuf::from))
        })
        .unwrap_or_else(|| defaults.storage.data_dir.clone());
    let max_message_size = args
        .max_message_size
        .or_else(|| file_config.as_ref().and_then(|cfg| cfg.max_message_size))
        .unwrap_or(defaults.network.max_message_size);
    let block_interval = args
        .block_interval_ms
        .or_else(|| file_config.as_ref().and_then(|cfg| cfg.block_interval_ms))
        .map(Duration::from_millis)
        .unwrap_or(defaults.consensus.block_interval);

    let bootstrap_peer_strings = if !args.bootstrap_peer.is_empty() || !args.dial_peer.is_empty() {
        args.bootstrap_peer
            .into_iter()
            .chain(args.dial_peer)
            .collect::<Vec<_>>()
    } else {
        file_config
            .as_ref()
            .and_then(|cfg| cfg.bootstrap_peers.clone())
            .unwrap_or_default()
    };
    let bootstrap_peers = bootstrap_peer_strings
        .into_iter()
        .map(|value| {
            parse_bootstrap_peer(&value)
                .map_err(|reason| ConfigError::InvalidBootstrapPeer { value, reason })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let validators = if !args.validator.is_empty() {
        Some(parse_validator_list(args.validator)?)
    } else {
        match file_config.and_then(|cfg| cfg.validators) {
            Some(values) => Some(parse_validator_list(values)?),
            None => None,
        }
    };

    Ok(NodeConfig {
        network: NetworkConfig {
            namespace: network_namespace
                .map(String::into_bytes)
                .unwrap_or(defaults.network.namespace),
            listen_addr,
            dialable_addr,
            bootstrap_peers,
            max_message_size,
        },
        identity: IdentityConfig {
            seed: validator_seed,
        },
        rpc: RpcConfig { bind_addr: rpc_addr },
        storage: StorageConfig { data_dir },
        consensus: ConsensusStartupConfig {
            namespace: consensus_namespace
                .map(String::into_bytes)
                .unwrap_or(defaults.consensus.namespace),
            block_interval,
        },
        validators,
    })
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
        let validators = if args.validator.is_empty() {
            None
        } else {
            Some(
                args.validator
                    .into_iter()
                    .map(|validator| match parse_validator_hex(&validator) {
                        Ok(parsed) => parsed,
                        Err(err) => panic!("failed to parse validator '{validator}': {err}"),
                    })
                    .collect(),
            )
        };

        Self {
            network: NetworkConfig {
                namespace: args
                    .network_namespace
                    .map(String::into_bytes)
                    .unwrap_or(defaults.network.namespace),
                listen_addr: args.listen_addr.unwrap_or(defaults.network.listen_addr),
                dialable_addr: args.dialable_addr.unwrap_or(defaults.network.dialable_addr),
                bootstrap_peers,
                max_message_size: args.max_message_size.unwrap_or(defaults.network.max_message_size),
            },
            identity: IdentityConfig {
                seed: args.validator_seed.unwrap_or(defaults.identity.seed),
            },
            rpc: RpcConfig {
                bind_addr: args.rpc_addr.unwrap_or(defaults.rpc.bind_addr),
            },
            storage: StorageConfig {
                data_dir: args.data_dir.unwrap_or(defaults.storage.data_dir),
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
            validators,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commonware_cryptography::Signer;
    use commonware_p2p::Ingress;
    use commonware_utils::hex;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_CONFIG_ID: AtomicU64 = AtomicU64::new(0);

    fn validator_hexes(seeds: &[u64]) -> Vec<String> {
        seeds
            .iter()
            .map(|seed| hex(ed25519::PrivateKey::from_seed(*seed).public_key().as_ref()))
            .collect()
    }

    fn write_config_file(contents: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "whirlpool-node-test-config-{}.toml",
            NEXT_TEMP_CONFIG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, contents).expect("failed to write config file");
        path
    }

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
        assert_eq!(config.validators, None);
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

    #[test]
    fn test_parse_bootstrap_peer_malformed_variants() {
        assert!(parse_bootstrap_peer("deadbeef127.0.0.1:3000").is_err());
        assert!(parse_bootstrap_peer("@127.0.0.1:3000").is_err());
        let pk = ed25519::PrivateKey::from_seed(1).public_key();
        let pk_hex = hex(pk.as_ref());
        assert!(parse_bootstrap_peer(&format!("{pk_hex}@")).is_err());
        assert!(parse_bootstrap_peer("ZZZZ@127.0.0.1:3000").is_err());
        assert!(parse_bootstrap_peer("aabb@127.0.0.1:3000").is_err());
        assert!(parse_bootstrap_peer(&format!("{pk_hex}@not-an-addr")).is_err());
    }

    #[test]
    fn test_node_args_to_node_config_full_custom() {
        let pk1 = ed25519::PrivateKey::from_seed(10).public_key();
        let pk2 = ed25519::PrivateKey::from_seed(20).public_key();
        let peer1 = format!("{}@10.0.0.1:5000", hex(pk1.as_ref()));
        let peer2 = format!("{}@10.0.0.2:6000", hex(pk2.as_ref()));
        let validators = validator_hexes(&[10, 20]);

        let args = NodeArgs {
            config: None,
            listen_addr: Some("0.0.0.0:9000".parse().unwrap()),
            dialable_addr: Some("1.2.3.4:9000".parse().unwrap()),
            bootstrap_peer: vec![peer1],
            dial_peer: vec![peer2],
            validator_seed: Some(42),
            validator: validators.clone(),
            rpc_addr: Some("0.0.0.0:8080".parse().unwrap()),
            data_dir: Some(PathBuf::from("/tmp/whirlpool")),
            max_message_size: Some(2_000_000),
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
        assert_eq!(config.validators.unwrap().len(), validators.len());
    }

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

        assert_eq!(config.network.bootstrap_peers.len(), 3);
        assert_eq!(config.network.bootstrap_peers[0].0, pk1);
        assert_eq!(config.network.bootstrap_peers[1].0, pk2);
        assert_eq!(config.network.bootstrap_peers[2].0, pk3);
    }

    #[test]
    fn test_node_args_default_roundtrip() {
        let args = NodeArgs::parse_from(["whirlpool-node"]);
        let from_args = NodeConfig::from(args);
        let from_default = NodeConfig::default();

        assert_eq!(from_args, from_default);
    }

    #[test]
    fn tst_01_toml_file_loading() {
        let validator = validator_hexes(&[44]).pop().unwrap();
        let bootstrap_key = ed25519::PrivateKey::from_seed(45).public_key();
        let bootstrap_peer = format!("{}@127.0.0.1:4010", hex(bootstrap_key.as_ref()));
        let path = write_config_file(&format!(
            "listen_addr = \"127.0.0.1:4011\"\ndialable_addr = \"10.0.0.1:4011\"\nbootstrap_peers = [\"{bootstrap_peer}\"]\nvalidator_seed = 99\nrpc_addr = \"127.0.0.1:9555\"\ndata_dir = \"custom-data\"\nmax_message_size = 2097152\nnetwork_namespace = \"toml-net\"\nconsensus_namespace = \"toml-consensus\"\nblock_interval_ms = 1234\nvalidators = [\"{validator}\"]\n"
        ));

        let config = load_config(NodeArgs {
            config: Some(path),
            listen_addr: None,
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: None,
            validator: vec![],
            rpc_addr: None,
            data_dir: None,
            max_message_size: None,
            network_namespace: None,
            consensus_namespace: None,
            block_interval_ms: None,
        })
        .expect("toml config should load");

        assert_eq!(config.network.listen_addr, "127.0.0.1:4011".parse().unwrap());
        assert_eq!(config.network.dialable_addr, "10.0.0.1:4011".parse().unwrap());
        assert_eq!(config.network.bootstrap_peers.len(), 1);
        assert_eq!(config.identity.seed, 99);
        assert_eq!(config.rpc.bind_addr, "127.0.0.1:9555".parse().unwrap());
        assert_eq!(config.storage.data_dir, PathBuf::from("custom-data"));
        assert_eq!(config.network.max_message_size, 2_097_152);
        assert_eq!(config.network.namespace, b"toml-net");
        assert_eq!(config.consensus.namespace, b"toml-consensus");
        assert_eq!(config.consensus.block_interval, Duration::from_millis(1234));
        assert_eq!(config.validators.unwrap().len(), 1);
    }

    #[test]
    fn tst_02_cli_overrides_toml() {
        let path = write_config_file(
            "listen_addr = \"127.0.0.1:4011\"\nrpc_addr = \"127.0.0.1:9555\"\nvalidator_seed = 7\nmax_message_size = 1000\nnetwork_namespace = \"toml-net\"\nvalidators = [\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"]\n",
        );
        let cli_validators = validator_hexes(&[2, 3]);

        let config = load_config(NodeArgs {
            config: Some(path),
            listen_addr: Some("0.0.0.0:5000".parse().unwrap()),
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: Some(42),
            validator: cli_validators.clone(),
            rpc_addr: Some("0.0.0.0:8546".parse().unwrap()),
            data_dir: None,
            max_message_size: Some(2048),
            network_namespace: Some("cli-net".into()),
            consensus_namespace: None,
            block_interval_ms: None,
        })
        .expect("cli should override toml");

        assert_eq!(config.network.listen_addr, "0.0.0.0:5000".parse().unwrap());
        assert_eq!(config.rpc.bind_addr, "0.0.0.0:8546".parse().unwrap());
        assert_eq!(config.identity.seed, 42);
        assert_eq!(config.network.max_message_size, 2048);
        assert_eq!(config.network.namespace, b"cli-net");
        assert_eq!(config.validators.unwrap().len(), cli_validators.len());
    }

    #[test]
    fn tst_03_no_config_backward_compat() {
        let validator = validator_hexes(&[8]).pop().unwrap();
        let args = NodeArgs {
            config: None,
            listen_addr: Some("0.0.0.0:9000".parse().unwrap()),
            dialable_addr: Some("1.2.3.4:9000".parse().unwrap()),
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: Some(88),
            validator: vec![validator],
            rpc_addr: Some("0.0.0.0:8546".parse().unwrap()),
            data_dir: Some(PathBuf::from("compat-data")),
            max_message_size: Some(4096),
            network_namespace: Some("compat-net".into()),
            consensus_namespace: Some("compat-cons".into()),
            block_interval_ms: Some(555),
        };

        let expected = NodeConfig::from(args.clone());
        let actual = load_config(args).expect("load_config without file should match From<NodeArgs>");

        assert_eq!(actual, expected);
    }

    #[test]
    fn tst_04_multi_validator_from_toml() {
        let validator_hexes = validator_hexes(&[1, 2, 3, 4]);
        let validator_list = validator_hexes
            .iter()
            .map(|value| format!("\"{value}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let path = write_config_file(&format!("validators = [{validator_list}]\n"));

        let config = load_config(NodeArgs {
            config: Some(path),
            listen_addr: None,
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: None,
            validator: vec![],
            rpc_addr: None,
            data_dir: None,
            max_message_size: None,
            network_namespace: None,
            consensus_namespace: None,
            block_interval_ms: None,
        })
        .expect("multi-validator toml should parse");

        let validators = config.validators.expect("validators should exist");
        assert_eq!(validators.len(), 4);
        assert_eq!(validators[0], ed25519::PrivateKey::from_seed(1).public_key());
        assert_eq!(validators[3], ed25519::PrivateKey::from_seed(4).public_key());
    }

    #[test]
    fn tst_08_missing_config_file_error() {
        let missing = std::env::temp_dir().join("whirlpool-node-missing-config.toml");
        let err = load_config(NodeArgs {
            config: Some(missing.clone()),
            listen_addr: None,
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: None,
            validator: vec![],
            rpc_addr: None,
            data_dir: None,
            max_message_size: None,
            network_namespace: None,
            consensus_namespace: None,
            block_interval_ms: None,
        })
        .expect_err("missing config should error");

        assert!(matches!(err, ConfigError::ReadConfig { ref path, .. } if path == &missing));
        assert!(err.to_string().contains("failed to read config file"));
    }

    #[test]
    fn tst_09_invalid_toml_error() {
        let path = write_config_file("listen_addr = [\n");
        let err = load_config(NodeArgs {
            config: Some(path),
            listen_addr: None,
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: None,
            validator: vec![],
            rpc_addr: None,
            data_dir: None,
            max_message_size: None,
            network_namespace: None,
            consensus_namespace: None,
            block_interval_ms: None,
        })
        .expect_err("invalid toml should error");

        assert!(matches!(err, ConfigError::ParseToml { .. }));
        assert!(err.to_string().contains("failed to parse TOML config"));
    }

    #[test]
    fn tst_10_partial_toml_cli_merge() {
        let path = write_config_file(
            "dialable_addr = \"10.0.0.2:5000\"\ndata_dir = \"toml-data\"\nconsensus_namespace = \"toml-cons\"\n",
        );

        let config = load_config(NodeArgs {
            config: Some(path),
            listen_addr: Some("0.0.0.0:5000".parse().unwrap()),
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: None,
            validator: vec![],
            rpc_addr: None,
            data_dir: None,
            max_message_size: Some(8192),
            network_namespace: Some("cli-net".into()),
            consensus_namespace: None,
            block_interval_ms: Some(2500),
        })
        .expect("partial merge should work");

        assert_eq!(config.network.listen_addr, "0.0.0.0:5000".parse().unwrap());
        assert_eq!(config.network.dialable_addr, "10.0.0.2:5000".parse().unwrap());
        assert_eq!(config.storage.data_dir, PathBuf::from("toml-data"));
        assert_eq!(config.network.namespace, b"cli-net");
        assert_eq!(config.consensus.namespace, b"toml-cons");
        assert_eq!(config.network.max_message_size, 8192);
        assert_eq!(config.consensus.block_interval, Duration::from_millis(2500));
    }

    #[test]
    fn tst_11_empty_validators_rejection() {
        let path = write_config_file("validators = []\n");
        let err = load_config(NodeArgs {
            config: Some(path),
            listen_addr: None,
            dialable_addr: None,
            bootstrap_peer: vec![],
            dial_peer: vec![],
            validator_seed: None,
            validator: vec![],
            rpc_addr: None,
            data_dir: None,
            max_message_size: None,
            network_namespace: None,
            consensus_namespace: None,
            block_interval_ms: None,
        })
        .expect_err("empty validators should fail");

        assert!(matches!(err, ConfigError::EmptyValidators));
    }
}
