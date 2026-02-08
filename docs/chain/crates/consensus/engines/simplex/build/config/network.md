## Network config (Simplex planes)

Per-plane channel configuration used when registering the three Simplex network planes.

These values are passed to `core::network::ChannelNetwork::register_channel(...)` during
`Planes::new(...)`.

```rust
// Pseudocode — not compile-ready.

pub struct NetworkConfig {
  pub pending: core::network::ChannelConfig,
  pub recovered: core::network::ChannelConfig,
  pub resolver: core::network::ChannelConfig,
}

impl NetworkConfig {
  pub fn prod_defaults() -> Self {
    // Three independent defaults (even if values are equal).
    //
    // Alto uses per-plane quotas (pending/recovered/resolver) plus a shared backlog.
    // See: `vendor/alto/chain/src/bin/validator.rs`.
    Self {
      pending: core::network::ChannelConfig { quota: 128, backlog: 16_384 },
      recovered: core::network::ChannelConfig { quota: 128, backlog: 16_384 },
      resolver: core::network::ChannelConfig { quota: 128, backlog: 16_384 },
    }
  }
}
```

Notes:

- `ChannelConfig` is intentionally generic at the `core` boundary. The concrete network defines the
  real quota/backlog semantics.
- These plane meanings are Simplex-specific; the core network traits remain consensus-agnostic.
