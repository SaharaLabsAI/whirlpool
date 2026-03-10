# Architecture Context — EVM Block Production

## Scope and grounding
- Inputs synthesized from:
  - `docs/design/evm-block-production/.design-scratch/run-20260301-1330/shared-context.md`
  - `docs/design/evm-block-production/CRATES.md`
  - `docs/design/evm-block-production/domains/*.md`
  - `docs/design/evm-block-production/wiring/*.md`
  - `docs/design/evm-block-production/*/README.md`
- Source grounding traced from:
  - `crates/whirlpool-node/src/{lib.rs,main.rs,app.rs,block.rs,config.rs}`
  - `crates/app-evm/src/{lib.rs,config.rs,executor.rs,error.rs}`
  - `crates/app/src/{lib.rs,traits.rs,types.rs,adapter.rs,error.rs}`
  - `crates/state/src/{lib.rs,db.rs,error.rs}`
  - `crates/consensus/src/{lib.rs,app.rs,block.rs,engine.rs,event.rs}`
  - `crates/consensus-simplex/src/{lib.rs,config.rs,types.rs,adapter.rs,engine.rs,mailbox.rs,sink.rs,tests.rs}`

## Subsystem map candidates and ownership boundaries

### Candidate subsystem map
| Subsystem | Primary owner crate(s) | Boundary role | Key grounded entrypoints |
|---|---|---|---|
| Node bootstrap/runtime orchestration | `whirlpool-node` | Wires signer/network/consensus/app stack; process lifecycle | `crates/whirlpool-node/src/main.rs::main` |
| Consensus abstraction contracts | `consensus` | Defines engine/app/event contracts consumed by node + simplex | `crates/consensus/src/engine.rs::ConsensusEngine`, `crates/consensus/src/app.rs::ConsensusApp`, `crates/consensus/src/event.rs::EventSink` |
| Consensus implementation adapter/runtime | `consensus-simplex` | Implements `ConsensusEngine` and adapter/sink/mailbox glue for commonware simplex | `crates/consensus-simplex/src/engine.rs::CommonwareEngine`, `crates/consensus-simplex/src/adapter.rs::AppAdapter`, `crates/consensus-simplex/src/sink.rs::FinalizationSink` |
| App-layer contract and bridge | `app` | Defines `Application`, `TxSource`, `EvmBlock`, `ExecutionResult`; maps app to consensus callbacks | `crates/app/src/traits.rs::Application`, `crates/app/src/adapter.rs::ApplicationAdapter` |
| EVM execution domain | `app-evm` | Concrete `Application` implementation for genesis/propose/verify and EVM config wrapper | `crates/app-evm/src/executor.rs::EvmApplication`, `crates/app-evm/src/config.rs::WhirlpoolEvmConfig` |
| State domain | `state` | In-memory state, root derivation, bundle commit; revm DB traits | `crates/state/src/db.rs::InMemoryStateDb::{with_genesis,commit,state_root}` |
| Legacy empty-block path (parallel/minimal) | `whirlpool-node` | Stateless `EmptyBlock` + `EmptyBlockApp` path (not wired in node main) | `crates/whirlpool-node/src/app.rs::EmptyBlockApp`, `crates/whirlpool-node/src/block.rs::EmptyBlock` |

### Ownership boundaries (grounded)
- `whirlpool-node` owns composition, not execution semantics: it creates `EvmApplication`, wraps with `ApplicationAdapter`, and passes to `CommonwareEngine` (`crates/whirlpool-node/src/main.rs::main`).
- `app` owns interface semantics at consensus boundary: `ApplicationAdapter::propose` drops execution result and converts proposal errors to abstain (`None`), while `verify` maps to `ConsensusError::InvalidBlock` (`crates/app/src/adapter.rs::ApplicationAdapter::{propose,verify}`).
- `app-evm` owns current proposal/verify semantics, including current MVP empty block behavior and state-root-only verification (`crates/app-evm/src/executor.rs::EvmApplication::{propose,verify}`).
- `state` owns data model and deterministic hash implementation currently used as state root (non-MPT simplification), and supports bundle apply via `commit` (`crates/state/src/db.rs`).
- `consensus` owns trait contracts only; no finalize callback is present in `ConsensusApp` (`crates/consensus/src/app.rs::ConsensusApp`).
- `consensus-simplex` currently exposes adapter/mailbox/sink components, but `CommonwareEngine::start` contains explicit STUB simulation comments and spawns a simulated height loop (`crates/consensus-simplex/src/engine.rs::CommonwareEngine::start`).

