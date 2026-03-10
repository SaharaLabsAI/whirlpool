# EVM Execution

## Definition
EVM Execution is the domain that implements `app::Application` semantics for Ethereum-style blocks by deriving genesis state metadata, proposing candidate blocks, and verifying candidate blocks against local state.
Grounded evidence:
- `crates/app-evm/src/executor.rs::EvmApplication` owns `genesis`, `propose`, and `verify` implementations.
- `crates/app-evm/src/executor.rs::StateProvider` defines the state-root read contract consumed by execution logic.

## Derived crates
- Primary crate: `app-evm` (`crates/app-evm`).
- Consumed crate contracts: `app` (`Application`, `TxSource`, `EvmBlock`, `ExecutionResult`), `state` (database implementations satisfying `StateProvider`), and reth/revm traits used by config and header conversion.
- Runtime consumer wiring enters from `whirlpool-node` via `EvmApplication::new(...)` in node bootstrap (see shared context grounding).

## Key public contracts
- `crates/app-evm/src/executor.rs::EvmApplication<DB>`
- `crates/app-evm/src/executor.rs::StateProvider`
- `crates/app-evm/src/config.rs::WhirlpoolEvmConfig`
- `crates/app-evm/src/config.rs::build_sahara_chain_spec`
- `crates/app-evm/src/lib.rs` re-exports: `WhirlpoolEvmConfig`, `build_sahara_chain_spec`, `SAHARA_CHAIN_ID`

## Core workflows
- Genesis derivation (`EvmApplication::genesis`): reads `db.state_root()` and returns block 0 with empty tx/receipt roots (`EMPTY_ROOT_HASH`) and zero gas usage.
- Proposal path (`EvmApplication::propose`): currently emits an empty block with `timestamp = parent.timestamp + 12`, `transactions = vec![]`, `gas_used = 0`, and roots derived from current DB + empty constants.
- Verification path (`EvmApplication::verify`): currently recomputes only `state_root` from current DB and rejects on mismatch via `EvmAppError::StateRootMismatch`; returns `ExecutionResult` without transaction replay.
- Header conversion helpers (`build_header_from_evm_block`, `build_sealed_header`) exist and are tested, but are not wired to a full execution/replay pipeline.

## Open questions / TODOs
- BLOCKER: `propose` has no transaction execution (`// MVP: Empty block execution (no transaction processing)` in `crates/app-evm/src/executor.rs::EvmApplication::propose`).
- BLOCKER: `verify` does not re-execute transactions and only checks state root (`crates/app-evm/src/executor.rs::EvmApplication::verify`).
- BLOCKER: Runtime tx ingress is effectively empty because node wiring uses `NoopTxSource`; non-empty execution artifacts cannot be produced.
- UNKNOWN: canonical tx decode/validation path from `TxSource::pending() -> Vec<Vec<u8>>` into executable transaction primitives is not present in in-scope evidence.
- UNKNOWN/BLOCKER: finalize-to-commit integration for canonical state application is not visible in consumed consensus app interface/wiring evidence.

INV constraints impacts (seed tests INV-01..INV-07):
- INV-01 Execution Visibility: BLOCKED by empty proposal path and noop tx source.
- INV-02 Verification Integrity: BLOCKED/PARTIAL because verification does not recompute tx/receipt roots or gas from replay.
- INV-03 Verification Read-Only: currently LIKELY SATISFIED for present code path (read lock + state_root read only).
- INV-04 Snapshot Safety: UNKNOWN/BLOCKER for full execution path; no explicit snapshot/rollback orchestration evidence in runtime wiring.
- INV-05 Commit Atomicity: BLOCKER; commit API exists in state crate but finalize callback/commit wiring is not evidenced in in-scope integration.
- INV-06 Root Consistency: BLOCKER for non-empty blocks; proposal currently hardcodes empty tx/receipt roots and zero gas.
- INV-07 Proposal Determinism: currently TRIVIALLY SATISFIED for empty-block semantics; UNKNOWN for real tx ordering/selection policy.
