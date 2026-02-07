## Build Config

This page defines the caller-facing build configuration for the Simplex engine.

Goals:

- Keep the chain-facing config surface small.
- Make default values explicit.
- Do not require callers to construct engine internals (`Marshaled`, `Planes`).

### Defaults and profiles

We provide two default profiles:

- `prod_defaults`: production-flavored defaults (aligned to Alto validator values).
- `dev_defaults`: faster turnover for local/dev runs.

Both profiles keep persistence sizing aligned to Alto (`vendor/alto/chain/src/engine.rs`) and only
vary view liveness thresholds (`activity_timeout`, `skip_timeout`).

### Pseudocode

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

/// Caller-facing build configuration for the Simplex engine.
///
/// Keep this "surface" small, but make defaults explicit.
///
/// Important: `Marshaled` + network `Planes` are engine internals; callers should not
/// construct them.
pub struct SimplexBuildConfig<
  I,
  S = DefaultScheme,
  ElectorCfg = commonware_consensus::simplex::elector::RoundRobin,
  Strat = commonware_parallel::Sequential,
> {
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
  pub scheme: S,

  /// Peer blocking / authorization policy.
  ///
  /// Default is `FromNetworkOracle`, i.e. derive a blocker from the provided network.
  /// Alto reference:
  /// - `vendor/alto/chain/src/bin/validator.rs:199` (authenticated network returns `oracle`)
  /// - `vendor/alto/chain/src/bin/validator.rs:246` (passes `oracle.clone()` as blocker)
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

/// How the engine sources the commonware "blocker" implementation.
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
```

### Notes

- The engine still needs additional build-time dependencies (e.g., `Network` to derive planes;
  `App` to construct `Marshaled`). `SimplexBuildConfig` only covers knobs and explicit defaults.
- `commonware_consensus::simplex::Config::assert()` enforces constraints like
  `leader_timeout <= notarization_timeout` and `skip_timeout <= activity_timeout`.
  See: `vendor/commonware/consensus/src/simplex/config.rs`.
