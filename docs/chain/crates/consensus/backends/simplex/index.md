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

This backend is used with the parent-level `ConsensusDriver<App, Backend, Network, Cfg>` composition
defined in [`consensus/driver`](../../driver.md).

The intent is:

- `core/*` stays engine-agnostic.
- `consensus/backends/simplex/*` wires `core` to `commonware_consensus::simplex`.

### 1) Backend implements `ConsensusBackend`

```rust
// Pseudocode — not compile-ready.

// Type aliases keep the docs readable. (Alto does the same; see
// `vendor/alto/types/src/consensus.rs:13`.)
//
// Default crypto scheme matches commonware's BLS12-381 threshold scheme:
// `vendor/commonware/consensus/src/simplex/scheme/bls12381_threshold.rs`.
type DefaultScheme = commonware_consensus::simplex::scheme::bls12381_threshold::Scheme<
  commonware_cryptography::ed25519::PublicKey,
  commonware_cryptography::bls12381::primitives::variant::MinSig,
>;

/// Caller-facing build configuration for the Simplex backend.
///
/// Keep this "surface" small, but make defaults explicit.
///
/// Important: `Marshaled` + network `Planes` are backend internals; callers should not
/// construct them.
pub struct SimplexBuildConfig<I, S = DefaultScheme, ElectorCfg = commonware_consensus::simplex::elector::RoundRobin, Strat = commonware_parallel::Sequential> {
  /// Storage namespace prefix used to derive commonware `partition` strings.
  pub partition_prefix: String,

  /// Optional indexer/pusher that receives finalized artifacts.
  ///
  /// If `None`, we still finalize blocks locally; we just don't push them out.
  pub indexer: Option<I>,

  /// Consensus crypto scheme.
  ///
  /// Default type is commonware's BLS12-381 threshold scheme:
  /// `vendor/commonware/consensus/src/simplex/scheme/bls12381_threshold.rs`.
  /// (Alto uses a type alias for this: `vendor/alto/types/src/consensus.rs:13`.)
  pub scheme: S,

  /// Peer blocking / authorization policy.
  ///
  /// Default is `FromNetworkOracle`, i.e. derive a blocker from the provided network.
  /// Alto reference:
  /// - `vendor/alto/chain/src/bin/validator.rs:199` (authenticated network returns `oracle`)
  /// - `vendor/alto/chain/src/bin/validator.rs:245` (passes `oracle.clone()` as blocker)
  pub blocker: BlockerConfig,

  /// Leader election policy.
  ///
  /// Default: deterministic rotation (`RoundRobin`).
  pub elector: ElectorCfg,

  /// Parallel signature verification / aggregation strategy.
  ///
  /// Default: `commonware_parallel::Sequential`.
  pub strategy: Strat,

  /// Fixed epoch used for the run.
  ///
  /// Default: `Epoch::zero()`.
  pub epoch: Epoch,

  /// Capacity of internal mailboxes used by consensus + payload tasks.
  pub mailbox_size: usize,

  /// Persistence buffers used by simplex replay / storage journals.
  ///
  /// Defaults match Alto (`vendor/alto/chain/src/engine.rs`).
  pub replay_buffer: NonZeroUsize,
  pub write_buffer: NonZeroUsize,

  /// Buffer pool used by persistence structures.
  ///
  /// Default matches Alto (`vendor/alto/chain/src/engine.rs`).
  pub buffer_pool: PoolRef,

  /// Simplex timing parameters.
  pub leader_timeout: Duration,
  pub notarization_timeout: Duration,
  pub nullify_retry: Duration,

  /// View tracking / liveness.
  pub activity_timeout: ViewDelta,
  pub skip_timeout: ViewDelta,

  /// Missing-artifact fetch behavior.
  pub fetch_timeout: Duration,
  pub fetch_concurrent: usize,
}

/// How the backend sources the commonware "blocker" implementation.
///
/// Default is to derive it from the network oracle.
pub enum BlockerConfig {
  FromNetworkOracle,
  // Optional extension: `Custom(B)` for tests.
}

impl<I> SimplexBuildConfig<I> {
  /// Production-flavored defaults (aligned to Alto validator defaults).
  pub fn prod_defaults(partition_prefix: String, scheme: DefaultScheme) -> Self {
    Self {
      partition_prefix,
      indexer: None,

      scheme,
      blocker: BlockerConfig::FromNetworkOracle,
      elector: commonware_consensus::simplex::elector::RoundRobin::default(),
      strategy: commonware_parallel::Sequential,
      epoch: Epoch::zero(),

      mailbox_size: 1024,

      // Alto persistence defaults.
      replay_buffer: NonZeroUsize::new(8 * 1024 * 1024).unwrap(),
      write_buffer: NonZeroUsize::new(1024 * 1024).unwrap(),
      buffer_pool: PoolRef::new(NZU16!(4_096), NZUsize!(8_192)),

      leader_timeout: Duration::from_secs(1),
      notarization_timeout: Duration::from_secs(2),
      nullify_retry: Duration::from_secs(10),

      activity_timeout: ViewDelta::new(256),
      skip_timeout: ViewDelta::new(32),

      fetch_timeout: Duration::from_secs(2),
      fetch_concurrent: 4,
    }
  }

  /// Dev/test-flavored defaults (faster turnover in local runs).
  pub fn dev_defaults(partition_prefix: String, scheme: DefaultScheme) -> Self {
    Self {
      // Keep non-timing defaults the same as prod.
      activity_timeout: ViewDelta::new(10),
      skip_timeout: ViewDelta::new(5),
      ..Self::prod_defaults(partition_prefix, scheme)
    }
  }
}

pub struct SimplexBackend<E, App, Net> {
  // Minimal pre-start backend state.
  engine: commonware_consensus::simplex::Engine<E, /* ... */>,
  planes: Planes,

  _app: std::marker::PhantomData<App>,
  _net: std::marker::PhantomData<Net>,
}

impl<E, App, Net> SimplexBackend<E, App, Net> {
  pub async fn build<I>(
    app: App,
    network: &Net,
    cfg: SimplexBuildConfig<I>,
  ) -> Result<Self, anyhow::Error> {
    // 1) derive Planes from network
    // 2) derive commonware `simplex::Config` from `cfg` + backend defaults
    // 3) consume `app` to construct Marshaled
    // 4) consume Marshaled into simplex::Config (automaton + relay)
    // 5) construct simplex::Engine::new(...)
    Ok(Self { /* engine, planes */ })
  }

  pub async fn start(self) -> Result<(), anyhow::Error> {
    // Consumes backend state and starts simplex with owned planes.
    Ok(())
  }
}
```

