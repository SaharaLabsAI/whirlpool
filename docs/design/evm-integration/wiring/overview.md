# Wiring

## Scope & method

This document maps how domain capabilities are wired between crates — which crate owns a capability, what trait interface enables swapping, and which types flow across boundaries.

Focus: the new `app` and `app-evm` crates and their integration with existing `consensus` and vendor `reth-evm` crates.

## Domain index

| Domain | Capabilities | File |
|---|---|---|
| Application | Block proposal, block verification, genesis, consensus-app bridging | wiring/application.md |
| EVM Execution | EVM env construction, block execution, block assembly, receipt building | wiring/evm-execution.md |
| State Storage | REVM database, bundle state commitment, state root computation, snapshots | wiring/state-storage.md |

## Blockers

- **ChainSpec selection**: `WhirlpoolEvmConfig` needs a chain spec. Decision: reuse reth `ChainSpec` with custom chain ID, or define `SaharaChainSpec`. Affects `app-evm` construction and all `evm_env` / `next_evm_env` methods.
- ~~**State DB generic**~~: Resolved in round 2 — `InMemoryStateDb` from `state` crate satisfies `Database + Clone`. See `wiring/state-storage.md`.
- **Transaction source**: `Application::propose()` needs pending transactions. Tx pool is out of scope but the interface must accommodate it.

<!-- continuation round 2 -->
## State Storage wiring (B-002)

The `state` crate provides the concrete `InMemoryStateDb` which satisfies the `Database` requirements of the EVM execution pipeline. It handles the transition from execution outputs (BundleState) to persistent state roots.
