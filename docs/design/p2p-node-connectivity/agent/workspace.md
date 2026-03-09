# Workspace Integration Plan

## Goal
- Make the existing Commonware-backed P2P provider complete enough for node startup to seed validators, supply bootstrap peers, and preserve inbound channel metadata.
- Limit this pass to the integration path spanning `crates/whirlpool-node` -> `crates/p2p-commonware` -> stable `crates/p2p` contracts.

## Integration Path
1. `crates/whirlpool-node/src/main.rs` derives node signer identity and validator set during startup.
2. The node constructs `CommonwareNetworkProviderBuilder` with namespace, listen address, dialable address, max message size, bootstrap peers, and initial validators.
3. `crates/p2p-commonware/src/provider.rs` builds the Commonware runtime/provider, injects bootstrappers into the vendor setup, and immediately seeds validators through `OracleHandle::update_validators(...)`.
4. `CommonwareNetworkProvider::start()` exposes sender/receiver handles implementing the stable `crates/p2p` traits.
5. `crates/p2p-commonware/src/receiver.rs` emits `NetworkMessage` values tagged with the actual mux channel, preserving compatibility for later consensus relay wiring.

## Workspace-Level Decisions

### Single seeding point
- Validator seeding happens exactly once in `CommonwareNetworkProviderBuilder::build(context)`.
- This avoids duplicated seeding responsibilities between node startup and provider startup.
- The workspace contract becomes: callers provide initial validators to the builder; the provider applies them to the oracle.

### Explicit bootstrap threading
- Bootstrap peers are a first-class startup input, not an implicit side effect of dial peers.
- The node wiring layer owns deciding which bootstrap peers to provide.
- The provider layer owns passing those peers into Commonware discovery without reinterpretation.

### Stable channel contract
- Channel IDs remain defined only in `crates/p2p`.
- `crates/p2p-commonware` is responsible for faithfully transporting those IDs across the vendor mux boundary.
- Downstream crates should not add remapping logic to compensate for the current `Channel(0)` bug.

## Implementation Ordering
1. Fix `crates/p2p-commonware/src/receiver.rs` constructor and message wrapping so channel metadata is preserved.
2. Update `crates/p2p-commonware/src/provider.rs` and `crates/p2p-commonware/src/lib.rs` call sites to pass real channels and apply validator seeding.
3. Update `crates/whirlpool-node/src/main.rs` to populate builder bootstrapper and validator inputs.
4. Leave downstream relay/config work untouched until Sub-Intent B and C passes consume the corrected provider behavior.

## Validation Expectations
- Unit/integration coverage should confirm provider build with non-empty validators triggers an oracle update path.
- Startup tests should confirm bootstrap peers are present in the provider configuration when node wiring supplies them.
- Receive-path tests should confirm inbound vote/certificate/resolver messages retain the originating channel in `NetworkMessage.channel`.

## Risks Managed In This Pass
- Startup with no validator seeding currently creates discovery/admission blind spots; centralized oracle initialization removes that gap.
- Empty bootstrapper wiring currently prevents discovery-based expansion beyond direct peers; explicit builder threading addresses this.
- Channel metadata loss currently blocks correct downstream dispatch; receiver-owned channel state resolves it without trait churn.

## Deferred To Later Passes
- CLI/config parsing and user-facing peer configuration surfaces.
- Consensus relay enablement and mailbox delivery.
- Multi-node end-to-end topology orchestration and operational defaults.
