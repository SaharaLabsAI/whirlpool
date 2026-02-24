# Consensus Crates Implementation

## TL;DR
> **Summary**: Implement `consensus-core` (traits + types) and `consensus-commonware` (Simplex BFT adapter) crates per `docs/design/consensus.md`, including workspace setup, mock engine, unit tests, and ConsensusStatus API.
> **Deliverables**: 2 new Rust crates, workspace Cargo.toml, mock/noop engine, test suites
> **Effort**: Medium
> **Parallel**: YES — 5 waves
> **Critical Path**: Task 1 (workspace) → Tasks 2-6 (core modules) → Tasks 7-8 (mock + tests) → Tasks 9-13 (adapter) → Task 14 (adapter tests) → F1-F4 (verification)

## Context

### Original Request
Implement consensus crates as specified in `docs/design/consensus.md`. User confirmed: include tests, mock engine, and ConsensusStatus query API.

### Interview Summary
- **Scope**: P0 (consensus-core) + P1 (consensus-commonware) + P3 (mock/noop engine) + ConsensusStatus
- **Test strategy**: Unit tests for core, integration tests for adapter, mock engine for application testing
- **Open questions resolved**: All 5 open questions from design doc resolved per doc leanings (tokio JoinHandle, Vec<u8> proof, no ValidatorSet, no timing hints, YES ConsensusStatus)

### Metis Review (gaps addressed)

**Critical finding — Reporter/Activity mapping correction**:
The design doc (§4.1) incorrectly states the adapter maps `Activity::Finalized` → `ConsensusEvent::Finalized`. In reality, commonware's architecture routes simplex `Activity` to the marshal `Mailbox`, and the Application's `Reporter` receives `Update<B>` from marshal — NOT simplex `Activity`. The adapter must implement `Reporter<Activity = Update<B>>` and map:
- `Update::Block(block, ack)` → `ConsensusEvent::Finalized { block, height, proof: vec![] }` + call `ack.acknowledge()`
- `Update::Tip(height, commitment)` → log tip update (not a finalization event)

**Fault events**: Deferred. Simplex fault Activity goes to marshal, not Application. `ConsensusEvent::Fault` variant stays in the enum for future use but adapter won't emit it yet.

**ConsensusStatus**: Defined as `{ current_height: u64, is_running: bool }`. Exposed via `Arc<AtomicU64>` + `Arc<AtomicBool>` shared between adapter and RunningEngine.

**tokio in consensus-core**: Design doc acknowledges this pragmatic choice. Use minimal tokio dep: `tokio = { version = "1", default-features = false, features = ["rt"] }`.

**CommonwareConfig missing fields**: Added `elector`, `strategy`, and `buffer_pool` fields that simplex::Config requires but design doc omitted.

**Handle bridging**: `simplex::Engine::start()` returns commonware `Handle<()>`, not `tokio::JoinHandle`. Adapter spawns commonware runtime components on tokio, then wraps outer handle.

## Work Objectives

### Core Objective
Create a backend-agnostic consensus abstraction layer (`consensus-core`) and its first concrete implementation wrapping commonware-consensus Simplex BFT (`consensus-commonware`), following the architecture in `docs/design/consensus.md`.

### Deliverables
- Root workspace `Cargo.toml`
- `crates/consensus-core/` — traits + types (block, engine, app, event, error)
- `crates/consensus-commonware/` — Simplex adapter (adapter, engine, config, types)
- Mock/noop consensus engine for testing
- Unit + integration test suites
- ConsensusStatus query API

### Definition of Done (verifiable conditions with commands)
- `cargo build --workspace` succeeds with zero errors
- `cargo nextest run --workspace` — all tests pass
- `cargo clippy --workspace -- -D warnings` — zero warnings
- `cargo tree -p consensus-core --depth 1` shows only: thiserror, tokio (minimal)
- `cargo tree -p consensus-commonware --depth 1` shows consensus-core + commonware crates
- No files in `vendor/` modified

### Must Have
- All 5 core traits from design doc §3 (Block, ConsensusApp, ConsensusEvent+EventSink, ConsensusEngine+RunningEngine, ConsensusError)
- CommonwareEngine implementing ConsensusEngine
- AppAdapter mapping ConsensusApp ↔ commonware Application+VerifyingApplication+Reporter
- CommonwareConfig struct with all required simplex fields
- ConsensusStatus struct + status() on RunningEngine
- MockEngine implementing ConsensusEngine for testing
- Tests proving trait compilation, mock engine lifecycle, adapter mapping