## Candidate end-to-end flows and triggers

### Flow A: Node startup and engine bring-up (grounded)
**Trigger**: process start (`main`).
1. Initialize tracing and runtime runner (`crates/whirlpool-node/src/main.rs::main`).
2. Build signer + network provider (`CommonwareNetworkProviderBuilder`) (`crates/whirlpool-node/src/main.rs::main`).
3. Construct `CommonwareConfig` with timeout/buffer values (`crates/whirlpool-node/src/main.rs::main`, `crates/consensus-simplex/src/config.rs::CommonwareConfig`).
4. Construct app stack: `TestStateDb` + `WhirlpoolEvmConfig` + `NoopTxSource` + `EvmApplication` + `ApplicationAdapter` (`crates/whirlpool-node/src/main.rs::main`).
5. Start engine via `ConsensusEngine::start` (`crates/consensus/src/engine.rs::ConsensusEngine`, `crates/consensus-simplex/src/engine.rs::CommonwareEngine::start`).

### Flow B: Proposal callback path (grounded current behavior)
**Trigger**: consensus engine requests proposal (`ConsensusApp::propose`).
1. Engine calls adapter proposal callback (`crates/consensus/src/app.rs::ConsensusApp`).
2. `ApplicationAdapter::propose` calls inner `Application::propose` and converts `Ok((block,_))` -> `Some(block)`, `Err(_)` -> `None` (`crates/app/src/adapter.rs::ApplicationAdapter::propose`).
3. `EvmApplication::propose` reads state root and emits block with empty tx list and empty tx/receipt roots (`crates/app-evm/src/executor.rs::EvmApplication::propose`).
4. `ExecutionResult` is produced in app layer but dropped by adapter before consensus sees it (`crates/app/src/adapter.rs::ApplicationAdapter::propose`).

### Flow C: Verification callback path (grounded current behavior)
**Trigger**: consensus engine verifies candidate block (`ConsensusApp::verify`).
1. Engine calls adapter verify callback (`crates/consensus/src/app.rs::ConsensusApp`).
2. `ApplicationAdapter::verify` calls inner `Application::verify`; `Err` becomes `ConsensusError::InvalidBlock(err.to_string())` (`crates/app/src/adapter.rs::ApplicationAdapter::verify`).
3. `EvmApplication::verify` recomputes current state root from DB and compares to block state root only; returns `EvmAppError::StateRootMismatch` on mismatch (`crates/app-evm/src/executor.rs::EvmApplication::verify`).

### Flow D: Finalization signal path (partially grounded)
**Trigger**: simplex reports finalized block/event.
- `consensus::ConsensusEvent::Finalized` carries block/height/proof shape (`crates/consensus/src/event.rs::ConsensusEvent`).
- `consensus-simplex::FinalizationSink` writes finalized height to `AtomicU64` (`crates/consensus-simplex/src/sink.rs::FinalizationSink::handle`).
- Node wires `FinalizationSink` with shared height (`crates/whirlpool-node/src/main.rs::main`).
- `UNKNOWN/BLOCKER`: canonical finalize -> `state::InMemoryStateDb::commit` integration is not evidenced in this path.

### Flow E: Intended full EVM block production ([PROPOSED] + blockers)
**Trigger**: non-empty `TxSource` data available and consensus requests proposal/verify.
- [PROPOSED] decode tx bytes from `TxSource::pending()` and execute via EVM pipeline.
- [PROPOSED] derive non-empty `transactions_root`, `receipts_root`, `gas_used`, and post-state root from execution artifacts.
- [PROPOSED] verify by replay/recompute artifacts from parent state snapshot.
- BLOCKER: current `TxSource` is noop in runtime and current app logic is MVP-empty.

## Crate handoff contracts and error propagation paths

