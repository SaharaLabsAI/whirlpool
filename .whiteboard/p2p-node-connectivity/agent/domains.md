# Domains and Cross-Cutting Concerns

## Domain Model

### Startup config aggregate
- `crates/whirlpool-node` owns a single runtime startup aggregate:

```rust
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub identity: IdentityConfig,
    pub rpc: RpcConfig,
    pub storage: StorageConfig,
    pub consensus: ConsensusStartupConfig,
}
```

- `NodeConfig` is assembled from CLI args before async runtime startup.
- `NodeConfig` is the only source of truth for startup values previously hardcoded in `main.rs`.

### Network config domain
- Network-specific startup state is modeled as:

```rust
pub struct NetworkConfig {
    pub network_namespace: Vec<u8>,
    pub listen_addr: SocketAddr,
    pub dialable_addr: SocketAddr,
    pub bootstrap_peers: Vec<Bootstrapper<ed25519::PublicKey>>,
    pub max_message_size: u32,
}
```

- `network_namespace` is the byte namespace consumed by `CommonwareNetworkProviderBuilder::new(...)`.
- `listen_addr` is the local socket bind address.
- `dialable_addr` is the externally advertised ingress address.
- `bootstrap_peers` is the complete startup peer list supplied to Commonware discovery.
- `max_message_size` remains a node-owned numeric startup limit.

### Peer bootstrap domain
- The concrete peer entry type is:

```rust
pub type BootstrapPeer = Bootstrapper<ed25519::PublicKey>;
```

- In the underlying Commonware dependency, `Bootstrapper<P>` is `(P, Ingress)` and `Ingress` is satisfied here by `SocketAddr`.
- Each configured peer therefore has two required pieces of authenticated startup identity:
  - `ed25519::PublicKey`
  - `SocketAddr`
- This means the node cannot represent startup peers as bare addresses if it wants to stay inside the existing builder contract.

### Identity domain
- Startup identity remains intentionally minimal:

```rust
pub struct IdentityConfig {
    pub validator_seed: u64,
}
```

- The signer is deterministically derived as `ed25519::PrivateKey::from_seed(validator_seed)`.
- The startup validator set remains `Vec<ed25519::PublicKey>` with the single local signer public key.
- No additional identity source such as private key hex or keystore path is part of this pass.

### RPC domain
- RPC startup state is independent from P2P startup state:

```rust
pub struct RpcConfig {
    pub bind_addr: SocketAddr,
}
```

- It is parsed from the same CLI/config module so all externally visible bind addresses come from one startup source.

### Storage domain
- Storage startup state uses one root path:

```rust
pub struct StorageConfig {
    pub data_dir: PathBuf,
}
```

- Derived subpaths remain fixed helpers rather than independently configured fields:
  - `runtime_dir() -> PathBuf` yields `<data-dir>/runtime`
  - `state_db_path() -> PathBuf` yields `<data-dir>/state`
  - `mempool_db_path() -> PathBuf` yields `<data-dir>/mempool`
- This keeps the domain model small while still enabling separate per-node state roots.

### Consensus startup domain
- Consensus retains its own startup-specific values:

```rust
pub struct ConsensusStartupConfig {
    pub consensus_namespace: String,
    pub block_interval: Duration,
}
```

- `consensus_namespace` remains distinct from `network_namespace` because the two values are consumed by different subsystems today.
- `block_interval` stays in the config domain even though REQ-4/REQ-5 are network-oriented, because it already belongs to startup-owned constants and should no longer be left outside the aggregate config model.

## CLI Input Model
- Recommended parser type:

```rust
#[derive(clap::Parser, Debug, Clone)]
pub struct NodeArgs {
    pub network_namespace: String,
    pub consensus_namespace: String,
    pub listen_addr: SocketAddr,
    pub dialable_addr: SocketAddr,
    pub bootstrap_peers: Vec<String>,
    pub dial_peers: Vec<String>,
    pub validator_seed: u64,
    pub rpc_addr: SocketAddr,
    pub data_dir: PathBuf,
    pub max_message_size: u32,
}
```

- `NodeArgs` is a parse model.
- `NodeConfig` is the normalized runtime model.
- The conversion from `NodeArgs` to `NodeConfig` is where string peer inputs become `Vec<Bootstrapper<ed25519::PublicKey>>`.

## Ownership Boundaries
- `crates/whirlpool-node` owns:
  - CLI parsing
  - default values
  - peer-string parsing
  - namespace separation
  - storage path derivation
  - conversion into concrete builder input types
- `crates/p2p-commonware` owns:
  - the builder contract
  - consumption of `SocketAddr`, bootstrapper tuples, validator sets, and max message size
- `crates/p2p` owns stable channel and transport abstraction contracts and is not part of this sub-intent.

## Cross-Cutting Invariants
- Startup values must be explicit and typed before the runtime starts.
- Defaults must preserve today's behavior when no CLI flags are provided.
- `--dial-peer` and `--bootstrap-peer` must normalize into the same internal `bootstrap_peers` field; they are not separate routing domains in this pass.
- Bootstrap peers must always include authenticated peer identity plus address; a bare `SocketAddr` is not a valid builder input.
- Network namespace and consensus namespace must remain separate fields so their consumers are unambiguous.
- Storage subpaths must derive from a single root `data_dir` to avoid partial path drift between runtime, state, and mempool storage.

## Data Flow
1. `NodeArgs` is parsed from CLI via Clap.
2. Each `bootstrap_peers` and `dial_peers` string is parsed from `PUBKEY@HOST:PORT` into `Bootstrapper<ed25519::PublicKey>`.
3. Parsed peers are merged into `NodeConfig.network.bootstrap_peers`.
4. `validator_seed` becomes `ed25519::PrivateKey`, then `ed25519::PublicKey`.
5. `main.rs` feeds `network_namespace`, `listen_addr`, `dialable_addr`, `bootstrap_peers`, `max_message_size`, and `validators` into `CommonwareNetworkProviderBuilder`.
6. `main.rs` feeds `consensus_namespace`, `block_interval`, `rpc.bind_addr`, and derived storage paths into the rest of node startup.

## Failure and Edge Cases
- Invalid bootstrap peer strings fail CLI parsing early.
- Empty bootstrap peer list is valid and preserves current local-dev startup.
- Repeated `--bootstrap-peer` and `--dial-peer` values are allowed; deduplication is not required in this pass.
- `dialable_addr` may remain `127.0.0.1:0` by default even though it is not production-useful; preserving current behavior takes priority for this pass.
- A single validator derived from `validator_seed` remains the only supported validator identity mode.

## Testability Notes
- `parse_bootstrap_peer("PUBKEY@127.0.0.1:3000")` is unit-testable without starting the runtime.
- `NodeArgs` to `NodeConfig` conversion is unit-testable for default preservation and dial/bootstrap normalization.
- Storage helper methods are unit-testable with simple `PathBuf` assertions.
- `main.rs` startup wiring remains testable by constructing a builder with config-derived values and asserting the wiring path compiles and sequences correctly.
