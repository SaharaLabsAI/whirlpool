# `consensus` — driver abstractions

This file defines the **parent-level abstractions** for running consensus.

Motivation: we may benchmark multiple consensus algorithms over the same chain/execution layer.

`core` remains engine-agnostic (traits for propose/verify/report). `consensus` defines the
driver/engine boundary and algorithm-specific implementations live under `engines/*`.

## Shapes (pseudocode)

```rust
// NOTE: high-level shapes, not compile-ready.

/// Returned by `ConsensusEngine::start`.
///
/// Must support graceful shutdown (signal stop + await completion).
pub trait ConsensusHandle {
  type Error;

  /// Graceful shutdown: signal stop, then await completion.
  async fn stop(self) -> Result<(), Self::Error>;
}

/// Algorithm/engine interface.
///
/// The engine runs the *consensus algorithm* once it is fully constructed.
///
/// IMPORTANT: engines have different build-time needs (message types, payload handling), so
/// construction is engine-defined and is intentionally *not* part of this trait.
pub trait ConsensusEngine {
  type Error;

  /// Runtime handle returned by `start`.
  type Handle: ConsensusHandle<Error = Self::Error>;

  /// Start the engine and return a runtime handle.
  ///
  /// This consumes the engine and transfers ownership into runtime tasks.
  async fn start(self) -> Result<Self::Handle, Self::Error>;
}

/// Chain-facing driver: a thin wrapper that starts a pre-built engine.
///
/// The caller is responsible for constructing the engine from its direct dependencies
/// (e.g. `app`, `network`, engine-specific config).
pub struct ConsensusDriver<Engine> {
  engine: Engine,
}

impl<Engine> ConsensusDriver<Engine>
where
  Engine: ConsensusEngine,
{
  pub fn new(engine: Engine) -> Self {
    Self { engine }
  }

  pub async fn start(self) -> Result<Engine::Handle, Engine::Error> {
    self.engine.start().await
  }
}
```

## How Simplex fits

The Simplex implementation is a concrete engine + driver composition that wires:

- Simplex engine (`commonware_consensus::simplex::Engine`)
- payload networking (buffered broadcast + marshal actor + mailbox/subscriptions)
- the 3 Simplex network planes (votes/certs/resolver)
- a reporter that produces `ConsensusEvent::Finalized(types::FinalizedBlock)`

See: [`simplex`](./engines/simplex/index.md).

## Notes on dynamic dispatch

We select the engine at **compile time** (different binaries / feature flags) for benchmarking.

So the intended pattern is:

- `ConsensusDriver<Engine>` holds a concrete `Engine: ConsensusEngine`.
- The caller constructs `Engine` explicitly (from `app`, `network`, and engine-specific config).
- No trait objects (`Box<dyn ...>`) and no runtime selection.

If runtime selection is ever needed, it will require additional type erasure/adapters and is out of
scope for this design.
