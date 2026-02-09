# Marshal mailbox (payload networking)

Simplex consensus votes on **commitments/digests** that refer to proposal payloads (blocks).
Consensus should not carry full block bytes in its votes/certificates channels.

The **marshal mailbox** is the payload-facing API used by:

- the `Marshaled` application wrapper (during propose/verify)
- reporters/indexers (to fetch the block bytes referenced by finalization activity)

It is the interface to a long-running **marshal actor**.

If you feel unclear about how marshal is *constructed* and *wired into* the engine, start with:

- [`construct`](./construct.md) — build-time construction (inside `SimplexEngine::new`)
- [`runtime`](./runtime.md) — runtime wiring (inside `SimplexEngine::start`)
- [`channels`](./channels.md) — marshal-specific network channels

In the recommended wiring, the Simplex engine owns the marshal actor and retains a cloneable
mailbox handle for:

- the marshaled application wrapper (propose/verify)
- reporters/indexers (finalization -> fetch bytes -> emit `FinalizedBlock`)

In Alto, marshal payload networking uses *two* additional P2P channels in addition to the 3
Simplex consensus channels:

- `BROADCASTER_CHANNEL` constant (payload broadcast)
- `MARSHAL_CHANNEL` constant (payload backfill request/response)

See: [`channels`](./channels.md).

## Sub-pages

- [`actor`](./actor.md) — what the marshal task owns/does
- [`subscription`](./subscription.md) — how "wait for payload X" works
- [`channels`](./channels.md) — marshal-specific network channels + constants
- [`construct`](./construct.md) — build-time construction
- [`runtime`](./runtime.md) — runtime wiring and start order
