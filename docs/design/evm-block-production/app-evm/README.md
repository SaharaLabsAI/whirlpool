# app-evm

## Purpose

Concrete implementation of the `Application` trait for EVM execution. This crate provides the `EvmApplication` struct that drives block proposal, verification, and genesis logic for the Sahara chain, along with chain specification and EVM configuration wrappers. It is the sole owner of the EVM execution domain. <!-- GROUNDED -->

## Public API at a glance (crate root exports)
- `executor::EvmApplication`: Main application logic for block proposal and verification. <!-- GROUNDED -->
- `executor::StateProvider`: Trait for accessing current state root. <!-- GROUNDED -->
- `config::WhirlpoolEvmConfig`: EVM configuration wrapper. <!-- GROUNDED -->
- `config::build_sahara_chain_spec`: Helper to build the Sahara chain specification. <!-- GROUNDED -->
- `config::SAHARA_CHAIN_ID`: The unique identifier for the Sahara chain (313,371). <!-- GROUNDED -->
- `error::EvmAppError`: Crate-specific error types. <!-- GROUNDED -->

## Modules
- `config`: Chain specification and EVM configuration. <!-- GROUNDED -->
- `executor`: Logic for genesis, proposal, and verification. <!-- GROUNDED -->
- `error`: Error definitions and conversions. <!-- GROUNDED -->

## Types & traits (public contract)
- `StateProvider`: Trait requiring `fn state_root(&self) -> B256`. <!-- GROUNDED -->
- `EvmApplication<DB>`: Struct holding `evm_config`, `state_db: Arc<RwLock<DB>>`, and `tx_source`. <!-- GROUNDED -->
- `WhirlpoolEvmConfig`: Wraps `EthEvmConfig` and implements `ConfigureEvm`. <!-- GROUNDED -->
- `EvmAppError`: Enum covering `Execution`, `StateRootMismatch`, `State`, and `InvalidBlock`. <!-- GROUNDED -->

## Functions & macros
- `build_sahara_chain_spec()`: Returns a `ChainSpec` with Cancun activated and 30M gas limit. <!-- GROUNDED -->
- `build_header_from_evm_block()`: Internal helper for header conversion. <!-- GROUNDED -->

## Config schema

- `SAHARA_CHAIN_ID`: `u64` constant defining the Sahara chain identifier. <!-- GROUNDED -->
- `ChainSpec`: Built via `build_sahara_chain_spec()`, containing chain ID, gas limit, difficulty, and hardfork activations. Not runtime-configurable. <!-- GROUNDED -->

## Config defaults table
| Field | Type | Default | Source | Override path | Evidence |
|---|---|---|---|---|---|
| `chain_id` | `u64` | `313,371` | `SAHARA_CHAIN_ID` | N/A | `config.rs` <!-- GROUNDED --> |
| `gas_limit` | `u64` | `30_000_000` | `build_sahara_chain_spec` | N/A | `config.rs` <!-- GROUNDED --> |
| `difficulty` | `U256` | `0` | `build_sahara_chain_spec` | N/A | `config.rs` <!-- GROUNDED --> |
| `hardforks` | `Cancun` | `Activated` | `build_sahara_chain_spec` | N/A | `config.rs` <!-- GROUNDED --> |

## Provider interfaces & swap points
- `TxSource`: Provided by `app` crate, held by `EvmApplication` but currently unused in the MVP empty-block path. <!-- PROPOSED -->
- `StateProvider`: Implemented by the database (e.g., `TestStateDb`) to provide state roots. <!-- GROUNDED -->
- `ConfigureEvm`: Implemented by `WhirlpoolEvmConfig` to define EVM behavior (reth/revm). <!-- GROUNDED -->

## Feature flags & cfg
- None currently specified. <!-- GROUNDED -->

## SemVer & stability
- Internal crate for development; API stability is not guaranteed. <!-- PROPOSED -->

## Primary flows

### Genesis
- Acquires a read lock on `state_db`. <!-- GROUNDED -->
- Reads `state_root()` from the `StateProvider`. <!-- GROUNDED -->
- Returns block 0 with `state_root` from `db.state_root()` and `EMPTY_ROOT_HASH` for `transactions_root` and `receipts_root`. <!-- GROUNDED -->
### Block Proposal (MVP Stub)
- Increments timestamp by 12 seconds from parent. <!-- GROUNDED -->
- **BLOCKER**: Produces empty blocks only; no transaction execution (INV-01). <!-- GROUNDED -->
- **BLOCKER**: Roots are hardcoded to `EMPTY_ROOT_HASH` (INV-06). <!-- GROUNDED -->
- Satisfied for empty blocks (INV-07). <!-- GROUNDED -->

### Block Verification
- Reads `state_root` from the provider and compares it to `block.state_root`. <!-- GROUNDED -->
- Returns `StateRootMismatch` if they do not match. <!-- GROUNDED -->
- **BLOCKER**: Does not replay transactions, receipts, or gas (INV-02). <!-- GROUNDED -->
- Uses read lock only (INV-03). <!-- GROUNDED -->

## API omissions report
- No mechanism for transaction ordering policy (INV-07 for non-empty blocks). <!-- PROPOSED -->
- Missing integration for actual transaction execution in `propose` (INV-01). <!-- GROUNDED -->
- No snapshot/rollback orchestration visible in this crate (INV-04). <!-- UNKNOWN -->
- `EvmAppError::InvalidBlock` variant exists but is never constructed in current code. <!-- GROUNDED -->
## Open questions / TODOs

- **BLOCKER (INV-01)**: `propose()` must execute transactions to produce non-empty blocks. <!-- PROPOSED -->
- **BLOCKER (INV-02)**: `verify()` must replay transactions and validate receipts/gas, not just state root. <!-- PROPOSED -->
- **BLOCKER (INV-06)**: Roots must be derived from actual execution, not hardcoded `EMPTY_ROOT_HASH`. <!-- PROPOSED -->
- **BLOCKER (INV-04)**: Snapshot Safety — no explicit snapshot/rollback orchestration in propose/verify. <!-- UNKNOWN -->
- **BLOCKER (INV-05)**: Commit Atomicity — no finalize trigger exists in the application layer. <!-- PROPOSED -->
- **UNKNOWN (INV-07)**: Deterministic transaction ordering for non-empty proposals is not yet defined. <!-- PROPOSED -->
