# Crate Contract: whirlpool-node

## Changes Summary
Add TOML config file support, multi-validator configuration, and extract `start_node()` for testability.

## New Dependencies
- `toml = "0.8"` — TOML deserialization
- `serde = { version = "1", features = ["derive"] }` — Deserialize derive

## API Changes

### config.rs

#### New: `TomlConfig` struct
```rust
#[derive(Debug, Default, Deserialize)]
pub struct TomlConfig {
    pub listen_addr: Option<String>,
    pub dialable_addr: Option<String>,
    pub bootstrap_peers: Option<Vec<String>>,
    pub validator_seed: Option<u64>,
    pub rpc_addr: Option<String>,
    pub data_dir: Option<String>,
    pub max_message_size: Option<usize>,
    pub network_namespace: Option<String>,
    pub consensus_namespace: Option<String>,
    pub block_interval_ms: Option<u64>,
    pub validators: Option<Vec<String>>,  // hex-encoded ed25519 pubkeys
}
```

#### Modified: `NodeArgs`
```rust
// New fields added:
#[arg(long)]
pub config: Option<PathBuf>,  // --config <path.toml>

#[arg(long)]
pub validator: Vec<String>,  // --validator <hex_pubkey> (repeatable)
```

#### Modified: `NodeConfig`
```rust
// New field:
pub validators: Option<Vec<ed25519::PublicKey>>,
```

#### New: `load_config(args: NodeArgs) -> Result<NodeConfig>`
Replaces direct `From<NodeArgs>` with TOML-aware loading:
1. If `args.config` is Some, read and parse TOML
2. Merge CLI over TOML over defaults
3. Parse validator hex strings to PublicKey

### main.rs / lib.rs

#### New: `start_node(config: NodeConfig) -> NodeHandle`
Extracted from main(). Runs node in background, returns handle with:
- RPC address (actual bound port)
- Shutdown signal
- JoinHandle

#### Modified: `main()`
Calls `load_config(NodeArgs::parse())?` then `start_node(config)`.

#### Modified: Validator setup
```rust
// Before:
let validators = vec![signer.public_key()];

// After:
let validators = config.validators
    .unwrap_or_else(|| vec![signer.public_key()]);
```

## Behavioral Contracts
- BC-1: With no --config flag, behavior is identical to current (backward compat)
- BC-2: CLI args override TOML values when both are specified
- BC-3: Invalid TOML or missing file produces clear error and exits
- BC-4: Empty validators list is rejected at startup
- BC-5: start_node() is async and returns when the node is ready to accept connections
