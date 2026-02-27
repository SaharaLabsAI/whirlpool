# Domain: Application

## Definition

The application domain defines how blocks are proposed, executed, and verified — bridging consensus ordering with state transition logic. Currently implemented as the trivial `EmptyBlockApp`; this design introduces an abstract `Application` trait that supports EVM-backed execution.

## Derived crates

| Crate | Role | Status |
|---|---|---|
| `app` | [PROPOSED] Abstract `Application` trait, `EvmBlock` type, execution result types | Proposed |
| `whirlpool-node` | Current `EmptyBlockApp` + `EmptyBlock` (trivial impl) | Grounded |

## Key public contracts

### Current: EmptyBlockApp (grounded)
**Grounded**: `crates/whirlpool-node/src/app.rs::EmptyBlockApp`
- ZST (zero-sized type), stateless
- `ConsensusApp::Block = EmptyBlock`
- `genesis()`: returns `EmptyBlock` at height 0
- `propose()`: creates `EmptyBlock::new(height, parent.id())`
- `verify()`: enforces 5 rules (height continuity, parent link, genesis rules)

### Current: EmptyBlock (grounded)
**Grounded**: `crates/whirlpool-node/src/block.rs::EmptyBlock`
- Fields: `height: u64`, `parent_id: [u8; 32]`
- `consensus::Block::Id = [u8; 32]`
- `compute_id()`: SHA-256 of `height || parent_id`
- Implements: `consensus::Block`, `commonware_codec::{Write, Read, EncodeSize}`, `commonware_cryptography::{Digestible, Committable}`

### [PROPOSED] Application trait
```rust
/// Abstract application that supports EVM-aware block execution.
/// Extends the concept of ConsensusApp with execution results.
pub trait Application: Send + Sync + 'static {
    /// Block type produced by this application
    type Block: consensus::Block;

    /// Execution output type (e.g., receipts, state diff)
    type ExecutionResult: Send + Sync;

    /// Error type for execution failures
    type Error: std::error::Error + Send + Sync;

    /// Genesis block
    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

    /// Propose a new block given parent context and pending transactions
    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl Future<Output = Result<(Self::Block, Self::ExecutionResult), Self::Error>> + Send;

    /// Verify and execute a proposed block
    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl Future<Output = Result<Self::ExecutionResult, Self::Error>> + Send;
}
```

**Rationale**: `propose` returns `(Block, ExecutionResult)` because block assembly requires execution (state root comes from executing txs). `verify` re-executes and returns results for the verifier to apply.

### [PROPOSED] EvmBlock type
```rust
/// Block type that carries EVM execution data alongside consensus identity.
pub struct EvmBlock {
    /// Consensus identity
    pub height: u64,
    pub parent_id: [u8; 32],

    /// EVM header fields
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub timestamp: u64,

    /// Transaction list (encoded)
    pub transactions: Vec<Vec<u8>>,
}
```

**BLOCKER**: The exact serialization format for `EvmBlock` needs alignment with commonware codec requirements (`Write`, `Read`, `EncodeSize`, `Digestible`, `Committable`). The `EmptyBlock` pattern shows these are required.

## Core workflows

### Block proposal (proposed)
1. Consensus engine calls `app.propose(parent, height)`
2. `Application` impl gathers pending transactions
3. Constructs EVM environment via `ConfigureEvm::next_evm_env(parent, attributes)`
4. Executes transactions via `BlockExecutorFactory` → `Executor::execute()`
5. Assembles block via `BlockAssembler::assemble_block()`
6. Returns `(EvmBlock, ExecutionResult)`

### Block verification (proposed)
1. Consensus engine calls `app.verify(parent, block)`
2. `Application` impl re-executes transactions from `block.transactions`
3. Compares computed `state_root` against `block.state_root`
4. Returns `ExecutionResult` on match, error on mismatch

## Open questions / TODOs

- BLOCKER: How does `Application` get pending transactions for `propose()`? Transaction pool is out of scope, but the interface needs a source.
  - Possible: `propose()` takes an iterator/vec of transactions as input, or `Application` holds a reference to a tx pool.
- UNKNOWN: Whether `EvmBlock` should wrap reth's `Block` type directly or maintain its own struct.
- UNKNOWN: Whether `ExecutionResult` needs to carry the full `BundleState` (state diff) or just summary hashes. **Resolved**: `ExecutionResult` carries summary hashes only; the `BundleState` is persisted separately by `EvmApplication` internally during execution. Post-finalization state commitment uses re-execution from the finalized block (see architecture/consensus-app-bridge.md).
