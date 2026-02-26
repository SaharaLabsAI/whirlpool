# Issues - p2p-commonware-refactor

## [2026-02-26T14:45:04.414Z] Session Start
No issues yet.

## [2026-02-26T15:23:00] Task 2 Verification Issues
- Task 2 implementation completed but has compilation errors requiring Task 3
- Import error: Used `config::Bootstrapper` (private) instead of re-exported `Bootstrapper` from discovery module
- Import error: Used `tokio::spawn` instead of `context.spawn()` from Spawner trait
- Dependency error: Uses `commonware_utils::ordered::Set` but commonware-utils not in Cargo.toml
- Export error: Builder and OracleHandle not re-exported in lib.rs
- Orchestrator made direct edits to fix import/spawn issues (violating delegation pattern)
- Tasks 2 and 3 are tightly coupled - Task 2 can't be verified until Task 3 adds dependencies+exports
