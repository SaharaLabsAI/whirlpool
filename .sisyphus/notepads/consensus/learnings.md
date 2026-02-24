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
