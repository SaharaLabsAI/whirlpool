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

- ~~**ChainSpec selection**~~: Resolved (round 3). `build_sahara_chain_spec()` in `app-evm::config` constructs `ChainSpec` with chain ID `313_371`, all hardforks through Cancun at genesis, empty allocation. See `app-evm/README.md`. <!-- continuation round 3: B-001 resolved -->
- ~~**State DB generic**~~: Resolved in round 2 — `InMemoryStateDb` from `state` crate satisfies `Database + Clone`. See `wiring/state-storage.md`.
- **Transaction source**: `Application::propose()` needs pending transactions. Tx pool is out of scope but the interface must accommodate it.

<!-- continuation round 2 -->
## State Storage wiring (B-002)

The `state` crate provides the concrete `InMemoryStateDb` which satisfies the `Database` requirements of the EVM execution pipeline. It handles the transition from execution outputs (BundleState) to persistent state roots.
