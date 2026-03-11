# Architecture Flows

## Flow 1: Config Loading (startup)

```
CLI args (clap::parse)
    │
    ├── --config <path> provided?
    │   ├── YES: Read file → toml::from_str → TomlConfig
    │   │         Merge: CLI non-defaults overlay TOML values
    │   │         → NodeConfig
    │   └── NO:  NodeArgs → NodeConfig (existing path, unchanged)
    │
    └── NodeConfig ready
        ├── config.identity.seed → ed25519 signer
        ├── config.validators (if set) → Vec<PublicKey>
        │   OR fallback → vec![signer.public_key()]
        ├── config → CommonwareConfig { validators, ... }
        ├── config → CommonwareNetworkProviderBuilder { initial_validators, ... }
        └── config → RPC server, storage paths, etc.
```

## Flow 2: Multi-Node Test Lifecycle

```
test_four_node_consensus()
    │
    ├── Phase 1: Key Generation
    │   for seed in [0, 1, 2, 3]:
    │       signer[seed] = ed25519::from_seed(seed)
    │       pubkey[seed] = signer[seed].public_key()
    │   validators = [pubkey[0], pubkey[1], pubkey[2], pubkey[3]]
    │
    ├── Phase 2: Node Startup
    │   for seed in [0, 1, 2, 3]:
    │       config = NodeConfig {
    │           listen_addr: "127.0.0.1:0",  // OS-assigned
    │           validator_seed: seed,
    │           validators: validators.clone(),
    │           data_dir: tempdir(),
    │           rpc_addr: "127.0.0.1:0",
    │           block_interval: 1s,
    │       }
    │       handle[seed] = start_node(config).await
    │       // Discover actual ports from handle
    │
    ├── Phase 3: Peer Wiring (post-startup)
    │   // Nodes discover each other via bootstrap_peers
    │   // OR: pre-wire sequential boot (node 0 first, others bootstrap to it)
    │
    ├── Phase 4: Connectivity & Height Verification
    │   loop with timeout(60s):
    │       for node in nodes:
    │           height[node] = rpc_call(node.rpc_addr, "eth_blockNumber")
    │       // Log: "Node {seed}: height={h}, connected_peers={p}"
    │       if all(height > target):
    │           PASS → proceed to peer count check
    │       sleep(1s)
    │
    │   // Peer connectivity: with 4 validators, BFT needs 2f+1=3.
    │   // If all 4 reach same height, all must be connected.
    │   // Additionally: each node logs peer connections at INFO level.
    │   // Test captures tracing logs to verify peer connection events.
    │
    │   Assertions:
    │     1. All 4 nodes: block_height > 0 (proves consensus progress)
    │     2. All 4 nodes: block_height within ±1 of each other (proves sync)
    │     3. Tracing logs show peer connection events for each node
    │
    └── Phase 5: Cleanup
        for handle in handles:
            handle.abort()
        // tempdirs auto-cleaned on drop
```

## Flow 3: Config Merge Logic

```
fn merge_config(args: NodeArgs) -> NodeConfig:
    let toml_config = if let Some(path) = args.config:
        read_and_parse(path)?  // TomlConfig
    else:
        TomlConfig::default()  // all None

    // For each field: CLI value if non-default, else TOML value if Some, else default
    NodeConfig {
        network: NetworkConfig {
            listen_addr: args.listen_addr.or(toml_config.listen_addr).unwrap_or(DEFAULT),
            ...
        },
        validators: args.validators.or(toml_config.validators),
        ...
    }
```
