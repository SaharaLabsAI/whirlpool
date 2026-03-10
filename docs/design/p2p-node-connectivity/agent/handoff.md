# Implementation Handoff

## Intent
- Finalize Sub-Intent B for `node-config-startup-wiring`.
- Scope is limited to `REQ-4` and `REQ-5`.
- Only implementation crate changed in this pass: `crates/whirlpool-node`.
- `crates/p2p-commonware` remains read-only and is consumed through its existing builder API.

## Verified Source Baseline
- `crates/whirlpool-node/src/config.rs` currently contains only startup constants and no structured config model.
- `crates/whirlpool-node/src/main.rs` currently owns multiple hardcoded startup literals that must move into config.
- The finalized design intentionally preserves current no-flag behavior while making startup values explicit and configurable.

## Implementation Order
1. Update `crates/whirlpool-node/Cargo.toml`.
   - Add `clap = { version = "4.5", features = ["derive"] }`.
   - Why first: `NodeArgs` depends on clap derive support.
2. Expand `crates/whirlpool-node/src/config.rs`.
   - Add `BootstrapPeer`, `NodeArgs`, `NodeConfig`, nested config structs, defaults, storage helpers, and `parse_bootstrap_peer`.
   - Why second: this file becomes the single startup source of truth used by `main.rs`.
3. Refactor startup wiring in `crates/whirlpool-node/src/main.rs`.
   - Parse CLI before creating the runtime.
   - Convert `NodeArgs` into `NodeConfig`.
   - Replace hardcoded network, identity, storage, RPC, and consensus literals with config fields.
   - Why third: startup wiring depends on the normalized config contract being available.
4. Add unit tests in `crates/whirlpool-node`.
   - Cover defaults, peer parsing, conversion, and storage helpers.
   - Why fourth: validates the config module independently before startup integration checks.
5. Add startup wiring tests in `crates/whirlpool-node`.
   - Cover default backwards compatibility and custom config propagation into builder inputs.
   - Why last: validates the completed config-to-startup flow.

## File-by-File Change Summary

### `crates/whirlpool-node/Cargo.toml`
- Add clap derive dependency only.
- No workspace-wide dependency promotion.
- No additional third-party config library in this pass.

### `crates/whirlpool-node/src/config.rs`
- Retire the current role of this file as a loose constant bag.
- Introduce the canonical startup model:
  - `NodeArgs`
  - `NodeConfig`
  - `NetworkConfig`
  - `IdentityConfig`
  - `RpcConfig`
  - `StorageConfig`
  - `ConsensusStartupConfig`
- Add `parse_bootstrap_peer("PUBKEY@HOST:PORT") -> Result<BootstrapPeer, String>`.
- Merge `bootstrap_peers` and `dial_peers` into the single internal `network.bootstrap_peers` vector.
- Add derived storage helpers:
  - `runtime_dir()`
  - `state_dir()`
  - `mempool_dir()`
- Encode all current startup defaults in one place.

### `crates/whirlpool-node/src/main.rs`
- Keep tracing initialization unchanged.
- Parse `NodeArgs` before `commonware_runtime::tokio::Runner::new(...)`.
- Build runtime storage from `config.storage.runtime_dir()`.
- Derive signer from `config.identity.seed`.
- Pass config-owned values into `p2p_commonware::CommonwareNetworkProviderBuilder`:
  - namespace
  - listen address
  - dialable address
  - bootstrap peers
  - max message size
  - initial validators
- Open state DB from `config.storage.state_dir()`.
- Open mempool DB from `config.storage.mempool_dir()`.
- Bind JSON-RPC server to `config.rpc.bind_addr`.
- Replace the current consensus namespace literal path with `config.consensus.namespace`.
- Wire `config.consensus.block_interval` into the consensus startup path where the current code uses fixed `Duration::from_secs(5)` timing.
- Preserve the current `oracle_handle` lifetime behavior.

## Dependencies Between Changes
- `main.rs` depends on the new `config.rs` types and helpers.
- Tests for startup wiring depend on the config model existing first.
- No implementation step depends on changes to `crates/p2p-commonware`.
- Namespace handling depends on preserving two separate fields because source already shows two different consumers:
  - network namespace for Commonware networking
  - consensus namespace for `consensus_simplex::CommonwareConfig`

## Verification Steps
1. Run crate-local tests covering defaults, parse helpers, conversion, and storage derivation.
2. Run startup wiring tests proving no-flag behavior matches the current source baseline.
3. Run startup wiring tests proving custom config values replace all previous literals.
4. Confirm `crates/whirlpool-node/src/main.rs` no longer hardcodes:
   - `APPLICATION_NAMESPACE`
   - listen and dialable localhost addresses
   - empty bootstrapper list by construction
   - `MAX_MESSAGE_SIZE`
   - `DEFAULT_DB_PATH`
   - `DEFAULT_RUNTIME_STORAGE_DIR`
   - `DEFAULT_MEMPOOL_DB_PATH`
   - inline RPC bind parsing
5. Confirm `crates/p2p-commonware` is untouched.

## Acceptance Checks
- `REQ-4`: node accepts explicit CLI startup inputs for listen address, dialable address, peers, validator seed, RPC, storage root, max message size, and namespaces.
- `REQ-5`: node startup threads configured values into the existing Commonware builder and remaining startup sequence.
- Default launch remains equivalent to today's local-dev startup when no flags are supplied.
- Invalid peer inputs fail before the runtime starts.

## Deferred Beyond This Pass
- Config-file support.
- Multi-validator CLI inputs.
- Keystore or private-key material inputs.
- Peer deduplication.
- Any `crates/p2p-commonware` API redesign.
