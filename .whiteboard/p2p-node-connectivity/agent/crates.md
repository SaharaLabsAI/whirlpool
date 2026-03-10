# Crate Change Specifications

## crates/whirlpool-node

### Scope
- This is the only crate modified in Sub-Intent B.
- It owns REQ-4 and REQ-5: accepting explicit startup configuration and wiring that configuration into the existing P2P builder.
- `crates/p2p-commonware` remains read-only; it is only the consumer boundary for builder inputs.

### Files in Scope
- `crates/whirlpool-node/Cargo.toml`
- `crates/whirlpool-node/src/config.rs`
- `crates/whirlpool-node/src/main.rs`

### `Cargo.toml`
- Add a direct dependency on `clap = { version = "4.5", features = ["derive"] }`.
- No workspace dependency promotion is needed.
- No other new third-party dependencies are required for this pass.

### `src/config.rs`

#### Current problems to resolve
- The file only exposes loose constants today.
- `BIND_ADDR` is unused by startup wiring.
- The runtime/network/storage values consumed by `main.rs` are split between constants here and unrelated hardcoded literals in `main.rs`.

#### Required design outcome
- Convert this module into the canonical startup-config module for the binary.
- Keep small default constants where helpful, but the main product of the module should be a typed `NodeConfig` assembled from Clap args.
- Recommended exported types:

```rust
pub type BootstrapPeer = p2p_commonware::Bootstrapper<ed25519::PublicKey>;

pub struct NodeConfig {
    pub network: NetworkConfig,
    pub identity: IdentityConfig,
    pub rpc: RpcConfig,
    pub storage: StorageConfig,
    pub consensus: ConsensusStartupConfig,
}

pub struct NetworkConfig {
    pub network_namespace: Vec<u8>,
    pub listen_addr: SocketAddr,
    pub dialable_addr: SocketAddr,
    pub bootstrap_peers: Vec<BootstrapPeer>,
    pub max_message_size: u32,
}

pub struct IdentityConfig {
    pub validator_seed: u64,
}

pub struct RpcConfig {
    pub bind_addr: SocketAddr,
}

pub struct StorageConfig {
    pub data_dir: PathBuf,
}

pub struct ConsensusStartupConfig {
    pub consensus_namespace: String,
    pub block_interval: Duration,
}
```

#### Parser types and conversions
- Add a derive-based parser type such as `NodeArgs`:

```rust
#[derive(clap::Parser, Debug, Clone)]
pub struct NodeArgs {
    #[arg(long = "network-namespace", default_value = "whirlpool-dev")]
    pub network_namespace: String,

    #[arg(long = "consensus-namespace", default_value = "sahara-chain-v0")]
    pub consensus_namespace: String,

    #[arg(short = 'l', long = "listen-addr", default_value = "127.0.0.1:0")]
    pub listen_addr: SocketAddr,

    #[arg(long = "dialable-addr", default_value = "127.0.0.1:0")]
    pub dialable_addr: SocketAddr,

    #[arg(short = 'b', long = "bootstrap-peer", value_name = "PUBKEY@HOST:PORT")]
    pub bootstrap_peers: Vec<String>,

    #[arg(long = "dial-peer", alias = "peer", value_name = "PUBKEY@HOST:PORT")]
    pub dial_peers: Vec<String>,

    #[arg(short = 's', long = "validator-seed", default_value_t = 0)]
    pub validator_seed: u64,

    #[arg(short = 'r', long = "rpc-addr", default_value = "127.0.0.1:8545")]
    pub rpc_addr: SocketAddr,

    #[arg(short = 'd', long = "data-dir", default_value = "data")]
    pub data_dir: PathBuf,

    #[arg(long = "max-message-size", default_value_t = 1024 * 1024)]
    pub max_message_size: u32,
}
```

- Implement `NodeArgs::into_config(self) -> Result<NodeConfig, String>` or an equivalent `TryFrom<NodeArgs>` conversion.
- Conversion responsibilities:
  - convert `network_namespace: String` to `Vec<u8>`
  - normalize `bootstrap_peers` and `dial_peers` into one `Vec<BootstrapPeer>`
  - carry defaults forward without relying on `main.rs`

#### Bootstrap peer parsing
- Add a dedicated parse helper in this module:

```rust
pub fn parse_bootstrap_peer(input: &str) -> Result<BootstrapPeer, String>;
```

- Expected input format: `PUBKEY@HOST:PORT`.
- Expected output type: `Bootstrapper<ed25519::PublicKey>` which resolves to `(ed25519::PublicKey, SocketAddr)` at this boundary.
- Parse rules:
  1. split once on `@`
  2. parse address as `SocketAddr`
  3. decode pubkey hex bytes
  4. construct `ed25519::PublicKey` from the 32-byte value
- Parsing failure is a CLI error, not a runtime warning.

