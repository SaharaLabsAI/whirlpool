# Strategy

## Scope
- This synthesize pass covers Sub-Intent A only: REQ-1, REQ-2, and REQ-3.
- The goal is to make the existing `crates/p2p` contract fully usable by `crates/p2p-commonware` and `crates/whirlpool-node` without changing trait definitions, vendor code, or introducing new crates.
- Out of scope for this pass: CLI/config surface expansion (REQ-4, REQ-5 follow-on design details), relay wiring in `crates/consensus-simplex` (REQ-6), and broader app compatibility concerns (REQ-7, REQ-8 except for preserving channel IDs).

## Design Intent
- Keep `crates/p2p` as the stable abstraction boundary and close the completeness gaps in the Commonware-backed provider implementation.
- Treat validator seeding, bootstrap peer injection, and channel preservation as startup/runtime wiring fixes rather than architectural redesign.
- Preserve existing commonware discovery and muxing behavior by feeding correct inputs into the existing builder and receiver paths.

## Approach
1. Extend the `CommonwareNetworkProviderBuilder` usage path so caller-supplied validator identities and bootstrap peers are captured before transport start.
2. Apply validator seeding immediately after the Commonware provider runtime is constructed and before node services depend on peer discovery, using the returned `OracleHandle`.
3. Preserve per-channel metadata by carrying the actual mux channel through `CommonwareReceiver` instead of synthesizing `Channel(0)`.
4. Keep canonical imports through `crate::traits::...` in `crates/p2p-commonware` to match local conventions and avoid ad hoc cross-module paths.

## Key Decisions

### Validator Seeding
- The builder remains responsible for collecting `initial_validators` as `Vec<PublicKey>`/peer identities in the Commonware domain.
- `CommonwareNetworkProviderBuilder::build` in `crates/p2p-commonware/src/provider.rs` becomes the single place that applies the initial validator set to the oracle by calling `oracle_handle.update_validators(...)` after the provider/oracle pair is created.
- Seeding occurs once during provider construction, before `CommonwareNetworkProvider::start` is used by downstream consumers, so startup discovery/admission has the intended validator baseline from the first network activity.
- Empty validator lists remain legal; in that case the builder performs no oracle update.

### Bootstrap Peer Injection
- `bootstrappers` stays a builder-owned input on `CommonwareNetworkProviderBuilder` and is populated by the node integration boundary rather than inferred inside the provider.
- `crates/whirlpool-node/src/main.rs` supplies bootstrap peer addresses into the builder alongside the existing namespace, listen address, dialable address, and message size settings.
- `provider.rs` threads `bootstrappers` directly into the Commonware network construction path so vendor bootstrapper-based discovery can activate without additional discovery wrappers.
- Static dial peers and bootstrap peers remain distinct concepts: dial targets are immediate outbound peers; bootstrappers are discovery seeds advertised to commonware.

### Channel Metadata Preservation
- `CommonwareReceiver` in `crates/p2p-commonware/src/receiver.rs` must hold the concrete `Channel` assigned when the per-channel receiver is created.
- `recv()` wraps inbound bytes into `NetworkMessage { channel: self.channel, ... }` instead of hard-coding `Channel(0)`.
- Construction sites in `crates/p2p-commonware/src/provider.rs` and/or `crates/p2p-commonware/src/lib.rs` must pass the actual `VOTE`, `CERTIFICATE`, `RESOLVER`, or other mux channel value into each receiver instance.
- This preserves compatibility with existing `crates/p2p` channel constants and unblocks later relay wiring without changing the trait crate.

## Sequence Plan
1. Update `provider.rs` builder-to-runtime assembly so bootstrappers and validators are both consumed at build time.
2. Update `receiver.rs` and its constructor call sites so each receiver instance owns the real channel identifier.
3. Update node startup wiring in `whirlpool-node/src/main.rs` so the provider builder is fed concrete validator and bootstrap inputs instead of only ephemeral defaults.
4. Leave consensus relay behavior untouched in this pass, but ensure the resulting provider emits correct `NetworkMessage.channel` values for the later Sub-Intent C design.

## Compatibility Rules
- Do not alter `crates/p2p/src/traits.rs`, `types.rs`, or channel constants.
- Do not modify `vendor/commonware/*`.
- Do not add alternative discovery stacks, config loaders, or new transport abstractions.
- Keep node startup behavior compatible with current ephemeral local defaults when no explicit bootstrap peers are supplied, while still seeding validators from the already-created validator set.

## Exit Criteria
- Builder-created providers apply non-empty validator seeds through `OracleHandle` during startup.
- Bootstrap peer inputs are explicitly threaded from node wiring into Commonware provider construction.
- `CommonwareReceiver` preserves the originating channel in every `NetworkMessage`.
- No hard blockers remain for implementing REQ-1, REQ-2, and REQ-3 in the existing crate structure.
