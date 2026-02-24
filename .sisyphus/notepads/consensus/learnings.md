# Consensus Implementation Learnings

This file tracks conventions, patterns, and wisdom accumulated during consensus crate implementation.

---

## [2026-02-24] Task 1: Workspace Scaffold
- Workspace uses resolver="2" for dependency resolution
- All commonware deps use path dependencies to vendor/ (not crates.io)
- consensus-core has minimal tokio dep: only "rt" feature, no default features
- Both crates use workspace.package for version and edition inheritance
- Vendor commonware crates have their own workspace.toml with extensive dependencies
- Path dependencies to vendor crates work correctly when vendor/commonware/Cargo.toml exists
- Successful compilation requires gcc toolchain for building native dependencies

## [2026-02-24] Task 1.5: Workspace Compilation Fix
- Adding `exclude = ["vendor"]` to [workspace] section prevents cargo from resolving vendor/* as workspace members
- Path dependencies to vendor crates still work correctly after exclusion
- The exclude directive must be placed immediately after `[workspace]` line
- This solves the workspace.package inheritance conflict between root and vendor/commonware workspaces
- Final `cargo clean && cargo check --workspace` exits 0 successfully

## Task 3: block.rs
- Block trait is the core abstraction for consensus layer
- Associated type `Id` requires: Copy + Eq + Hash + Debug + Send + Sync + 'static
  - This ensures block identifiers are lightweight, hashable, and can be used in collections
  - Copy trait ensures cheap copying for ID values (not Clone)
- Three essential methods: id(), parent_id(), height()
- Block trait itself requires: Send + Sync + 'static for thread-safety and dynamic dispatch
- No serialization traits needed at this core level (added at higher layers)
- Minimal, focused trait definition avoids over-specification for concrete implementations
- Successfully compiled with `cargo check -p consensus-core` (exit 0)

## Task 5: event.rs
- ConsensusEvent enum uses generic type parameter B: Block for block abstraction
- Three event variants: Finalized (with proof), PreFinalized, and Fault (with evidence)
- All proof/evidence stored as Vec<u8> (opaque bytes) — not typed structs
- EventSink trait uses RPITIT (impl Future) — no async-trait crate needed
- Associated type Block ensures event handlers work with specific block implementations
- Module successfully added to lib.rs with reexports for public API
- Concurrent compilation with app.rs/engine.rs shows module structure is isolated correctly
- event.rs file created with exact spec: use statements, enum, and trait definitions match requirements

## Task 6: engine.rs
- ConsensusEngine is a synchronous trait with single method `start(self) -> Result<RunningEngine, ConsensusError>`
- RunningEngine wraps a JoinHandle and provides graceful shutdown:
  - `status()` method queries current height and running state atomically (Ordering::Relaxed)
  - `wait()` awaits handle completion
  - `shutdown()` invokes closure then awaits handle
- _shutdown field uses underscore prefix because it's explicitly invoked via function pointer syntax, not auto-dropped
- ConsensusStatus is a simple Copy struct (not static) containing current_height: u64 and is_running: bool
- RunningEngine::new() factory takes all four components (shutdown closure, handle, height Arc, running Arc)
- Map_err converts JoinHandle errors to ConsensusError::Runtime
- Task 2 (error.rs) had to be created concurrently to allow Task 6 cargo check to pass — both tasks depend on each other
- Concurrent task execution created lib.rs race conditions — always READ file before APPEND to handle concurrent edits

## Task 4: app.rs - ConsensusApp Trait

- Created `crates/consensus-core/src/app.rs` with ConsensusApp trait
- ConsensusApp is the primary application interface trait for consensus engine callbacks
- Uses RPITIT (Return Position Impl Trait In Trait) for async methods — no async-trait crate needed
- Associated type Block constrains the application to work with a specific Block implementation
- Three core methods: `genesis()`, `propose()`, and `verify()`
  - `genesis()` produces the initial block
  - `propose()` allows nodes to abstain from proposing (returns Option)
  - `verify()` validates blocks against their parent
- Updated lib.rs to declare `pub mod app` and reexport `pub use app::ConsensusApp`
- Coordinate with other Wave 2 tasks: error.rs (Task 2), block.rs (Task 3), event.rs (Task 5), engine.rs (Task 6)
- RPITIT syntax is stable in Rust 2021 edition and works well for trait methods returning Futures
- All dependencies (Block trait, ConsensusError enum) are properly resolved when all Wave 2 tasks complete

## Task 7: Mock Module for Testing

- Created mock module with MockBlock and MockEngine for testing consensus abstractions
- MockBlock uses `[u8; 32]` as block identifier with genesis() and child() helper methods
- MockEngine is generic over EventSink implementation `MockEngine<S: EventSink<Block = MockBlock>>`
  - **CRITICAL**: Cannot use `Arc<dyn EventSink>` because RPITIT makes EventSink not dyn-compatible
  - Must use generic type parameter instead: `pub struct MockEngine<S: EventSink<Block = MockBlock>>`
- MockEngine spawns tokio task that iterates blocks and emits Finalized events
- Uses tokio::sync::oneshot channel for graceful shutdown coordination
- Added tokio "sync" feature to dependencies (required for oneshot channel)
- Added `[features] mock = []` to Cargo.toml for conditional compilation
- Added dev-dependencies for tokio with "rt" and "macros" features
- Conditional module in lib.rs: `#[cfg(any(test, feature = "mock"))] pub mod mock;`
- MockBlock::child() encodes height in first 8 bytes of id (deterministic, no randomness)
- Successfully compiles with `cargo check -p consensus-core --features mock` (exit 0)

## Task 8: Unit Tests for consensus-core

- Created comprehensive test suite in `crates/consensus-core/src/tests.rs` with 7 test cases
- Tests cover: MockBlock (genesis, child), MockEngine (lifecycle, shutdown, status), ConsensusError (display), EventSink (event collection)
- CollectorSink test helper pattern: returns `(Arc<Self>, Arc<Mutex<Vec<u64>>>)` to allow verification after engine completion
  - Must wrap sink in Arc because MockEngine::new() takes `Arc<S>` where `S: EventSink<Block = MockBlock>`
  - Shared events Vec allows assertion after async engine completes
- Import fix: use `crate::mock::MockBlock` and `crate::mock::MockEngine` (public re-exports), NOT `crate::mock::block::MockBlock` (private modules)
- Type annotation required for Result types in async contexts: `let result: Result<(), ConsensusError> = running.wait().await;`
  - Compiler cannot infer Result type when only calling `.is_ok()` without storing value
  - Alternative: use `.expect()` directly which consumes Result and returns inner value
- All 7 tests pass with cargo nextest (exit 0)
- Clippy clean with --features mock -- -D warnings (zero warnings)
- Evidence saved to `.sisyphus/evidence/task-8-tests.txt`
- Added `#[cfg(test)] mod tests;` to lib.rs to include test module

## Task 9: Module Structure for consensus-commonware

- Created module structure for the new `consensus-commonware` adapter crate
- Updated `crates/consensus-commonware/src/lib.rs` with module declarations for: types, config, adapter, engine
- Added `pub use` statements to re-export CommonwareBlock and CommonwareConfig
- Created 4 stub files (types.rs, config.rs, adapter.rs, engine.rs) with minimal placeholder types
  - types.rs: `pub trait CommonwareBlock {}` (marker trait)
  - config.rs: `pub struct CommonwareConfig;` (empty struct)
  - adapter.rs: `pub struct AppAdapter;` (empty struct)
  - engine.rs: `pub struct CommonwareEngine;` (empty struct)
- Added dependencies to Cargo.toml: tokio (with rt, sync, macros features) and tracing 0.1
- All dependencies already present: consensus-core, and all commonware-* path dependencies
- Successfully compiled with `cargo check -p consensus-commonware` (exit 0)

## CommonwareBlock Super-Trait Implementation

**Task**: Replace placeholder `CommonwareBlock` trait with proper super-trait combining both `consensus_core::Block` and `commonware_consensus::Block`.

**Implementation Pattern**:
- Super-trait: `pub trait CommonwareBlock: CoreBlock + VendorBlock + Clone {}`
- Blanket impl: `impl<T> CommonwareBlock for T where T: CoreBlock + VendorBlock + Clone`
- This allows automatic implementation for any type satisfying both trait requirements

**Key Details**:
- Both Block traits have `height() -> u64` methods from different sources (no name conflict in Rust)
- CoreBlock requires: `id()`, `parent_id()`, `height()` methods
- VendorBlock requires: `Heightable + Codec + Digestible + Committable + Send + Sync` super-traits
- Clone bound needed to ensure concrete types are cloneable

**Verification**:
- `cargo check -p consensus-commonware` ✅ PASSED
- No compilation errors
- Blanket impl pattern means users don't need to manually implement CommonwareBlock

## CommonwareConfig Implementation

**Date**: 2025-02-24

### Completed
- Replaced placeholder `pub struct CommonwareConfig;` with full real struct
- Struct has 11 public fields covering all user-facing params for Simplex BFT engine
- Field mapping from vendor `simplex::Config<...>`:
  - `partition` → `namespace: String`
  - `leader_timeout`, `notarization_timeout`, `nullify_retry` → `Duration` fields
  - `activity_timeout`, `skip_timeout` → `u64` (from `ViewDelta` which is `u64`)
  - `mailbox_size` → `usize`
  - `replay_buffer`, `write_buffer` → `NonZeroUsize`
  - `epoch` → `u64` (from `Epoch` which is `u64`)
  - `fetch_timeout`, `fetch_concurrent` → Duration and usize

### Design Decisions
- Used concrete `u64` for timeout values (vs. type aliases) to avoid vendor import in public API
- No `Default` impl per design constraint (users must explicitly set all values)
- No builder pattern per design constraint
- No generic type parameters (config is concrete, not parameterized)
- Internal details (scheme, elector, blocker, automaton, relay, reporter, strategy, buffer_pool) are NOT exposed—handled by engine

### Compilation Status
- `cargo check -p consensus-commonware` ✓ PASS
- `cargo clippy` has unrelated vendor errors but config.rs has no issues

