# Domains and Cross-Cutting Concerns

## Domain Model

### Peer identity domain
- Canonical peer identity originates from the existing Ed25519/public-key material already created during node startup.
- `crates/whirlpool-node` owns deriving the initial validator identity list from startup state.
- `crates/p2p-commonware` consumes those identities as `initial_validators` and applies them to Commonware oracle state.

### Discovery domain
- Commonware vendor discovery supports bootstrapper-based peer discovery and manager/oracle-driven peer-set updates.
- `bootstrappers` represent network entry points for discovering additional peers.
- `initial_validators` represent the allowed/expected validator peer set that should be active in the oracle from startup.
- These are related but distinct data sets and must remain separate builder inputs.

### Transport/channel domain
- `crates/p2p` defines `Channel(u64)` and the canonical constants `VOTE`, `CERTIFICATE`, and `RESOLVER`.
- Commonware provides muxed channels underneath the provider implementation.
- `crates/p2p-commonware` is the translation layer that must preserve channel identity when converting vendor receiver output into `NetworkMessage`.

## Ownership Boundaries
- `crates/p2p`: owns network abstraction traits and channel constants; no redesign permitted.
- `crates/p2p-commonware`: owns Commonware-specific runtime assembly, oracle/bootstrap configuration, and translation between vendor receiver/sender primitives and `crates/p2p` contracts.
- `crates/whirlpool-node`: owns startup-time selection/provision of listen address, dialable address, validator seeds, and bootstrap peers.
- `crates/consensus-simplex`: consumes channel-correct `NetworkMessage` values later, but is not modified in this pass.

## Cross-Cutting Invariants
- Builder inputs must be explicit: no hidden derivation of validators or bootstrap peers inside provider internals.
- Oracle seeding must happen before the network provider is handed off for long-running startup usage.
- Channel values must be preserved end-to-end; no receiver may substitute a sentinel/default channel.
- `crate::traits::` is the canonical local import path inside `crates/p2p-commonware` for transport trait references.

## Data Flow
1. Node startup derives signer and validator identities.
2. Node startup constructs builder inputs, including bootstrap peers and initial validators.
3. Provider build creates runtime/oracle state and immediately seeds validator membership.
4. Provider start exposes sender/receiver handles.
5. Per-channel receivers emit `NetworkMessage` values with the stored channel identity intact.

## Failure/Edge Cases
- Empty validator list: provider skips oracle update and behaves as today, but without panicking.
- Empty bootstrapper list: provider still starts, but only direct peer paths are available.
- Unknown/new channels: receiver should preserve the provided `Channel` generically rather than restricting to hard-coded constants.
- Ephemeral listen addresses: still allowed in this pass; completeness work does not depend on stable external addresses.

## Testability Notes
- Validator seeding is testable at the provider build boundary by asserting the oracle update side effect for non-empty input.
- Bootstrap injection is testable by asserting the provider/runtime configuration includes supplied bootstrappers.
- Channel preservation is testable by constructing per-channel receivers and asserting emitted `NetworkMessage.channel` matches the receiver's configured channel.
