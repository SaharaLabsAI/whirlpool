# Marshal actor

The marshal **actor** is a dedicated async task responsible for **payload custody**.

It owns the state machine for:

- mapping `commitment/digest -> payload bytes` (and metadata)
- tracking in-flight fetches (so multiple requesters don’t duplicate work)
- persisting payloads (optional) and enforcing retention policies

It is called an "actor" because it serializes access to this shared state behind a message loop.

## Responsibilities

1. **Publish** local payloads
   - when we propose a block, we register its bytes so peers can request it by commitment.

2. **Resolve** missing payloads
   - if consensus sees a commitment it does not have, the actor triggers fetch (storage, then network).

3. **Serve** payloads to peers
   - handle inbound payload requests and respond with the bytes if available.

4. **Notify** local consumers
   - fulfill subscriptions for payload availability (see `subscription.md`).

## Interface: mailbox

Consumers do not call the actor directly; they use a **mailbox** (client handle) to:

- `publish(commitment, bytes)`
- `get(commitment)` / `resolve(commitment)`
- `subscribe(commitment)`

Commonware references:

- marshal config: `vendor/commonware/consensus/src/marshal/config.rs`
- optional P2P resolver: `vendor/commonware/consensus/src/marshal/resolver/p2p.rs`
- marshaled wrapper: `vendor/commonware/consensus/src/application/marshaled.rs`

Alto wiring reference:

- marshal actor + mailbox initialized in `vendor/alto/chain/src/engine.rs`:
  - `marshal::Actor::init(...) -> (marshal, marshal_mailbox, _)` (lines ~226-250)
  - the marshaled app is constructed with `marshal_mailbox.clone()` (lines ~254-259)
  - consensus config uses `marshaled` as `automaton` and `relay` (lines ~275-282)

Runtime start order (Alto):

- `buffer.start(broadcast)` then `marshal.start(...)` then `consensus.start(votes, certificates, resolver)`
  (`vendor/alto/chain/src/engine.rs`, lines ~369-382)

See also:

- [`construct`](./construct.md) — how marshal + mailbox are built
- [`runtime`](./runtime.md) — how marshal networking is wired at start
- [`channels`](./channels.md) — marshal-specific channel IDs