The driver (`ConsensusDriver<App, SimplexBackend<...>, Net, SimplexBuildConfig<_>>`) should only be responsible for:

- providing caller-owned dependencies (`app`, `network`, `SimplexBuildConfig`)
- invoking backend lifecycle (`build` then `start`)
- supervising task lifetime and shutdown propagation

`Marshaled` and `Planes` are backend internals and must not be required from callers.
Current backend state is intentionally minimal: only `engine` + `planes` before `start()`.

If the generic backend trait in `consensus/driver` requires a context type, keep that as an
internal implementation detail. Do not require callers to construct a `SimplexContext`.

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

1. caller/driver prepares caller-owned dependencies (`app`, `network`, `SimplexBuildConfig`)
2. backend `build(app, network, cfg)` derives consensus planes from `network`
3. backend `build(app, network, cfg)` consumes `app` to construct `application::marshaled::Marshaled`
4. backend `build(app, network, cfg)` builds and validates `simplex::Config` from `cfg` + backend defaults
5. backend `build(app, network, cfg)` constructs `simplex::Engine::new(...)`
6. backend now holds minimal pre-start state: `engine` + `planes`
7. backend `start(self)` consumes that state and starts simplex with owned planes
8. driver supervises all tasks and propagates shutdown on fatal error

Behavioral reference for ordering and components remains Alto (`vendor/alto/chain/src/engine.rs`),
while this doc keeps those details behind the backend API.
