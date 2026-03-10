# Domains and Wiring

## Scope
- Sub-Intent C only: `REQ-6`, `REQ-7`, and `REQ-8`.
- Focus crates:
  - `crates/consensus-simplex`
  - `crates/p2p`
  - `crates/p2p-commonware`
  - `crates/whirlpool-node`

## Domain Model

### Relay payload domain
- Relay activation introduces one application-level data domain distinct from vendor protocol traffic:
  - proposed block payload distribution keyed by digest
- Canonical logical envelope:

```rust
pub struct PayloadRelayMessage {
    pub digest: Digest,
    pub payload: bytes::Bytes,
}
```

- This domain is owned by `crates/consensus-simplex`, not by `crates/p2p` or vendor simplex.

### Shared block cache domain
- `BlockStore<A::Block>` remains the shared digest-indexed cache for consensus blocks.
- It is the single lookup surface used across:
  - local proposal persistence
  - outbound relay lookup
  - inbound payload persistence
  - local verification lookup
  - finalization reporting
- Design invariant: relay activation must reuse this cache rather than introduce a second payload store.

### Transport channel domain
- `crates/p2p` owns channel identity.
- Final channel map for this sub-intent:
  - `VOTE = 0`
  - `CERTIFICATE = 1`
  - `RESOLVER = 2`
  - `PAYLOAD = 3`
- `crates/p2p-commonware` owns registration of those channels into the Commonware transport provider.

## Wiring Boundaries

### `crates/p2p`
- Owns the stable `Channel` constants only.
- Does not define how consensus payloads are serialized.
- Provides the contract boundary that downstream crates align to.

### `crates/p2p-commonware`
- Owns transport registration and dedicated channel-pair exposure.
- Extends `PerChannelNetwork<S, R>` to include `payload: (S, R)`.
- Does not inspect or decode consensus payload envelopes.

### `crates/consensus-simplex`
- Owns relay behavior and payload cache wiring.
- `Mailbox` becomes the outbound relay adapter.
- `CommonwareEngine::start()` becomes the inbound payload persistence owner.
- `AppAdapter` remains finalization/reporting glue and is intentionally unchanged in semantics.

### `crates/whirlpool-node`
- Continues to instantiate `CommonwareEngine` and network provider.
- Does not own payload relay logic.
- Serves as the compatibility boundary proving relay activation does not break node startup or finalization wiring.

## Cross-Crate Wiring

### Outbound path
1. `MailboxActor` writes proposed block into `BlockStore`.
2. Vendor simplex engine calls `Relay::broadcast(digest)` on `Mailbox`.
3. `Mailbox` reads `BlockStore[digest]`.
4. `Mailbox` serializes `PayloadRelayMessage`.
5. `Mailbox` sends bytes through the payload sender using `Channel::PAYLOAD` semantics.

### Inbound path
1. `p2p-commonware` payload receiver yields bytes from channel `3`.
2. `CommonwareEngine` payload task decodes the relay envelope and block.
3. `CommonwareEngine` writes the block into the shared `BlockStore` under the validated digest.
4. Later verification paths resolve the same digest from that store.

### Vendor boundary
- Vendor simplex engine still owns vote/certificate/resolver traffic only.
- It does not learn about `Channel::PAYLOAD` directly.
- The additive payload path lives entirely outside the vendor boundary.

## Invariants
- `BlockStore` must be shared across all mailbox clones and engine-owned helper tasks.
- Payload receive must validate digest consistency before writing to the cache.
- Vote/certificate/resolver channel meanings must remain unchanged.
- `whirlpool-node` startup remains an integration consumer, not the owner of relay semantics.
- No change under `vendor/` is allowed.

## Failure Boundaries
- Missing outbound digest in local `BlockStore` -> trace and no-send.
- Malformed inbound envelope -> trace and drop.
- Inbound digest mismatch -> trace and drop.
- Zero connected peers -> outbound send is still safe and must not break single-node startup.

## Traceability
- `REQ-6` -> relay payload domain, shared block cache domain, outbound and inbound wiring
- `REQ-7` -> transport channel domain and channel map invariants
- `REQ-8` -> whirlpool-node compatibility boundary and unchanged app/finalization ownership
