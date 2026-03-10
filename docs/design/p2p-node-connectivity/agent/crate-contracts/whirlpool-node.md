# Crate Contract: whirlpool-node

## Scope
- Sub-Intent B only: `REQ-4` and `REQ-5`.
- Primary implementation crate: `crates/whirlpool-node`.
- Read-only dependency boundary: `crates/p2p-commonware`.
- Source verification anchors:
  - `crates/whirlpool-node/src/config.rs`
  - `crates/whirlpool-node/src/main.rs`

## Current Baseline Verified From Source
- `crates/whirlpool-node/src/config.rs` currently exports only constants:
  - `pub const NAMESPACE: &[u8] = b"sahara-chain-v0";`
  - `pub const BLOCK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);`
  - `pub const BIND_ADDR: &str = "127.0.0.1:0";`
  - `pub const VALIDATOR_SEED: u64 = 0;`
  - `pub const RPC_BIND_ADDR: &str = "127.0.0.1:8545";`
- `crates/whirlpool-node/src/main.rs` currently hardcodes:
  - network namespace `b"whirlpool-dev"`
  - listen address `127.0.0.1:0`
  - dialable address `127.0.0.1:0`
  - empty bootstrappers
  - max message size `1024 * 1024`
  - state path `data/state`
  - runtime storage path `data/runtime`
  - mempool path `data/mempool`
  - consensus namespace from `config::NAMESPACE`
  - RPC bind address from `config::RPC_BIND_ADDR`
- Finalized design replaces those split constants and literals with a single config-owned startup contract while preserving no-flag behavior.

## Final Public API Surface

### Module exports from `crates/whirlpool-node/src/config.rs`

```rust
pub type BootstrapPeer = p2p_commonware::Bootstrapper<commonware_cryptography::ed25519::PublicKey>;

#[derive(clap::Parser, Debug, Clone)]
pub struct NodeArgs {
    pub listen_addr: std::net::SocketAddr,
    pub dialable_addr: std::net::SocketAddr,
    pub bootstrap_peers: Vec<String>,
    pub dial_peers: Vec<String>,
    pub validator_seed: u64,
    pub rpc_addr: std::net::SocketAddr,
    pub data_dir: std::path::PathBuf,
    pub max_message_size: u32,
    pub network_namespace: String,
    pub consensus_namespace: String,
    pub block_interval_ms: u64,
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
    pub listen_addr: std::net::SocketAddr,
    pub dialable_addr: std::net::SocketAddr,
    pub bootstrap_peers: Vec<BootstrapPeer>,
    pub max_message_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityConfig {
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpcConfig {
    pub bind_addr: std::net::SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    pub data_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusStartupConfig {
    pub namespace: String,
    pub block_interval: std::time::Duration,
}

impl Default for NodeConfig {
    fn default() -> Self;
}

impl From<NodeArgs> for NodeConfig {
    fn from(args: NodeArgs) -> Self;
}

impl StorageConfig {
    pub fn runtime_dir(&self) -> std::path::PathBuf;
    pub fn state_dir(&self) -> std::path::PathBuf;
    pub fn mempool_dir(&self) -> std::path::PathBuf;
}

pub fn parse_bootstrap_peer(input: &str) -> Result<BootstrapPeer, String>;
```

## CLI Contract
- `--listen-addr` / `-l` -> `std::net::SocketAddr`
- `--dialable-addr` -> `std::net::SocketAddr`
- `--bootstrap-peer` / `-b` -> repeatable `PUBKEY@HOST:PORT`
- `--dial-peer` -> repeatable alias using the same `PUBKEY@HOST:PORT` format; normalized into `network.bootstrap_peers`
- `--validator-seed` / `-s` -> `u64`
- `--rpc-addr` -> `std::net::SocketAddr`
- `--data-dir` -> `std::path::PathBuf`
- `--max-message-size` -> `u32`
- `--network-namespace` -> `String`; converted to `Vec<u8>` for the Commonware builder
- `--consensus-namespace` -> `String`; passed to `consensus_simplex::CommonwareConfig.namespace`
- `--block-interval-ms` -> `u64`; converted to `std::time::Duration`

