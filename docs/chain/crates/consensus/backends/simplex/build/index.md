## Build

The Simplex backend exposes a two-phase lifecycle:

1. construction (backend-defined; typically `SimplexBackend::new(...)`) wires all dependencies and
   produces a minimal pre-start state.
2. `start(self) -> Result<Handle>` consumes that state and spawns runtime tasks.

The returned handle supports graceful shutdown: signal stop, then await completion.

This split is important because:

- `commonware_consensus::simplex::Engine::new(...)` is a build-time wiring step.
- `commonware_consensus::simplex::Engine::start(...)` requires runtime network planes.
- `application::marshaled::Marshaled::new(...)` consumes the chain `App` and must be created inside
  the backend (callers should not know how).

### Inputs

At build time, the caller provides only caller-owned dependencies:

- `App`: the chain application implementation.
- `Network`: a network implementation that the backend can query to derive the 3 Simplex planes.
- `SimplexBuildConfig`: backend-specific build knobs and defaults.

See: [`config`](./config.md).

### Backend-owned internals

The backend constructs and owns:

- `Planes` (vote/cert/resolver), derived from `Network`.
- `Marshaled` (block/payload availability + relay adapter).
- `simplex::Config` (constructed from `SimplexBuildConfig` + derived deps).

### Output

After `build(...)`, the backend should hold only the minimal state required to start:

- `engine: commonware_consensus::simplex::Engine<...>`
- `planes: Planes`

### Sketch

```rust
// Pseudocode.

let cfg = SimplexBuildConfig::prod_defaults(partition_prefix, scheme);
let backend = SimplexBackend::new(app, network, cfg)?;

let driver = ConsensusDriver::new(backend);
let handle = driver.start().await?;

// Later...
handle.stop().await?;
```


- [`simplex`](../index.md)
- [`network planes`](../network/planes.md)
- [`marshal mailbox`](../network/marshal_mailbox/index.md)
