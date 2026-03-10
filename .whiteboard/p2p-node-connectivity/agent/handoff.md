# Implementation Handoff

## Intent
- Finalize Sub-Intent C for `consensus-relay-activation`.
- Scope is limited to `REQ-6`, `REQ-7`, and `REQ-8`.
- Primary implementation crate: `crates/consensus-simplex`.
- Supporting crates with narrow additive changes:
  - `crates/p2p`
  - `crates/p2p-commonware`
- Node integration boundary to preserve:
  - `crates/whirlpool-node`

## Verified Source Baseline
- `crates/consensus-simplex/src/mailbox.rs` currently implements `Relay::broadcast()` as a no-op.
- `crates/consensus-simplex/src/mailbox.rs` already stores genesis and proposed blocks in a shared `BlockStore` through `MailboxActor::remember_block()`.
- `crates/consensus-simplex/src/engine.rs` already creates one shared `BlockStore` and passes it to both `MailboxActor` and `AppAdapter`.
- `crates/p2p/src/types.rs` currently reserves only three channel constants: vote, certificate, resolver.
- `crates/p2p-commonware/src/provider.rs` currently registers only those three channels in `start_per_channel()`.
- `crates/whirlpool-node/src/main.rs` already delegates consensus/network startup to `CommonwareEngine` and does not own relay logic itself.

## Implementation Order
1. Update `crates/p2p/src/types.rs`.
   - Add `Channel::PAYLOAD = Channel(3)`.
   - Why first: every downstream relay registration and assertion depends on the stable constant existing.
2. Update `crates/p2p-commonware/src/provider.rs` and related transport tests.
   - Register the payload channel in `start_per_channel()`.
   - Extend `PerChannelNetwork` with `payload: (S, R)`.
   - Why second: consensus wiring needs the extra dedicated channel pair before relay activation can consume it.
3. Refactor `crates/consensus-simplex/src/mailbox.rs`.
   - Give `Mailbox` access to shared `BlockStore` and a payload sender.
   - Replace the no-op relay with digest lookup plus payload send.
   - Why third: this is the core outbound relay activation point.
4. Extend `crates/consensus-simplex/src/engine.rs`.
   - Extract `per_channel.payload`.
   - Construct the updated `Mailbox`.
   - Spawn a payload receive task that decodes inbound payloads and stores them in `BlockStore`.
   - Why fourth: engine wiring ties outbound relay and inbound persistence together.
5. Run and update tests.
   - Add relay broadcast tests, payload receive/store tests, channel alignment tests, multi-node round-trip coverage, and single-node compatibility checks.
   - Why last: test coverage validates the completed end-to-end relay path.

## File-by-File Change Summary

### `crates/p2p/src/types.rs`
- Add one additive channel constant only:
  - `pub const PAYLOAD: Channel = Channel(3);`
- Preserve existing values for `VOTE`, `CERTIFICATE`, and `RESOLVER`.

### `crates/p2p-commonware/src/provider.rs`
- Extend `PerChannelNetwork<S, R>` with `payload: (S, R)`.
- Register `Channel::PAYLOAD.0` in `start_per_channel()` using the same quota/backlog policy as the existing channels.
- Return the payload sender/receiver pair in the `PerChannelNetwork` bundle.
- Keep the vendor-facing dedicated-channel contract intact for vote/certificate/resolver.

### `crates/consensus-simplex/src/mailbox.rs`
- Change `Mailbox::new(...)` to accept relay dependencies in addition to the mailbox command sender.
- Store the shared `BlockStore` inside the mailbox so `broadcast(digest)` can resolve the block payload.
- Add or use a small relay sender helper that serializes `PayloadRelayMessage` and sends to `Recipients::All`.
- Replace the current no-op `broadcast` implementation with real payload relay behavior.
- Keep `Automaton` and `CertifiableAutomaton` behavior intact.

### `crates/consensus-simplex/src/engine.rs`
- Continue creating one shared `BlockStore` for proposal, verification, and finalization paths.
- Extract `per_channel.payload` before handing `vote`, `cert`, and `resolver` to the vendor engine.
- Spawn a background payload receiver task that:
  - reads payload channel messages
  - decodes the relay envelope
  - decodes the block
  - verifies digest consistency
  - stores the block into `BlockStore`
- Continue calling `simplex::Engine::start(per_channel.vote, per_channel.cert, per_channel.resolver)` unchanged.

### `crates/whirlpool-node/src/main.rs`
- Prefer no source change.
- If implementation requires any edit here, it must remain narrow and compatibility-preserving.
- Do not move payload relay logic into the node binary.

## Dependencies Between Changes
- `p2p` payload channel constant must exist before `p2p-commonware` can register it.
- `p2p-commonware` payload registration must exist before `consensus-simplex` can consume `per_channel.payload`.
- `Mailbox` relay activation depends on the existing `BlockStore` contract remaining shared.
- End-to-end relay tests depend on both transport registration and consensus wiring being in place.

## Verification Steps
1. Confirm `crates/p2p/src/types.rs` exposes `PAYLOAD = 3` while existing constants remain unchanged.
2. Confirm `crates/p2p-commonware/src/provider.rs` registers four channels in `start_per_channel()` and returns a populated `payload` pair.
3. Confirm `crates/consensus-simplex/src/mailbox.rs` no longer contains a no-op `broadcast` implementation.
4. Confirm `crates/consensus-simplex/src/engine.rs` spawns a payload receive task and still starts the vendor engine with only vote/cert/resolver pairs.
5. Run:
   - `nix develop --command cargo build`
   - `nix develop --command cargo test -p consensus-simplex`
   - `nix develop --command cargo test -p p2p`
   - `nix develop --command cargo test -p p2p-commonware`
   - `nix develop --command cargo test -p whirlpool-node`

## Acceptance Checks
- `REQ-6`: relay broadcast sends application payload bytes for a proposed digest and inbound payloads are stored for later verification.
- `REQ-7`: vote/certificate/resolver channels remain aligned on `0`, `1`, and `2`, with payload isolated on `3`.
- `REQ-8`: existing node startup and app-layer finalization flow remain compatible while multi-node relay becomes functional.
- No vendor code is modified.

## Deferred Beyond This Pass
- Vendor simplex engine redesign.
- Payload gossip policies beyond `Recipients::All`.
- Specialized payload backpressure tuning separate from the default channel quota/backlog.
- Any redesign of application-level block encoding beyond what is minimally required to serialize and verify relay payloads.
