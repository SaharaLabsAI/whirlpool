# Exploration: Node Config And Startup Wiring

## Scope
- Sub-Intent B covers REQ-4 and REQ-5 only.
- Focus crate: `crates/whirlpool-node`.
- Integration boundary: `crates/p2p-commonware` builder inputs are already sufficient and remain read-only in this pass.

## Current Config Surface

### `crates/whirlpool-node/src/config.rs`
- `NAMESPACE: &[u8] = b"sahara-chain-v0"`
- `BLOCK_INTERVAL: Duration = Duration::from_secs(5)`
- `BIND_ADDR: &str = "127.0.0.1:0"`
- `VALIDATOR_SEED: u64 = 0`
- `RPC_BIND_ADDR: &str = "127.0.0.1:8545"`

### Hardcoded Startup Values In `crates/whirlpool-node/src/main.rs`
- `APPLICATION_NAMESPACE: &[u8] = b"whirlpool-dev"` is separate from `config::NAMESPACE` and hardcoded in the binary.
- `MAX_MESSAGE_SIZE: u32 = 1024 * 1024` is fixed in the binary.
- Runtime storage path is hardcoded to `data/runtime`.
- State DB path is hardcoded to `data/state`.
- Mempool DB path is hardcoded to `data/mempool`.
- Signer is always built from `config::VALIDATOR_SEED`.
- Validator set is always `[self]`.
- P2P listen address is always `127.0.0.1:0` via `SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)`.
- P2P dialable address is always `127.0.0.1:0` via a second inline `SocketAddr` construction.
- Bootstrappers are always `vec![]`.
- RPC bind address is parsed from `config::RPC_BIND_ADDR`, so it is constant-backed rather than user-supplied.

## Gap Analysis
- REQ-4 requires explicit listen addresses from CLI/config, but the node always binds to ephemeral localhost.
- REQ-4 requires dial peer input, but no dial peer collection exists in `config.rs` or `main.rs`.
- REQ-4 requires bootstrap peer input, but startup always sends `vec![]` into `.bootstrappers(...)`.
- REQ-5 requires configured values to reach the provider builder, but the builder currently receives inline defaults rather than user-owned configuration.
- Identity input is not yet a configurable concept; the node always derives the keypair from `VALIDATOR_SEED`. That is workable for a local dev default, but multi-node startup needs a decision on whether to keep seed-based input, add explicit private key input, or support both.
- `config::BIND_ADDR` exists but is unused by startup wiring, so the config module does not currently control actual network startup behavior.
- `APPLICATION_NAMESPACE` and `config::NAMESPACE` diverge; this can confuse which namespace actually drives consensus versus network isolation when CLI/config is introduced.

## CLI Framework Recommendation
- Use `clap` with derive macros for argument parsing.
- Reasoning:
  - `clap` is the de facto Rust CLI standard and fits a single-binary crate well.
  - Sub-Intent B needs typed parsing for `SocketAddr`, repeated peer flags, and defaults; `clap` handles those cases cleanly.
  - A search across `Cargo.toml` files found no first-party workspace member already using `clap`, but vendored ecosystems in this repository already use Clap 4.x, including `vendor/commonware/Cargo.toml` with `clap = "4.5.18"` and `vendor/reth/Cargo.toml` with `clap = "4"`.
  - Because the top-level workspace excludes `vendor` and has no `[workspace.dependencies]` entry for `clap`, `crates/whirlpool-node` would need its own direct dependency declaration rather than `workspace = true`.
- Recommendation: adopt Clap 4 derive support directly in `crates/whirlpool-node` and prefer a `4.5.x` line to stay close to vendored usage patterns.

## Config Struct Design Sketch

### Recommended Layering
- Keep `src/config.rs` as the home for defaults and parse helpers.
- Introduce a runtime-owned `NodeConfig` struct produced from CLI args.
- Treat current constants as fallback defaults for local development instead of as the only configuration source.

### Proposed `NodeConfig` Shape
```rust
pub struct NodeConfig {
    pub namespace: Vec<u8>,
    pub block_interval: Duration,
    pub listen_addr: SocketAddr,
    pub dialable_addr: SocketAddr,
    pub dial_peers: Vec<SocketAddr>,
    pub bootstrap_peers: Vec<BootstrapPeerConfig>,
    pub validator_source: ValidatorIdentityConfig,
    pub rpc_bind_addr: SocketAddr,
    pub state_db_path: PathBuf,
    pub runtime_storage_dir: PathBuf,
    pub mempool_db_path: PathBuf,
    pub max_message_size: u32,
}
```

### Supporting Types
```rust
pub struct BootstrapPeerConfig {
    pub public_key: ed25519::PublicKey,
    pub address: SocketAddr,
}

pub enum ValidatorIdentityConfig {
    Seed(u64),
    // Optional future path if explicit key material is required.
    PrivateKeyHex(String),
}
```

