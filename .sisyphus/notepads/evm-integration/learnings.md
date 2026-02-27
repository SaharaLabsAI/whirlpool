## 2026-02-27 Task 1 scaffolding findings
- alloy-* in `vendor/reth/Cargo.toml` `[workspace.dependencies]` are version-based crates.io deps (e.g., `alloy-primitives = { version = "1.5.0", ... }`), not local path deps; no local alloy path wiring is needed from whirlpool crates.
- `app-evm` uses reth path deps only (`../../vendor/reth/crates/...`) for the 7 required reth crates; alloy crates remain transitive through reth crates.
- `revm` does expose primitives via re-exported namespace usage in this repo (`revm::primitives::*` observed under vendor/reth), so `state/Cargo.toml` omits direct `alloy-primitives`.
- `nix develop --command cargo check` first failed due to crates.io timeout downloading `linux-raw-sys v0.12.1`; rerunning the same command succeeded with exit 0 after download completed.
- Non-blocking warnings seen during check/build: deprecated `try_next` in `vendor/commonware/utils/src/channels/tracked.rs` and dead_code field `sink` in `crates/consensus-simplex/src/engine.rs`; no action required for this scaffolding task.


## [2026-02-27T14:45] Task 1: Workspace Scaffolding

**Critical Finding: revm vs reth-revm**
- The plan incorrectly specified `revm = { path = "../../vendor/reth/crates/revm" }`
- Actual vendor crate name: **`reth-revm`** (reth's wrapper with utilities)
- Standalone `revm` crate (core EVM, Database trait) comes from **crates.io version 34**
- Usage pattern:
  - `state` crate: uses `revm = "34"` from crates.io for core Database trait
  - `app-evm` crate: uses `reth-revm = { path = "..." }` for reth-specific execution wiring
- This is the CORRECT pattern, not an error.

**Dependency Resolution**
- alloy-* crates are crates.io deps in reth's workspace (version 0.8), not path deps
- reth-* vendor crates correctly use path deps: `../../vendor/reth/crates/{evm,chainspec,execution-types,...}`
- No async-trait dep added to app crate (uses RPITIT like ConsensusApp)

**Build Status**
- cargo check: PASSED (exit 0, 2 non-blocking warnings)
- cargo build: PASSED (exit 0, "Finished `dev` profile")
- All 3 new crates scaffold correctly with empty stubs
