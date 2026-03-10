# Strategy

## Scope
- This synthesize pass covers Sub-Intent B only: REQ-4 and REQ-5.
- Primary implementation crate: `crates/whirlpool-node`.
- `crates/p2p-commonware` is read-only in this pass; its builder API is consumed as-is.
- Out of scope: REQ-1, REQ-2, REQ-3, REQ-6, REQ-7, REQ-8; new config file formats; any `p2p-commonware` API change.

## Design Intent
- Replace the hardcoded startup values in `crates/whirlpool-node/src/main.rs` with a typed startup configuration object parsed before runtime startup.
- Preserve today's local-dev behavior when the binary is launched with no flags.
- Make the node accept explicit network inputs for multi-node startup while staying within the current Commonware builder contract.

## Concrete Decisions

### CLI model
- This pass is CLI-first only. No config file format is introduced.
- `crates/whirlpool-node` adds a direct dependency on `clap = { version = "4.5", features = ["derive"] }`.
- Parsing happens synchronously at the beginning of `main()` before `tokio::Runner::new(...)` is constructed.
- `config.rs` becomes the canonical home for defaults, parse helpers, and `NodeConfig` assembly.

### Canonical config shape
- `config.rs` should expose a runtime-owned `NodeConfig` with nested domains instead of more ad hoc constants.
- Recommended shape:

