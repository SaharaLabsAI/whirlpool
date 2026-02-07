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

/// Chain-level constants intentionally not exposed as caller knobs.
///
/// Used to derive the commonware `partition` strings for persistence.
const PARTITION_PREFIX: &str = "whirlpool-";

/// Consensus scheme type used by this chain.
///
/// Note: a concrete scheme *value* depends on validator membership and is derived from the
/// membership/network layer during engine construction.
type Scheme = commonware_consensus::simplex::scheme::bls12381_threshold::Scheme<
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
  ElectorCfg = commonware_consensus::simplex::elector::RoundRobin,
  Strat = commonware_parallel::Sequential,
> {
  /// Optional indexer/pusher that receives finalized artifacts.
  ///
  /// If `None`, we still finalize blocks locally; we just don't push them out.
  pub indexer: Option<I>,

  /// Peer blocking / authorization policy.

  ///
  /// Default is [`BlockerPolicy::NetworkOracleControl`], i.e. derive a concrete commonware
  /// `Blocker` from the network's oracle during engine construction.
  ///
  /// Concretely, the engine needs:
  /// - a network-provided oracle (from the network layer)
  /// - the local validator's public key (typically from the derived consensus `scheme` value)
  ///
  /// And then derives the blocker with a pattern like `oracle.control(self_public_key)`.
  ///
  /// Alto references (wiring blocker from oracle):
  /// - `vendor/alto/chain/src/bin/validator.rs:199` (authenticated network returns `oracle`)
  /// - `vendor/alto/chain/src/bin/validator.rs:246` (passes `oracle.clone()` as blocker)
  pub blocker: BlockerPolicy,

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

/// Policy for how the engine derives a commonware `Blocker` implementation.
///
/// In commonware simplex, the runtime config wants a concrete type `B` where
/// `B: commonware_p2p::Blocker<PublicKey = S::PublicKey>`.
///
/// This policy exists to keep defaults pure (no runtime dependencies) while still
/// making it explicit that the engine will derive the concrete blocker during
/// `SimplexEngine::new(...)`.
pub enum BlockerPolicy {
  /// Derive a blocker from the network's oracle and the local validator identity.
  ///
  /// Typical derivation: `oracle.control(self_public_key)`.
  NetworkOracleControl,

  /// Do not provide a peer blocker.
  ///
  /// This is typically only useful for local/dev networks or tests where peer authorization is
  /// not required.
  ///
  /// Note: this assumes the engine can construct a `Blocker` that effectively permits peers.
  /// If the underlying network/oracle does not support this, the engine should reject the build.
  Disabled,

  // Optional extension: `Custom(B)` for tests.
}

impl<I> SimplexBuildConfig<I> {
  /// Production-flavored defaults (aligned to Alto validator defaults).
pub fn prod_defaults() -> Self {
    Self {
      indexer: None,

      blocker: BlockerPolicy::NetworkOracleControl,
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
  pub fn dev_defaults() -> Self {
    Self {
      // Keep non-timing defaults the same as prod.
      blocker: BlockerPolicy::Disabled,
      activity_timeout: ViewDelta::new(10),
      skip_timeout: ViewDelta::new(5),
      ..Self::prod_defaults()
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
