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

- `vendor/alto/chain/src/indexer.rs` waits on the marshal subscription before uploading notarized/finalized artifacts:
  - notarization path: `marshal.subscribe(Some(notarization.round()), notarization.proposal.payload)` (lines ~137-146)
  - finalization path: `marshal.subscribe(Some(finalization.round()), finalization.proposal.payload)` (lines ~179-187)

Note the shape: `subscribe(..).await.await` — the first await registers the subscription; the second
await waits for the block to arrive (or be cancelled).

## Subscription vs mailbox

- **Mailbox**: the request interface to the marshal actor.
- **Subscription**: the returned handle representing "notify me when payload X is available".