#### Dial peer decision
- Do not introduce a separate `dial_peers: Vec<SocketAddr>` field in `NodeConfig`.
- In Commonware, bootstrappers are already the peers dialed on startup.
- Therefore `--dial-peer` is only an operator-friendly alias that feeds the same `bootstrap_peers` collection.
- This crate must document and implement that normalization explicitly so REQ-4 remains unambiguous.

#### Storage helpers
- Add derived-path helpers on `StorageConfig`:

```rust
impl StorageConfig {
    pub fn runtime_dir(&self) -> PathBuf;
    pub fn state_db_path(&self) -> PathBuf;
    pub fn mempool_db_path(&self) -> PathBuf;
}
```

- The derived layout must remain:
  - `<data-dir>/runtime`
  - `<data-dir>/state`
  - `<data-dir>/mempool`

### `src/main.rs`

#### Startup ordering
- Parse CLI args before initializing the runtime executor.
- Replace ad hoc constants and inline `SocketAddr::new(...)` calls with `NodeConfig` fields.
- Keep tracing setup at the top of `main()`.

#### Runtime and storage wiring
- Replace:
  - `DEFAULT_RUNTIME_STORAGE_DIR`
  - `DEFAULT_DB_PATH`
  - `DEFAULT_MEMPOOL_DB_PATH`
- With the values derived from `node_config.storage.data_dir`.
- No separate storage redesign is needed beyond consuming the new helper methods.

#### Network builder wiring
- Replace the current hardcoded builder assembly with config-owned values:

```rust
let (network_provider, oracle_handle) =
    CommonwareNetworkProviderBuilder::new(
        signer.clone(),
        node_config.network.network_namespace.clone(),
    )
    .listen_addr(node_config.network.listen_addr)
    .dialable_addr(node_config.network.dialable_addr)
    .max_message_size(node_config.network.max_message_size)
    .initial_validators(0, validators.clone())
    .bootstrappers(node_config.network.bootstrap_peers.clone())
    .build(context.with_label("network"))
    .await;
```

- Types threaded into the builder are concrete and unchanged from the existing API:
  - `SocketAddr` for `listen_addr`
  - `SocketAddr` for `dialable_addr`
  - `Vec<Bootstrapper<ed25519::PublicKey>>` for `bootstrappers`
  - `Vec<ed25519::PublicKey>` for `initial_validators`

#### Validator identity wiring
- Continue deriving the signer with:

```rust
let signer = ed25519::PrivateKey::from_seed(node_config.identity.validator_seed);
```

- Continue using `vec![signer.public_key()]` as the startup validator set.
- No multi-validator CLI surface is introduced in this pass.

#### Namespace wiring
- Replace hardcoded `APPLICATION_NAMESPACE` usage with `node_config.network.network_namespace`.
- Replace `String::from_utf8_lossy(config::NAMESPACE).to_string()` with `node_config.consensus.consensus_namespace.clone()`.
- This crate is where the current namespace divergence is made explicit and resolved without changing runtime behavior.

#### RPC wiring
- Replace inline parsing of `config::RPC_BIND_ADDR` with `node_config.rpc.bind_addr`.
- Keep the RPC server startup flow otherwise unchanged.

#### Constants to retire or narrow
- Remove or internalize `APPLICATION_NAMESPACE`, `DEFAULT_DB_PATH`, `DEFAULT_RUNTIME_STORAGE_DIR`, and `DEFAULT_MEMPOOL_DB_PATH` once they are represented by `NodeConfig` defaults.
- `MAX_MESSAGE_SIZE` may remain as a config-module default constant, but `main.rs` should stop owning it.

#### Tests in `main.rs`
- Existing startup-wiring tests can remain lightweight, but they should shift from builder-call sequencing to config-to-builder intent.
- Suitable minimal tests for this crate after implementation:
  - default `NodeArgs` conversion preserves current startup behavior
  - bootstrap peer parsing accepts a valid `PUBKEY@HOST:PORT`
  - `dial_peers` and `bootstrap_peers` merge into a single config vector
  - `data_dir` derives the expected `runtime`, `state`, and `mempool` paths

## crates/p2p-commonware
- Read-only in this pass.
- Existing consumed contracts:
  - `CommonwareNetworkProviderBuilder::new(signer, namespace)`
  - `.listen_addr(SocketAddr)`
  - `.dialable_addr(SocketAddr)`
  - `.bootstrappers(Vec<Bootstrapper<PublicKey>>)`
  - `.initial_validators(epoch, Vec<PublicKey>)`
  - `.max_message_size(u32)`
- No crate-local redesign, no new setter, and no startup-side workaround should be proposed here.

## Other crates
- `crates/p2p`: no changes.
- `crates/consensus-simplex`: no changes.
- `crates/app`: no changes.
- `crates/p2p-commonware` design artifacts from Sub-Intent A are not revised in this pass.
