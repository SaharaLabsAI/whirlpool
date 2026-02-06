# `consensus` — driver abstractions

This file defines the **parent-level abstractions** for running consensus.

Motivation: we may benchmark multiple consensus algorithms over the same chain/execution layer.

`core` remains engine-agnostic (traits for propose/verify/report). `consensus` defines the
driver/backend boundary and algorithm-specific implementations live under `backends/*`.

## Shapes (pseudocode)

```rust
// NOTE: high-level shapes, not compile-ready.

/// Algorithm/backend interface.
///
/// The backend runs the *consensus algorithm* given application + networking inputs.
///
/// IMPORTANT: backends have different needs (message types, payload handling), so input shape
/// is backend-defined. Callers should pass direct dependencies, not backend internals.
pub trait ConsensusBackend {
  type Error;
  type BuildInput<'a>
  where
    Self: 'a;

  async fn build(input: Self::BuildInput<'_>) -> Result<Self, Self::Error>
  where
    Self: Sized;

  async fn start(self) -> Result<(), Self::Error>;
}

/// Chain-facing driver composition: owns chain app + wiring.
///
/// Backend selection is compile-time (different binaries / feature flags).
///
/// The driver typically also owns a backend-specific build config `Cfg`.
pub struct ConsensusDriver<App, Backend, Network, Cfg> {
  app: App,
  network: Network,
  cfg: Cfg,
  _backend: std::marker::PhantomData<Backend>,
}

impl<App, Backend, Network, Cfg> ConsensusDriver<App, Backend, Network, Cfg>
where
  Backend: ConsensusBackend,
{
  pub async fn start(self) -> Result<(), Backend::Error> {
    // Build backend from direct caller-owned dependencies.
    let backend = Backend::build(/* app, network, cfg */).await?;

    // Start backend; ownership moves into runtime tasks.
    backend.start().await
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

- `ConsensusDriver<App, Backend, Network, Cfg>` is generic over a concrete `Backend: ConsensusBackend`.
- No trait objects (`Box<dyn ...>`) and no runtime selection.

If runtime selection is ever needed, it will require additional type erasure/adapters and is out of
scope for this design.
