# Application Layer

## Definition
Application Layer is the consensus-facing abstraction domain that defines application lifecycle contracts (`genesis`, `propose`, `verify`), transaction ingress seam, and EVM block/result data model, then bridges these contracts into consensus via `ApplicationAdapter`.

Grounded scope evidence:
- `crates/app/src/traits.rs::Application` defines lifecycle methods and associated `Block/Result/Error` types.
- `crates/app/src/traits.rs::TxSource` defines ingress as `pending() -> Vec<Vec<u8>>`.
- `crates/app/src/types.rs::EvmBlock` and `crates/app/src/types.rs::ExecutionResult` define block/result contracts consumed by this layer.
- `crates/app/src/adapter.rs::ApplicationAdapter` implements `consensus::ConsensusApp` for an `Application<Block = EvmBlock>`.

INV constraint impact in this domain scope (INV-01..INV-07):
- INV-01 (Execution Visibility): BLOCKER in current runtime usage because `NoopTxSource` returns empty pending txs (`crates/app/src/traits.rs::NoopTxSource::pending`).
- INV-02 (Verification Integrity): PARTIAL/BLOCKER at bridge level because adapter only maps pass/fail from inner verify and does not expose richer artifact checks (`crates/app/src/adapter.rs::ApplicationAdapter::verify`).
- INV-03 (Verification Read-Only): UNKNOWN at this domain boundary because mutability guarantees are delegated to inner `Application::verify` implementation.
- INV-04 (Snapshot Safety): UNKNOWN because no snapshot/rollback contract exists in `Application` or `ApplicationAdapter` interfaces.
- INV-05 (Commit Atomicity): BLOCKER at interface boundary because `Application`/`ConsensusApp` contracts here expose no finalize/commit callback (`crates/app/src/traits.rs::Application`, `crates/consensus/src/app.rs::ConsensusApp`).
- INV-06 (Root Consistency): UNKNOWN at abstraction level; consistency depends on inner implementation populating `EvmBlock` roots and `ExecutionResult` from actual execution.
- INV-07 (Proposal Determinism): UNKNOWN because `TxSource::pending()` carries no ordering or determinism semantics.

## Derived crates
| Crate | Role in this domain | Evidence |
|---|---|---|
| `app` | Owns application traits, block/result types, and consensus bridge adapter. | `crates/app/src/traits.rs`, `crates/app/src/types.rs`, `crates/app/src/adapter.rs` |
| `consensus` (consumed) | Provides `ConsensusApp` trait implemented by adapter bridge. | `crates/consensus/src/app.rs::ConsensusApp` |
| `app-evm` | Provides a concrete `Application` implementation consumed by adapter. | `crates/app-evm/src/executor.rs::EvmApplication` |
| `whirlpool-node` | Instantiates and injects concrete app into `ApplicationAdapter`. | `crates/whirlpool-node/src/main.rs::main` |

## Key public contracts
| Contract | Why it matters for Application Layer | Evidence |
|---|---|---|
| `Application` trait | Canonical lifecycle boundary for propose/verify/genesis and associated result/error typing. | `crates/app/src/traits.rs::Application` |
| `TxSource` trait | Only explicit transaction ingress seam in this layer. | `crates/app/src/traits.rs::TxSource` |
| `NoopTxSource` | Default empty provider demonstrating current empty-input behavior. | `crates/app/src/traits.rs::NoopTxSource` |
| `EvmBlock` | Consensus-visible block payload with roots, gas, timestamp, and raw transactions. | `crates/app/src/types.rs::EvmBlock` |
| `ExecutionResult` | Execution artifact summary returned by `propose/verify` at application boundary. | `crates/app/src/types.rs::ExecutionResult` |
| `ApplicationAdapter` | Bridge from `Application` to `ConsensusApp` with proposal/verify result mapping. | `crates/app/src/adapter.rs::ApplicationAdapter` |

## Core workflows
1) Lifecycle contract exposure
- Consensus-facing app behavior is modeled in `Application::genesis/propose/verify` (`crates/app/src/traits.rs::Application`).

2) Block/result contract propagation
- `Application::propose` returns `(EvmBlock, ExecutionResult)` and `verify` returns `ExecutionResult`, preserving execution summaries at app boundary (`crates/app/src/traits.rs::Application`, `crates/app/src/types.rs`).

3) Adapter bridge to consensus
- `ApplicationAdapter::genesis` pass-throughs inner genesis.
- `ApplicationAdapter::propose` converts `Result<(Block, Result), Error>` into `Option<Block>` (`Err(_) => None`).
- `ApplicationAdapter::verify` converts inner errors into `ConsensusError::InvalidBlock(err.to_string())`.
- Evidence: `crates/app/src/adapter.rs::ApplicationAdapter::{genesis,propose,verify}`.

4) Transaction ingress seam
- Tx ingress is pulled through `TxSource::pending`; default provided implementation is empty (`NoopTxSource`).
- Evidence: `crates/app/src/traits.rs::{TxSource,NoopTxSource}`.

## Open questions / TODOs
- BLOCKER: Provide non-empty `TxSource` implementation for runtime injection; current default in this domain is empty (`crates/app/src/traits.rs::NoopTxSource`).
- BLOCKER: Adapter proposal path drops error details (`Err(_) => None`), which weakens diagnosability for INV-01/INV-02 failures (`crates/app/src/adapter.rs::ApplicationAdapter::propose`).
- BLOCKER: No finalize/commit callback exists in this domain interface set, leaving INV-05 finalize-to-commit wiring out of scope here (`crates/app/src/traits.rs::Application`, `crates/consensus/src/app.rs::ConsensusApp`).
- UNKNOWN: Deterministic ordering/selection guarantees for `TxSource::pending()` responses are unspecified for INV-07.
- UNKNOWN: Verify read-only and snapshot-safety guarantees (INV-03/INV-04) are not encoded in trait-level contracts and must be enforced by concrete implementations.