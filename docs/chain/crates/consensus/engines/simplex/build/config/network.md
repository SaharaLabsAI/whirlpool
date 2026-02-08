## Network config (Simplex channels)

Per-channel configuration used when registering the three Simplex network channels.

These values are passed to `core::network::ChannelNetwork::register_channel(...)` during
`SimplexChannels::new(...)`.

```rust
// Pseudocode — not compile-ready.

pub struct NetworkConfig {
  pub votes: core::network::ChannelConfig,
  pub certificates: core::network::ChannelConfig,
  pub resolver: core::network::ChannelConfig,
}

impl NetworkConfig {
  pub fn prod_defaults() -> Self {
    // Three independent defaults (even if values are equal).
    //
    // Alto uses per-channel quotas (votes/certificates/resolver; Alto labels differ)
    // plus a shared backlog.
    // See: `vendor/alto/chain/src/bin/validator.rs`.
    Self {
      votes: core::network::ChannelConfig { quota: 128, backlog: 16_384 },
      certificates: core::network::ChannelConfig { quota: 128, backlog: 16_384 },
      resolver: core::network::ChannelConfig { quota: 128, backlog: 16_384 },
    }
  }
}
```

Notes:

- `ChannelConfig` is intentionally generic at the `core` boundary. The concrete network defines the
  real quota/backlog semantics.
- These plane meanings are Simplex-specific; the core network traits remain consensus-agnostic.
