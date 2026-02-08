# Simplex engine wrapper

Our chain wraps Commonware’s Simplex implementation to standardize how we:

- construct the network planes from a consensus-agnostic network boundary, and
- start the underlying `commonware_consensus::simplex::Engine`.

This wrapper is where engine-specific wiring lives (channel IDs, plane meanings, mailbox wiring).

The wrapper follows the two-phase lifecycle described in [`build`](./build/index.md):

1. `new(...)` wires build-time dependencies and produces a minimal pre-start state.
2. `start(self) -> Result<Handle>` consumes that state and spawns runtime tasks.

See also:

- [`network/channels`](./network/planes.md)
- [`network/marshal_mailbox`](./network/marshal_mailbox/index.md)
- [`build`](./build/index.md)
- [`build config`](./build/config/index.md)

```rust
// NOTE: pseudocode for boundaries (not a spec; syntax intentionally loose).

/// Wrapper that owns the minimal pre-start state.
pub struct SimplexEngine<Net> {
  // Derived during `new(...)`.
  //
  // Note: these are the two items called out in `build/index.md` as the minimal pre-start state.
  pub channels: SimplexChannels<Net::Sender, Net::Receiver>,
  pub engine: commonware_consensus::simplex::Engine<...>,
}

impl<Net> SimplexEngine<Net>
where
  Net: core::network::Network,
{
  pub fn new(app: App, mut network: Net, cfg: SimplexBuildConfig) -> Result<Self> {
    // Engine-owned internals (callers do not construct these):
    // - SimplexChannels derived from the network
    // - Marshaled application wrapper
    // - commonware simplex::Config (including derived blocker)

    // Derive the three Simplex planes from the consensus-agnostic channel network.
    //
// Channel IDs + per-channel quotas/backlogs are engine-specific; see `network/channels`.
    // The exact config surface is intentionally not shown here.
    // `SimplexChannels::new(...)` registers three channel IDs via `network.register_channel(...)`.
    //
    // Per-plane channel configs live under `SimplexBuildConfig::network`.
    let channels = SimplexChannels::new(&mut network, cfg.network);

    // Details omitted: derive scheme from membership/network state, derive blocker
    // according to cfg.blocker policy (e.g. `oracle.control(self_public_key)`),
    // construct `Marshaled::new(app, ...)`, build `simplex::Config`, then:
    let engine = commonware_consensus::simplex::Engine::new(/* env, simplex::Config */);

    Ok(Self { channels, engine })
  }

  pub fn start(self) -> Result<Handle<()>> {
    // Spawn engine-owned runtime tasks (marshal actor, broadcast engine, etc.)
    // and start consensus by passing the (Sender, Receiver) pairs.
    Ok(self.engine.start(
      self.channels.votes,
      self.channels.certificates,
      self.channels.resolver,  // resolver channel (request/response)
    ))
  }
}
```

### Sketch

This matches the flow in [`build`](./build/index.md):

```rust
// Pseudocode.

let cfg = SimplexBuildConfig::prod_defaults();
let engine = SimplexEngine::new(app, network, cfg)?;

let driver = ConsensusDriver::new(engine);
let handle = driver.start().await?;

// Later...
handle.stop().await?;
```

## Notes

- The channels are logical; the network implementation may multiplex them.
- Payload bytes are handled via marshal mailbox wiring, not the three Simplex channels.
