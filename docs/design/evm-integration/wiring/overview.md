# Wiring

## Scope & method

This document maps how domain capabilities are wired between crates — which crate owns a capability, what trait interface enables swapping, and which types flow across boundaries.

Focus: the new `app` and `app-evm` crates and their integration with existing `consensus` and vendor `reth-evm` crates.

## Domain index

| Domain | Capabilities | File |
|---|---|---|
| Application | Block proposal, block verification, genesis, consensus-app bridging | wiring/application.md |
| EVM Execution | EVM env construction, block execution, block assembly, receipt building | wiring/evm-execution.md |

## Blockers

- **ChainSpec selection**: `WhirlpoolEvmConfig` needs a chain spec. Decision: reuse reth `ChainSpec` with custom chain ID, or define `SaharaChainSpec`. Affects `app-evm` construction and all `evm_env` / `next_evm_env` methods.
- **State DB generic**: `Executor<DB>` requires `Database` impl. `app-evm` must define its DB boundary. Currently out of scope but blocks full end-to-end wiring.
- **Transaction source**: `Application::propose()` needs pending transactions. Tx pool is out of scope but the interface must accommodate it.
