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
