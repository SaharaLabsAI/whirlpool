# p2p-commonware — Contract Document

## Purpose

Expose per-channel `(Sender, Receiver)` pairs from the P2P network provider, enabling the vendor simplex engine to receive its required 3 separate channel pairs (vote, certificate, resolver).

## Public Interface Changes

### Added: `PerChannelNetwork` [PROPOSED]

```rust
pub struct PerChannelNetwork<S, R> {
    pub vote: (CommonwareSender<S>, CommonwareReceiver<R>),
    pub certificate: (CommonwareSender<S>, CommonwareReceiver<R>),
    pub resolver: (CommonwareSender<S>, CommonwareReceiver<R>),
    pub network_handle: Handle<()>,
}
```

### Added: `CommonwareNetworkProvider::start_per_channel()` [PROPOSED]

```rust
impl<E, C> CommonwareNetworkProvider<E, C> {
    pub fn start_per_channel(mut self) -> Result<PerChannelNetwork<...>, NetworkError>
}
```

Same internal logic as current `start()` — registers 3 channels on discovery::Network — but returns individual pairs instead of wrapping them in Multiplex.

### Unchanged: `NetworkProvider::start()` (existing)

The multiplexed `start()` remains for backward compatibility.

### Exposed: `OracleHandle` blocker access [PROPOSED]

Ensure `OracleHandle.control(public_key)` → `Oracle` (which impls `Blocker`) is accessible to engine configuration. May require exposing `OracleHandle` field or adding a method.

## Internal Changes

- Factor channel registration logic into shared helper used by both `start()` and `start_per_channel()`
- `start_per_channel()` returns raw per-channel pairs instead of wrapping in Multiplex

## Dependencies

No new dependencies. Uses existing `commonware_p2p::authenticated::discovery` types.

## Risks

- Sender/Receiver types must match what vendor simplex::Engine::start() expects
- Network handle lifetime must be preserved to keep P2P alive
