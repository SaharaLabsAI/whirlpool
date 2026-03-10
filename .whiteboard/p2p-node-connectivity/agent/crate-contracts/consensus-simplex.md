# Crate Contract: consensus-simplex

## Scope
- Sub-Intent C only: `REQ-6`, `REQ-7`, and `REQ-8`.
- Primary implementation crate: `crates/consensus-simplex`.
- Supporting contract boundaries:
  - `crates/p2p`
  - `crates/p2p-commonware`
  - `crates/whirlpool-node`
- Out of scope:
  - `vendor/**`
  - vendor simplex trait or engine API changes
  - vote/certificate/resolver protocol redesign
  - application business-logic changes outside relay activation

## Current Baseline Verified From Source
- `crates/consensus-simplex/src/mailbox.rs` defines `Mailbox<B>` as the adapter passed to the vendor simplex engine as both `Automaton` and `Relay`.
- `Mailbox::broadcast(digest)` is currently a no-op.
- `MailboxActor::remember_block(&block)` already stores genesis and proposed blocks into the shared `BlockStore<A::Block>` keyed by digest.
- `crates/consensus-simplex/src/engine.rs` already constructs one shared `BlockStore<A::Block>` and passes it to both `MailboxActor` and `AppAdapter`.
- `CommonwareEngine::start()` already calls `network.start_per_channel()` and passes `vote`, `cert`, and `resolver` into `simplex::Engine::start(...)`.
- Therefore the missing behavior is application payload relay, not vendor protocol transport.

## Public API Surface Changes

### `crates/consensus-simplex/src/mailbox.rs`
- `Mailbox<B>::new(...)` changes from a mailbox-channel-only constructor to one that also receives relay dependencies.
- `Mailbox<B>` gains new stored fields:

```rust
pub struct Mailbox<B, S> {
    sender: futures::channel::mpsc::Sender<Message>,
    block_store: crate::BlockStore<B>,
    payload_sender: S,
    _phantom: std::marker::PhantomData<B>,
}
```

- If implementation prefers not to make `Mailbox` generic over the sender type, it may store a crate-local relay adapter trait object or wrapper struct instead, but the externally visible crate contract remains: the mailbox owns a payload-broadcast capability plus the shared block store.

### Relay behavior
- `impl Relay for Mailbox<...>` remains the vendor-required integration point.
- Trait shape remains unchanged:

```rust
impl<B, S> Relay for Mailbox<B, S> {
    type Digest = commonware_cryptography::sha256::Digest;

    async fn broadcast(&mut self, digest: Self::Digest);
}
```

- Behavioral contract changes from no-op to active payload distribution.

## Internal Relay Types

### Payload envelope
- Add a crate-local relay message type used on `Channel::PAYLOAD`:

```rust
pub struct PayloadRelayMessage {
    pub digest: commonware_cryptography::sha256::Digest,
    pub payload: bytes::Bytes,
}
```

- The exact serde/codec derive set is implementation-defined.
- Required invariant: a decoded payload must be accepted only if its embedded digest matches the digest recomputed from the decoded block.

### Optional relay sender adapter
- Recommended helper type if the raw per-channel sender does not directly satisfy the local ergonomics needed by the mailbox:

```rust
pub struct PayloadRelaySender<S> {
    inner: S,
}
```

- This helper may own serialization and `Recipients::All` routing so `Mailbox::broadcast()` stays small.

## Function and Method Contracts

### `Mailbox::new(...)`
- Purpose: construct the vendor-facing automaton/relay adapter with access to local command handling, payload lookup, and payload egress.
- Required inputs:
  - mailbox actor command sender
  - shared `BlockStore<B>`
  - payload channel sender or equivalent relay adapter
- Postconditions:
  - every mailbox clone shares the same `BlockStore`
  - every mailbox clone can broadcast payloads over the payload transport path

