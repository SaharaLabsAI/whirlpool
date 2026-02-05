# `core`

**Purpose**: component *interfaces* expressed as Rust traits.

Owns: traits that define the boundaries between modules (e.g. `Storage`, `Executor`, `Consensus`, `Network`, `Mempool`, `Rpc`).

Why: components depend on trait bounds, not concrete implementations.

Inputs/outputs: trait methods use `types` as the shared data model.

Depends on: `types` (and minimal `async`/error plumbing if needed).

Not in scope: concrete implementations, networking transports, wire formats.

## Consensus interfaces (shape)

Extracted from Alto’s pattern: consensus owns the algorithm; the chain-specific “application” provides `genesis`, `propose`, and `verify` over an ancestry stream, and receives activity notifications.

In Alto this shows up as:

- `Application`: provides `genesis` + `propose`
- `VerifyingApplication`: provides `verify`
- `Reporter`: receives activity (e.g. finalized blocks)

```rust
// NOTE: pseudocode for boundaries (not a spec; syntax intentionally loose).

// NOTE: naming this crate `core` may collide with Rust's built-in `core` crate.
// Consider `chain-core` or `interfaces` when you turn docs into real crates.

pub trait Ancestry<Block> {
  // Yields: candidate block, then its parent(s) as needed.
  fn next(&mut self) -> Option<Block>;
}

pub trait ConsensusApplication {
  type Block;    // usually `types::Block`
  type Context;  // consensus-local context (view/epoch, proposer id, etc.)
  type Error;

  async fn genesis(&mut self) -> Self::Block;

  async fn propose(
    &mut self,
    ctx: Self::Context,
    ancestry: &mut dyn Ancestry<Self::Block>,
  ) -> Result<Option<Self::Block>, Self::Error>;
}

pub trait VerifyingApplication {
  type Block;
  type Context;
  type Error;

  async fn verify(
    &mut self,
    ctx: Self::Context,
    ancestry: &mut dyn Ancestry<Self::Block>,
  ) -> Result<bool, Self::Error>;
}

pub trait Reporter<A> {
  async fn report(&mut self, activity: A);
}

// Example “activity” payload.
pub enum Activity {
  Finalized(types::FinalizedBlock),
}

// Optional: a higher-level driver boundary used by the node.
pub trait ConsensusDriver {
  type Error;
  async fn start(&mut self) -> Result<(), Self::Error>;
}

Where this usually lands:

- `ConsensusApplication` / `VerifyingApplication`: implemented by the “app” layer (often backed by `executor` + `storage`).
- `Reporter`: implemented by something that wants notifications (logs, metrics, indexer).
- `ConsensusDriver`: implemented by the concrete `consensus` crate.
```

Design rule: `core` depends only on `types` (no dependency on concrete crates like `consensus`/`network`).