## Defaults and Backwards Compatibility Baseline
- `NodeConfig::default()` must preserve today's no-flag startup behavior verified from `crates/whirlpool-node/src/main.rs` and `crates/whirlpool-node/src/config.rs`.
- Required defaults:
  - `network.namespace = b"whirlpool-dev".to_vec()`
  - `network.listen_addr = "127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap()`
  - `network.dialable_addr = "127.0.0.1:0".parse::<std::net::SocketAddr>().unwrap()`
  - `network.bootstrap_peers = Vec::new()`
  - `network.max_message_size = 1_048_576`
  - `identity.seed = 0`
  - `rpc.bind_addr = "127.0.0.1:8545".parse::<std::net::SocketAddr>().unwrap()`
  - `storage.data_dir = std::path::PathBuf::from("data")`
  - `consensus.namespace = "sahara-chain-v0".to_owned()`
  - `consensus.block_interval = std::time::Duration::from_secs(5)`
- Guarantee: launching the node with no CLI flags remains behaviorally equivalent to the current binary.

## Function and Method Contracts

### `impl Default for NodeConfig`
- Purpose: provide the canonical startup baseline previously scattered across constants and `main.rs` literals.
- Preconditions:
  - none
- Postconditions:
  - returns a fully populated config with no empty required fields
  - matches current hardcoded startup values exactly
  - yields storage root `data` whose helpers derive `data/runtime`, `data/state`, and `data/mempool`
- Error handling:
  - no fallible behavior exposed; default literals must be valid by construction

### `impl From<NodeArgs> for NodeConfig`
- Purpose: normalize parsed CLI inputs into runtime-owned types consumed by startup.
- Preconditions:
  - `NodeArgs` has already passed `clap` parsing for scalar fields like `SocketAddr`, `u64`, `u32`, and `PathBuf`
  - peer strings in `bootstrap_peers` and `dial_peers` must be valid `PUBKEY@HOST:PORT` values when converted
- Postconditions:
  - `network.namespace` contains the byte representation of `args.network_namespace`
  - `network.bootstrap_peers` contains parsed `bootstrap_peers` followed by parsed `dial_peers`
  - `identity.seed == args.validator_seed`
  - `rpc.bind_addr == args.rpc_addr`
  - `storage.data_dir == args.data_dir`
  - `consensus.namespace == args.consensus_namespace`
  - `consensus.block_interval == std::time::Duration::from_millis(args.block_interval_ms)`
- Error handling strategy:
  - malformed peer strings are a configuration error and must cause immediate failure at conversion time rather than deferred runtime behavior
  - if implementation keeps `From<NodeArgs>`, malformed peer inputs must already be rejected by parser helpers invoked from clap value parsing or by a constructor that exits before runtime startup
  - if conversion needs explicit fallibility during implementation, `TryFrom<NodeArgs>` is an acceptable refinement so long as the external contract remains fail-fast before the async runtime starts

### `pub fn parse_bootstrap_peer(input: &str) -> Result<BootstrapPeer, String>`
- Purpose: parse operator-facing peer text into the exact Commonware bootstrapper type.
- Input format:
  - `PUBKEY@HOST:PORT`
  - `PUBKEY` is hex-encoded bytes for `commonware_cryptography::ed25519::PublicKey`
  - `HOST:PORT` must parse as `std::net::SocketAddr`
- Preconditions:
  - `input` must contain exactly one `@`
  - pubkey segment must be non-empty and valid for `ed25519::PublicKey`
  - address segment must be non-empty and parse as `SocketAddr`
- Postconditions on success:
  - returns `(public_key, socket_addr)` with no lossy transformation
  - returned tuple is directly consumable by `p2p_commonware::CommonwareNetworkProviderBuilder::bootstrappers(...)`
- Postconditions on failure:
  - returns `Err(String)` describing the malformed segment class: missing separator, invalid hex, wrong key length, invalid public key bytes, or invalid socket address
- Error handling strategy:
  - no warnings, skips, or partial acceptance
  - one malformed peer entry invalidates startup configuration

### `impl StorageConfig`

