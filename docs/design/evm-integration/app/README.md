# app

## Purpose

Abstract application trait crate that decouples consensus ordering from execution logic. Defines the `Application` trait — a richer version of `ConsensusApp` that supports EVM-backed state transitions with execution results (state root, receipts, gas). Provides an adapter (`ApplicationAdapter`) to bridge `Application` impls back to `ConsensusApp` for compatibility with the existing consensus engine.

## Public API at a glance (crate root exports)

[PROPOSED] — all items below are proposed; this crate does not yet exist.

```rust
// lib.rs
pub mod error;
pub mod types;

pub use error::ApplicationError;
pub use types::{ExecutionResult, EvmBlock};

/// Core application trait
pub trait Application: Send + Sync + 'static { ... }

/// Adapter bridging Application -> ConsensusApp
pub struct ApplicationAdapter<A: Application> { ... }
impl<A: Application> ConsensusApp for ApplicationAdapter<A> { ... }
```

## Modules

| Module | Responsibilities |
|---|---|
| `lib.rs` (root) | `Application` trait, `ApplicationAdapter` struct + `ConsensusApp` impl |
| `error` | `ApplicationError` enum — execution failures, verification failures |
| `types` | `ExecutionResult` struct, `EvmBlock` struct |

## Types & traits (public contract)

### Application trait [PROPOSED]

```rust
pub trait Application: Send + Sync + 'static {
    /// Block type — must satisfy consensus::Block for integration
    type Block: consensus::Block + Send + Sync;

    /// Execution output (receipts, gas used, state diff summary)
    type ExecutionResult: Send + Sync;

    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Produce the genesis block (no execution)
    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

    /// Propose a new block: gather txs, execute, assemble
    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl Future<Output = Result<(Self::Block, Self::ExecutionResult), Self::Error>> + Send;

    /// Verify a proposed block: re-execute and compare
    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl Future<Output = Result<Self::ExecutionResult, Self::Error>> + Send;
}
```

**Design decisions**:
- `propose()` returns `(Block, ExecutionResult)` because block assembly depends on execution (state root, receipts root come from tx execution).
- `verify()` returns `ExecutionResult` so the caller can persist state changes after consensus agreement.
- `genesis()` returns bare `Block` — no execution needed for genesis.

### ApplicationAdapter [PROPOSED]

```rust
pub struct ApplicationAdapter<A: Application> {
    inner: A,
}

impl<A: Application> ApplicationAdapter<A> {
    pub fn new(app: A) -> Self { Self { inner: app } }
    pub fn inner(&self) -> &A { &self.inner }
}
```

Implements `ConsensusApp` by delegating to `Application`, discarding execution results at the consensus boundary.

### ExecutionResult [PROPOSED]

```rust
pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}
```

Minimal summary of execution outcome. The full `BlockExecutionResult<Receipt>` from reth is internal to `app-evm`.

### EvmBlock [PROPOSED]

```rust
pub struct EvmBlock {
    // Consensus identity (satisfies consensus::Block)
    pub height: u64,
    pub parent_id: [u8; 32],

    // EVM execution summary
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub timestamp: u64,

    // Encoded transactions
    pub transactions: Vec<Vec<u8>>,
}
```

BLOCKER: Must implement `consensus::Block` + commonware codec traits (`Write`, `Read`, `EncodeSize`, `Digestible`, `Committable`). Pattern from `EmptyBlock` (`crates/whirlpool-node/src/block.rs`).

### ApplicationError [PROPOSED]

```rust
pub enum ApplicationError {
    /// EVM execution failed
    Execution(String),
    /// Block verification failed (state root mismatch, etc.)
    Verification(String),
    /// Missing state / database error
    State(String),
}
```

## Config schema

No configuration types in this crate. Configuration lives in `app-evm` (chain spec, EVM params) and `whirlpool-node` (node-level config).

## Config defaults table

N/A — this is a trait-only crate.

## Provider interfaces & swap points

| Interface | Provider trait | Default provider | Swap point |
|---|---|---|---|
| Application logic | `Application` | `EvmApplication` (in `app-evm`) | Any `Application` impl |
| Consensus bridge | `ConsensusApp` (via `ApplicationAdapter`) | `ApplicationAdapter<EvmApplication>` | Automatic from `Application` impl |

## Feature flags & cfg

[PROPOSED]:
- No feature flags initially
- `mock` feature (future): mock `Application` impl for testing

## SemVer & stability

UNKNOWN — workspace is pre-1.0. All interfaces are unstable until declared otherwise.

## Primary flows

### 1. Propose + execute flow [PROPOSED]
```pseudo
consensus_engine.propose(parent, height)
  → ApplicationAdapter.propose(parent, height)
    → Application.propose(parent, height)
      → (gather transactions from tx source)
      → (execute via ConfigureEvm → Executor)
      → (assemble block via BlockAssembler)
      → return (EvmBlock, ExecutionResult)
    → return Some(EvmBlock)  // ExecutionResult discarded at consensus boundary
```

### 2. Verify + re-execute flow [PROPOSED]
```pseudo
consensus_engine.verify(parent, block)
  → ApplicationAdapter.verify(parent, block)
    → Application.verify(parent, block)
      → (re-execute block.transactions against parent state)
      → (compare computed state_root vs block.state_root)
      → return Ok(ExecutionResult) or Err(ApplicationError::Verification)
    → return Ok(()) or Err(ConsensusError)
```

## API omissions report

- Transaction pool / tx source interface: out of scope, but `Application::propose()` needs one. Recommendation: inject at construction time.
- State persistence after finalization: out of scope. The `ExecutionResult` is returned but not persisted by this crate.

## Open questions / TODOs

- BLOCKER: `EvmBlock` serialization — must implement commonware codec traits. Need to verify exact trait bounds from `crates/consensus-simplex/`.
- BLOCKER: Transaction source for `propose()` — how does `Application` get pending transactions?
- **Resolved**: `ExecutionResult` carries summary hashes only (state_root, receipts_root, gas_used, receipt_count). The full `BundleState` (state diff) is internal to `EvmApplication` and is committed to the state DB upon finalization. Post-finalization state commitment is triggered by the `EventSink` handler, which either (a) calls a `commit_state()` method on `EvmApplication`, or (b) re-executes the finalized block. See architecture/consensus-app-bridge.md.
- UNKNOWN: Whether `Application` should have a `finalize()` method for post-consensus state commitment.