### Contract handoff map
| From | To | Contract seam | Current mapping behavior |
|---|---|---|---|
| `whirlpool-node` | `consensus-simplex` | `CommonwareEngine::new(...).start()` | Node passes app+sink+config+network and receives `RunningEngine` (`crates/whirlpool-node/src/main.rs::main`) |
| `consensus-simplex` | `consensus` | `impl ConsensusEngine for CommonwareEngine` | Returns `Result<RunningEngine, ConsensusError>` (`crates/consensus-simplex/src/engine.rs`) |
| `consensus` | `app` | `ConsensusApp` trait | Callbacks: `genesis/propose/verify` only (`crates/consensus/src/app.rs`) |
| `app` | `app-evm` | `Application` trait implementation | `EvmApplication<DB>` implements `Application<Block=EvmBlock,Result=ExecutionResult,Error=EvmAppError>` (`crates/app-evm/src/executor.rs`) |
| `app-evm` | `state` | `StateProvider` + `revm::Database` on DB type | Node `TestStateDb` delegates to `InMemoryStateDb` (`crates/whirlpool-node/src/main.rs`) |

### Error propagation paths (grounded)
1. `EvmApplication::verify` mismatch -> `EvmAppError::StateRootMismatch` (`crates/app-evm/src/executor.rs`).
2. Adapter verify maps any app error string to `ConsensusError::InvalidBlock` (`crates/app/src/adapter.rs::ApplicationAdapter::verify`).
3. `EvmAppError` can map into `ApplicationError` via `From<EvmAppError> for ApplicationError` (verification/state/execution mapping) (`crates/app-evm/src/error.rs`).
4. Proposal errors are swallowed at consensus boundary (`Err(_) => None`), losing cause detail (`crates/app/src/adapter.rs::ApplicationAdapter::propose`).
5. Engine/network startup failures map to `ConsensusError::Other` in simplex start path (`crates/consensus-simplex/src/engine.rs::CommonwareEngine::start`).

### Data-loss/observability boundaries
- Proposal path discards `ExecutionResult` before consensus (`crates/app/src/adapter.rs::ApplicationAdapter::propose`).
- Verification path reduces rich errors to `InvalidBlock(String)` for consensus (`crates/app/src/adapter.rs::ApplicationAdapter::verify`).
- [PROPOSED] richer typed errors across consensus/app seam may improve diagnostics but is not present in current contracts.

## Implementation slice candidates with acceptance hooks and test-first hooks

### Slice 1: Replace noop ingress with concrete transaction source
- Goal: introduce non-empty transaction ingress replacing runtime `NoopTxSource`.
- Touch points:
  - `crates/app/src/traits.rs::TxSource`
  - `crates/whirlpool-node/src/main.rs::main` (provider injection)
  - [PROPOSED] new tx source module in in-scope crate.
- Acceptance hooks:
  - Node wiring no longer constructs `Arc::new(NoopTxSource)`.
  - Proposal path sees non-zero `pending()` count under test harness.
- Test-first hooks:
  - Unit: `TxSource` implementation returns deterministic ordering contract ([PROPOSED] explicit policy).
  - Integration: proposal includes tx bytes in `EvmBlock.transactions` when source non-empty.

### Slice 2: Implement non-empty proposal execution in `EvmApplication::propose`
- Goal: execute tx list and derive all block/execution artifacts from execution results.
- Touch points:
  - `crates/app-evm/src/executor.rs::EvmApplication::propose`
  - `crates/app/src/types.rs::{EvmBlock,ExecutionResult}` (only if fields insufficient) [PROPOSED].
- Acceptance hooks:
  - `transactions_root` and `receipts_root` differ from `EMPTY_ROOT_HASH` when tx list non-empty.
  - `gas_used > 0` when executable txs are present.
  - `state_root` corresponds to post-execution state, not pre-read root.
- Test-first hooks:
  - Red test: non-empty `TxSource` still yields empty roots (current behavior).
  - Green test: non-empty proposal computes roots/gas/receipt_count deterministically.

### Slice 3: Strengthen verification integrity (`verify` replay/recompute)
- Goal: verify block by recomputing artifacts and rejecting tampered fields.
- Touch points:
  - `crates/app-evm/src/executor.rs::EvmApplication::verify`
  - `crates/app-evm/src/error.rs::EvmAppError` (add precise mismatch variants) [PROPOSED].
