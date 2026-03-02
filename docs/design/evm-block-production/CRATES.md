# Crate Index — EVM Block Production

## In-scope crates

| Crate | Path | Purpose | Provides | Depends on (workspace) | Used by | Status |
|---|---|---|---|---|---|---|
| `whirlpool-node` | `crates/whirlpool-node` | Primary node binary and orchestration | `main` (whirlpool-node/src/main.rs), `config::VALIDATOR_SEED` | `app`, `app-evm`, `state`, `consensus`, `consensus-simplex`, `p2p`, `p2p-commonware` | — | Grounded (needs enhancement) |
| `app-evm` | `crates/app-evm` | EVM execution engine and block production logic | `EvmApplication` (app-evm/src/executor.rs), `WhirlpoolEvmConfig` (app-evm/src/config.rs) | `app`, `state`, `consensus` | `whirlpool-node` | Grounded (needs enhancement) |
| `app` | `crates/app` | Abstract application and transaction interfaces | `Application` (app/src/traits.rs), `TxSource` (app/src/traits.rs), `EvmBlock` (app/src/types.rs), `ApplicationAdapter` (app/src/adapter.rs) | `consensus` | `whirlpool-node`, `app-evm` | Grounded (needs enhancement) |
| `state` | `crates/state` | In-memory EVM state management | `InMemoryStateDb` (state/src/db.rs), `DbAccount` (state/src/db.rs) | — | `whirlpool-node`, `app-evm` | Grounded (exists) |

## Out-of-scope crates (referenced)

| Crate | Path | Role in this design |
|---|---|---|
| `consensus` | `crates/consensus` | Defines the `ConsensusApp` and `Block` traits used for engine-to-app communication. |
| `consensus-simplex` | `crates/consensus-simplex` | Provides the `CommonwareEngine` implementation that orchestrates the consensus process. |
| `p2p` | `crates/p2p` | Abstract peer-to-peer networking traits. |
| `p2p-commonware` | `crates/p2p-commonware` | Concrete P2P implementation using Commonware primitives. |

## Vendor crates (key external dependencies)

| Crate | Version | Used by | Role |
|---|---|---|---|
| `revm` | 34 | `state`, `app-evm`, `whirlpool-node` | The core Ethereum Virtual Machine implementation. |
| `reth-evm` | local | `app-evm` | Reth's EVM abstraction layer for transaction execution. |
| `reth-evm-ethereum` | local | `app-evm` | Ethereum-specific EVM configuration from Reth. |
| `commonware-consensus` | local | `whirlpool-node`, `app` | Consensus primitives and traits from Commonware. |
| `alloy-primitives` | 1.5.0 | `app-evm`, `whirlpool-node` | Fixed-size types for Ethereum (hashes, addresses). |
| `alloy-consensus` | 1.4.3 | `app-evm` | Ethereum consensus-related types (headers, transactions). |
