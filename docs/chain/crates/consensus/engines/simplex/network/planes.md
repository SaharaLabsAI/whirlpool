# Simplex channels

Simplex requires **three** distinct P2P message channels. Think of these as separate logical
channels (they may share an underlying transport implementation).

These are the three channels passed into `commonware_consensus::simplex::Engine::start(...)`.

In our engine boundary (see `../index.md`), callers provide a network implementation
that can register/open logical channels (see `core` network traits: `docs/chain/crates/core/network.md`).
The Simplex engine derives/constructs these channels internally during engine construction/start by
registering three channel IDs and retaining the resulting `(Sender, Receiver)` pairs.

Recommended shape: keep these handles in an internal `SimplexChannels` struct owned by the engine.

```rust
// Pseudocode (shape only): build planes from a consensus-agnostic channel network.
//
// The engine chooses channel IDs; the network does not interpret them.

struct SimplexChannels<NS, NR> {
  votes: (NS, NR),
  certificates: (NS, NR),
  resolver: (NS, NR),
}

impl<Net> SimplexChannels<Net::Sender, Net::Receiver>
where
  Net: core::network::Network,
{
  // `cfg` is `SimplexBuildConfig::network` (see `../build/config/index.md`).
  fn new(network: &mut Net, cfg: NetworkConfig) -> Self {
    let votes = network.register_channel(PENDING_CHANNEL, cfg.votes);
    let certificates = network.register_channel(RECOVERED_CHANNEL, cfg.certificates);
    let resolver = network.register_channel(RESOLVER_CHANNEL, cfg.resolver);
    Self { votes, certificates, resolver }
  }
}
```

## 1) Votes channel (`vote_network`)

Carries **individual votes** (small, frequent messages), such as:

- `Notarize`
- `Nullify`
- `Finalize`

Alto example wiring: `vendor/alto/chain/src/bin/validator.rs` registers the `PENDING_CHANNEL` for votes.

Concrete references (Alto):

- channel IDs: `PENDING_CHANNEL` / `RECOVERED_CHANNEL` / `RESOLVER_CHANNEL` (lines ~28-31)
- registrations: `network.register(PENDING_CHANNEL, ...)` etc. (lines ~206-217)
- engine start: `engine.start(votes, certificates, resolver, ...)` (line ~286)

## 2) Certificates channel (`certificate_network`)

Carries **aggregated certificates** (quorum proofs), such as:

- `Notarization`
- `Nullification`
- `Finalization`

Alto example wiring: `vendor/alto/chain/src/bin/validator.rs` registers the `RECOVERED_CHANNEL` for certificates.

Concrete references (Alto):

- `RECOVERED_CHANNEL` constant (line ~29)
- registration block (lines ~210-213)

## 3) Resolver / fetch channel (`resolver_network`)

Carries **request/response** traffic for missing consensus artifacts (e.g. certificates / views)
so a node can catch up.

Alto example wiring: `vendor/alto/chain/src/bin/validator.rs` registers `resolver` for fetch.

Concrete references (Alto):

- `RESOLVER_CHANNEL` constant (line ~30)
- registration block (lines ~215-217)

## Notes

- These channels are about **consensus messages**, not block/payload bytes.
- Payload bytes are handled via the marshal mailbox (see [`marshal_mailbox`](./marshal_mailbox/index.md)).

References:

- `vendor/commonware/consensus/src/simplex/engine.rs` (`Engine::start`)
- Alto end-to-end wiring: `vendor/alto/chain/src/bin/validator.rs` (channel registration + `engine.start(...)`)