### `Relay::broadcast(&mut self, digest)`
- Purpose: send the full proposed block payload corresponding to `digest` to remote peers.
- Preconditions:
  - the proposed block should already have been inserted into the shared `BlockStore` during the proposal path
- Required behavior:
  1. read `digest` from the shared `BlockStore`
  2. if present, serialize the block into `PayloadRelayMessage { digest, payload }`
  3. send the serialized message to all peers via the payload sender
  4. return after the send attempt completes
- Error handling:
  - if `digest` is absent from the store, emit tracing and return without panic
  - if serialization fails, emit tracing and return without panic
  - if the network send fails, emit tracing and return without panic
- Postconditions on success:
  - all currently connected peers are targeted through `Recipients::All`
  - no vote/certificate/resolver channel is used for payload distribution

### Payload receive task
- Recommended location: `crates/consensus-simplex/src/engine.rs` as a helper spawned from `CommonwareEngine::start()`.
- Purpose: persist inbound payloads into the shared `BlockStore` so later `verify(digest)` calls can resolve them.
- Inputs:
  - payload receiver from `PerChannelNetwork`
  - shared `BlockStore<A::Block>`
- Required behavior:
  1. receive bytes from the payload channel loop
  2. decode `PayloadRelayMessage`
  3. decode the block payload into `A::Block`
  4. recompute the block digest
  5. compare recomputed digest to the envelope digest
  6. store the block in `BlockStore` if the check passes
- Error handling:
  - malformed frames are dropped with tracing
  - digest mismatch is dropped with tracing
  - duplicate digest insertion is allowed to overwrite identical content or keep the latest value; no error path is required

### `CommonwareEngine::start()`
- New required steps in addition to current behavior:
  1. obtain `per_channel.payload` from `network.start_per_channel()`
  2. create `block_store` before mailbox construction
  3. construct `Mailbox` with `mailbox_tx`, shared `block_store`, and payload sender
  4. spawn the payload receive task with the payload receiver and shared `block_store`
  5. continue passing only `vote`, `cert`, and `resolver` to `simplex::Engine::start(...)`
- Postconditions:
  - propose, relay broadcast, payload receive, verify, and finalization reporting share one digest-indexed cache
  - vendor engine integration remains unchanged at the call boundary

## Data Ownership and Invariants
- `BlockStore<A::Block>` remains the sole shared cache for digest-to-block lookup.
- Proposed blocks must enter `BlockStore` before relay broadcast attempts.
- Inbound payload persistence must write into the same `BlockStore` type used by local proposal and finalization logic.
- Vote/certificate/resolver transport remains vendor-owned and must not be repurposed for payload bytes.
- Relay payload transport is additive and uses only `Channel::PAYLOAD`.

## Serialization Contract
- The relay path requires a stable way to turn `A::Block` into bytes and back.
- Implementation may satisfy this using existing block codec traits if available, or by introducing narrow crate-local bounds/helpers needed for serialization.
- The design requirement is behavioral rather than trait-prescriptive:
  - outbound path must send the full block payload bytes
  - inbound path must reconstruct `A::Block`
  - recomputed digest must match the envelope digest before storage
- If implementation discovers the current `CommonwareBlock` bound is insufficient for this, the acceptable refinement is to tighten local generic bounds in `CommonwareEngine`/`Mailbox` without leaking a vendor-facing API redesign.

## Traceability
- `REQ-6` -> active `Relay::broadcast`, payload receive task, shared `BlockStore` lookup/store behavior
- `REQ-7` -> strict separation between `PAYLOAD = 3` and existing vote/certificate/resolver channels
- `REQ-8` -> unchanged `AppAdapter`/finalization flow and additive relay-only integration

## Compatibility Rules
- Do not modify anything under `vendor/`.
- Do not change the vendor `Relay` trait, automaton traits, or `simplex::Engine::start(...)` signature.
- Preserve existing single-node behavior.
- Preserve existing `AppAdapter` and `FinalizationSink` semantics.
- Keep relay activation minimal: no redesign of consensus state machine responsibilities.
