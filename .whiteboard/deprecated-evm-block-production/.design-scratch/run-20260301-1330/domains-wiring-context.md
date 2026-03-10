# Domains & Wiring Context — EVM Block Production

## Scope and grounding
- Inputs consumed: `docs/design/evm-block-production/.design-scratch/run-20260301-1330/shared-context.md`, `docs/design/evm-block-production/CRATES.md`, and seed-test docs under `docs/design/evm-block-production/tests/`.
- Source evidence limited to in-scope crates plus consumed consensus interfaces:
  - `crates/whirlpool-node/src/`
  - `crates/app-evm/src/`
  - `crates/app/src/`
  - `crates/state/src/`
  - `crates/consensus/src/` (consumed interfaces)
- Claims below are grounded with `path::symbol` and/or quoted code snippets.

## Candidate domain list with evidence

| Candidate domain | Why this is a domain | Grounding evidence |
|---|---|---|
| Node orchestration & runtime wiring | Bootstraps runtime, network, consensus engine, and app wiring. | `crates/whirlpool-node/src/main.rs::main` wires `CommonwareNetworkProviderBuilder`, `CommonwareEngine::new(...)`, and `ApplicationAdapter::new(evm_app)`. |
| Consensus-facing application bridge | Converts app trait outcomes to consensus trait contract (`Option<Block>` / `Result<(), ConsensusError>`). | `crates/app/src/adapter.rs::ApplicationAdapter` implements `consensus::ConsensusApp`; `propose` maps `Err(_) => None`, `verify` maps app error to `ConsensusError::InvalidBlock(...)`. |
| Application contract & block model | Defines abstract lifecycle (`genesis/propose/verify`), tx ingress seam, and block/result data contracts. | `crates/app/src/traits.rs::Application`; `crates/app/src/traits.rs::TxSource`; `crates/app/src/types.rs::{EvmBlock,ExecutionResult}`. |
| EVM execution pipeline | Intended location for tx execution + block/verification logic, currently MVP-empty behavior. | `crates/app-evm/src/executor.rs::EvmApplication` and comments in `propose`: `// MVP: Empty block execution (no transaction processing)`. |
| State management & commitment substrate | In-memory account/storage/code DB with commit and root computation used by execution and node. | `crates/state/src/db.rs::InMemoryStateDb::{commit,state_root}` and `impl Database/DatabaseRef`. |
| Consensus interface boundary (consumed) | Defines exact engine↔app contract consumed by adapter and node wiring. | `crates/consensus/src/app.rs::ConsensusApp`, `crates/consensus/src/block.rs::Block`, `crates/consensus/src/engine.rs::ConsensusEngine`. |

## Domain-to-crate mapping

| Domain | Primary crate(s) | Secondary/consumed crate(s) | Evidence |
|---|---|---|---|
| Node orchestration & runtime wiring | `whirlpool-node` | `consensus-simplex` (engine impl), `consensus` (trait), `app`, `app-evm`, `state` | `crates/whirlpool-node/src/main.rs::main` |
| Consensus-facing application bridge | `app` | `consensus` | `crates/app/src/adapter.rs::ApplicationAdapter` + `crates/consensus/src/app.rs::ConsensusApp` |
| Application contract & block model | `app` | `consensus` | `crates/app/src/traits.rs`, `crates/app/src/types.rs` |
| EVM execution pipeline | `app-evm` | `app`, `state`, `revm/reth` abstractions | `crates/app-evm/src/executor.rs::EvmApplication`, `crates/app-evm/src/config.rs::WhirlpoolEvmConfig` |
| State management & commitment substrate | `state` | consumed by `app-evm`, `whirlpool-node` | `crates/state/src/db.rs::InMemoryStateDb` |
| Consensus interface boundary (consumed) | `consensus` | consumed by `app`, `whirlpool-node` | `crates/consensus/src/{app.rs,block.rs,engine.rs}` |

## Capability-level wiring skeleton per domain

### 1) Node orchestration & runtime wiring

