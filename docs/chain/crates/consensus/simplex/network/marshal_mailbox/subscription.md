# Subscriptions

A **subscription** is how a consumer waits for a payload to become available.

Conceptually:

- `subscribe(commitment)` returns a receiver/future/stream
- the marshal actor fulfills it when the payload bytes arrive

This pattern is used when you have a commitment (from consensus activity) but not the bytes yet.

## Typical usage

1. Simplex finalizes a commitment.
2. Reporter receives finalization activity.
3. Reporter calls `marshal.subscribe(commitment)`.
4. When bytes are available, reporter constructs and emits:
   - `ConsensusEvent::Finalized(types::FinalizedBlock { certificate, block })`

Alto example:

- `vendor/alto/chain/src/indexer.rs` waits on the marshal subscription before uploading notarized/finalized artifacts.

## Subscription vs mailbox

- **Mailbox**: the request interface to the marshal actor.
- **Subscription**: the returned handle representing "notify me when payload X is available".
