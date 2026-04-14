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

mod parse;

use self::parse::{load_toml_config, parse_socket_addr, parse_validator_hex, parse_validator_list};

pub type BootstrapPeer = (ed25519::PublicKey, Ingress);

pub const APPLICATION_NAMESPACE: &[u8] = b"whirlpool-dev";
pub const NAMESPACE: &[u8] = b"sahara-chain-v0";
pub const BLOCK_INTERVAL: Duration = Duration::from_secs(5);
pub const BIND_ADDR: &str = "127.0.0.1:0";
pub const VALIDATOR_SEED: u64 = 0;
pub const RPC_BIND_ADDR: &str = "127.0.0.1:8545";
pub const MEM_RPC_BIND_ADDR: &str = "127.0.0.1:8645";
pub const DEFAULT_DATA_DIR: &str = "data";
pub const DEFAULT_MAX_MESSAGE_SIZE: u32 = 1024 * 1024;

pub use self::parse::parse_bootstrap_peer;

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
    pub mem_rpc_addr: Option<SocketAddr>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub identity: IdentityConfig,
    pub rpc: RpcConfig,
    pub storage: StorageConfig,
    pub consensus: ConsensusStartupConfig,
    /// Optional startup peer set hint used only for discovery bootstrap.
    ///
    /// This does not define simplex consensus authority; simplex validators are
    /// sourced from the genesis-backed registry.
    pub bootstrap_validators: Option<Vec<ed25519::PublicKey>>,
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
    pub mem_bind_addr: SocketAddr,
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
    pub mem_rpc_addr: Option<String>,
    pub data_dir: Option<String>,
    pub max_message_size: Option<u32>,
    pub network_namespace: Option<String>,
    pub consensus_namespace: Option<String>,
    pub block_interval_ms: Option<u64>,
    #[serde(alias = "validators")]
    pub bootstrap_validators: Option<Vec<String>>,
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
    EmptyBootstrapValidators,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadConfig { path, source } => {
                write!(
                    f,
                    "failed to read config file '{}': {source}",
                    path.display()
                )
            }
            Self::ParseToml { path, source } => {
                write!(
                    f,
                    "failed to parse TOML config '{}': {source}",
                    path.display()
                )
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
            Self::EmptyBootstrapValidators => {
                write!(
                    f,
                    "bootstrap validators must not be empty when explicitly configured"
                )
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
            | Self::EmptyBootstrapValidators => None,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            namespace: APPLICATION_NAMESPACE.to_vec(),
            listen_addr: BIND_ADDR
                .parse()
                .expect("default listen address must be valid"),
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
            mem_bind_addr: MEM_RPC_BIND_ADDR
                .parse()
                .expect("default mem RPC bind address must be valid"),
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

pub fn load_config(args: NodeArgs) -> Result<NodeConfig, ConfigError> {
    let defaults = NodeConfig::default();
    let file_config = match args.config.as_deref() {
        Some(path) => Some(load_toml_config(path)?),
        None => None,
    };

    let network_namespace = args.network_namespace.clone().or_else(|| {
        file_config
            .as_ref()
            .and_then(|cfg| cfg.network_namespace.clone())
    });
    let consensus_namespace = args.consensus_namespace.clone().or_else(|| {
        file_config
            .as_ref()
            .and_then(|cfg| cfg.consensus_namespace.clone())
    });
    let listen_addr = match args.listen_addr {
        Some(addr) => addr,
        None => match file_config.as_ref().and_then(|cfg| cfg.listen_addr.clone()) {
            Some(value) => parse_socket_addr("listen_addr", value)?,
            None => defaults.network.listen_addr,
        },
    };
    let dialable_addr = match args.dialable_addr {
        Some(addr) => addr,
        None => match file_config
            .as_ref()
            .and_then(|cfg| cfg.dialable_addr.clone())
        {
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
    let mem_rpc_addr = match args.mem_rpc_addr {
        Some(addr) => addr,
        None => match file_config
            .as_ref()
            .and_then(|cfg| cfg.mem_rpc_addr.clone())
        {
            Some(value) => parse_socket_addr("mem_rpc_addr", value)?,
            None => defaults.rpc.mem_bind_addr,
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

    let bootstrap_validators = if !args.validator.is_empty() {
        Some(parse_validator_list(args.validator)?)
    } else {
        match file_config.and_then(|cfg| cfg.bootstrap_validators) {
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
        rpc: RpcConfig {
            bind_addr: rpc_addr,
            mem_bind_addr: mem_rpc_addr,
        },
        storage: StorageConfig { data_dir },
        consensus: ConsensusStartupConfig {
            namespace: consensus_namespace
                .map(String::into_bytes)
                .unwrap_or(defaults.consensus.namespace),
            block_interval,
        },
        bootstrap_validators,
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
        let bootstrap_validators = if args.validator.is_empty() {
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
                max_message_size: args
                    .max_message_size
                    .unwrap_or(defaults.network.max_message_size),
            },
            identity: IdentityConfig {
                seed: args.validator_seed.unwrap_or(defaults.identity.seed),
            },
            rpc: RpcConfig {
                bind_addr: args.rpc_addr.unwrap_or(defaults.rpc.bind_addr),
                mem_bind_addr: args.mem_rpc_addr.unwrap_or(defaults.rpc.mem_bind_addr),
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
            bootstrap_validators,
        }
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
