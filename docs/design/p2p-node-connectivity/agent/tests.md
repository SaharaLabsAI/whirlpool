# Test Contracts

## Scope
- Sub-Intent B only: `REQ-4` and `REQ-5`.
- Primary test targets:
  - `crates/whirlpool-node/src/config.rs`
  - `crates/whirlpool-node/src/main.rs`
- Read-only dependency boundary under test-by-consumption:
  - `crates/p2p-commonware`

## Test Strategy
- Framework:
  - Rust crate-local unit tests in `crates/whirlpool-node`
  - startup-focused integration tests in `crates/whirlpool-node/tests/` only if config extraction from `main.rs` makes integration coverage clearer
- Assertion style:
  - exact value assertions for defaults and derived paths
  - exact type-shape assertions for peer parsing and config normalization
  - no network mocking required for config-only tests
- Mocking approach:
  - prefer deterministic pure-function tests over runtime or socket-heavy tests
  - for startup wiring coverage, isolate config-to-builder input assembly into testable helper boundaries if needed; do not mock `p2p-commonware` APIs by redefining their contract
- Failure philosophy:
  - malformed operator input should fail early and explicitly
  - backwards-compatibility tests must prove that no-flag startup preserves current behavior

## Requirement Traceability
- `REQ-4` -> `TST-REQ4-001`, `TST-REQ4-002`, `TST-REQ4-003`, `TST-REQ4-004`, `TST-REQ4-005`
- `REQ-5` -> `TST-REQ5-001`, `TST-REQ5-002`

## Unit Tests

### `TST-REQ4-001` NodeConfig defaults preserve current hardcoded values
- Requirement: `REQ-4`
- Target file:
  - `crates/whirlpool-node/src/config.rs`
- Test type: unit test
- Setup:
  - construct `NodeConfig::default()`
- Assertions:
  - `network.namespace == b"whirlpool-dev".to_vec()`
  - `network.listen_addr == "127.0.0.1:0".parse().unwrap()`
  - `network.dialable_addr == "127.0.0.1:0".parse().unwrap()`
  - `network.bootstrap_peers.is_empty()`
  - `network.max_message_size == 1_048_576`
  - `identity.seed == 0`
  - `rpc.bind_addr == "127.0.0.1:8545".parse().unwrap()`
  - `storage.data_dir == PathBuf::from("data")`
  - `consensus.namespace == "sahara-chain-v0"`
  - `consensus.block_interval == Duration::from_secs(5)`
- Failure caught:
  - default drift from current source-verified startup behavior

### `TST-REQ4-002` `parse_bootstrap_peer` accepts valid peer input
- Requirement: `REQ-4`
- Target file:
  - `crates/whirlpool-node/src/config.rs`
- Test type: unit test
- Setup:
  - generate a deterministic `commonware_cryptography::ed25519::PrivateKey` from a seed
  - serialize its public key to the exact hex format expected by the parser
  - build input `PUBKEY@127.0.0.1:3000`
- Assertions:
  - parser returns `Ok((expected_public_key, "127.0.0.1:3000".parse().unwrap()))`
- Failure caught:
  - parser accepts the format but produces the wrong tuple shape or wrong key bytes

### `TST-REQ4-003` `parse_bootstrap_peer` rejects malformed inputs
- Requirement: `REQ-4`
- Target file:
  - `crates/whirlpool-node/src/config.rs`
- Test type: unit test table
- Cases:
  - missing `@`
  - empty pubkey segment
  - empty address segment
  - invalid hex
  - wrong key length
  - invalid socket address
- Assertions:
  - every case returns `Err(...)`
  - no malformed case is silently accepted or partially normalized
- Failure caught:
  - deferred runtime failure caused by weak CLI validation

### `TST-REQ4-004` `NodeArgs` converts into `NodeConfig` correctly
- Requirement: `REQ-4`
- Target file:
  - `crates/whirlpool-node/src/config.rs`
