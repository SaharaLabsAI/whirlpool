# Strategy

## Existing Finalized Scope

### Sub-Intent B
- This synthesize pass covers Sub-Intent B only: REQ-4 and REQ-5.
- Primary implementation crate: `crates/whirlpool-node`.
- `crates/p2p-commonware` is read-only in this pass; its builder API is consumed as-is.
- Out of scope: REQ-1, REQ-2, REQ-3, REQ-6, REQ-7, REQ-8; new config file formats; any `p2p-commonware` API change.

## Sub-Intent C Scope
- This design pass covers Sub-Intent C only: `REQ-6`, `REQ-7`, and `REQ-8`.
- Primary implementation crate: `crates/consensus-simplex`.
- Supporting crates with narrow, contract-preserving changes:
  - `crates/p2p`
  - `crates/p2p-commonware`
  - `crates/whirlpool-node`
- Out of scope:
  - any modification under `vendor/`
  - any redesign of vendor simplex vote/certificate/resolver transport
  - any change to the `Relay` trait shape
  - any redesign of application finalization semantics
  - any overwrite of `docs/design/p2p-node-connectivity/agent/shared-intent-splits.md`

## Design Intent
- Activate the simplex relay by making `Mailbox::broadcast(digest)` send the full proposed block payload to peers.
- Reuse the existing mailbox-owned `BlockStore` as the canonical digest-to-block cache for both outbound relay lookup and inbound verification lookup.
- Introduce one dedicated application payload transport channel without disturbing the vendor-managed vote, certificate, and resolver channels.
- Keep the implementation additive and local: vendor engine wiring stays intact while `CommonwareEngine::start()` layers one payload receive task beside it.

## Concrete Decisions

### Payload transport choice
- Add a new P2P channel constant `PAYLOAD = 3` in `crates/p2p/src/types.rs`.
- Register that channel in `crates/p2p-commonware/src/provider.rs` alongside the existing `VOTE`, `CERTIFICATE`, and `RESOLVER` channels.
- Extend `PerChannelNetwork<S, R>` with a fourth field:

```rust
pub struct PerChannelNetwork<S, R> {
    pub vote: (S, R),
    pub cert: (S, R),
    pub resolver: (S, R),
    pub payload: (S, R),
    pub network_handle: commonware_runtime::Handle<()>,
}
```

- Rationale: payload distribution is application-level data, not a vote/certificate/resolver protocol message, so it should not be multiplexed onto those existing vendor channels.

### Mailbox relay activation
- `Mailbox<B>` gains shared access to:
  - the existing `BlockStore<B>`
  - a payload-channel sender implementing the `p2p::NetworkSender` contract or an adapter over the raw per-channel sender returned by `start_per_channel()`
- `Relay::broadcast(digest)` must:
  1. look up `digest` in the shared `BlockStore`
  2. clone or encode the stored block payload into bytes
  3. send those bytes to `Recipients::All` on `Channel::PAYLOAD`
- Missing-block lookup is treated as a guarded no-send condition with tracing, not a panic path, because consensus progress should not crash the process if the local cache is unexpectedly absent.

### Payload message format
- The payload channel carries a minimal envelope containing:
  - the block digest
  - the serialized block payload bytes
- Recommended logical shape:

```rust
pub struct PayloadRelayMessage {
    pub digest: Digest,
    pub payload: bytes::Bytes,
}
```

- Serialization format remains implementation-defined, but the design requires deterministic decoding and digest cross-checking before storing inbound data.
- The receiver path must reject malformed payload frames or frames whose embedded digest does not match the decoded block's computed digest.

### Inbound payload persistence
- `CommonwareEngine::start()` spawns one background task dedicated to `per_channel.payload.1`.
- That task decodes each inbound payload message and inserts the block into the shared `BlockStore` under the digest key before local verification needs it.
- The task must share the same `BlockStore<A::Block>` instance already passed to `MailboxActor` and `AppAdapter`.
- This keeps one canonical digest-indexed cache across propose, relay broadcast, relay receive, verify, and finalization reporting.

### Engine wiring plan
- `CommonwareEngine::start()` remains responsible for:
  - calling `network.start_per_channel()`
  - constructing `Mailbox`
  - spawning `MailboxActor`
  - constructing `AppAdapter`
  - starting the vendor simplex engine with `vote`, `cert`, and `resolver`
- New relay-specific wiring added in this pass:
  1. extract `per_channel.payload`
  2. construct `Mailbox` with a payload sender handle plus shared `BlockStore`
  3. spawn a payload receiver task that stores inbound blocks into `BlockStore`
  4. keep the payload task alive for the same lifetime as the running engine
- The vendor simplex engine call remains exactly three-channel:

```rust
engine.start(per_channel.vote, per_channel.cert, per_channel.resolver)
```

- This preserves the vendor boundary and ensures no vendor API change is required.

### Channel alignment rule
- Existing protocol-channel assignments remain unchanged:
  - `VOTE = 0`
  - `CERTIFICATE = 1`
  - `RESOLVER = 2`
- New assignment:
  - `PAYLOAD = 3`
- `REQ-7` is satisfied only if vote/certificate/resolver semantics stay exactly where they are today and the payload path is strictly additive.

### App compatibility rule
- `AppAdapter` and finalization sink behavior remain unchanged.
- The relay path only adds missing payload availability needed for remote verification.
- No finalization-side redesign is proposed.
- Existing single-node behavior remains valid because payload broadcast to `Recipients::All` is harmless when no remote peers are present.

## Flow Summary
1. Propose path stores the newly created block into `BlockStore` before the vendor engine invokes relay broadcast.
2. Relay broadcast looks up the block by digest and sends it over `Channel::PAYLOAD`.
3. Remote payload receiver decodes and stores the block into its own `BlockStore`.
4. Later `Automaton::verify(digest)` resolves that digest from the store and can validate it locally.
5. Vote/certificate/resolver traffic continues flowing through the existing vendor-managed channels unchanged.

## Exit Criteria
- `crates/p2p` defines an additive payload channel constant without disturbing existing channel IDs.
- `crates/p2p-commonware` registers and exposes a payload channel pair in `PerChannelNetwork`.
- `crates/consensus-simplex` has an implementation-ready relay design for outbound payload broadcast and inbound payload persistence.
- `crates/whirlpool-node` needs no architectural redesign beyond consuming the updated consensus/network crates.
- Agent and review lane artifacts for Sub-Intent C are internally consistent and support a final verdict of PASS.