| Capability | Provider | Consumer | Interface/type | Current wiring status | Evidence |
|---|---|---|---|---|---|
| Runtime bootstrap | `whirlpool-node::main` | whole node | `commonware_runtime::Runner` | WIRED | `crates/whirlpool-node/src/main.rs::main` |
| Network provider build | `CommonwareNetworkProviderBuilder` setup in node | `CommonwareEngine` | builder output consumed by engine | WIRED | `crates/whirlpool-node/src/main.rs::main` |
| Consensus engine startup | `CommonwareEngine::new(...).start()` | node process | `consensus::ConsensusEngine` trait contract | WIRED | `crates/whirlpool-node/src/main.rs::main`, `crates/consensus/src/engine.rs::ConsensusEngine` |
| App injection into consensus | `ApplicationAdapter<EvmApplication<_>>` | consensus engine | `consensus::ConsensusApp` | WIRED | `crates/whirlpool-node/src/main.rs::main`, `crates/app/src/adapter.rs::ApplicationAdapter` |
| Tx source injection | `Arc::new(NoopTxSource)` | `EvmApplication` | `app::TxSource` | WIRED but empty semantics | `crates/whirlpool-node/src/main.rs::main`, `crates/app/src/traits.rs::NoopTxSource` |

### 2) Consensus-facing application bridge

| Capability | Provider | Consumer | Interface/type | Current wiring status | Evidence |
|---|---|---|---|---|---|
| Genesis pass-through | `ApplicationAdapter::genesis` | consensus engine | `ConsensusApp::genesis` | WIRED | `crates/app/src/adapter.rs::ApplicationAdapter::genesis` |
| Proposal adaptation | `ApplicationAdapter::propose` | consensus engine | `ConsensusApp::propose -> Option<Block>` | WIRED, error details dropped | `crates/app/src/adapter.rs::ApplicationAdapter::propose` (`Err(_) => None`) |
| Verification adaptation | `ApplicationAdapter::verify` | consensus engine | `ConsensusApp::verify -> Result<(), ConsensusError>` | WIRED | `crates/app/src/adapter.rs::ApplicationAdapter::verify` |
| Error surface shaping | Adapter error mapping | consensus engine | `ConsensusError::InvalidBlock(String)` | WIRED, lossy (string conversion) | `crates/app/src/adapter.rs::ApplicationAdapter::verify` |

### 3) Application contract & block model

| Capability | Provider | Consumer | Interface/type | Current wiring status | Evidence |
|---|---|---|---|---|---|
| App lifecycle contract | `app::Application` trait | `app-evm` impl + adapter | `genesis/propose/verify` futures | WIRED | `crates/app/src/traits.rs::Application` |
| Tx ingress contract | `app::TxSource` trait | `EvmApplication` field | `pending() -> Vec<Vec<u8>>` | CONTRACT EXISTS; runtime impl is noop | `crates/app/src/traits.rs::TxSource`, `crates/app-evm/src/executor.rs::EvmApplication` |
| Block data model | `app::EvmBlock` | adapter, app-evm, consensus | roots/gas/timestamp/transactions fields | WIRED | `crates/app/src/types.rs::EvmBlock` |
| Execution result model | `app::ExecutionResult` | app-evm + adapter tests | `state_root/receipts_root/gas_used/receipt_count` | WIRED | `crates/app/src/types.rs::ExecutionResult` |

### 4) EVM execution pipeline

| Capability | Provider | Consumer | Interface/type | Current wiring status | Evidence |
|---|---|---|---|---|---|
| Genesis block derivation from state | `EvmApplication::genesis` | consensus/app bridge | reads DB `state_root`, builds block | WIRED | `crates/app-evm/src/executor.rs::EvmApplication::genesis` |
| Proposal execution | `EvmApplication::propose` | adapter/consensus | returns `(EvmBlock, ExecutionResult)` | BLOCKED (empty block only) | `crates/app-evm/src/executor.rs::EvmApplication::propose` + quote `MVP: Empty block execution (no transaction processing)` |
| Verification execution | `EvmApplication::verify` | adapter/consensus | returns `ExecutionResult` | PARTIAL/BLOCKED (state_root check only) | `crates/app-evm/src/executor.rs::EvmApplication::verify` |
| Header conversion helpers | internal fns | future reth execution path | `build_header_from_evm_block`, `build_sealed_header` | PRESENT but not wired to tx execution | `crates/app-evm/src/executor.rs::{build_header_from_evm_block,build_sealed_header}` |
| EVM config abstraction | `WhirlpoolEvmConfig` | `EvmApplication` field | `reth_evm::ConfigureEvm` | PRESENT, not consumed in propose/verify path | `crates/app-evm/src/config.rs::WhirlpoolEvmConfig`, `crates/app-evm/src/executor.rs::EvmApplication` |

