# Crate Contract: whirlpool-node

## Scope
- Crate: `crates/whirlpool-node`
- Requirement focus: `REQ-2` primary, `REQ-1` integration support
- In-scope file:
  - `crates/whirlpool-node/src/main.rs`
- Out of scope:
  - CLI/config expansion for user-specified peer lists
  - consensus relay wiring
  - app-layer or vendor networking changes

## Public API Changes
- No exported API changes.
- The node binary remains responsible for startup-time construction of `CommonwareNetworkProviderBuilder` in `crates/whirlpool-node/src/main.rs`.
- The runtime contract changes only at the wiring layer:
  - node startup must pass initial validator identities into `CommonwareNetworkProviderBuilder::initial_validators(...)`
  - node startup must pass bootstrap peers into `CommonwareNetworkProviderBuilder::bootstrappers(...)`

## Internal Changes

### `crates/whirlpool-node/src/main.rs`
- Reuse the already-derived validator set created during startup as the canonical source for the provider builder.
- Convert that validator set once into the Commonware public-key form expected by `CommonwareNetworkProviderBuilder::initial_validators(epoch, validators)`.
- Supply a bootstrap peer list through `CommonwareNetworkProviderBuilder::bootstrappers(...)` instead of always leaving it empty by construction.
- Preserve existing defaults for:
  - `APPLICATION_NAMESPACE`
  - `MAX_MESSAGE_SIZE`
  - ephemeral `127.0.0.1:0` listen and dialable addresses when explicit config is absent
- Continue keeping the returned `oracle_handle` alive for runtime lifetime management.
- Do not call `oracle_handle.update_validators(...)` directly in `main.rs`; seeding remains centralized in `crates/p2p-commonware/src/provider.rs`.

## New Types
- No new types are introduced in `crates/whirlpool-node/src/main.rs` for this pass.
- Bootstrap peers may be materialized as a local value compatible with `p2p_commonware::Bootstrapper`, but no new persistent struct is required.

## Modified Functions

### `crates/whirlpool-node/src/main.rs`
- `fn main()`
  - derive startup bootstrap inputs before provider build
  - pass `initial_validators(...)` into `CommonwareNetworkProviderBuilder`
  - pass `bootstrappers(...)` into `CommonwareNetworkProviderBuilder`
  - preserve oracle-handle lifetime and existing engine startup flow

## Exact File Paths Referenced
- `crates/whirlpool-node/src/main.rs`
- `crates/p2p-commonware/src/provider.rs`

## Traceability
- `REQ-1` -> node startup provides the initial validator list to the builder so provider-side seeding can occur centrally
- `REQ-2` -> node startup provides bootstrap peers to the builder so Commonware discovery can activate

## Implementation Constraints
- Do not add CLI/config flags in this sub-intent.
- Do not change consensus engine construction beyond the provider input wiring.
- Do not modify `e2e-state.md`, source files outside the documented scope, or vendor code.
