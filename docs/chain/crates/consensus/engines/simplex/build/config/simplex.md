## Simplex config

Algorithm/runtime knobs used to build `commonware_consensus::simplex::Config`.

```rust
// Pseudocode — not compile-ready.

pub struct SimplexConfig {
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

impl SimplexConfig {
  pub fn prod_defaults() -> Self {
    Self {
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

  pub fn dev_defaults() -> Self {
    Self {
      // Keep non-timing defaults the same as prod.
      activity_timeout: ViewDelta::new(10),
      skip_timeout: ViewDelta::new(5),
      ..Self::prod_defaults()
    }
  }
}
```
