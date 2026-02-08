## Build Config

This section defines the caller-facing build configuration for the Simplex engine.

Split:

- [`simplex`](./simplex.md): algorithm/runtime knobs (used to build `simplex::Config`)
- [`network`](./network.md): per-plane channel configuration (used to register network planes)

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

// Defined in subpages:
// - `SimplexConfig`: see `./simplex.md`
// - `NetworkConfig`: see `./network.md`
pub struct SimplexConfig { /* ... */ }
pub struct NetworkConfig { /* ... */ }

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
  pub blocker: BlockerPolicy,

  /// Leader election policy.
  ///
  /// Default: deterministic rotation (`RoundRobin`).
  pub elector: ElectorCfg,

  /// Parallel signature verification / aggregation strategy.
  ///
  /// Default: `commonware_parallel::Sequential`.
  pub strategy: Strat,

  /// Simplex algorithm/runtime knobs (see `./simplex.md`).
  pub simplex: SimplexConfig,

  /// Simplex per-plane network configuration (see `./network.md`).
  pub network: NetworkConfig,
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
  Disabled,
}

impl<I> SimplexBuildConfig<I> {
  /// Production-flavored defaults (aligned to Alto validator defaults).
  pub fn prod_defaults() -> Self {
    Self {
      indexer: None,

      blocker: BlockerPolicy::NetworkOracleControl,
      elector: commonware_consensus::simplex::elector::RoundRobin::default(),
      strategy: commonware_parallel::Sequential,

      simplex: SimplexConfig::prod_defaults(),
      network: NetworkConfig::prod_defaults(),
    }
  }

  /// Dev/test-flavored defaults (faster turnover in local runs).
  pub fn dev_defaults() -> Self {
    let mut cfg = Self::prod_defaults();
    cfg.blocker = BlockerPolicy::Disabled;
    cfg.simplex = SimplexConfig::dev_defaults();
    cfg
  }
}
```

### Notes

- The engine still needs additional build-time dependencies (e.g., `Network` to derive planes;
  `App` to construct `Marshaled`). `SimplexBuildConfig` only covers knobs and explicit defaults.
- `commonware_consensus::simplex::Config::assert()` enforces constraints like
  `leader_timeout <= notarization_timeout` and `skip_timeout <= activity_timeout`.
  See: `vendor/commonware/consensus/src/simplex/config.rs`.
