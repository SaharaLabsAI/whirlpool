# Blockers — EVM Transaction Execution

## Active Blockers

### B-1: `builder.finish()` requires `StateRootProvider`
- **Type**: `information-gap`
- **Severity**: Degraded (workaround exists)
- **Description**: reth's `BlockBuilder::finish()` expects a `StateRootProvider` implementation for computing state root with hashed state updates. `InMemoryStateDb` does not implement this reth trait.
- **Evidence**: `vendor/reth/crates/evm/evm/src/execute.rs::BlockBuilder::finish` signature
- **Workaround**: Bypass reth's state root computation. Extract `BundleState` manually via `state.take_bundle()`, commit to `InMemoryStateDb`, then compute state root via `InMemoryStateDb::state_root()`. This means we cannot use `builder.finish()` directly and must implement a custom finish flow.
- **Impact**: Medium — requires custom block assembly logic instead of using reth's built-in assembler

### B-2: No finalization callback in ConsensusApp
- **Type**: `decision-gap`
- **Severity**: Degraded (MVP workaround proposed)
- **Description**: `ConsensusApp` trait has only `genesis/propose/verify` — no `finalize` or `commit` callback. This means `propose()` must commit state speculatively, and there's no mechanism to rollback if consensus rejects the block.
- **Evidence**: `crates/consensus/src/app.rs::ConsensusApp` — only 3 methods
- **Proposed resolution**: Clone-based snapshots (STRATEGY D-2). Proposer clones state before execution, commits to canonical only on success. If consensus rejects, canonical state has already been mutated. Acceptable for single-proposer MVP.
- **Impact**: Low for MVP (single proposer = proposed block is always finalized). High for multi-validator future.

### B-3: Decision — skip vs fail on invalid transactions
- **Type**: `decision-gap`
- **Severity**: Degraded
- **Description**: When `propose()` encounters an invalid transaction (decode failure or EVM execution error), should it skip the tx and continue, or fail the entire block proposal?
- **Proposed resolution**: Skip invalid txs (STRATEGY D-4), matching Ethereum mainnet behavior where invalid txs are excluded from blocks.
- **Impact**: Low — either approach is valid for MVP

### B-4: Decision — clone vs COW state snapshots
- **Type**: `decision-gap`
- **Severity**: Degraded
- **Description**: State snapshot mechanism for propose rollback safety. Clone is O(n) in state size; COW is more efficient but complex.
- **Proposed resolution**: Clone for MVP (STRATEGY D-2). State is in-memory HashMap, Clone is straightforward.
- **Impact**: Low for MVP. Performance concern only at scale.

## Resolved Blockers

*None yet — this is the initial design round.*