- Acceptance hooks:
  - Tampered `transactions_root`, `receipts_root`, or `gas_used` causes rejection.
  - Verify remains read-only on canonical state (no persistent mutation).
- Test-first hooks:
  - Mutation tests over candidate block fields.
  - Invariant test: state root and DB content unchanged after failed verify.

### Slice 4: Snapshot/rollback orchestration for proposal/verify safety
- Goal: ensure speculative execution does not leak into canonical state.
- Touch points:
  - `crates/state/src/db.rs::InMemoryStateDb` clone/commit usage.
  - `crates/app-evm/src/executor.rs` orchestration around proposal/verify paths.
- Acceptance hooks:
  - Failed proposal/verify leaves canonical DB unchanged.
  - Successful path exposes explicit bundle suitable for final commit ([PROPOSED] artifact type).
- Test-first hooks:
  - Clone-db before execution, inject failure, assert roots/storage unchanged.
  - Differential test between speculative and canonical DB instances.

### Slice 5: Finalize-to-commit integration seam
- Goal: connect finalized canonical block to `InMemoryStateDb::commit` atomically.
- Touch points:
  - `crates/consensus/src/app.rs::ConsensusApp` contract (may require extension) [PROPOSED].
  - `crates/consensus-simplex/src/{adapter.rs,sink.rs,engine.rs}` finalize report path.
  - `crates/whirlpool-node/src/main.rs` ownership wiring.
- Acceptance hooks:
  - On finalized block event, corresponding state bundle commit is invoked once.
  - Reorg/fault behavior defined as UNKNOWN until callback contract exists.
- Test-first hooks:
  - Engine-to-app integration test asserting commit invocation after finalization event.
  - Atomicity test: partial commit cannot be observed under failure injection.

### Slice 6: Improve boundary error fidelity
- Goal: reduce loss of failure detail at adapter boundary.
- Touch points:
  - `crates/app/src/adapter.rs::ApplicationAdapter::{propose,verify}`
  - `crates/consensus/src/error.rs::ConsensusError` [PROPOSED] richer variants.
- Acceptance hooks:
  - Proposal abstain reasons observable (metrics/log or typed error path) [PROPOSED].
  - Verify invalid block reasons remain structured and classifiable.
- Test-first hooks:
  - Unit tests asserting specific error mapping for each `EvmAppError` class.

## Explicit UNKNOWN candidates
- UNKNOWN: canonical tx decode/validation pipeline from `TxSource::pending() -> Vec<Vec<u8>>` into executable transaction primitives is not present in in-scope sources.
- UNKNOWN: deterministic ordering/pool policy for non-empty proposals is not encoded in `TxSource` contract (`crates/app/src/traits.rs::TxSource`).
- UNKNOWN: exact ownership boundary for finalized execution artifacts (where bundle is stored before commit) is not visible in current contracts.
- UNKNOWN: how `consensus-simplex::AppAdapter` mailbox/verify/propose logic is intended to replace/compose with current STUB `CommonwareEngine::start` path.
- UNKNOWN: production persistence backend plan for state (current state crate is in-memory only).

## Explicit BLOCKER candidates
- BLOCKER: proposal path is MVP-empty (`crates/app-evm/src/executor.rs::EvmApplication::propose` comment and behavior).
- BLOCKER: verification path checks only state root and does not replay tx artifacts (`crates/app-evm/src/executor.rs::EvmApplication::verify`).
- BLOCKER: runtime uses `NoopTxSource` (`crates/whirlpool-node/src/main.rs::main`, `crates/app/src/traits.rs::NoopTxSource`).
- BLOCKER: no grounded finalize callback in `ConsensusApp` contract; finalize->commit seam absent (`crates/consensus/src/app.rs::ConsensusApp`, `crates/state/src/db.rs::InMemoryStateDb::commit`).
- BLOCKER: `consensus-simplex` engine path is explicitly STUB/simulated in current `start`, limiting end-to-end grounding for real simplex execution (`crates/consensus-simplex/src/engine.rs::CommonwareEngine::start`).

## Notes on claim confidence
- Claims marked as grounded tie to explicit source paths/symbols above.
- Any behavior requiring new callbacks/new artifact types/new policy contracts is marked `UNKNOWN` or `[PROPOSED]`.
