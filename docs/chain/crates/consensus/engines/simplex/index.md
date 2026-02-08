This chain uses `vendor/commonware/consensus` Simplex as the consensus engine.

This section is implementation-oriented: it enumerates the concrete components you must provide
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

## Build and Start (encapsulated engine)

The engine owns all Simplex-specific internals.

- Callers must not construct `Marshaled` or `SimplexChannels`.
- Callers provide only caller-owned dependencies (`app`, `network`, build config).

When started, the engine returns a runtime handle (see `ConsensusHandle` in
`docs/chain/crates/consensus/driver.md`). This handle supports graceful shutdown:

- `handle.stop()` signals shutdown and awaits task completion.

Build details:

- [`build`](./build/index.md)
- [`build config`](./build/config/index.md)

Engine wrapper:

- [`engine`](./engine.md)

## Adapter types you will actually use (commonware)

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

### Network channels (3 channels required by Simplex)

See: [`network/channels`](./network/planes.md).

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

## Startup sequence (encapsulated Simplex engine)

The typical start order should be:

1. caller constructs the engine from caller-owned dependencies (`app`, `network`, `SimplexBuildConfig`)
2. engine construction derives consensus planes from `network`
3. engine construction consumes `app` to construct `application::marshaled::Marshaled`
4. engine construction builds and validates `simplex::Config` from `SimplexBuildConfig` + engine defaults
5. engine construction constructs `simplex::Engine::new(...)`
6. engine holds minimal pre-start state: `engine` + `planes`
7. caller starts the engine via `engine.start()` (ownership moves into runtime tasks)
8. the caller/driver supervises tasks and propagates shutdown on fatal error

Behavioral reference for ordering and components remains Alto (`vendor/alto/chain/src/engine.rs`),
while this doc keeps those details behind the engine API.
