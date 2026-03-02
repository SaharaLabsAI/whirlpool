# Block Production

## Definition
Block Production is the runtime orchestration domain that wires node startup, consensus engine lifecycle, and consensus-to-application callbacks for EVM block proposal/verification in `whirlpool-node`.

Grounded scope evidence:
- Node runtime + engine bootstrap is assembled in `crates/whirlpool-node/src/main.rs::main`.
- Consensus callbacks are constrained to `genesis/propose/verify` by `crates/consensus/src/app.rs::ConsensusApp`.
- Engine startup contract is `crates/consensus/src/engine.rs::ConsensusEngine::start`.

INV constraint impact in current domain scope:
- INV-01 (Execution Visibility): BLOCKER (assessment; runtime wiring uses `NoopTxSource` in `crates/whirlpool-node/src/main.rs::main`, which blocks non-empty execution visibility in current flow).
- INV-02 (Verification Integrity): BLOCKER/PARTIAL (assessment from grounded app-path evidence in `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md`, which cites state-root-oriented verify behavior in `crates/app-evm/src/executor.rs::EvmApplication::verify`).
- INV-03 (Verification Read-Only): currently likely satisfied for existing minimal verify path (assessment from `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md` evidence on read-only/root-check verify behavior).
- INV-04 (Snapshot Safety): UNKNOWN/BLOCKER (assessment; snapshot/rollback seam is not explicit in in-scope wiring, and this unknown is a blocker for intended non-empty execution safety in current architecture).
- INV-05 (Commit Atomicity): BLOCKER (assessment; consumed callback surface is limited to `genesis/propose/verify` in `crates/consensus/src/app.rs::ConsensusApp`, and no finalize->commit call is visible in `crates/whirlpool-node/src/main.rs::main`).
- INV-06 (Root Consistency): BLOCKER for non-empty transactions (assessment from grounded proposal-path evidence in `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md`, which cites MVP-empty proposal behavior).
- INV-07 (Proposal Determinism): trivially satisfied for current empty proposal semantics; UNKNOWN for non-empty ordering policy (assessment based on current runtime wiring in `crates/whirlpool-node/src/main.rs::main` and tx-policy gaps captured in `docs/design/evm-block-production/.design-scratch/run-20260301-1330/domains-wiring-context.md`).

## Derived crates
| Crate | Role in this domain | Evidence |
|---|---|---|
| `whirlpool-node` | Primary runtime orchestration and wiring point. | `crates/whirlpool-node/src/main.rs::main` |
| `app` | Provides adapter that satisfies consensus app boundary and maps app errors. | `crates/app/src/adapter.rs::ApplicationAdapter` |
| `app-evm` | Provides concrete app implementation injected into adapter. | `crates/app-evm/src/executor.rs::EvmApplication` |
| `state` | Provides backing state DB/root used by app and node wrapper. | `crates/state/src/db.rs::InMemoryStateDb` |
| `consensus` (consumed) | Defines consumed `ConsensusApp` and `ConsensusEngine` contracts. | `crates/consensus/src/app.rs::ConsensusApp`, `crates/consensus/src/engine.rs::ConsensusEngine` |
| `consensus-simplex` (consumed) | Provides `CommonwareEngine` and `FinalizationSink` used by node startup path. | `crates/whirlpool-node/src/main.rs::main` |

## Key public contracts
| Contract | Why it matters for Block Production wiring | Evidence |
|---|---|---|
| `consensus::ConsensusApp` | Defines only `genesis/propose/verify`; this bounds what node can wire into consensus callbacks. | `crates/consensus/src/app.rs::ConsensusApp` |
| `consensus::ConsensusEngine::start` | Node uses this to transition from configured engine to running handle. | `crates/consensus/src/engine.rs::ConsensusEngine::start` |
| `app::ApplicationAdapter` | Bridges `app::Application` to `ConsensusApp`, including proposal/verify error shaping. | `crates/app/src/adapter.rs::ApplicationAdapter` |
| `app::TxSource` | Defines tx ingress seam (`pending() -> Vec<Vec<u8>>`) used by app wiring. | `crates/app/src/traits.rs::TxSource` |
| `app_evm::executor::EvmApplication` | Concrete application instance injected into adapter by node runtime. | `crates/whirlpool-node/src/main.rs::main`, `crates/app-evm/src/executor.rs::EvmApplication` |
| `app_evm::executor::StateProvider` | Node-local DB wrapper implements state-root access consumed by EVM app. | `crates/whirlpool-node/src/main.rs::impl StateProvider for TestStateDb` |

## Core workflows
Current-vs-intended note: non-empty execution and finalize->commit persistence are intended/proposed behavior, while the currently grounded runtime remains MVP-empty (`NoopTxSource` in `crates/whirlpool-node/src/main.rs::main`) and lacks a visible finalize->commit seam in in-scope wiring (`crates/consensus/src/app.rs::ConsensusApp`, `crates/whirlpool-node/src/main.rs::main`).
1) Runtime bootstrap and dependency wiring
- Node initializes runtime and network provider, builds `CommonwareConfig`, constructs app stack (`TestStateDb` + `WhirlpoolEvmConfig` + `NoopTxSource` + `EvmApplication` + `ApplicationAdapter`), then starts `CommonwareEngine` in `crates/whirlpool-node/src/main.rs::main`.

2) Proposal path (consensus -> adapter -> app)
- Consensus invokes `ConsensusApp::propose` (contract in `crates/consensus/src/app.rs::ConsensusApp`).
- Adapter forwards to app and maps app proposal failure to abstain (`Err(_) => None`) in `crates/app/src/adapter.rs::ApplicationAdapter::propose`.
- Current runtime tx source is `NoopTxSource` (`crates/whirlpool-node/src/main.rs::main`), so INV-01 and INV-06 are blocked for non-empty execution semantics.

3) Verification path (consensus -> adapter -> app)
- Consensus invokes `ConsensusApp::verify` and expects `Result<(), ConsensusError>`.
- Adapter maps app errors into `ConsensusError::InvalidBlock(String)` in `crates/app/src/adapter.rs::ApplicationAdapter::verify`.
- Current verify behavior is root-check-oriented in the app stack (from shared context evidence), leaving INV-02 partial for artifact-level tampering detection.

4) Finalization visibility and state commitment boundary
- Node wires `FinalizationSink` height tracking in `crates/whirlpool-node/src/main.rs::main`.
- `InMemoryStateDb::commit` exists (`crates/state/src/db.rs::InMemoryStateDb::commit`), but finalize-to-commit integration is not visible in in-scope node/consensus callback contract (BLOCKER for INV-05).

## Open questions / TODOs
- BLOCKER: Replace runtime `NoopTxSource` wiring in `crates/whirlpool-node/src/main.rs::main` with a non-empty transaction source path to unlock INV-01/INV-06.
- BLOCKER: Strengthen verification integrity behavior in app verification path so INV-02 covers full execution artifacts, not only state-root equality.
- BLOCKER: Define and wire finalize-to-commit ownership seam for canonical state updates to satisfy INV-05; current consumed `ConsensusApp` contract has no finalize callback (`crates/consensus/src/app.rs::ConsensusApp`).
- UNKNOWN: Snapshot/rollback orchestration point for failed propose/verify (INV-04) in runtime wiring is not explicit in node evidence.
- UNKNOWN: Deterministic tx ordering/selection policy for non-empty proposal (INV-07) is not visible from `TxSource::pending()` contract (`crates/app/src/traits.rs::TxSource`).
