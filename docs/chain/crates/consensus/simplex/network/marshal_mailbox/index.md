# Marshal mailbox (payload networking)

Simplex consensus votes on **commitments/digests** that refer to proposal payloads (blocks).
Consensus should not carry full block bytes in its vote/certificate planes.

The **marshal mailbox** is the payload-facing API used by:

- the `Marshaled` application wrapper (during propose/verify)
- reporters/indexers (to fetch the block bytes referenced by finalization activity)

It is the interface to a long-running **marshal actor**.

In Alto, marshal payload networking uses a *separate* P2P channel in addition to the 3 Simplex
planes:

- `MARSHAL_CHANNEL` constant (see `vendor/alto/chain/src/bin/validator.rs`, line ~32)
- `network.register(MARSHAL_CHANNEL, ...)` (lines ~227-229)
- a P2P resolver is initialized and passed into the engine start:
  - `marshal::resolver::p2p::init(...)` (lines ~271-283)
  - `engine.start(..., marshal_resolver)` (line ~286)

## Sub-pages

- [`actor`](./actor.md) — what the marshal task owns/does
- [`subscription`](./subscription.md) — how "wait for payload X" works