- Test type: unit test
- Setup:
  - construct `NodeArgs` with explicit non-default values for every field
  - include at least one `bootstrap_peer` and one `dial_peer`
- Assertions:
  - `network.listen_addr`, `network.dialable_addr`, `network.max_message_size`, `identity.seed`, `rpc.bind_addr`, `storage.data_dir`, and `consensus.block_interval` all match input intent
  - `network.bootstrap_peers.len() == bootstrap_peers.len() + dial_peers.len()`
  - peer order is deterministic and documented by the conversion
- Failure caught:
  - alias inputs are lost, fields remain hardcoded, or units are converted incorrectly

### `TST-REQ4-005` storage path derivation stays consistent
- Requirement: `REQ-4`
- Target file:
  - `crates/whirlpool-node/src/config.rs`
- Test type: unit test
- Setup:
  - construct `StorageConfig { data_dir: PathBuf::from("node-a") }`
- Assertions:
  - `runtime_dir() == PathBuf::from("node-a/runtime")`
  - `state_dir() == PathBuf::from("node-a/state")`
  - `mempool_dir() == PathBuf::from("node-a/mempool")`
- Failure caught:
  - inconsistent layout between runtime, state, and mempool storage roots

## Integration Tests

### `TST-REQ5-001` default startup config remains backwards compatible
- Requirement: `REQ-5`
- Target files:
  - `crates/whirlpool-node/src/main.rs`
  - `crates/whirlpool-node/src/config.rs`
- Test type: construction-level integration test
- Setup:
  - parse `NodeArgs` from an empty CLI input equivalent or use `NodeConfig::default()`
  - feed resulting config into the startup assembly path used by `main.rs`
- Assertions:
  - signer derives from seed `0`
  - builder input values match the source-verified legacy literals
  - storage paths remain `data/runtime`, `data/state`, `data/mempool`
  - RPC bind address remains `127.0.0.1:8545`
- Failure caught:
  - config refactor unintentionally changes no-flag node behavior

### `TST-REQ5-002` custom startup config wires all builder inputs
- Requirement: `REQ-5`
- Target files:
  - `crates/whirlpool-node/src/main.rs`
  - `crates/whirlpool-node/src/config.rs`
- Test type: construction-level integration test
- Setup:
  - create explicit config with non-default listen address, dialable address, peers, validator seed, namespaces, RPC address, data dir, max message size, and block interval
- Assertions:
  - `CommonwareNetworkProviderBuilder::new(...)` receives the configured network namespace
  - `.listen_addr(...)`, `.dialable_addr(...)`, `.bootstrappers(...)`, `.max_message_size(...)`, and `.initial_validators(...)` all receive config-derived values
  - state, runtime, mempool, RPC, and consensus configuration use the same config object rather than local literals
- Failure caught:
  - only part of startup becomes configurable, leaving hidden hardcoded paths in `main.rs`

## Suggested Test Layout
- `crates/whirlpool-node/src/config.rs` test module:
  - `tst_req4_001_node_config_defaults_preserve_current_startup`
  - `tst_req4_002_parse_bootstrap_peer_accepts_valid_input`
  - `tst_req4_003_parse_bootstrap_peer_rejects_invalid_input`
  - `tst_req4_004_node_args_normalize_into_node_config`
  - `tst_req4_005_storage_helpers_derive_expected_paths`
- `crates/whirlpool-node/src/main.rs` or `crates/whirlpool-node/tests/startup_config.rs`:
  - `tst_req5_001_default_startup_wiring_is_backwards_compatible`
  - `tst_req5_002_custom_startup_wiring_uses_node_config`

## Completion Criteria
- Every in-scope requirement maps to at least one concrete `TST-*`.
- Pure config behavior is covered without requiring live network execution.
- One test explicitly proves backwards compatibility for no-flag startup.
- One test explicitly proves full custom wiring into the existing Commonware builder path.
