# `consensus` — types (shape)

Not a spec. These are the minimal public shapes the rest of the chain cares about.

```rust
// NOTE: high-level shapes, not a spec.

pub struct ConsensusConfig {
  pub chain_id: u64,
  pub quorum: types::Quorum,
  pub timeouts: Timeouts,
}

pub struct Timeouts {
  pub leader: std::time::Duration,
  pub vote: std::time::Duration,
}

pub enum ConsensusEvent {
  Finalized(types::FinalizedBlock),
  // e.g. view/round advanced, peer needed, etc.
}

// Driver/engine abstractions are defined in `consensus/driver.md`.
```
