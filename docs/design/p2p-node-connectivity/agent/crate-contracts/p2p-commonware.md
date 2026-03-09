# Crate Contract: p2p-commonware

## Scope
- Crate: `crates/p2p-commonware`
- Requirements: `REQ-1`, `REQ-2`, `REQ-3`
- In-scope files:
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/p2p-commonware/src/receiver.rs`
  - `crates/p2p-commonware/src/sender.rs`
  - `crates/p2p-commonware/src/lib.rs`
  - `crates/p2p-commonware/src/traits.rs`
- Out of scope:
  - `vendor/commonware/**`
  - `crates/p2p/**` API redesign
  - relay wiring in `crates/consensus-simplex/**`

## Public API Changes

### `crates/p2p-commonware/src/provider.rs`
- `CommonwareNetworkProviderBuilder::bootstrappers(...)` remains part of the builder API and becomes a required integration path for bootstrap discovery under `REQ-2`.
- `CommonwareNetworkProviderBuilder::initial_validators(epoch, validators)` remains part of the builder API and becomes the canonical input for validator seeding under `REQ-1`.
- `CommonwareNetworkProviderBuilder::build(context)` keeps returning `(CommonwareNetworkProvider<Ctx, C>, OracleHandle<C::PublicKey>)` with no trait-surface break, but its contract changes from passive construction to active initialization:
  - it must thread `bootstrappers` into `discovery::Config::local(...)`
  - it must apply non-empty `initial_validators` through `OracleHandle::update_validators(...)` before returning
- `OracleHandle::update_validators(...)` remains the only exposed validator update primitive; no new public seeding API is introduced elsewhere.

### `crates/p2p-commonware/src/receiver.rs`
- `CommonwareReceiver::new(...)` changes signature to require the concrete `p2p::Channel` for the wrapped receiver instance.
- `CommonwareReceiver<R>` gains a stored channel as part of its public construction contract, but it does not expose new methods.

### `crates/p2p-commonware/src/lib.rs`
- `MultiplexReceiver::new(...)` remains stable at the crate boundary.
- Construction of `CommonwareReceiver` values must now pass explicit channel identifiers for each registered channel.

### `crates/p2p-commonware/src/sender.rs`
- No public API change.
- Send path remains channel-directed by the caller-provided `p2p::Channel`.

### `crates/p2p-commonware/src/traits.rs`
- No interface redesign.
- This file remains the canonical local import surface for Commonware transport traits and `PerChannelNetwork`.

## Internal Changes

### `crates/p2p-commonware/src/provider.rs`
- `CommonwareNetworkProviderBuilder` continues to store:
  - `bootstrappers: Vec<Bootstrapper<C::PublicKey>>`
  - `initial_validators: Option<(u64, Vec<C::PublicKey>)>`
- `CommonwareNetworkProviderBuilder::build(context)` must:
  1. consume builder-owned bootstrap peers when building `discovery::Config::local(...)`
  2. construct `discovery::Network` and clone the oracle into `OracleHandle`
  3. if `initial_validators` is `Some((epoch, validators))` and `validators` is non-empty, call `oracle_handle.update_validators(epoch, validators.clone()).await` before returning from the builder path
  4. preserve empty-validator behavior by skipping the oracle update when the vector is empty
- `NetworkProvider::start()` implementation must instantiate:
  - `CommonwareReceiver::new(Channel::VOTE, vote_receiver)`
  - `CommonwareReceiver::new(Channel::CERTIFICATE, cert_receiver)`
  - `CommonwareReceiver::new(Channel::RESOLVER, res_receiver)`
- `start_per_channel()` remains dedicated-channel transport setup and does not add new relay behavior in this pass.

### `crates/p2p-commonware/src/receiver.rs`
- `CommonwareReceiver::recv()` must wrap inbound bytes as `NetworkMessage { channel: self.channel, data, peer_id }`.
- The existing authenticated sender extraction from the Commonware receiver stays unchanged.
- No receiver-local fallback such as `Channel(0)` may remain.

### `crates/p2p-commonware/src/lib.rs`
- `MultiplexReceiver::recv()` should stop compensating for broken receiver metadata and instead trust the per-channel `CommonwareReceiver` instances to emit correct channel values.
- The round-robin polling strategy remains unchanged.

### `crates/p2p-commonware/src/sender.rs`
- Confirm that `send(&self, channel, data, recipients)` continues to route via the sender associated with the selected `Channel`; no bootstrap or validator logic belongs here.

### `crates/p2p-commonware/src/traits.rs`
- Keep local trait imports consistent via `crate::traits::...` when sibling modules need transport traits.

## New Types
- No new top-level public types are required.
- `CommonwareReceiver<R>` gains a new stored field:
  - `channel: p2p::Channel` in `crates/p2p-commonware/src/receiver.rs`
- Existing builder state remains sufficient; no new builder companion type is needed.

## Modified Functions

### `crates/p2p-commonware/src/provider.rs`
- `CommonwareNetworkProviderBuilder::build<Ctx>(self, context: Ctx)`
  - apply validator seeding through the returned oracle handle before returning
  - keep bootstrap peers threaded into `discovery::Config::local(...)`
- `impl NetworkProvider for CommonwareNetworkProvider<E, C>::start(mut self)`
  - pass concrete channel IDs into each `CommonwareReceiver::new(...)` call
- `CommonwareNetworkProvider::start_per_channel(mut self)`
  - no behavioral redesign; confirm channel registration remains `VOTE`, `CERTIFICATE`, `RESOLVER`

### `crates/p2p-commonware/src/receiver.rs`
- `CommonwareReceiver::new(...)`
  - accept and store `Channel`
- `impl NetworkReceiver for CommonwareReceiver<R>::recv(&mut self)`
  - emit stored channel, not `Channel(0)`

### `crates/p2p-commonware/src/lib.rs`
- `impl NetworkReceiver for MultiplexReceiver<R>::recv(&mut self)`
  - preserve the existing round-robin behavior without channel repair logic once per-channel receivers are fixed

### `crates/p2p-commonware/src/sender.rs`
- `impl NetworkSender for CommonwareSender<S>::send(...)`
  - unchanged behavior; validate no regression in channel-based sender selection assumptions

## Traceability
- `REQ-1` -> `crates/p2p-commonware/src/provider.rs` builder seeding path
- `REQ-2` -> `crates/p2p-commonware/src/provider.rs` bootstrap threading path
- `REQ-3` -> `crates/p2p-commonware/src/receiver.rs` channel preservation, with matching constructor updates in `crates/p2p-commonware/src/provider.rs` and `crates/p2p-commonware/src/lib.rs`

## Implementation Constraints
- Do not change `crates/p2p/src/traits.rs` or `crates/p2p/src/types.rs`.
- Do not modify `vendor/commonware/**`.
- Do not add relay/mailbox behavior for `crates/consensus-simplex` in this sub-intent.
- Preserve compatibility with empty bootstrapper and empty validator inputs.
