# Marshal mailbox (payload networking)

Simplex consensus votes on **commitments/digests** that refer to proposal payloads (blocks).
Consensus should not carry full block bytes in its vote/certificate planes.

The **marshal mailbox** is the payload-facing API used by:

- the `Marshaled` application wrapper (during propose/verify)
- reporters/indexers (to fetch the block bytes referenced by finalization activity)

It is the interface to a long-running **marshal actor**.

## Sub-pages

- [`actor`](./actor.md) — what the marshal task owns/does
- [`subscription`](./subscription.md) — how "wait for payload X" works