### Must NOT Have (guardrails)
- No concrete block type outside test modules
- No `async-trait` crate dependency (use RPITIT: `impl Future`)
- No builder pattern for CommonwareConfig (plain struct; builder is future work)
- No P2P channel setup helpers (channels are injected, not created)
- No multi-node integration tests (that's P2)
- No metrics/tracing infrastructure
- No modifications to `vendor/` files
- No `ValidatorSet` trait (deferred per design doc)

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: Tests-after + framework: cargo nextest
- QA policy: Every task has agent-executed scenarios
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy

### Parallel Execution Waves

**Wave 1** (Foundation — 1 task):
- Task 1: Create workspace Cargo.toml + directory scaffold [quick]

**Wave 2** (Core Crate — 5 parallel tasks):
- Task 2: consensus-core/error.rs [quick]
- Task 3: consensus-core/block.rs [quick]
- Task 4: consensus-core/app.rs [quick]
- Task 5: consensus-core/event.rs [quick]
- Task 6: consensus-core/engine.rs + ConsensusStatus [quick]

**Wave 3** (Core Tests + Mock — 2 parallel tasks):
- Task 7: MockEngine + mock block [unspecified-low]
- Task 8: consensus-core unit tests [unspecified-low]

**Wave 4** (Adapter Crate — 5 tasks, partial parallel):
- Task 9: consensus-commonware/Cargo.toml + lib.rs [quick]
- Task 10: consensus-commonware/types.rs [quick]
- Task 11: consensus-commonware/config.rs [unspecified-low]
- Task 12: consensus-commonware/adapter.rs [unspecified-high]
- Task 13: consensus-commonware/engine.rs [unspecified-high]

**Wave 5** (Adapter Tests — 1 task):
- Task 14: consensus-commonware tests [unspecified-high]

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | — | 2,3,4,5,6 |
| 2 | 1 | 4,5,6,7,8 |
| 3 | 1 | 4,5,6,7,8 |
| 4 | 1,2,3 | 7,8,12 |
| 5 | 1,2,3 | 7,8,12 |
| 6 | 1,2 | 7,8,13 |
| 7 | 2,3,4,5,6 | 8 |
| 8 | 2,3,4,5,6,7 | — |
| 9 | 1 | 10,11,12,13 |
| 10 | 9,3 | 11,12,13 |
| 11 | 9,10 | 13 |
| 12 | 9,10,4,5 | 13,14 |
| 13 | 9,10,11,12,6 | 14 |
| 14 | 9-13 | — |

### Agent Dispatch Summary

| Wave | Tasks | Categories |
|------|-------|-----------| 
| 1 | 1 | quick |
| 2 | 5 | quick ×5 |
| 3 | 2 | unspecified-low ×2 |
| 4 | 5 | quick ×2, unspecified-low ×1, unspecified-high ×2 |
| 5 | 1 | unspecified-high |

## TODOs

<!-- TASKS_START -->

- [x] 1. Create Workspace Cargo.toml + Directory Scaffold

  **What to do**:
  Create the root workspace `Cargo.toml` and empty crate directory structure. This unblocks all subsequent tasks and is required for `flake.nix` to function.

  1. Create `Cargo.toml` at workspace root:
     ```toml
     [workspace]
     resolver = "2"
     members = [
         "crates/consensus-core",
         "crates/consensus-commonware",
     ]

     [workspace.package]
     version = "0.1.0"
     edition = "2021"
     ```
  2. Create directory structure: `crates/consensus-core/src/`, `crates/consensus-commonware/src/`
  3. Create `crates/consensus-core/Cargo.toml`:
     ```toml
     [package]
     name = "consensus-core"
     version.workspace = true
     edition.workspace = true

     [dependencies]
     thiserror = "2"
     tokio = { version = "1", default-features = false, features = ["rt"] }
     ```
  4. Create `crates/consensus-core/src/lib.rs` with placeholder: `// consensus-core`
  5. Create `crates/consensus-commonware/Cargo.toml`:
     ```toml
     [package]
     name = "consensus-commonware"
     version.workspace = true
     edition.workspace = true

     [dependencies]
     consensus-core = { path = "../consensus-core" }
     commonware-consensus = { path = "../../vendor/commonware/consensus" }
     commonware-broadcast = { path = "../../vendor/commonware/broadcast" }
     commonware-cryptography = { path = "../../vendor/commonware/cryptography" }
     commonware-p2p = { path = "../../vendor/commonware/p2p" }
     commonware-runtime = { path = "../../vendor/commonware/runtime" }
     commonware-storage = { path = "../../vendor/commonware/storage" }
     commonware-codec = { path = "../../vendor/commonware/codec" }
     commonware-utils = { path = "../../vendor/commonware/utils" }
     ```
  6. Create `crates/consensus-commonware/src/lib.rs` with placeholder: `// consensus-commonware`
  7. Run `cargo check --workspace` to verify workspace compiles

  **Must NOT do**: Do not add any trait/type definitions yet. Do not modify `flake.nix`. Do not add vendored crates as workspace members.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Simple file creation, no complex logic
  - Skills: [] — no special skills needed
  - Omitted: [`git-master`] — commit handled separately

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [2,3,4,5,6,9] | Blocked By: []

  **References**:
  - Pattern: `vendor/commonware/Cargo.toml` — workspace structure reference
  - Pattern: `vendor/alto/Cargo.toml` — workspace member pattern
  - Config: `flake.nix` — reads `workspace.package.version` from root Cargo.toml

  **Acceptance Criteria**:
  - [ ] `cargo check --workspace` exits 0
  - [ ] Both crate directories exist with `Cargo.toml` + `src/lib.rs`
  - [ ] Root `Cargo.toml` contains `[workspace.package]` with `version = "0.1.0"`

  **QA Scenarios**:
  ```
  Scenario: Workspace compiles
    Tool: Bash
    Steps: cargo check --workspace
    Expected: Exit code 0, no errors
    Evidence: .sisyphus/evidence/task-1-workspace.txt

  Scenario: Both crate dirs exist
    Tool: Bash
    Steps: find crates/ -name "Cargo.toml" -o -name "lib.rs" | sort
    Expected: Lists 4 files (2 Cargo.toml + 2 lib.rs)
    Evidence: .sisyphus/evidence/task-1-structure.txt
  ```

  **Commit**: YES | Message: `feat(consensus): scaffold workspace and crate directories` | Files: [Cargo.toml, crates/consensus-core/**, crates/consensus-commonware/**]

- [ ] 2. Implement consensus-core/error.rs — ConsensusError

  **What to do**:
  Create `crates/consensus-core/src/error.rs` with the `ConsensusError` enum exactly as specified in design doc §3.5.

  1. Create `crates/consensus-core/src/error.rs`:
     ```rust
     #[derive(Debug, thiserror::Error)]
     pub enum ConsensusError {
         #[error("invalid block: {0}")]
         InvalidBlock(String),

         #[error("proposal failed: {0}")]
         ProposalFailed(String),

         #[error("engine not ready: {0}")]
         NotReady(String),

         #[error("runtime error: {0}")]
         Runtime(String),

         #[error("shutdown requested")]
         Shutdown,

         #[error("{0}")]
         Other(Box<dyn std::error::Error + Send + Sync>),
     }
     ```
  2. Add `pub mod error;` to `lib.rs` and re-export: `pub use error::ConsensusError;`

  **Must NOT do**: Do not add variants beyond the design doc. Do not implement `From` conversions yet.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single file, exact spec in design doc
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 3,4,5,6) | Wave 2 | Blocks: [4,5,6,7,8] | Blocked By: [1]

  **References**:
  - Spec: `docs/design/consensus.md:261-284` — exact ConsensusError definition

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-core` exits 0
  - [ ] `ConsensusError` has exactly 6 variants: InvalidBlock, ProposalFailed, NotReady, Runtime, Shutdown, Other

  **QA Scenarios**:
  ```
  Scenario: Error type compiles
    Tool: Bash
    Steps: cargo check -p consensus-core
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-2-error.txt
  ```

  **Commit**: NO (batched with wave 2)

- [ ] 3. Implement consensus-core/block.rs — Block Trait

  **What to do**:
  Create `crates/consensus-core/src/block.rs` with the `Block` trait exactly as specified in design doc §3.1.

  1. Create `crates/consensus-core/src/block.rs`:
     ```rust
     /// A consensus block.
     ///
     /// Intentionally minimal — only identity + ordering.
     /// Serialization, digest computation, and proof attachment
     /// belong to the adapter or the application crate.
     pub trait Block: Send + Sync + 'static {
         /// Opaque block identifier (hash, commitment, etc.).
         type Id: Copy + Eq + core::hash::Hash + core::fmt::Debug + Send + Sync + 'static;

         /// This block's unique identifier.
         fn id(&self) -> Self::Id;

         /// Parent block's identifier. Genesis returns a well-known sentinel.
         fn parent_id(&self) -> Self::Id;

         /// Monotonically increasing block height. Genesis = 0.
         fn height(&self) -> u64;
     }
     ```
  2. Add `pub mod block;` to `lib.rs` and re-export: `pub use block::Block;`

  **Must NOT do**: Do not add codec/serialization. Do not add digest computation. Do not create a concrete block struct.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single trait, exact spec
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 2,4,5,6) | Wave 2 | Blocks: [4,5,6,7,8] | Blocked By: [1]

  **References**:
  - Spec: `docs/design/consensus.md:66-87` — exact Block trait definition
  - Rationale: `docs/design/consensus.md:89` — why minimal vs commonware's Block super-traits

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-core` exits 0
  - [ ] Block trait has exactly 3 methods: `id`, `parent_id`, `height`
  - [ ] Block trait has `Id` associated type with correct bounds (Copy + Eq + Hash + Debug + Send + Sync + 'static)

  **QA Scenarios**:
  ```
  Scenario: Block trait compiles
    Tool: Bash
    Steps: cargo check -p consensus-core
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-3-block.txt
  ```

  **Commit**: NO (batched with wave 2)

- [ ] 4. Implement consensus-core/app.rs — ConsensusApp Trait

  **What to do**:
  Create `crates/consensus-core/src/app.rs` with the `ConsensusApp` trait as specified in design doc §3.2.

  1. Create `crates/consensus-core/src/app.rs`:
     ```rust
     use core::future::Future;
     use crate::block::Block;
     use crate::error::ConsensusError;

     /// Application logic for block production and validation.
     ///
     /// Implemented by the application. The consensus engine calls
     /// these methods during its protocol rounds.
     pub trait ConsensusApp: Send + Sync + 'static {
         type Block: Block;

         /// Produce the genesis block. Called once at chain init.
         fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

         /// Propose a new block extending `parent`.
         /// Returns `None` to skip this slot.
         fn propose(
             &self,
             parent: &Self::Block,
             height: u64,
         ) -> impl Future<Output = Option<Self::Block>> + Send;

         /// Validate a block proposed by another participant.
         /// Must be deterministic for the same (parent, block) pair.
         fn verify(
             &self,
             parent: &Self::Block,
             block: &Self::Block,
         ) -> impl Future<Output = Result<(), ConsensusError>> + Send;
     }
     ```
  2. Add `pub mod app;` to `lib.rs` and re-export: `pub use app::ConsensusApp;`

  **Must NOT do**: Do not add runtime generic `E`. Do not add timing hints. Do not use `async-trait`.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single trait, exact spec
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 2,3,5,6) | Wave 2 | Blocks: [7,8,12] | Blocked By: [1,2,3]

  **References**:
  - Spec: `docs/design/consensus.md:96-131` — exact ConsensusApp definition
  - Design: `docs/design/consensus.md:133-138` — rationale for `impl Future` over `async-trait`

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-core` exits 0
  - [ ] ConsensusApp uses RPITIT (`impl Future`), NOT `async-trait`
  - [ ] No `async-trait` in `cargo tree -p consensus-core`

  **QA Scenarios**:
  ```
  Scenario: ConsensusApp trait compiles with RPITIT
    Tool: Bash
    Steps: cargo check -p consensus-core && cargo tree -p consensus-core | grep -v async-trait
    Expected: Exit code 0, no async-trait dependency
    Evidence: .sisyphus/evidence/task-4-app.txt
  ```

  **Commit**: NO (batched with wave 2)

- [ ] 5. Implement consensus-core/event.rs — ConsensusEvent + EventSink

  **What to do**:
  Create `crates/consensus-core/src/event.rs` with `ConsensusEvent` enum and `EventSink` trait as specified in design doc §3.3.

  1. Create `crates/consensus-core/src/event.rs`:
     ```rust
     use core::future::Future;
     use crate::block::Block;
     use crate::error::ConsensusError;

     #[derive(Debug)]
     pub enum ConsensusEvent<B: Block> {
         Finalized { block: B, height: u64, proof: Vec<u8> },
         PreFinalized { block: B, height: u64 },
         Fault { offender: Vec<u8>, evidence: Vec<u8> },
     }

     pub trait EventSink: Send + Sync + 'static {
         type Block: Block;
         fn handle(
             &self,
             event: ConsensusEvent<Self::Block>,
         ) -> impl Future<Output = Result<(), ConsensusError>> + Send;
     }
     ```
  2. Add `pub mod event;` to `lib.rs` and re-export: `pub use event::{ConsensusEvent, EventSink};`

  **Must NOT do**: Do not parse proof bytes. Do not add typed proof variants. Keep Fault variant even though adapter won't emit it yet.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single file, exact spec
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 2,3,4,6) | Wave 2 | Blocks: [7,8,12] | Blocked By: [1,2,3]

  **References**:
  - Spec: `docs/design/consensus.md:143-192` — exact ConsensusEvent + EventSink definition
  - Rationale: `docs/design/consensus.md:194-199` — design choices for opaque proof

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-core` exits 0
  - [ ] ConsensusEvent has exactly 3 variants: Finalized, PreFinalized, Fault
  - [ ] EventSink uses RPITIT, not `async-trait`

  **QA Scenarios**:
  ```
  Scenario: Event types compile
    Tool: Bash
    Steps: cargo check -p consensus-core
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-5-event.txt
  ```

  **Commit**: NO (batched with wave 2)

- [ ] 6. Implement consensus-core/engine.rs — ConsensusEngine + RunningEngine + ConsensusStatus

  **What to do**:
  Create `crates/consensus-core/src/engine.rs` with `ConsensusEngine` trait, `RunningEngine` struct, and `ConsensusStatus` struct per design doc §3.4 + ConsensusStatus extension.

  1. Create `crates/consensus-core/src/engine.rs`:
     ```rust
     use crate::error::ConsensusError;
     use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
     use std::sync::Arc;

     #[derive(Debug, Clone)]
     pub struct ConsensusStatus {
         pub current_height: u64,
         pub is_running: bool,
     }

     pub trait ConsensusEngine: Send + 'static {
         fn start(self) -> Result<RunningEngine, ConsensusError>;
     }

     pub struct RunningEngine {
         _shutdown: Box<dyn FnOnce() + Send>,
         handle: tokio::task::JoinHandle<Result<(), ConsensusError>>,
         height: Arc<AtomicU64>,
         running: Arc<AtomicBool>,
     }

     impl RunningEngine {
         pub fn new(shutdown: impl FnOnce() + Send + 'static, handle: tokio::task::JoinHandle<Result<(), ConsensusError>>, height: Arc<AtomicU64>, running: Arc<AtomicBool>) -> Self { ... }
         pub fn status(&self) -> ConsensusStatus { ... }
         pub async fn wait(self) -> Result<(), ConsensusError> { ... }
         pub async fn shutdown(self) -> Result<(), ConsensusError> { ... }
     }
     ```
  2. Add `pub mod engine;` to `lib.rs` and re-export: `pub use engine::{ConsensusEngine, ConsensusStatus, RunningEngine};`

  **Must NOT do**: Do not add epoch/view fields to ConsensusStatus. Do not make RunningEngine generic over handle type.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single file, spec + small extension
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 2,3,4,5) | Wave 2 | Blocks: [7,8,13] | Blocked By: [1,2]

  **References**:
  - Spec: `docs/design/consensus.md:204-257` — ConsensusEngine + RunningEngine
  - Extension: ConsensusStatus — user confirmed, design doc §8 Q5

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-core` exits 0
  - [ ] `RunningEngine` has `status()` returning `ConsensusStatus`
  - [ ] `ConsensusStatus` has `current_height: u64` and `is_running: bool`

  **QA Scenarios**:
  ```
  Scenario: Engine types compile
    Tool: Bash
    Steps: cargo check -p consensus-core
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-6-engine.txt
  ```

  **Commit**: YES | Message: `feat(consensus-core): implement all core traits and types` | Files: [crates/consensus-core/src/**]

- [ ] 7. Implement MockEngine + Mock Block Type

  **What to do**:
  Create mock/noop consensus engine and mock block in `consensus-core` behind `#[cfg(any(test, feature = "mock"))]`.

  1. Add to `crates/consensus-core/Cargo.toml`:
     ```toml
     [features]
     mock = []
     [dev-dependencies]
     tokio = { version = "1", features = ["rt", "macros"] }
     ```
  2. Create `crates/consensus-core/src/mock/mod.rs` — re-exports MockBlock + MockEngine
  3. Create `crates/consensus-core/src/mock/block.rs`:
     - `MockBlock { id: [u8; 32], parent_id: [u8; 32], height: u64 }` implementing `Block`
     - `MockBlock::genesis()` — zeroed id, zeroed parent, height 0
     - `MockBlock::child(parent)` — extends parent with height+1
  4. Create `crates/consensus-core/src/mock/engine.rs`:
     - `MockEngine` holds blocks to finalize + `Arc<dyn EventSink<Block = MockBlock>>`
     - Implements `ConsensusEngine`: spawns tokio task delivering Finalized events
     - Shutdown via `tokio::sync::oneshot`, updates height/running atomics
  5. Add to `lib.rs`: `#[cfg(any(test, feature = "mock"))] pub mod mock;`

  **Must NOT do**: Do not make complex. No networking. No timing. Keep deterministic.

  **Recommended Agent Profile**:
  - Category: `unspecified-low` — Reason: Multiple files, straightforward
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 8) | Wave 3 | Blocks: [8] | Blocked By: [2,3,4,5,6]

  **References**:
  - Spec: `docs/design/consensus.md:603-605` — P3 noop/mock engine
  - Traits: `crates/consensus-core/src/engine.rs` — ConsensusEngine to implement
  - Traits: `crates/consensus-core/src/event.rs` — EventSink to call

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-core --features mock` exits 0
  - [ ] MockBlock implements Block, MockEngine implements ConsensusEngine
  - [ ] MockEngine delivers Finalized events to EventSink

  **QA Scenarios**:
  ```
  Scenario: Mock engine lifecycle
    Tool: Bash
    Steps: cargo nextest run -p consensus-core mock
    Expected: All mock-related tests pass
    Evidence: .sisyphus/evidence/task-7-mock.txt
  ```

  **Commit**: NO (batched with task 8)

- [ ] 8. Write consensus-core Unit Tests

  **What to do**:
  Comprehensive unit tests: block trait, engine lifecycle, shutdown, status, errors, event delivery.

  Tests:
  - `test_mock_block_genesis`: height 0, zeroed ids
  - `test_mock_block_child`: increments height, sets parent correctly
  - `test_mock_engine_lifecycle`: 3 blocks → 3 Finalized events → clean exit
  - `test_mock_engine_shutdown`: many blocks → shutdown mid-stream → clean exit
  - `test_consensus_status`: is_running + current_height update correctly
  - `test_consensus_error_display`: all variants produce expected strings
  - `test_event_sink_error_propagation`: EventSink returns Err → engine handles it

  **Must NOT do**: Do not test commonware integration. Do not add benchmarks.

  **Recommended Agent Profile**:
  - Category: `unspecified-low` — Reason: Multiple test cases
  - Skills: []

  **Parallelization**: Can Parallel: NO (needs task 7) | Wave 3 | Blocks: [] | Blocked By: [2,3,4,5,6,7]

  **References**:
  - Mock: `crates/consensus-core/src/mock/` — from task 7
  - Traits: `crates/consensus-core/src/` — all trait files

  **Acceptance Criteria**:
  - [ ] `cargo nextest run -p consensus-core` — all pass
  - [ ] At least 7 test cases covering all areas

  **QA Scenarios**:
  ```
  Scenario: All consensus-core tests pass
    Tool: Bash
    Steps: cargo nextest run -p consensus-core
    Expected: 0 failures
    Evidence: .sisyphus/evidence/task-8-tests.txt

  Scenario: No clippy warnings
    Tool: Bash
    Steps: cargo clippy -p consensus-core --features mock -- -D warnings
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-8-clippy.txt
  ```

  **Commit**: YES | Message: `feat(consensus-core): add mock engine and unit tests` | Files: [crates/consensus-core/src/mock/**, crates/consensus-core/Cargo.toml]

- [ ] 9. Create consensus-commonware Crate Structure + lib.rs

  **What to do**:
  Set up `consensus-commonware` crate with proper dependencies and module structure.

  1. Update `crates/consensus-commonware/Cargo.toml` with all required deps:
     ```toml
     [package]
     name = "consensus-commonware"
     version.workspace = true
     edition.workspace = true

     [dependencies]
     consensus-core = { path = "../consensus-core" }
     commonware-consensus = { path = "../../vendor/commonware/consensus" }
     commonware-broadcast = { path = "../../vendor/commonware/broadcast" }
     commonware-cryptography = { path = "../../vendor/commonware/cryptography" }
     commonware-p2p = { path = "../../vendor/commonware/p2p" }
     commonware-runtime = { path = "../../vendor/commonware/runtime" }
     commonware-storage = { path = "../../vendor/commonware/storage" }
     commonware-codec = { path = "../../vendor/commonware/codec" }
     commonware-utils = { path = "../../vendor/commonware/utils" }
     ```
     Note: Run `cargo check` and add any missing transitive path deps.
  2. Create `crates/consensus-commonware/src/lib.rs`:
     ```rust
     pub mod adapter;
     pub mod config;
     pub mod engine;
     pub mod types;
     ```
  3. Create stub files for each module.
  4. Run `cargo check -p consensus-commonware` to verify dep resolution.

  **Must NOT do**: Do not implement logic. All commonware deps must be path-based to vendor/.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: File setup, dep configuration
  - Skills: []

  **Parallelization**: Can Parallel: NO (wave 4 start) | Wave 4 | Blocks: [10,11,12,13] | Blocked By: [1]

  **References**:
  - Spec: `docs/design/consensus.md:19-36` — crate layout
  - Deps: `docs/design/consensus.md:40-54` — dependency graph
  - Vendor: `vendor/commonware/*/Cargo.toml` — actual crate names

  **Acceptance Criteria**:
  - [ ] All path dependencies resolve
  - [ ] `cargo tree -p consensus-commonware --depth 1` shows consensus-core + commonware crates

  **QA Scenarios**:
  ```
  Scenario: Dependencies resolve
    Tool: Bash
    Steps: cargo tree -p consensus-commonware --depth 1
    Expected: Shows consensus-core and commonware-* crates
    Evidence: .sisyphus/evidence/task-9-deps.txt
  ```

  **Commit**: NO (batched with wave 4)

- [ ] 10. Implement consensus-commonware/types.rs — Block Trait Bindings

  **What to do**:
  Create type aliases and `CommonwareBlock` super-trait binding core + commonware trait requirements.

  1. Create `crates/consensus-commonware/src/types.rs`:
     ```rust
     /// A block type satisfying both core and commonware trait requirements.
     pub trait CommonwareBlock:
         consensus_core::Block
         + commonware_consensus::Block
         + commonware_consensus::Heightable
         + commonware_codec::Codec
         + commonware_cryptography::Digestible
         + commonware_cryptography::Committable
         + Clone
     {}

     impl<T> CommonwareBlock for T where T:
         consensus_core::Block
         + commonware_consensus::Block
         + commonware_consensus::Heightable
         + commonware_codec::Codec
         + commonware_cryptography::Digestible
         + commonware_cryptography::Committable
         + Clone
     {}
     ```

  **Must NOT do**: Do not define a concrete block struct. Do not hardcode signing scheme.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Type aliases and trait bounds
  - Skills: []

  **Parallelization**: Can Parallel: YES (once task 9 done) | Wave 4 | Blocks: [11,12,13] | Blocked By: [9,3]

  **References**:
  - Spec: `docs/design/consensus.md:337-375` — block type mapping
  - Pattern: `vendor/alto/types/src/block.rs` — how Alto's block implements both trait sets
  - Traits: `vendor/commonware/consensus/src/lib.rs` — Block, Heightable
  - Traits: `vendor/commonware/cryptography/src/lib.rs` — Digestible, Committable

  **Acceptance Criteria**:
  - [ ] `CommonwareBlock` requires both `core::Block` and `commonware::Block`
  - [ ] Blanket impl auto-derives for conforming types

  **QA Scenarios**:
  ```
  Scenario: Types compile
    Tool: Bash
    Steps: cargo check -p consensus-commonware
    Expected: types.rs compiles
    Evidence: .sisyphus/evidence/task-10-types.txt
  ```

  **Commit**: NO (batched with wave 4)

- [ ] 11. Implement consensus-commonware/config.rs — CommonwareConfig

  **What to do**:
  Create `CommonwareConfig` struct per design doc §4.3, including missing fields found by Metis (elector, strategy, buffer_pool).

  1. Create `crates/consensus-commonware/src/config.rs`:
     ```rust
     pub struct CommonwareConfig<S, B, L, T> {
         // Identity
         pub scheme: S,
         pub namespace: Vec<u8>,
         // Consensus tuning
         pub leader_timeout: Duration,
         pub notarization_timeout: Duration,
         pub nullify_retry: Duration,
         pub activity_timeout: ViewDelta,
         pub skip_timeout: ViewDelta,
         // Networking
         pub blocker: B,
         pub mailbox_size: usize,
         // Storage
         pub partition_prefix: String,
         pub replay_buffer: usize,
         pub write_buffer: usize,
         // Epoch
         pub epoch: Epoch,
         pub epoch_length: u64,
         // Fetch
         pub fetch_timeout: Duration,
         pub fetch_concurrent: usize,
         // Engine components (Metis finding: missing from design doc)
         pub elector: L,
         pub strategy: T,
         pub buffer_pool: PoolRef,
     }
     ```

  **Must NOT do**: No builder pattern. No Default impl. No validation logic. Keep generic over S/B/L/T.

  **Recommended Agent Profile**:
  - Category: `unspecified-low` — Reason: Many fields, careful type alignment with simplex::Config
  - Skills: []

  **Parallelization**: Can Parallel: YES (with 10,12) | Wave 4 | Blocks: [13] | Blocked By: [9,10]

  **References**:
  - Spec: `docs/design/consensus.md:380-412` — CommonwareConfig (partial)
  - Complete: `vendor/commonware/consensus/src/simplex/config.rs` — all simplex::Config fields
  - Pattern: `vendor/alto/chain/src/engine.rs:274-298` — how Config maps to simplex::Config

  **Acceptance Criteria**:
  - [ ] Config has all fields from design doc PLUS: elector, strategy, buffer_pool
  - [ ] Generic over S (scheme), B (blocker), L (elector), T (strategy)

  **QA Scenarios**:
  ```
  Scenario: Config compiles
    Tool: Bash
    Steps: cargo check -p consensus-commonware
    Expected: config.rs compiles
    Evidence: .sisyphus/evidence/task-11-config.txt
  ```

  **Commit**: NO (batched with wave 4)

- [ ] 12. Implement consensus-commonware/adapter.rs — AppAdapter

  **What to do**:
  Create `AppAdapter<A>` bridging `ConsensusApp` + `EventSink` to commonware `Application<E>` + `VerifyingApplication<E>` + `Reporter<Activity = Update<B>>`.

  **CRITICAL**: The design doc’s §4.1 mapping table is INCORRECT about Reporter. The adapter receives `Update<B>` from marshal, NOT simplex `Activity`. See Metis review.

  **Key struct:**
  ```rust
  pub struct AppAdapter<A, E> {
      app: Arc<A>,
      sink: Arc<dyn EventSink<Block = A::Block>>,
      // Phantom for E
  }
  ```

  **Trait implementations:**
  a. `Application<E>` for `AppAdapter`:
     - `genesis()`: delegate to `self.app.genesis().await`
     - `propose((runtime, context), ancestry)`: resolve parent from ancestry stream, call `self.app.propose(&parent, height).await`

  b. `VerifyingApplication<E>` for `AppAdapter`:
     - `verify((runtime, context), ancestry)`: resolve block+parent from ancestry, call `self.app.verify(&parent, &block).await`, return bool (Ok→true, Err→false)

  c. `Reporter` for `AppAdapter`:
     - `type Activity = Update<A::Block>`
     - `report(Update::Block(block, ack))`: call `self.sink.handle(Finalized{block, height, proof: vec![]})` + `ack.acknowledge()`
     - `report(Update::Tip(height, _))`: log only, no event to sink

  **Must NOT do**: Do not handle Fault events (deferred). Do not implement Relay/Monitor. Do not add retry logic. Do not populate proof bytes.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Complex trait bridging, must match commonware exact signatures
  - Skills: [] — agent should use `ast_grep_search` to verify commonware trait signatures

  **Parallelization**: Can Parallel: YES (with 11) | Wave 4 | Blocks: [13,14] | Blocked By: [9,10,4,5]

  **References**:
  - Pattern: `vendor/alto/chain/src/application.rs` — Application/VerifyingApplication/Reporter impl
  - Trait: `vendor/commonware/consensus/src/lib.rs` — Application<E>, VerifyingApplication<E>
  - Trait: `vendor/commonware/consensus/src/reporter.rs` — Reporter trait
  - Update: `vendor/commonware/consensus/src/marshal/mod.rs:78-95` — Update<B> enum (Tip, Block)
  - Marshaled: `vendor/commonware/consensus/src/application/marshaled.rs` — delegation pattern
  - Core: `crates/consensus-core/src/app.rs`, `crates/consensus-core/src/event.rs`

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-commonware` exits 0
  - [ ] AppAdapter implements Application<E>, VerifyingApplication<E>, Reporter<Activity = Update<B>>
  - [ ] AppAdapter requires Clone (commonware mandates this)
  - [ ] propose() resolves parent from ancestry, delegates to ConsensusApp::propose
  - [ ] verify() resolves parent+block from ancestry, delegates to ConsensusApp::verify, returns bool
  - [ ] report() handles Update::Block by calling EventSink::handle(Finalized) + acknowledging

  **QA Scenarios**:
  ```
  Scenario: Adapter compiles against commonware trait bounds
    Tool: Bash
    Steps: cargo check -p consensus-commonware
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-12-adapter.txt

  Scenario: All trait impls present
    Tool: Bash
    Steps: grep -c 'impl.*Application\|impl.*Reporter' crates/consensus-commonware/src/adapter.rs
    Expected: At least 3 impl blocks
    Evidence: .sisyphus/evidence/task-12-adapter-traits.txt
  ```

  **Commit**: NO (batched with wave 4)

- [ ] 13. Implement consensus-commonware/engine.rs — CommonwareEngine

  **What to do**:
  Create `CommonwareEngine` that builds and starts the full Simplex stack, implementing `ConsensusEngine`.
  Follow Alto's `engine.rs`: buffer → marshal → Marshaled → simplex.

  **Key struct:**
  ```rust
  pub struct CommonwareEngine<A, E, S, B, L, T> {
      buffer: buffered::Engine<...>,
      marshal: marshal::Actor<E, ...>,
      consensus: simplex::Engine<E, S, ...>,
      height: Arc<AtomicU64>,
      running: Arc<AtomicBool>,
  }
  ```

  **Constructor** (`async fn new`):
  1. Create buffered::Engine
  2. Open archives (finalized_blocks, finalizations_by_height)
  3. Create marshal::Actor with archives
  4. Create AppAdapter wrapping app + sink
  5. Create Marshaled wrapping AppAdapter + marshal mailbox
  6. Create Reporters tuple (marshal_mailbox, ...)
  7. Create simplex::Engine with Config
  8. Return Self

  **ConsensusEngine impl** (`fn start`):
  1. Set running = true
  2. Spawn tokio task: start buffer → marshal → consensus, try_join_all
  3. Set running = false on exit
  4. Return RunningEngine with shutdown oneshot

  **Must NOT do**: Do not create P2P channels (injected). Do not open network connections. No retry/reconnect. No metrics.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Complex async wiring, must match Alto's engine.rs precisely
  - Skills: []

  **Parallelization**: Can Parallel: NO (needs 9-12) | Wave 4 (tail) | Blocks: [14] | Blocked By: [9,10,11,12,6]

  **References**:
  - Pattern: `vendor/alto/chain/src/engine.rs:126-298` — Engine::new() construction
  - Pattern: `vendor/alto/chain/src/engine.rs:370-389` — Engine::start() sequence
  - Config: `vendor/commonware/consensus/src/simplex/config.rs` — simplex::Config
  - Marshal: `vendor/commonware/consensus/src/marshal/` — Actor, Config
  - Buffered: `vendor/commonware/broadcast/src/buffered/` — buffered::Engine
  - Core: `crates/consensus-core/src/engine.rs` — ConsensusEngine trait

  **Acceptance Criteria**:
  - [ ] `cargo check -p consensus-commonware` exits 0
  - [ ] CommonwareEngine implements ConsensusEngine
  - [ ] `new()` follows buffer → marshal → Marshaled → simplex order
  - [ ] `start()` returns RunningEngine with working shutdown/status

  **QA Scenarios**:
  ```
  Scenario: Engine compiles
    Tool: Bash
    Steps: cargo check -p consensus-commonware
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-13-engine.txt

  Scenario: Full workspace compiles
    Tool: Bash
    Steps: cargo build --workspace
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-13-workspace.txt
  ```

  **Commit**: YES | Message: `feat(consensus-commonware): implement Simplex BFT adapter` | Files: [crates/consensus-commonware/src/**]

- [ ] 14. Write consensus-commonware Tests

  **What to do**:
  Unit tests for adapter crate. Focus on type-level verification, config construction, adapter method logic with mocks.

  Tests:
  - `test_adapter_satisfies_application_bounds`: AppAdapter<MockApp, E> compiles
  - `test_adapter_satisfies_reporter_bounds`: AppAdapter impl Reporter<Activity = Update<B>>
  - `test_config_construction`: create CommonwareConfig, verify fields
  - `test_update_block_to_finalized_event`: Update::Block → EventSink receives Finalized
  - `test_update_tip_no_event`: Update::Tip → no event emitted
  - `test_verify_delegates_correctly`: verify() calls ConsensusApp::verify with correct args

  Test utilities needed:
  - Test block type implementing both core::Block and CommonwareBlock
  - Mock ConsensusApp implementation
  - Recording EventSink that captures events

  **Must NOT do**: No multi-node integration tests. No P2P network. No Simplex protocol testing.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Complex test setup bridging two trait ecosystems
  - Skills: []

  **Parallelization**: Can Parallel: NO (needs all adapter code) | Wave 5 | Blocks: [] | Blocked By: [9-13]

  **References**:
  - Mock: `vendor/commonware/consensus/src/marshal/mocks/` — commonware mock patterns
  - Core mock: `crates/consensus-core/src/mock/` — MockBlock, MockEngine
  - Adapter: `crates/consensus-commonware/src/adapter.rs`

  **Acceptance Criteria**:
  - [ ] `cargo nextest run -p consensus-commonware` — all pass
  - [ ] At least 6 test cases covering: trait bounds, config, event mapping, delegation
  - [ ] `cargo clippy -p consensus-commonware -- -D warnings` exits 0

  **QA Scenarios**:
  ```
  Scenario: All adapter tests pass
    Tool: Bash
    Steps: cargo nextest run -p consensus-commonware
    Expected: 0 failures
    Evidence: .sisyphus/evidence/task-14-tests.txt

  Scenario: Full workspace clippy
    Tool: Bash
    Steps: cargo clippy --workspace -- -D warnings
    Expected: Exit code 0
    Evidence: .sisyphus/evidence/task-14-clippy.txt
  ```

  **Commit**: YES | Message: `test(consensus-commonware): add adapter unit tests` | Files: [crates/consensus-commonware/tests/**, crates/consensus-commonware/Cargo.toml]
<!-- TASKS_END -->

## Final Verification Wave (4 parallel agents, ALL must APPROVE)

- [ ] F1. Plan Compliance Audit — oracle
  Verify all tasks match `docs/design/consensus.md` requirements. Check: all 5 core traits present, crate layout matches §2, dependency graph matches §3, no scope violations.

- [ ] F2. Code Quality Review — unspecified-high
  Run `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, `cargo doc --workspace --no-deps`. Verify no warnings, proper documentation on public items.

- [ ] F3. Real Manual QA — unspecified-high
  Execute: `cargo build --workspace`, `cargo nextest run --workspace`, `cargo tree -p consensus-core --depth 1` (verify minimal deps), `cargo tree -p consensus-commonware --depth 1` (verify correct deps).

- [ ] F4. Scope Fidelity Check — deep
  Verify: no files outside `crates/` and root `Cargo.toml` were created/modified. No `vendor/` changes. No undocumented features added. Mock engine is behind feature flag. All acceptance criteria from all tasks are met.

## Commit Strategy

| Commit Point | Message | Scope |
|---|---|---|
| After Wave 1 | `feat(consensus): scaffold workspace and crate directories` | Root Cargo.toml, crate dirs |
| After Wave 2 | `feat(consensus-core): implement all core traits and types` | consensus-core/src/** |
| After Wave 3 | `feat(consensus-core): add mock engine and unit tests` | consensus-core mock + tests |
| After Wave 4 | `feat(consensus-commonware): implement Simplex BFT adapter` | consensus-commonware/src/** |
| After Wave 5 | `test(consensus-commonware): add adapter unit tests` | consensus-commonware tests |

## Success Criteria

- [ ] `cargo build --workspace` — zero errors
- [ ] `cargo nextest run --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` — zero warnings
- [ ] `cargo tree -p consensus-core --depth 1` — only thiserror + tokio
- [ ] `cargo tree -p consensus-commonware --depth 1` — consensus-core + commonware crates
- [ ] No files in `vendor/` modified
- [ ] All 5 core traits from design doc present and correctly defined
- [ ] CommonwareEngine implements ConsensusEngine
- [ ] MockEngine implements ConsensusEngine
- [ ] ConsensusStatus API functional