### 5) State management & commitment substrate

| Capability | Provider | Consumer | Interface/type | Current wiring status | Evidence |
|---|---|---|---|---|---|
| Canonical state DB | `InMemoryStateDb` | node, app-evm | `revm::Database`/`DatabaseRef` | WIRED | `crates/state/src/db.rs::InMemoryStateDb` |
| State application | `InMemoryStateDb::commit` | intended finalize flow | apply `BundleState` | IMPLEMENTED in crate | `crates/state/src/db.rs::InMemoryStateDb::commit` |
| State root derivation | `InMemoryStateDb::state_root` | genesis/propose/verify | `B256` root | WIRED (simplified hash algorithm) | `crates/state/src/db.rs::InMemoryStateDb::state_root` |
| Node-local DB wrapper | `TestStateDb(InMemoryStateDb)` | `EvmApplication` in node | `StateProvider` + `revm::Database` impls | WIRED | `crates/whirlpool-node/src/main.rs::{TestStateDb,impl StateProvider for TestStateDb}` |

### 6) Consensus interface boundary (consumed)

| Capability | Provider | Consumer | Interface/type | Current wiring status | Evidence |
|---|---|---|---|---|---|
| Engine-facing app callbacks | `consensus::ConsensusApp` | adapter and engine impls | `genesis/propose/verify` | CONSUMED | `crates/consensus/src/app.rs::ConsensusApp` |
| Block identity/lineage contract | `consensus::Block` | `app::EvmBlock` impl | `id/parent_id/height` | WIRED | `crates/consensus/src/block.rs::Block`, `crates/app/src/types.rs::impl CoreBlock for EvmBlock` |
| Engine run handle contract | `consensus::ConsensusEngine` | `CommonwareEngine` in node | `start() -> RunningEngine` | CONSUMED | `crates/consensus/src/engine.rs::ConsensusEngine`, `crates/whirlpool-node/src/main.rs::main` |

## Explicit unknowns and blocker candidates

### Confirmed blockers from code
1. **No transaction execution in proposal path**
   - Evidence: `crates/app-evm/src/executor.rs::EvmApplication::propose` comment and construction of `transactions: vec![]`, `gas_used: 0`, roots set to `EMPTY_ROOT_HASH`.
2. **No transaction re-execution in verify path**
   - Evidence: `crates/app-evm/src/executor.rs::EvmApplication::verify` only compares current DB root vs `block.state_root`; does not iterate `block.transactions`.
3. **No non-noop tx source in runtime wiring**
   - Evidence: `crates/whirlpool-node/src/main.rs::main` uses `let tx_source = Arc::new(NoopTxSource);`; `NoopTxSource::pending()` returns empty vec in `crates/app/src/traits.rs::NoopTxSource`.
4. **No finalize/commit integration visible in in-scope wiring**
   - Evidence: `consensus::ConsensusApp` in `crates/consensus/src/app.rs` exposes only `genesis/propose/verify`; no finalize callback. `whirlpool-node/src/main.rs` wires `FinalizationSink` height tracking but no call to `InMemoryStateDb::commit` is present in in-scope code.

### Explicit UNKNOWN items (grounded absence)
1. **UNKNOWN: canonical transaction decoding/type pipeline**
   - Grounding: `TxSource` contract is raw `Vec<Vec<u8>>` (`crates/app/src/traits.rs::TxSource`); no in-scope symbol defines decode/validation pipeline from bytes into executable tx type.
2. **UNKNOWN: verification scope for header fields beyond state_root**
   - Grounding: current `verify` checks only state root in `crates/app-evm/src/executor.rs::EvmApplication::verify`; expected checks for `transactions_root`, `receipts_root`, and `gas_used` are not present.
