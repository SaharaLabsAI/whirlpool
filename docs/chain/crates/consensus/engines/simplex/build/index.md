## Build

The Simplex engine exposes a two-phase lifecycle:

1. construction (engine-defined; typically `SimplexEngine::new(...)`) wires all dependencies and
   produces a minimal pre-start state.
2. `start(self) -> Result<Handle>` consumes that state and spawns runtime tasks.

The returned handle supports graceful shutdown: signal stop, then await completion.

This split is important because:

- `commonware_consensus::simplex::Engine::new(...)` is a build-time wiring step.
- `commonware_consensus::simplex::Engine::start(...)` requires runtime network channels.
- `application::marshaled::Marshaled::new(...)` consumes the chain `App` and must be created inside
  the engine (callers should not know how).

### Inputs

At build time, the caller provides only caller-owned dependencies:

- `App`: the chain application implementation.
- `Network`: a network implementation that the engine can query to derive the 3 Simplex channels.
- `SimplexBuildConfig`: engine-specific build knobs and defaults.

See: [`config`](./config/index.md).

Notes:

- `SimplexBuildConfig` hardcodes chain-level constants like the persistence `PARTITION_PREFIX`
  (currently `"whirlpool-"`). The engine uses this to derive commonware partition strings.
- The consensus scheme *type* is hardcoded (BLS12-381 threshold). The scheme *value* is derived
  from membership/network state during `SimplexEngine::new(...)`.
- The build config includes a [`BlockerPolicy`](./config/index.md) rather than a concrete blocker.
  The engine derives the concrete commonware `Blocker` during `SimplexEngine::new(...)` using
  runtime dependencies (the network's oracle and the local validator's public key).

### Engine-owned internals

The engine constructs and owns:

- `SimplexChannels` (votes/certificates/resolver), derived from `Network`.
- `Marshaled` (block/payload availability + relay adapter).
- `simplex::Config` (constructed from `SimplexBuildConfig` + derived deps).

### Output

After construction, the engine should hold only the minimal state required to start:

- `engine: commonware_consensus::simplex::Engine<...>`
- `channels: SimplexChannels`

### Sketch

```rust
// Pseudocode.

let cfg = SimplexBuildConfig::prod_defaults();
let engine = SimplexEngine::new(app, network, cfg)?;

let driver = ConsensusDriver::new(engine);
let handle = driver.start().await?;

// Later...
handle.stop().await?;
```


- [`simplex`](../index.md)
- [`network channels`](../network/planes.md)
- [`marshal mailbox`](../network/marshal_mailbox/index.md)