#### `pub fn runtime_dir(&self) -> std::path::PathBuf`
- Preconditions:
  - `self.data_dir` may be relative or absolute; no existence guarantee required at config time
- Postconditions:
  - returns `self.data_dir.join("runtime")`
  - does not mutate `self`
- Error handling:
  - infallible path derivation

#### `pub fn state_dir(&self) -> std::path::PathBuf`
- Preconditions:
  - same as `runtime_dir`
- Postconditions:
  - returns `self.data_dir.join("state")`
- Error handling:
  - infallible path derivation

#### `pub fn mempool_dir(&self) -> std::path::PathBuf`
- Preconditions:
  - same as `runtime_dir`
- Postconditions:
  - returns `self.data_dir.join("mempool")`
- Error handling:
  - infallible path derivation

## Startup Wiring Contract in `crates/whirlpool-node/src/main.rs`

### `fn main()`
- Required startup order:
  1. initialize tracing exactly as today
  2. parse `NodeArgs` before constructing `commonware_runtime::tokio::Runner`
  3. convert `NodeArgs` into `NodeConfig`
  4. create runtime storage config from `config.storage.runtime_dir()`
  5. derive signer with `commonware_cryptography::ed25519::PrivateKey::from_seed(config.identity.seed)`
  6. derive validators as `vec![signer.public_key()]`
  7. build `p2p_commonware::CommonwareNetworkProviderBuilder` from config-owned fields
  8. construct consensus engine, state DB, mempool DB, and RPC server from config-owned fields
  9. keep `oracle_handle` alive for process lifetime
- Preconditions:
  - configuration parsing completed successfully
  - `config.storage` paths are derivable
- Postconditions:
  - no hardcoded startup literals remain in `main.rs` for listen address, dialable address, validator seed, RPC address, storage directories, namespace selection, or max message size
  - Commonware builder receives:
    - `config.network.namespace.clone()`
    - `config.network.listen_addr`
    - `config.network.dialable_addr`
    - `config.network.bootstrap_peers.clone()`
    - `config.network.max_message_size`
    - `vec![signer.public_key()]` via `.initial_validators(0, ...)`
  - consensus engine receives:
    - `config.consensus.namespace.clone()`
    - `config.consensus.block_interval` where block interval is wired
  - JSON-RPC server binds to `config.rpc.bind_addr`
  - state DB uses `config.storage.state_dir()`
  - runtime storage uses `config.storage.runtime_dir()`
  - mempool DB uses `config.storage.mempool_dir()`
- Error handling strategy:
  - malformed CLI/config input fails before runtime startup
  - storage or DB open failures continue to use existing `expect(...)`-style startup termination unless implementation deliberately upgrades them to richer startup errors without changing behavior
  - no fallback to silent defaults after explicit invalid user input

## Data Ownership and Invariants
- `NodeArgs` is parse-only and may contain operator-friendly strings.
- `NodeConfig` is runtime-owned and contains only normalized values consumed during startup.
- `network.namespace` and `consensus.namespace` remain distinct because current source already shows two different consumers and two different default values.
- `--dial-peer` is a UX alias, not a second peer-routing model.
- bootstrap peers must always include authenticated peer identity; bare socket addresses are not valid Commonware builder inputs in this pass.

## Error Handling Summary
- CLI scalar parsing errors are owned by `clap`.
- Peer normalization errors are owned by `parse_bootstrap_peer` and surfaced immediately.
- No malformed peer entry may be ignored.
- No duplicate-peer deduplication is required in this pass.
- Empty peer lists remain valid.

## Backwards Compatibility Guarantees
- Running `whirlpool-node` with no flags preserves current local-dev behavior.
- Existing single-validator startup remains intact by continuing to derive one signer from a deterministic seed.
- Current directory layout remains intact under `data/` when `--data-dir` is not provided.
- `crates/p2p-commonware` public API is consumed as-is; no upstream builder changes are introduced.
- The node binary gains new flags, but no previously working no-flag startup path regresses.

## Non-Goals
- No config file format in this pass.
- No private-key hex or keystore support in this pass.
- No anonymous dial target support outside `BootstrapPeer`.
- No `crates/p2p-commonware` redesign.
