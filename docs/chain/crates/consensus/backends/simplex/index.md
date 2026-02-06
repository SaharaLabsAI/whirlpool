# `consensus/backends/simplex` — Simplex backend (commonware)

This chain uses `vendor/commonware/consensus` Simplex as the consensus engine.

This page is implementation-oriented: it enumerates the concrete components you must provide
and how they connect. The canonical end-to-end example in this repo is `vendor/alto/chain`.

## What you are wiring

At minimum you are building and starting:

1. A **Simplex consensus engine**: `commonware_consensus::simplex::Engine`
2. A **marshaling layer** for blocks/payloads: `commonware_consensus::marshal::*` +
   `commonware_consensus::application::marshaled::Marshaled`
3. A **broadcast/relay plane** for block dissemination (Alto uses `commonware_broadcast::buffered`)
4. A **reporter** that converts commonware `Activity` into our `ConsensusEvent`

## Our app boundary vs commonware boundary

Our `core` traits are engine-agnostic ports.

- [`core/consensus`](../../../core/consensus.md): `ConsensusApplication`, `VerifyingApplication`, `Reporter`

The Simplex adapter layer is responsible for implementing the commonware traits and delegating
to our `core` traits.

## Concrete adapter structure (recommended)

This backend is used with the parent-level `ConsensusDriver<App, Backend, Network>` composition
defined in [`consensus/driver`](../../driver.md).

The intent is:

- `core/*` stays engine-agnostic.
- `consensus/backends/simplex/*` wires `core` to `commonware_consensus::simplex`.

### 1) Backend implements `ConsensusBackend`

```rust
// Pseudocode — not compile-ready.

pub struct SimplexBackend<E, App, Net> {
  // Internal backend state (for now): simplex engine + 3 consensus planes.
  engine: commonware_consensus::simplex::Engine<E, /* ... */>,
  planes: Planes,

  _app: std::marker::PhantomData<App>,
  _net: std::marker::PhantomData<Net>,
}

pub struct SimplexContext<'a, E, App, Net> {
  // App is consumed during build and wrapped by Marshaled.
  pub app: App,

  // Caller-owned dependencies injected into build.
  pub network: &'a Net,
  pub options: &'a SimplexOptions,

  _env: std::marker::PhantomData<E>,
}

impl<E, App, Net> consensus::ConsensusBackend for SimplexBackend<E, App, Net> {
  type Error = anyhow::Error;
  type Context<'a> = SimplexContext<'a, E, App, Net>
  where
    Self: 'a,
    App: 'a,
    Net: 'a;

  async fn build(ctx: Self::Context<'_>) -> Result<Self, Self::Error> {
    // Backend derives Planes from network implementation.
    // Backend consumes `ctx.app` to construct Marshaled.
    // Backend consumes Marshaled into simplex::Config (automaton + relay).
    // Backend constructs simplex::Engine::new(...).
    Ok(Self { /* engine, planes */ })
  }

  async fn start(self) -> Result<(), Self::Error> {
    // Consumes backend state and starts simplex with owned planes.
    Ok(())
  }
}
```

The driver (`ConsensusDriver<App, SimplexBackend<...>, Net>`) should only be responsible for:

- providing caller-owned dependencies (`app`, `network`, `SimplexOptions`)
- invoking backend lifecycle (`build` then `start`)
- supervising task lifetime and shutdown propagation

`Marshaled` and `Planes` are backend internals and must not be required from callers.
Current backend state is intentionally minimal: only `engine` + `planes` before `start()`.

Alto remains the behavioral reference for startup order and component composition,
but this adapter boundary keeps Alto-style internals encapsulated inside the backend.

### 2) Backend is “Simplex” (not a trait object)

If we are using Simplex, prefer holding the concrete type `commonware_consensus::simplex::Engine`.
This keeps the wiring explicit and prevents “dynamic dispatch everywhere”.

For the parent-level driver/backend abstractions (used to benchmark multiple algorithms), see:

- [`consensus/driver`](../../driver.md)

### 3) Adapter types you will actually use (commonware)

These are the concrete adapter/wrapper patterns worth mirroring:

- `commonware_consensus::application::marshaled::Marshaled<...>`
  - wraps your app and supplies `Automaton`/`Relay` behaviors needed by Simplex
  - file: `vendor/commonware/consensus/src/application/marshaled.rs`

