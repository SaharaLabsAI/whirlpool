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
/// The backend runs the *consensus algorithm* given an application adapter + networking.
///
/// IMPORTANT: backends have different needs (message types, payload handling), so we keep
/// the boundary minimal and allow each backend to define its required "context".
pub trait ConsensusBackend {
  type Error;
  type Context<'a>
  where
    Self: 'a;

  async fn start(&mut self, ctx: Self::Context<'_>) -> Result<(), Self::Error>;
}

/// Chain-facing driver composition: owns chain app + backend + wiring.
///
/// Backend selection is compile-time (different binaries / feature flags).
pub struct ConsensusDriver<App, Backend, Network> {
  app: App,
  backend: Backend,
  network: Network,
}

impl<App, Backend, Network> ConsensusDriver<App, Backend, Network>
where
  Backend: ConsensusBackend,
{
  pub async fn start(&mut self) -> Result<(), Backend::Error> {
    // 1) build backend context from (app, network, storage bindings)
    // 2) start backend
    self.backend.start(/* ctx */).await
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

- `ConsensusDriver<App, Backend, Network>` is generic over a concrete `Backend: ConsensusBackend`.
- No trait objects (`Box<dyn ...>`) and no runtime selection.

If runtime selection is ever needed, it will require additional type erasure/adapters and is out of
scope for this design.