```rust
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
    pub bootstrap_peers: Vec<Bootstrapper<ed25519::PublicKey>>,
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

- `StorageConfig` should provide helper methods instead of storing duplicated paths:

```rust
impl StorageConfig {
    pub fn runtime_dir(&self) -> PathBuf;
    pub fn state_db_path(&self) -> PathBuf;
    pub fn mempool_db_path(&self) -> PathBuf;
}
```

### Defaults
- Defaults must preserve current behavior when no CLI flags are passed.
- Concrete defaults:
  - `network.network_namespace = b"whirlpool-dev".to_vec()`
  - `network.listen_addr = "127.0.0.1:0".parse::<SocketAddr>().unwrap()`
  - `network.dialable_addr = "127.0.0.1:0".parse::<SocketAddr>().unwrap()`
  - `network.bootstrap_peers = Vec::<Bootstrapper<ed25519::PublicKey>>::new()`
  - `network.max_message_size = 1024 * 1024`
  - `identity.validator_seed = 0`
  - `rpc.bind_addr = "127.0.0.1:8545".parse::<SocketAddr>().unwrap()`
  - `storage.data_dir = PathBuf::from("data")`
  - `consensus.consensus_namespace = "sahara-chain-v0".to_string()`
  - `consensus.block_interval = Duration::from_secs(5)`

### Namespace divergence resolution
- The existing `config::NAMESPACE` and `APPLICATION_NAMESPACE` values represent two different startup consumers and should not be silently merged.
- `NodeConfig` therefore carries two explicit namespace fields:
  - `NetworkConfig.network_namespace: Vec<u8>` for `CommonwareNetworkProviderBuilder::new(...)`
  - `ConsensusStartupConfig.consensus_namespace: String` for `CommonwareConfig.namespace`
- Default values preserve today's behavior: `whirlpool-dev` for network isolation and `sahara-chain-v0` for consensus naming.
- This resolves the ambiguity by making the distinction explicit in config instead of leaving one namespace hidden in `main.rs` and another in `config.rs`.

### Data directory layout
- Use a single CLI flag, `--data-dir`, as the canonical storage root for this pass.
- Derived subpaths stay fixed to preserve current layout:
  - runtime storage: `<data-dir>/runtime`
  - state DB: `<data-dir>/state`
  - mempool DB: `<data-dir>/mempool`
- No separate `--state-db-path`, `--runtime-dir`, or `--mempool-db-path` flags are added in Sub-Intent B.
- Rationale: REQ-4 and REQ-5 need startup configurability, not storage-layout redesign, and one root flag keeps the CLI small while still enabling multi-node runs from different directories.

### Validator identity
- Validator identity remains seed-based in this pass.
- The only explicit identity flag is `--validator-seed <u64>`.
- `main.rs` continues deriving `ed25519::PrivateKey` via `ed25519::PrivateKey::from_seed(node_config.identity.validator_seed)`.
- The initial validator set remains `vec![signer.public_key()]` for this pass.
- No explicit private key material flag is introduced yet.

### Bootstrap and dial peer semantics
- Commonware already defines `Bootstrapper<P>` as `(P, Ingress)` and documents bootstrappers as peers dialed on startup.
- There is no separate builder setter for raw dial peers.
- Design decision: for Sub-Intent B, `dial peers` and `bootstrap peers` are the same operational concept at the `whirlpool-node` boundary.
- The node will therefore normalize both CLI surfaces into the single internal field `NetworkConfig.bootstrap_peers: Vec<Bootstrapper<ed25519::PublicKey>>`.
- Consequences:
  - `--bootstrap-peer` is the canonical flag name.
  - `--dial-peer` is accepted as an alias for operator intent, but it uses the same `PUBKEY@HOST:PORT` value format and lands in the same `bootstrap_peers` vector.
  - There is no separate `Vec<SocketAddr>` field in `NodeConfig` because the existing builder cannot consume anonymous dial targets.
- This keeps REQ-4 truthful without inventing unsupported startup behavior outside the builder.

### Bootstrap peer parse format
- Each peer flag value must be `PUBKEY@HOST:PORT`.
- Parsed target type:

```rust
pub type BootstrapPeer = Bootstrapper<ed25519::PublicKey>;
```

- Parsing steps:
  1. Split once on `@`; reject missing or repeated separator cases.
  2. Parse the right side as `SocketAddr`.
  3. Decode the left side from hex into `Vec<u8>`.
  4. Convert the bytes into `ed25519::PublicKey` via the Commonware codec read path.
  5. Return `(public_key, socket_addr)`.
- Malformed key bytes, odd-length hex, invalid socket addresses, or empty segments must fail during CLI parsing rather than later during runtime startup.

### Exact CLI surface
- Recommended parser type:

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

- Flag semantics:
  - `--network-namespace`: signed network namespace passed to the P2P builder.
  - `--consensus-namespace`: consensus namespace string passed into `CommonwareConfig`.
  - `--listen-addr` / `-l`: socket address the node binds to for inbound P2P.
  - `--dialable-addr`: externally advertised socket address passed to the builder.
  - `--bootstrap-peer` / `-b`: repeatable startup dial target in `PUBKEY@HOST:PORT` format.
  - `--dial-peer`: repeatable alias of `--bootstrap-peer`, normalized into the same parsed peer list.
  - `--validator-seed` / `-s`: deterministic local-dev signer seed.
  - `--rpc-addr` / `-r`: JSON-RPC bind address.
  - `--data-dir` / `-d`: root directory used to derive `runtime`, `state`, and `mempool` paths.
  - `--max-message-size`: P2P message cap passed into the builder.

## Startup Wiring Plan
1. Parse `NodeArgs` at process startup and convert them into `NodeConfig`.
2. Derive the signer from `node_config.identity.validator_seed`.
3. Derive `validators = vec![signer.public_key()]`.
4. Build `CommonwareNetworkProviderBuilder::new(signer.clone(), node_config.network.network_namespace.clone())`.
5. Thread in:
   - `.listen_addr(node_config.network.listen_addr)`
   - `.dialable_addr(node_config.network.dialable_addr)`
   - `.bootstrappers(node_config.network.bootstrap_peers.clone())`
   - `.initial_validators(0, validators.clone())`
   - `.max_message_size(node_config.network.max_message_size)`
6. Replace `DEFAULT_RUNTIME_STORAGE_DIR`, `DEFAULT_DB_PATH`, and `DEFAULT_MEMPOOL_DB_PATH` usage with paths derived from `node_config.storage.data_dir`.
7. Replace inline RPC address parsing with `node_config.rpc.bind_addr`.
8. Replace inline consensus namespace usage with `node_config.consensus.consensus_namespace.clone()`.

## Compatibility Rules
- No new traits, no new builder setters, and no public API changes to `crates/p2p-commonware`.
- Preserve the current single-validator local-dev startup when no flags are provided.
- Keep the returned `oracle_handle` alive exactly as today.
- Accept operator-facing `dial peer` terminology without creating a second peer-routing implementation path.

## Exit Criteria
- `crates/whirlpool-node` has a fully specified `NodeConfig` design with actual Rust field types.
- Exact CLI flags, defaults, and parse behavior are documented.
- The namespace mismatch is resolved through explicit config ownership.
- Dial peers are no longer ambiguous: they are aliases of Commonware bootstrappers for this pass.
- The startup handoff from parsed config into `CommonwareNetworkProviderBuilder` is implementation-ready.
