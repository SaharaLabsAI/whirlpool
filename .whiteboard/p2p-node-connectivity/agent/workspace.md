# Workspace Integration Plan

## Goal
- Deliver REQ-4 and REQ-5 by giving `crates/whirlpool-node` a real startup configuration surface and threading those configured values into the existing Commonware network builder.
- Keep the workspace impact intentionally narrow: one crate changes, one new dependency is added, and no transport or consensus contracts move.

## Workspace Impact
- Modified crate: `crates/whirlpool-node`.
- Read-only integration dependency: `crates/p2p-commonware`.
- Stable downstream consumers: `crates/consensus-simplex`, `crates/app`, `crates/p2p`.
- New dependency addition: `clap 4.5.x` with derive support in `crates/whirlpool-node` only.
- No `[workspace.dependencies]` update and no workspace resolver changes are required.

## Integration Path
1. `crates/whirlpool-node/src/config.rs` parses CLI args into `NodeConfig`.
2. `crates/whirlpool-node/src/main.rs` derives signer, validator set, storage paths, and RPC bind address from `NodeConfig`.
3. `crates/whirlpool-node/src/main.rs` constructs `CommonwareNetworkProviderBuilder` using configured namespace, listen address, dialable address, bootstrap peers, validators, and max message size.
4. `crates/p2p-commonware` consumes those values through its existing builder API with no API changes.
5. The rest of node startup remains behaviorally unchanged apart from now being configurable.

## Workspace-Level Decisions

### Single crate ownership
- All new config parsing and startup normalization live in `crates/whirlpool-node`.
- No shared config crate is introduced.
- No other crate becomes aware of Clap or CLI parsing concerns.

### Builder contract stays fixed
- `crates/p2p-commonware` is already sufficient for REQ-4/REQ-5.
- The workspace contract for this sub-intent becomes:
  - node parses startup inputs
  - node converts them into concrete builder types
  - provider consumes them without reinterpretation

### Dial peers map to bootstrappers
- Workspace-level interpretation is explicit: Commonware bootstrappers are the startup dial targets.
- `whirlpool-node` may accept both `--bootstrap-peer` and `--dial-peer` flags, but both normalize to `Vec<Bootstrapper<ed25519::PublicKey>>`.
- No second peer-routing path is introduced elsewhere in the workspace.

### Namespace separation is preserved
- The existing divergence between network namespace and consensus namespace is retained but made explicit.
- `whirlpool-node` owns two separate config values:
  - network namespace for `CommonwareNetworkProviderBuilder`
  - consensus namespace for `CommonwareConfig`
- This prevents accidental workspace-wide coupling between the two consumers.

### Storage configuration remains local
- `--data-dir` is the only new storage-related CLI flag.
- Derived runtime/state/mempool subpaths stay local to `whirlpool-node` and do not create new workspace contracts.

## Concrete Type Threading
- CLI-parsed and config-normalized types crossing crate boundaries:
  - `Vec<u8>` for network namespace
  - `SocketAddr` for P2P listen address
  - `SocketAddr` for P2P dialable address
  - `Vec<p2p_commonware::Bootstrapper<ed25519::PublicKey>>` for bootstrap peers
  - `Vec<ed25519::PublicKey>` for initial validators
  - `u32` for max message size
- Internal-only node startup types:
  - `u64` for validator seed
  - `SocketAddr` for RPC bind address
  - `PathBuf` for `data_dir`
  - `String` for consensus namespace
  - `Duration` for block interval

## Implementation Ordering
1. Update `crates/whirlpool-node/Cargo.toml` with `clap`.
2. Expand `crates/whirlpool-node/src/config.rs` into `NodeArgs` + `NodeConfig` + parse helpers.
3. Replace hardcoded startup literals in `crates/whirlpool-node/src/main.rs` with `NodeConfig` values.
4. Keep `crates/p2p-commonware` untouched; only consume its existing builder setters.

## Validation Expectations
- Default launch behavior remains equivalent to the current single-node local-dev startup when no flags are supplied.
- Passing explicit `--listen-addr`, `--dialable-addr`, and peer flags changes the concrete values sent into `CommonwareNetworkProviderBuilder`.
- Passing `--data-dir` changes all three derived storage locations consistently.
- Passing `--rpc-addr` changes JSON-RPC bind behavior without affecting P2P startup.

## Risks Managed In This Pass
- Hardcoded startup values in `main.rs` currently prevent running multiple nodes with distinct addresses or storage roots; `NodeConfig` removes that coupling.
- The unused `config::BIND_ADDR` currently suggests a config surface that does not really control startup; centralizing config fixes that mismatch.
- The current namespace split is easy to miss; naming both config values explicitly prevents accidental misuse during later implementation.
- The dial-peer requirement is ambiguous against the existing builder API; mapping dial peers to Commonware bootstrappers resolves that ambiguity without changing upstream crates.

## Deferred Beyond This Pass
- Config file support.
- Explicit private key material or keystore support.
- Separate startup semantics for anonymous dial targets that do not include authenticated peer identity.
- Any transport, discovery, or consensus relay changes outside `crates/whirlpool-node`.
