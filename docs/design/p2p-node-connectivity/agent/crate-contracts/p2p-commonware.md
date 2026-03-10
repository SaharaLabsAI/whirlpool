# Crate Contract: p2p-commonware

## Scope
- Sub-Intent C support contract for `REQ-6` and `REQ-7`.
- Crate: `crates/p2p-commonware`.
- In-scope files:
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/p2p-commonware/src/traits.rs`
  - tests that validate `start_per_channel()` behavior
- Out of scope:
  - vendor network/discovery changes
  - multiplex sender/receiver redesign
  - any reinterpretation of vote/certificate/resolver traffic

## Current Baseline Verified From Source
- `PerChannelNetwork<S, R>` currently exposes three channel pairs: `vote`, `cert`, and `resolver`.
- `CommonwareNetworkProvider::start_per_channel()` registers exactly three channels and returns those three pairs.
- `CommonwareNetworkProvider::start()` separately builds the multiplex sender/receiver over the same registered channels.
- Source comments currently describe the dedicated-channel API as `vote/certificate/resolver` only.

## Required Public API Changes

### `PerChannelNetwork<S, R>`
- Extend the struct with a fourth channel pair:

```rust
pub struct PerChannelNetwork<S, R> {
    pub vote: (S, R),
    pub cert: (S, R),
    pub resolver: (S, R),
    pub payload: (S, R),
    pub network_handle: commonware_runtime::Handle<()>,
}
```

- The existing fields remain unchanged.
- Field order may stay vote/cert/resolver/payload/network_handle for readability and compatibility with the design docs.

### `start_per_channel()` contract
- `CommonwareNetworkProvider::start_per_channel()` and the matching trait method in `crates/p2p-commonware/src/traits.rs` keep the same function signatures but return a `PerChannelNetwork` populated with four channel pairs instead of three.
- This is an additive shape change to the returned struct, not a method redesign.

## Internal Changes

### Channel registration
- `CommonwareNetworkProvider::start_per_channel()` must additionally register:

```rust
self.network.register(Channel::PAYLOAD.0, quota.clone(), backlog)
```

- Existing registrations remain:
  - `Channel::VOTE.0`
  - `Channel::CERTIFICATE.0`
  - `Channel::RESOLVER.0`
- The payload registration uses the same default quota/backlog policy as the existing three channels unless implementation discovers a concrete reason to tune it separately.

### Returned bundle
- The returned `PerChannelNetwork` must include:
  - `payload: (payload_sender, payload_receiver)`
- `network_handle` lifetime behavior remains unchanged.

### `NetworkProvider::start()`
- No change is required for the generic multiplexed sender/receiver contract unless implementation wants parity registration for `Channel::PAYLOAD` there as well.
- Minimum required guarantee for Sub-Intent C: `start_per_channel()` exposes the dedicated payload path consumed by `consensus-simplex`.
- If implementation elects to register `Channel::PAYLOAD` in `start()` too, that is acceptable so long as existing multiplex semantics do not regress.

## Behavioral Contract
- Vote, certificate, and resolver channel routing remains unchanged.
- Payload traffic is exposed as an additional dedicated channel pair and is not consumed by the vendor simplex engine directly.
- `p2p-commonware` remains transport-only here; it does not decode payload envelopes or store consensus blocks.

## Test Contract
- Update `start_per_channel()` tests to assert four dedicated pairs are returned.
- Add or update send/receive coverage proving the payload pair can carry bytes between peers without disturbing vote/certificate/resolver behavior.
- Existing tests for vote/certificate/resolver must continue to pass unchanged in meaning.

## Traceability
- `REQ-6` -> provides the transport leg used by relay activation in `consensus-simplex`
- `REQ-7` -> preserves current channel IDs while adding the dedicated payload channel

## Compatibility Rules
- Do not modify vendor code.
- Do not change the builder API shape.
- Do not remove or renumber any existing channel registration.
- Keep payload support additive and minimal.
