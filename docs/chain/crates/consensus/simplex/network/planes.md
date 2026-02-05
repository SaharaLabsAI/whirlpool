# Simplex network planes

Simplex requires **three** distinct P2P message planes. Think of these as separate logical
channels (they may share an underlying transport implementation).

These are the three channels passed into `commonware_consensus::simplex::Engine::start(...)`.

## 1) Vote plane (`vote_network`)

Carries **individual votes** (small, frequent messages), such as:

- `Notarize`
- `Nullify`
- `Finalize`

Alto example wiring: `vendor/alto/chain/src/bin/validator.rs` registers `pending` for votes.

## 2) Certificate plane (`certificate_network`)

Carries **aggregated certificates** (quorum proofs), such as:

- `Notarization`
- `Nullification`
- `Finalization`

Alto example wiring: `vendor/alto/chain/src/bin/validator.rs` registers `recovered` for certificates.

## 3) Resolver / fetch plane (`resolver_network`)

Carries **request/response** traffic for missing consensus artifacts (e.g. certificates / views)
so a node can catch up.

Alto example wiring: `vendor/alto/chain/src/bin/validator.rs` registers `resolver` for fetch.

## Notes

- These planes are about **consensus messages**, not block/payload bytes.
- Payload bytes are handled via the marshal mailbox (see [`marshal_mailbox`](./marshal_mailbox/index.md)).

References:

- `vendor/commonware/consensus/src/simplex/engine.rs` (`Engine::start`)
