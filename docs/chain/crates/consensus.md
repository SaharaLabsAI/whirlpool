# `consensus`

**Purpose**: decide canonical/finalized blocks.

Owns: vote/finality rules, consensus state machine/driver, fork choice (if any).

Inputs: candidate blocks (headers + execution results) + peer votes/messages.

Outputs: finalized head + consensus events.

Depends on: `types`, `core` (traits), `storage`.

## Key structs (shape)

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

// Main driver that wires the algorithm to the app boundaries.
pub struct ConsensusDriver<A> {
  pub app: A, // implements core::ConsensusApplication + core::VerifyingApplication
}
```