3. **UNKNOWN: snapshot/rollback mechanism for failed propose/verify in production path**
   - Grounding: `InMemoryStateDb` has `Clone` and `commit` APIs (`crates/state/src/db.rs`), but there is no explicit runtime snapshot manager type or rollback call in `whirlpool-node` / `app-evm` execution path.
4. **UNKNOWN: ordering and block-gas policy for tx selection**
   - Grounding: `TxSource::pending()` has no ordering/limit semantics in `crates/app/src/traits.rs`; no policy type found in in-scope crates.
5. **BLOCKER candidate: `evm_config` and `tx_source` are not used by execution logic today**
   - Grounding: fields exist on `EvmApplication` (`crates/app-evm/src/executor.rs::EvmApplication`), but `propose/verify` bodies do not invoke config-driven execution nor `tx_source.pending()`.

## Seed-test implications using INV-01..INV-07

Source constraints: `docs/design/evm-block-production/tests/overview.md` defines INV-01..INV-07 as proposed invariants.

| Invariant | Constraint (seed test) | Current code evidence | Implication for wiring/tests |
|---|---|---|---|
| INV-01 | Execution Visibility: non-empty tx input must yield visible execution artifacts. | `NoopTxSource` always empty (`crates/app/src/traits.rs::NoopTxSource`); `EvmApplication::propose` emits empty tx list (`crates/app-evm/src/executor.rs::EvmApplication::propose`). | **Currently unachievable** in wired runtime; happy-path non-empty flow in `tests/cross-crate-flows.md` is blocked. |
| INV-02 | Verification Integrity: `verify()` recomputes artifacts and rejects mismatches. | `verify` only checks `state_root` equality with current DB root (`crates/app-evm/src/executor.rs::EvmApplication::verify`). | **Blocked/partial**: tampered `transactions_root/gas_used/receipts_root` may not be detected by current logic. |
| INV-03 | Verification Read-Only: verify must not mutate canonical state. | `verify` takes read lock and calls only `state_root()` on DB (`crates/app-evm/src/executor.rs::EvmApplication::verify`). | **Likely satisfied for current implementation**, but only for current minimal check path; future re-exec path needs explicit safeguards. |
| INV-04 | Snapshot Safety: failed propose/verify leaves state byte-identical. | No explicit snapshot/rollback API in runtime path; `InMemoryStateDb` supports clone/commit (`crates/state/src/db.rs`). | **UNKNOWN/BLOCKER candidate** for full execution path; seed rollback tests need concrete snapshot orchestration seam. |
| INV-05 | Commit Atomicity: finalize applies all effects exactly once. | `InMemoryStateDb::commit` exists (`crates/state/src/db.rs::InMemoryStateDb::commit`), but no finalize callback in `ConsensusApp` (`crates/consensus/src/app.rs`) and no commit call in `whirlpool-node::main`. | **Blocked in current cross-crate wiring**: finalize-to-commit path is not visible in in-scope integration. |
| INV-06 | Root Consistency: roots derived from actual tx execution. | Proposal hardcodes empty roots and no tx execution (`crates/app-evm/src/executor.rs::EvmApplication::propose`). | **Blocked** for non-empty blocks; currently only trivial empty-root consistency. |
| INV-07 | Proposal Determinism with same state + tx source responses. | Current `propose` deterministic for empty block (`parent.timestamp + 12`, no txs) (`crates/app-evm/src/executor.rs::EvmApplication::propose`). | **Trivially satisfied now**, but not yet validated for real tx execution ordering/selection (policy unknown). |

## Practical wiring takeaways (grounded)
- Existing end-to-end runtime wiring is structurally in place (`whirlpool-node` -> `ApplicationAdapter` -> `EvmApplication` -> `state`), but execution is still MVP-empty.
- The strongest integration gap for intent fulfillment is not trait availability; it is missing behavior inside `EvmApplication::propose/verify` plus missing observable finalize->commit wiring in in-scope code.
- Seed tests already encode the right invariants (INV-01..INV-07), but INV-01/02/05/06 are currently blocked by concrete code paths above; INV-03/07 are only partially/trivially satisfied under empty-block semantics.