- `commonware_consensus::reporter::Reporters<...>`
  - combines reporters (tee activity to multiple sinks)
  - file: `vendor/commonware/consensus/src/reporter.rs`

Alto examples:

- app implements commonware traits directly: `vendor/alto/chain/src/application.rs`
- engine composition / start order: `vendor/alto/chain/src/engine.rs`

## Required ingredients (checklist)

### Runtime environment (`E`)

Simplex is generic over a runtime environment `E` with these capabilities:

- `Clock + CryptoRngCore + Spawner + Storage + Metrics`

See: `vendor/commonware/consensus/src/simplex/engine.rs`.

### Simplex engine config (`simplex::Config`)

You must provide a `commonware_consensus::simplex::Config` with (non-exhaustive) categories:

- **Consensus crypto**: `scheme`
- **Leader election**: `elector`
- **Peer blocking**: `blocker`
- **State machine hooks**:
  - `automaton` (proposal + verification + certifiability)
  - `relay` (broadcast proposed payloads)
  - `reporter` (activity out)
- **Parallelism**: `strategy`
- **Persistence**: `partition`, `replay_buffer`, `write_buffer`, `buffer_pool`
- **Timeouts / view tracking**: `leader_timeout`, `notarization_timeout`, `nullify_retry`,
  `activity_timeout`, `skip_timeout`
- **Fetching**: `fetch_timeout`, `fetch_concurrent`

See: `vendor/commonware/consensus/src/simplex/config.rs`.

Mapping into our docs:

- `consensus/types.md`: `ConsensusConfig` / `Timeouts` are the *chain-facing* knobs.
- The Simplex adapter will also need additional knobs that are commonware-specific
  (storage partition name, buffer sizes, fetch strategy), even if we keep them out of the
  chain-facing `ConsensusConfig`.

### Network planes (3 channels required by Simplex)

See: [`network/planes`](./network/planes.md).

### Marshaling layer (block/payload availability)

See: [`network/marshal_mailbox`](./network/marshal_mailbox/index.md).

### Broadcast / relay plane

Simplex itself only calls `Relay::broadcast(commitment)`. You must implement the actual
transport of payload bytes.

Alto uses:

- `commonware_broadcast::buffered::Engine` + `buffered::Mailbox`
  - constructed in `vendor/alto/chain/src/engine.rs`
  - started with a dedicated `broadcast` channel registered in `vendor/alto/chain/src/bin/validator.rs`

In our design, this is owned by `network`/`consensus` glue. The key requirement is:

- when `Relay::broadcast(digest)` is invoked, peers can later retrieve the block bytes
  (often via marshal/backfill requests).

### Reporter -> our `ConsensusEvent`

Commonware emits rich activity via `commonware_consensus::Reporter`.

We only need a minimal chain-facing event:

- `ConsensusEvent::Finalized(types::FinalizedBlock)`

So implement a reporter that:

1. receives commonware activity (e.g. finalization)
2. obtains the referenced block bytes (if necessary) via marshal subscription
3. attaches the consensus certificate to produce `types::FinalizedBlock { certificate, block }`
4. publishes the `ConsensusEvent`

Alto shows both styles:

- app-level reporter receiving `Update::Block` finalizations (`vendor/alto/chain/src/application.rs`)
- separate indexer/pusher reporter that fetches blocks via marshal subscription before uploading
  notarized/finalized artifacts (`vendor/alto/chain/src/indexer.rs`)

## Startup sequence (encapsulated Simplex backend)

The typical start order should be:

1. caller/driver prepares caller-owned dependencies (`app`, `network`, `SimplexOptions`)
2. backend `build(ctx)` derives consensus planes from `network`
3. backend `build(ctx)` consumes `app` to construct `application::marshaled::Marshaled`
4. backend `build(ctx)` consumes `Marshaled` into `simplex::Config` and constructs `simplex::Engine::new(...)`
5. backend now holds minimal pre-start state: `engine` + `planes`
6. backend `start(self)` consumes that state and starts simplex with owned planes
7. driver supervises all tasks and propagates shutdown on fatal error

Behavioral reference for ordering and components remains Alto (`vendor/alto/chain/src/engine.rs`),
while this doc keeps those details behind the backend API.
