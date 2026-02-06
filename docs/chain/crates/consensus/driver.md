# `consensus` — driver abstractions

This file defines the **parent-level abstractions** for running consensus.

Motivation: we may benchmark multiple consensus algorithms over the same chain/execution layer.

`core` remains engine-agnostic (traits for propose/verify/report). `consensus` defines the
driver/backend boundary and algorithm-specific implementations live under `backends/*`.

## Shapes (pseudocode)

```rust
// NOTE: high-level shapes, not compile-ready.

/// Returned by `ConsensusBackend::start`.
///
/// Must support graceful shutdown (signal stop + await completion).
pub trait ConsensusHandle {
  type Error;

  /// Graceful shutdown: signal stop, then await completion.
  async fn stop(self) -> Result<(), Self::Error>;
}

/// Algorithm/backend interface.
///
/// The backend runs the *consensus algorithm* once it is fully constructed.
///
/// IMPORTANT: backends have different build-time needs (message types, payload handling), so
/// construction is backend-defined and is intentionally *not* part of this trait.
pub trait ConsensusBackend {
  type Error;

  /// Runtime handle returned by `start`.
  type Handle: ConsensusHandle<Error = Self::Error>;

  /// Start the backend and return a runtime handle.
  ///
  /// This consumes the backend and transfers ownership into runtime tasks.
  async fn start(self) -> Result<Self::Handle, Self::Error>;
}

/// Chain-facing driver: a thin wrapper that starts a pre-built backend.
///
/// The caller is responsible for constructing the backend from its direct dependencies
/// (e.g. `app`, `network`, backend-specific config).
pub struct ConsensusDriver<Backend> {
  backend: Backend,
}

impl<Backend> ConsensusDriver<Backend>
where
  Backend: ConsensusBackend,
{
  pub fn new(backend: Backend) -> Self {
    Self { backend }
  }

  pub async fn start(self) -> Result<Backend::Handle, Backend::Error> {
    self.backend.start().await
  }
}
```

## How Simplex fits

The Simplex implementation is a concrete backend + driver composition that wires:

- Simplex engine (`commonware_consensus::simplex::Engine`)
- payload networking (buffered broadcast + marshal actor + mailbox/subscriptions)
- the 3 Simplex network planes (votes/certs/resolver)
- a reporter that produces `ConsensusEvent::Finalized(types::FinalizedBlock)`

See: [`simplex`](./backends/simplex/index.md).

## Notes on dynamic dispatch

We select the backend at **compile time** (different binaries / feature flags) for benchmarking.

So the intended pattern is:

- `ConsensusDriver<Backend>` holds a concrete `Backend: ConsensusBackend`.
- The caller constructs `Backend` explicitly (from `app`, `network`, and backend-specific config).
- No trait objects (`Box<dyn ...>`) and no runtime selection.

If runtime selection is ever needed, it will require additional type erasure/adapters and is out of
scope for this design.
