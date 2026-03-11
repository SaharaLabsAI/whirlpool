# Task 3: Extract start_node() for Testability

**Complexity**: M
**Covers**: Enables AC-5, AC-6, AC-7 (test infrastructure)

## Pre-Task Gate
- [ ] Task 2 complete (multi-validator config working)
- [ ] `nix develop --command cargo build -p whirlpool-node` passes
- [ ] `crates/whirlpool-node/src/main.rs` contains the node startup logic

## What to Do

### Step 1: Design NodeHandle
```rust
pub struct NodeHandle {
    pub rpc_addr: SocketAddr,     // actual bound RPC address
    pub listen_addr: SocketAddr,  // actual bound P2P address
    pub public_key: ed25519::PublicKey,
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub join_handle: tokio::task::JoinHandle<()>,
}
```

### Step 2: Extract start_node()
Move the body of `main()` (after CLI parsing) into:
```rust
pub async fn start_node(config: NodeConfig) -> Result<NodeHandle, Box<dyn std::error::Error>> {
    // All existing main.rs logic:
    // - signer creation
    // - validator set
    // - network provider build
    // - state DB open
    // - chain tip recovery
    // - CommonwareConfig creation
    // - mempool creation
    // - app creation
    // - engine start
    // - RPC server start
    // Return NodeHandle with actual bound addresses
}
```

### Step 3: Refactor main()
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = NodeArgs::parse();
    let config = load_config(args)?;
    let handle = start_node(config).await?;
    info!("Node started: rpc={}, p2p={}, pubkey={}", 
        handle.rpc_addr, handle.listen_addr, hex::encode(handle.public_key));
    // Wait for shutdown signal or ctrl-c
    tokio::signal::ctrl_c().await?;
    Ok(())
}
```

### Step 4: Export from lib.rs
```rust
pub mod config;
pub mod persisting_sink;
pub use config::{NodeConfig, NodeArgs, load_config, TomlConfig};
pub use crate::start_node;
pub use crate::NodeHandle;
```

### Step 5: Add peer connection logging
Add INFO-level tracing in start_node() when network provider is wired:
```rust
info!(
    listen_addr = %actual_listen_addr,
    bootstrap_peers = config.network.bootstrap_peers.len(),
    validators = validators.len(),
    "Node P2P layer starting"
);
```

### Step 6: Verify
```bash
nix develop --command cargo build -p whirlpool-node 2>&1
nix develop --command cargo test -p whirlpool-node 2>&1
```

## Post-Task Gate
- [ ] `nix develop --command cargo build -p whirlpool-node` passes
- [ ] `nix develop --command cargo test -p whirlpool-node` passes
- [ ] `start_node()` is exported as public API from `whirlpool_node` crate
- [ ] `main()` is a thin wrapper calling `load_config` + `start_node`
- [ ] Evidence saved to `.sisyphus/evidence/task-3-extract-start-node.txt`

## Mock Boundary
None — refactoring within whirlpool-node only.