### Default Intent
- `namespace`: preserve existing behavior by defaulting to current node/network namespace values until the naming mismatch is reconciled.
- `block_interval`: default to 5 seconds.
- `listen_addr`: default to `127.0.0.1:0`.
- `dialable_addr`: default to `127.0.0.1:0` unless the CLI chooses to reuse `listen_addr` when omitted.
- `dial_peers`: default empty.
- `bootstrap_peers`: default empty.
- `validator_source`: default to `Seed(0)` to preserve current single-node local behavior.
- `rpc_bind_addr`: default to `127.0.0.1:8545`.
- `state_db_path`: default `data/state`.
- `runtime_storage_dir`: default `data/runtime`.
- `mempool_db_path`: default `data/mempool`.
- `max_message_size`: default 1 MiB.

## Parsing Considerations
- `SocketAddr` can be parsed directly by Clap for `listen_addr`, `dialable_addr`, `dial_peers`, and `rpc_bind_addr`.
- `dial_peers` should likely use a repeatable flag such as `--dial-peer <ip:port>` so the startup config can collect multiple direct peers.
- `bootstrap_peers` need a compound parse because the builder expects `(PublicKey, SocketAddr)` tuples.
- Practical format recommendation for bootstrappers: a repeatable `PUBKEY@HOST:PORT` string parsed into `BootstrapPeerConfig`.
- If explicit key material replaces `VALIDATOR_SEED`, use a separate flag rather than overloading the seed field. That keeps dev defaults intact while allowing a later secure path.

## Startup Wiring Analysis

### Current Startup Points That Need To Change
- `main()` currently constructs runtime state with no parsed node-level config object.
- `let signer = ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED);`
  - Change to derive the signer from `NodeConfig.validator_source`.
- `let validators = vec![signer.public_key()];`
  - Keep the current self-validator default, but source the final validator list from configured identity input and any future explicit validator list policy.
- `let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);`
  - Replace with `node_config.listen_addr`.
- `let dialable_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);`
  - Replace with `node_config.dialable_addr`.
- `let bootstrappers = vec![];`
  - Replace with parsed bootstrap peer tuples converted into the builder's `Bootstrapper<PublicKey>` shape.
- `.initial_validators(0, validators.clone())`
  - Keep this call, but feed it the validator values derived from config rather than inline defaults.
- `.bootstrappers(bootstrappers)`
  - Keep this call, but ensure it receives parsed config values.
- RPC startup currently parses `config::RPC_BIND_ADDR` inline.
  - Replace with `node_config.rpc_bind_addr` so network-facing ports come from one configuration source.
- Path constants for runtime/state/mempool remain hardcoded.
  - They are not part of REQ-4/REQ-5, but if a `NodeConfig` is introduced it is cleaner to carry them in the same struct so startup configuration remains centralized.

### Builder Wiring Impact
- `CommonwareNetworkProviderBuilder` already accepts the required P2P inputs, so Sub-Intent B only needs to map parsed values into:
  - `.listen_addr(node_config.listen_addr)`
  - `.dialable_addr(node_config.dialable_addr)`
  - `.initial_validators(0, validators.clone())`
  - `.bootstrappers(parsed_bootstrappers)`
- Dial peers are not currently passed into the builder based on the explored API surface.
- Because REQ-4 explicitly calls for dial peer acceptance, one of two design outcomes is needed in implementation planning:
  - treat dial peers as startup-managed outbound connection targets handled outside the current builder, or
  - confirm whether the existing provider startup path already consumes dial targets through another existing hook not surfaced in this exploration.
- This is the narrowest remaining ambiguity inside REQ-4/REQ-5; listen and bootstrap wiring are directly supported today, while dial peer injection needs a concrete integration point in `whirlpool-node` startup design.

## Recommended Implementation Direction For Sub-Intent B
1. Add `clap` derive support to `crates/whirlpool-node`.
2. Introduce a `CliArgs` or `NodeArgs` parser type in `src/config.rs` and convert it into `NodeConfig`.
3. Preserve current constants as defaults so `cargo run -p whirlpool-node` still behaves like the existing local dev node.
4. Replace the inline P2P startup values in `src/main.rs` with `NodeConfig` fields.
5. Add dedicated parse helpers for bootstrap peer strings and validator identity input so startup wiring stays readable.

## Open Design Questions For Strategy
- Should Sub-Intent B be CLI-only, or should it also define a config file format in this pass?
- Should validator identity remain `--validator-seed` for now, or should the node move to explicit private key material immediately?
- What exact string format should be accepted for bootstrap peers so `PublicKey + SocketAddr` parsing is unambiguous?
- Where should direct dial peers be consumed if the current builder surface only exposes listen, dialable, bootstrap, and validators?
