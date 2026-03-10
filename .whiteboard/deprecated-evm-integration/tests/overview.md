# Test Contracts Overview
> High-level test strategy for EVM integration.

Status: **PROPOSED** | Crate: `N/A`

## Strategy
This strategy defines how to verify the EVM integration from unit level to cross-crate flows. It ensures the new Application trait implementations meet the consensus requirements while correctly wrapping Reth/Alloy execution logic.

## Async Test Harness
All async tests use `#[tokio::test]` as the async runtime. The `tokio` dev-dependency must be added to both `app` and `app-evm` crate `Cargo.toml` files with `features = ["macros", "rt-multi-thread"]`.
## Test Levels

### Unit Tests
Focus on data structures and trait implementations in `app` and `app-evm`. These tests verify that blocks, results, and configs behave correctly in isolation.

### Integration Tests
Verify the interaction between crates and external components. This includes the `ApplicationAdapter` wrapping the `EvmApplication`, and the `WhirlpoolEvmConfig` correctly wiring up the `EthBlockExecutorFactory`.

### Cross-Crate Flows
Exercise the full lifecycle of a block from proposal to verification. These tests ensure that the state roots match across different execution steps and that errors propagate correctly through the adapter.

## Priorities
1. Correctness of the `consensus::Block` implementation for `EvmBlock`.
2. State root matching between `propose` and `verify` flows.
3. Proper mapping of execution errors to consensus verification errors.
4. Validation of the `ApplicationAdapter` forwarding logic.

<!-- continuation round 2 -->
5. State database correctness: `InMemoryStateDb` implements `Database` correctly, `commit()` applies diffs, `state_root()` is deterministic.
6. Clone isolation: cloned state databases produce independent snapshots.

## Grounded vs Proposed
- **Grounded**: Tests existing traits with `path::Symbol` citations (e.g., `crates/consensus/src/app.rs::ConsensusApp`).
- **[PROPOSED]**: Tests new implementations such as `EvmApplication` and `WhirlpoolEvmConfig`.

<!-- continuation round 2 -->
- **Grounded**: Tests `revm::Database` trait contract (method signatures from `revm` cargo dependency).
- **[PROPOSED]**: Tests `InMemoryStateDb`, `commit()`, `state_root()`, `with_genesis()` from `state` crate.
