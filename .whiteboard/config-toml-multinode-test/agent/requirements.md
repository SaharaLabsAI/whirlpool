# Requirements

## Intent
Support config.toml file loading for whirlpool-node and set up a 4-node integration test that verifies P2P connectivity and block height growth.

## Functional Requirements

### REQ-1: Config TOML File Support
The whirlpool-node binary MUST support loading configuration from a TOML file via a `--config <path>` CLI flag. The TOML schema MUST cover all fields currently in `NodeArgs` (listen_addr, dialable_addr, bootstrap_peer, dial_peer, validator_seed, rpc_addr, data_dir, max_message_size, network_namespace, consensus_namespace, block_interval_ms). Additionally, a `validators` field MUST be supported to specify the full validator set (list of hex-encoded public keys).

### REQ-2: Config Layering (CLI > TOML > Defaults)
CLI arguments MUST override TOML values, and TOML values MUST override built-in defaults. This is a standard config precedence model.

### REQ-3: Multi-Validator Support
The node MUST accept a list of validator public keys (hex-encoded ed25519) from config, replacing the current hardcoded single-validator setup. All nodes in a network MUST share the same validator set.

### REQ-4: 4-Node Integration Test
An integration test MUST spin up 4 whirlpool-node instances in-process, each with:
- Unique validator_seed, listen_addr, dialable_addr, rpc_addr, data_dir
- Shared validator set (all 4 public keys)
- Bootstrap peers pointing to each other
- Same namespace and consensus configuration

### REQ-5: Block Height Growth Verification
The integration test MUST verify that block height grows across all 4 nodes within a bounded time window. This proves P2P connectivity, consensus participation, and block propagation are working.

### REQ-6: P2P Connectivity Verification
The test MUST verify that all 4 nodes establish P2P connections and can exchange messages (proven transitively by consensus progress).

## Non-Functional Requirements

### NFR-1: Backward Compatibility
Existing CLI-only usage (no --config flag) MUST continue to work identically.

### NFR-2: Test Determinism
The multi-node test MUST use deterministic seeds and localhost networking to ensure reproducibility.

### NFR-3: Test Timeout
The multi-node test MUST complete within a reasonable timeout (e.g., 60s) or fail explicitly.
