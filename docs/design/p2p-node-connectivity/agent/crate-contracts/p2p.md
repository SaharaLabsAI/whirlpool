# Crate Contract: p2p

## Scope
- Sub-Intent C support contract for `REQ-7`.
- Crate: `crates/p2p`.
- In-scope file:
  - `crates/p2p/src/types.rs`
- Out of scope:
  - trait redesign in `crates/p2p/src/traits.rs`
  - sender/receiver semantic changes
  - any channel ID reassignment for existing protocol channels

## Current Baseline Verified From Source
- `crates/p2p/src/types.rs` defines:
  - `Channel::VOTE = Channel(0)`
  - `Channel::CERTIFICATE = Channel(1)`
  - `Channel::RESOLVER = Channel(2)`
- These channel IDs are already consumed by `crates/p2p-commonware` and by consensus transport wiring.

## Required Public API Change
- Add one additive associated constant:

```rust
impl Channel {
    pub const PAYLOAD: Channel = Channel(3);
}
```

- No existing constant changes value.
- `Channel(pub u64)` remains unchanged.
- `Recipients`, `NetworkChannel`, and `NetworkMessage` remain unchanged.

## Behavioral Contract
- `Channel::PAYLOAD` is reserved for application-level consensus payload distribution.
- `Channel::PAYLOAD` must not replace or alias `VOTE`, `CERTIFICATE`, or `RESOLVER`.
- Consumers may rely on the following stable mapping after this change:
  - `0` -> vote messages
  - `1` -> certificate messages
  - `2` -> resolver messages
  - `3` -> proposed block payload relay messages

## Traceability
- `REQ-7` -> additive payload channel while preserving all existing protocol channel IDs.

## Compatibility Rules
- No p2p trait signatures change.
- Existing callers using `VOTE`, `CERTIFICATE`, or `RESOLVER` continue to behave exactly as before.
- This crate does not define payload message encoding; it only reserves the channel ID.
