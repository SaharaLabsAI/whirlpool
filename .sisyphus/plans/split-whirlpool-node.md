# Split whirlpool-node into Two Binary Crates

## TL;DR
> **Summary**: Refactor whirlpool-node into two separate binary crates — `whirlpool-node` (EVM, production) and `whirlpool-node-simple` (non-EVM, dev/test) — eliminating all `cfg(feature = "evm")` conditional compilation.
> **Deliverables**: Two independently-compilable binary crates, zero feature gates, all existing tests passing
> **Effort**: Short
> **Parallel**: YES - 2 waves
> **Critical Path**: Task 1 (baseline) → Task 2 (new crate) + Task 3 (modify existing) in parallel → Task 4 (workspace) → Task 5 (verify + cleanup) → Task 6 (llmdocs)

## Context

### Original Request
Bob asked to refactor whirlpool-node into two binary crates to eliminate the `evm` feature cfg.

### Interview Summary
- **Binary names**: `whirlpool-node` (EVM, keeps current name) + `whirlpool-node-simple` (non-EVM, new)
- **Structure**: Two separate crate directories under `crates/`
- **Shared code strategy**: Duplicate the ~35 lines of bootstrap code in each main.rs (user explicitly chose this over shared function extraction)
- **Library remains unchanged**: lib.rs, app.rs, block.rs, config.rs have zero feature gates and need no changes
- **whirlpool-node-simple depends on whirlpool-node as library** for EmptyBlockApp, config, block types

### Metis Review (gaps addressed)
1. **Dep bloat concern**: Metis flagged that making EVM deps non-optional could bloat whirlpool-node-simple if it depends on whirlpool-node lib. **Resolution**: Safe — lib.rs only exports `app`, `block`, `config` which don't reference EVM types. Cargo only links EVM deps for the binary target, not the lib. Verified: zero cfg gates in lib.rs/app.rs/block.rs/config.rs.
2. **Bootstrap drift**: Acknowledged risk of duplicated bootstrap code diverging. User explicitly chose duplication. Both binaries MUST use `whirlpool_node::config` constants to minimize drift.
3. **Integration tests**: Metis incorrectly claimed test files exist in `crates/whirlpool-node/tests/` — verified via glob: **no integration test files exist**. No test migration needed.
4. **RwLock import split**: Minor — EVM main.rs should unconditionally import `Arc` + `RwLock`. Addressed in task details.
5. **CI coverage**: Workspace `cargo build` and `cargo test` cover all members. No CI config changes needed (no CI files exist).

## Work Objectives

### Core Objective
Eliminate all `cfg(feature = "evm")` conditional compilation from whirlpool-node by splitting into two dedicated binary crates.

### Deliverables
- `crates/whirlpool-node/` — EVM-only binary crate (modified)
- `crates/whirlpool-node-simple/` — non-EVM binary crate (new)
- Updated workspace `Cargo.toml` with new member
- Zero `cfg(feature = "evm")` in any non-vendor crate

### Definition of Done (verifiable conditions with commands)
```bash
# Both binaries compile
nix develop --command cargo build -p whirlpool-node    # exit 0
nix develop --command cargo build -p whirlpool-node-simple  # exit 0

# All existing tests pass
nix develop --command cargo test   # exit 0 (workspace-wide)

# Zero cfg(feature="evm") in non-vendor code
grep -r 'cfg.*feature.*evm' crates/   # zero matches

# whirlpool-node has no [features] section
grep '\[features\]' crates/whirlpool-node/Cargo.toml   # zero matches

# whirlpool-node has no optional deps
grep 'optional' crates/whirlpool-node/Cargo.toml   # zero matches
```

### Must Have
- Both binaries compile and run independently
- whirlpool-node binary starts EVM consensus path
- whirlpool-node-simple binary starts EmptyBlockApp consensus path
- All existing unit tests in app.rs (11 tests) and block.rs (9 tests) continue to pass
- Config constants shared via `whirlpool_node::config` (not duplicated)

### Must NOT Have (guardrails, scope boundaries)
- Do NOT modify lib.rs, app.rs, block.rs, config.rs — they have zero feature gates
- Do NOT modify vendor/ code
- Do NOT extract shared bootstrap into a function (user chose duplication)
- Do NOT create a shared "node-common" library crate
- Do NOT add new features/functionality — this is a pure structural refactor
- Do NOT move TestStateDb out of main.rs — it stays as development scaffolding

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: Tests-after (existing tests must pass; no new tests needed since behavior is unchanged)
- QA policy: Every task has agent-executed verification via `nix develop --command cargo build/test`
- Evidence: .sisyphus/evidence/task-{N}-{slug}.{ext}

## Execution Strategy

### Parallel Execution Waves

Wave 1 (foundation): Task 1 — baseline verification
Wave 2 (parallel): Task 2 (create whirlpool-node-simple) + Task 3 (modify whirlpool-node) + Task 4 (update workspace)
Wave 3 (verification): Task 5 — full verification + cfg audit
Wave 4 (docs): Task 6 — update llmdocs

### Dependency Matrix
| Task | Depends On | Blocks |
|------|-----------|--------|
| 1. Baseline verification | — | 2, 3, 4 |
| 2. Create whirlpool-node-simple | 1 | 5 |
| 3. Modify whirlpool-node Cargo.toml + main.rs | 1 | 5 |
| 4. Update workspace Cargo.toml | 1 | 5 |
| 5. Full verification + cfg audit | 2, 3, 4 | 6 |
| 6. Update llmdocs | 5 | — |

### Agent Dispatch Summary
| Wave | Tasks | Categories |
|------|-------|-----------|
| 1 | 1 | quick |
| 2 | 3 | quick, quick, quick |
| 3 | 1 | unspecified-low |
| 4 | 1 | quick (ctx-update-doc skill) |

## TODOs

- [x] 1. Baseline Verification

  **What to do**: Run workspace build and test to confirm everything passes before making changes.
  ```bash
  nix develop --command cargo build
  nix develop --command cargo test
  ```
  Record exit codes as evidence.

  **Must NOT do**: Make any code changes. This is verification only.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single command execution, no code changes
  - Skills: [] — no special skills needed
  - Omitted: all — pure shell commands

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: 2, 3, 4 | Blocked By: none

  **References**:
  - AGENTS.md rule: "All cargo commands must be run via `nix develop --command <cmd>`"

  **Acceptance Criteria** (agent-executable only):
  - [x] `nix develop --command cargo build` exits 0
  - [x] `nix develop --command cargo test` exits 0

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Workspace builds clean
    Tool: Bash
    Steps: nix develop --command cargo build 2>&1
    Expected: exit code 0, no errors
    Evidence: .sisyphus/evidence/task-1-baseline-build.txt

  Scenario: Workspace tests pass
    Tool: Bash
    Steps: nix develop --command cargo test 2>&1
    Expected: exit code 0, "test result: ok" for all crates
    Evidence: .sisyphus/evidence/task-1-baseline-test.txt
  ```

  **Commit**: NO

- [x] 2. Create whirlpool-node-simple Crate

  **What to do**: Create a new binary crate at `crates/whirlpool-node-simple/` with two files:

  **File 1: `crates/whirlpool-node-simple/Cargo.toml`**
  ```toml
  [package]
  name = "whirlpool-node-simple"
  version.workspace = true
  edition.workspace = true

  [dependencies]
  whirlpool-node = { path = "../whirlpool-node" }
  consensus = { path = "../consensus" }
  consensus-simplex = { path = "../consensus-simplex" }
  p2p-commonware = { path = "../p2p-commonware" }
  commonware-cryptography = { path = "../../vendor/commonware/cryptography" }
  commonware-runtime = { path = "../../vendor/commonware/runtime", features = ["tokio"] }
  tokio = { version = "1", features = ["full"] }
  tracing = "0.1"
  tracing-subscriber = { version = "0.3", features = ["env-filter"] }
  ```

  **File 2: `crates/whirlpool-node-simple/src/main.rs`**
  Duplicate the non-EVM bootstrap path from current main.rs. The code should:
  1. Import shared types: `use commonware_cryptography::{Signer, ed25519}`, `use commonware_runtime::{tokio, Metrics, Runner}`, `use consensus::ConsensusEngine`, `use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink}`, `use p2p_commonware::CommonwareNetworkProviderBuilder`, `use std::sync::Arc`, `use std::sync::atomic::AtomicU64`, `use std::num::NonZeroUsize`, `use std::net::{IpAddr, Ipv4Addr, SocketAddr}`, `use std::time::Duration`, `use tracing::info`, `use whirlpool_node::app::EmptyBlockApp`, `use whirlpool_node::config`
  2. Define constants: `APPLICATION_NAMESPACE: &[u8] = b"whirlpool-dev"`, `MAX_MESSAGE_SIZE: u32 = 1024 * 1024`
  3. `fn main()` body: Initialize tracing (same pattern as current main.rs lines 88-93), create height `Arc<AtomicU64>`, sink `FinalizationSink`, tokio Runner. Inside `executor.start()`: create signer from `config::VALIDATOR_SEED`, setup listen/dialable addrs, build network provider, create `CommonwareConfig` (same 14 fields as current main.rs lines 121-134), create `EmptyBlockApp::new()`, wrap in `Arc`, create `CommonwareEngine::new(app, sink, engine_config, network_provider)`, call `engine.start()`, await `pending::<()>()`
  4. Add module-level doc comment: `//! Whirlpool simple consensus node (non-EVM) binary.`

  **Must NOT do**:
  - Do NOT import or depend on app, app-evm, state, revm, alloy-primitives
  - Do NOT extract a shared function — duplicate the bootstrap code directly
  - Do NOT add EVM-related code

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Creating two small files from known template
  - Skills: [] — no special skills needed
  - Omitted: all — straightforward file creation

  **Parallelization**: Can Parallel: YES (with Task 3, 4) | Wave 2 | Blocks: 5 | Blocked By: 1

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `crates/whirlpool-node/src/main.rs:86-161` — non-EVM bootstrap path to duplicate (lines 86-99 shared init, 102-134 executor.start body shared setup, 150-156 non-EVM app creation, 158-161 pending wait)
  - Pattern: `crates/whirlpool-node/src/main.rs:3-9,13-16,27-28` — non-EVM imports to use
  - Pattern: `crates/whirlpool-node/src/main.rs:31-32` — constants to duplicate
  - API/Type: `crates/whirlpool-node/src/app.rs:EmptyBlockApp` — the app type to use
  - API/Type: `crates/whirlpool-node/src/config.rs` — config constants (NAMESPACE, VALIDATOR_SEED, etc.)
  - API/Type: `crates/consensus-simplex/src/engine.rs:CommonwareEngine` — generic engine constructor
  - Package: `crates/whirlpool-node/Cargo.toml` — reference for dependency paths and versions

  **Acceptance Criteria** (agent-executable only):
  - [x] File `crates/whirlpool-node-simple/Cargo.toml` exists and is valid TOML
  - [x] File `crates/whirlpool-node-simple/src/main.rs` exists
  - [x] main.rs contains `EmptyBlockApp::new()` (not EVM types)
  - [x] main.rs contains `use whirlpool_node::config` (shared constants)
  - [x] main.rs contains NO `cfg(feature` directives
  - [x] main.rs contains NO imports from `app`, `app_evm`, `state`, `revm`

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Simple node binary compiles
    Tool: Bash
    Steps: nix develop --command cargo build -p whirlpool-node-simple 2>&1
    Expected: exit code 0, binary produced
    Evidence: .sisyphus/evidence/task-2-simple-build.txt

  Scenario: Simple node has no EVM dependencies
    Tool: Bash
    Steps: grep -E 'app-evm|state|revm|alloy' crates/whirlpool-node-simple/Cargo.toml
    Expected: exit code 1 (no matches)
    Evidence: .sisyphus/evidence/task-2-no-evm-deps.txt

  Scenario: No feature cfg in simple node
    Tool: Bash
    Steps: grep 'cfg.*feature' crates/whirlpool-node-simple/src/main.rs
    Expected: exit code 1 (no matches)
    Evidence: .sisyphus/evidence/task-2-no-cfg.txt
  ```

  **Commit**: NO (commit after all changes verified in Task 5)

- [x] 3. Modify whirlpool-node to EVM-Only

  **What to do**: Two file modifications in `crates/whirlpool-node/`:

  **File 1: `crates/whirlpool-node/Cargo.toml`**
  - Remove the entire `[features]` section (lines: `default = ["evm"]`, `evm = [...]`)
  - Convert all 5 optional dependencies to required (remove `optional = true`):
    - `app = { path = "../app" }` (was optional)
    - `app-evm = { path = "../app-evm" }` (was optional)
    - `state = { path = "../state" }` (was optional)
    - `revm = { version = "34", default-features = false }` (was optional)
    - `alloy-primitives = { version = "1.5.0" }` (was optional)
  - Keep all other dependencies unchanged

  **File 2: `crates/whirlpool-node/src/main.rs`**
  Rewrite to EVM-only code, removing all 14 `#[cfg(feature = "evm")]` and 3 `#[cfg(not(feature = "evm"))]` blocks:
  
  1. **Imports** (unconditional, merge both cfg paths):
     - Keep lines 3-9 as-is (shared imports)
     - Replace lines 10-13 with: `use std::sync::{Arc, RwLock};` (unconditional, both Arc and RwLock)
     - Keep lines 14-17 as-is
     - Replace lines 18-25 with unconditional: `use app::{ApplicationAdapter, NoopTxSource};`, `use app_evm::executor::{EvmApplication, StateProvider};`, `use app_evm::{WhirlpoolEvmConfig, build_sahara_chain_spec};`, `use state::InMemoryStateDb;`
     - Remove line 26-27 (`use whirlpool_node::app::EmptyBlockApp` — no longer needed in this binary)
     - Keep line 28 (`use whirlpool_node::config;`)
  
  2. **Constants** (lines 31-32): Keep unchanged
  
  3. **TestStateDb** (lines 34-84): Remove ALL `#[cfg(feature = "evm")]` annotations (lines 34, 38, 45, 52, 55). Keep the struct, impl blocks, and `use revm::Database;` — just remove the 5 cfg attributes.
  
  4. **fn main()** (lines 86-162):
     - Lines 86-134: Keep unchanged (shared bootstrap)
     - Lines 136-148: Remove `#[cfg(feature = "evm")]` on line 136. Keep the block content (lines 137-148) — this becomes the unconditional app creation path
     - Lines 150-156: Remove entirely (the `#[cfg(not(feature = "evm"))]` block with EmptyBlockApp)
     - Lines 158-161: Keep unchanged (pending await)
  
  5. **Doc comment** (line 1): Keep as-is or update to `//! Whirlpool EVM consensus node binary.`

  **Must NOT do**:
  - Do NOT modify lib.rs, app.rs, block.rs, config.rs
  - Do NOT change any logic — only remove cfg gates and the non-EVM code path
  - Do NOT move TestStateDb to another file
  - Do NOT change any dependency versions

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Surgical removal of cfg attributes and one code block
  - Skills: [] — no special skills needed
  - Omitted: all — simple text editing

  **Parallelization**: Can Parallel: YES (with Task 2, 4) | Wave 2 | Blocks: 5 | Blocked By: 1

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `crates/whirlpool-node/src/main.rs` — ENTIRE file, every line matters. See line-by-line instructions above.
  - Pattern: `crates/whirlpool-node/Cargo.toml` — current features and optional deps to modify
  - Guardrail: Do NOT touch `crates/whirlpool-node/src/lib.rs` (exports `app`, `block`, `config` — unchanged)

  **Acceptance Criteria** (agent-executable only):
  - [x] `grep '\[features\]' crates/whirlpool-node/Cargo.toml` returns no matches
  - [x] `grep 'optional' crates/whirlpool-node/Cargo.toml` returns no matches
  - [x] `grep 'cfg.*feature' crates/whirlpool-node/src/main.rs` returns no matches
  - [x] `grep 'EmptyBlockApp' crates/whirlpool-node/src/main.rs` returns no matches
  - [x] main.rs contains `use std::sync::{Arc, RwLock};` (unconditional)
  - [x] main.rs contains `TestStateDb` (still present, no cfg)
  - [x] lib.rs is byte-identical to before (sha256 check)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: EVM node binary compiles
    Tool: Bash
    Steps: nix develop --command cargo build -p whirlpool-node 2>&1
    Expected: exit code 0
    Evidence: .sisyphus/evidence/task-3-evm-build.txt

  Scenario: Unit tests still pass
    Tool: Bash
    Steps: nix develop --command cargo test -p whirlpool-node 2>&1
    Expected: exit code 0, 20 tests pass (11 in app, 9 in block)
    Evidence: .sisyphus/evidence/task-3-evm-test.txt

  Scenario: No cfg(feature) remains
    Tool: Bash
    Steps: grep -c 'cfg.*feature' crates/whirlpool-node/src/main.rs
    Expected: exit code 1 (zero matches)
    Evidence: .sisyphus/evidence/task-3-no-cfg.txt

  Scenario: lib.rs unchanged
    Tool: Bash
    Steps: git diff crates/whirlpool-node/src/lib.rs
    Expected: empty output (no changes)
    Evidence: .sisyphus/evidence/task-3-lib-unchanged.txt
  ```

  **Commit**: NO (commit after all changes verified in Task 5)

- [x] 4. Update Workspace Cargo.toml

  **What to do**: Add `"crates/whirlpool-node-simple"` to the workspace `members` array in the root `Cargo.toml`.

  Current members list (root Cargo.toml, under `[workspace]`):
  ```toml
  members = [
      "crates/consensus",
      "crates/consensus-simplex",
      "crates/p2p",
      "crates/p2p-commonware",
      "crates/whirlpool-node",
      "crates/state",
      "crates/app",
      "crates/app-evm",
  ]
  ```

  Add `"crates/whirlpool-node-simple"` after `"crates/whirlpool-node"` to keep alphabetical grouping:
  ```toml
  members = [
      "crates/consensus",
      "crates/consensus-simplex",
      "crates/p2p",
      "crates/p2p-commonware",
      "crates/whirlpool-node",
      "crates/whirlpool-node-simple",
      "crates/state",
      "crates/app",
      "crates/app-evm",
  ]
  ```

  **Must NOT do**: Change any other fields in the workspace Cargo.toml

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Single line addition
  - Skills: [] — no special skills needed

  **Parallelization**: Can Parallel: YES (with Task 2, 3) | Wave 2 | Blocks: 5 | Blocked By: 1

  **References**:
  - File: `Cargo.toml` (root) — workspace members list

  **Acceptance Criteria** (agent-executable only):
  - [x] `grep 'whirlpool-node-simple' Cargo.toml` returns a match
  - [x] Workspace resolves: `nix develop --command cargo metadata --format-version=1 | grep whirlpool-node-simple` returns a match

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Workspace includes new member
    Tool: Bash
    Steps: grep 'whirlpool-node-simple' Cargo.toml
    Expected: Line containing "crates/whirlpool-node-simple"
    Evidence: .sisyphus/evidence/task-4-workspace-member.txt
  ```

  **Commit**: NO (commit after all changes verified in Task 5)

- [x] 5. Full Verification, cfg Audit, and Commit

  **What to do**: Run comprehensive verification and create a single commit.

  1. **Full workspace build**:
     ```bash
     nix develop --command cargo build
     ```
  2. **Full workspace test**:
     ```bash
     nix develop --command cargo test
     ```
  3. **cfg audit** — verify zero `cfg(feature = "evm")` in non-vendor code:
     ```bash
     grep -r 'cfg.*feature.*evm' crates/
     ```
     Must return zero matches.
  4. **Verify both binaries exist**:
     ```bash
     ls -la target/debug/whirlpool-node target/debug/whirlpool-node-simple
     ```
  5. **Verify lib unchanged**:
     ```bash
     git diff crates/whirlpool-node/src/lib.rs crates/whirlpool-node/src/app.rs crates/whirlpool-node/src/block.rs crates/whirlpool-node/src/config.rs
     ```
     Must be empty.
  6. **Commit all changes** with message:
     ```
     refactor(whirlpool-node): split into two binary crates, remove evm feature cfg
     ```
     Files to stage:
     - `Cargo.toml` (workspace member addition)
     - `Cargo.lock` (auto-updated)
     - `crates/whirlpool-node/Cargo.toml` (features removed, deps made required)
     - `crates/whirlpool-node/src/main.rs` (EVM-only, no cfg)
     - `crates/whirlpool-node-simple/Cargo.toml` (new)
     - `crates/whirlpool-node-simple/src/main.rs` (new)

  **Must NOT do**: Skip any verification step. All must pass before committing.

  **Recommended Agent Profile**:
  - Category: `unspecified-low` — Reason: Multi-step verification + commit
  - Skills: [`git-master`] — for proper commit creation
  - Omitted: all others

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 6 | Blocked By: 2, 3, 4

  **References**:
  - AGENTS.md: "cargo build and cargo test must both pass. Fix any failures introduced by your changes before proceeding."
  - AGENTS.md: "All cargo commands must be run via `nix develop --command <cmd>`"

  **Acceptance Criteria** (agent-executable only):
  - [x] `nix develop --command cargo build` exits 0
  - [x] `nix develop --command cargo test` exits 0
  - [x] `grep -r 'cfg.*feature.*evm' crates/` returns zero matches
  - [x] Both binaries exist in target/debug/
  - [x] lib.rs, app.rs, block.rs, config.rs have zero git diff
  - [x] Git commit created successfully

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Full workspace builds
    Tool: Bash
    Steps: nix develop --command cargo build 2>&1
    Expected: exit code 0
    Evidence: .sisyphus/evidence/task-5-workspace-build.txt

  Scenario: Full workspace tests pass
    Tool: Bash
    Steps: nix develop --command cargo test 2>&1
    Expected: exit code 0, all test suites pass
    Evidence: .sisyphus/evidence/task-5-workspace-test.txt

  Scenario: Zero cfg(feature=evm) in crates
    Tool: Bash
    Steps: grep -r 'cfg.*feature.*evm' crates/ ; echo "exit: $?"
    Expected: no output, exit code 1
    Evidence: .sisyphus/evidence/task-5-cfg-audit.txt

  Scenario: Library code untouched
    Tool: Bash
    Steps: git diff crates/whirlpool-node/src/lib.rs crates/whirlpool-node/src/app.rs crates/whirlpool-node/src/block.rs crates/whirlpool-node/src/config.rs
    Expected: empty output
    Evidence: .sisyphus/evidence/task-5-lib-unchanged.txt
  ```

  **Commit**: YES | Message: `refactor(whirlpool-node): split into two binary crates, remove evm feature cfg` | Files: Cargo.toml, Cargo.lock, crates/whirlpool-node/Cargo.toml, crates/whirlpool-node/src/main.rs, crates/whirlpool-node-simple/Cargo.toml, crates/whirlpool-node-simple/src/main.rs

- [x] 6. Update llmdocs

  **What to do**: Use the `ctx-update-doc` skill to generate/update llmdocs for the affected crates:
  - `whirlpool-node` (modified)
  - `whirlpool-node-simple` (new)

  Per AGENTS.md: "After completing code changes, always use the `ctx-update-doc` skill to generate/update llmdocs for the affected crates."

  **Must NOT do**: Skip this step — AGENTS.md requires it.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Skill invocation, no complex logic
  - Skills: [`ctx-update-doc`] — required for llmdocs generation
  - Omitted: all others

  **Parallelization**: Can Parallel: NO | Wave 4 | Blocks: none | Blocked By: 5

  **References**:
  - AGENTS.md: "After completing code changes, always use the `ctx-update-doc` skill to generate/update llmdocs for the affected crates."

  **Acceptance Criteria** (agent-executable only):
  - [x] llmdocs updated for whirlpool-node
  - [x] llmdocs created for whirlpool-node-simple

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: llmdocs exist for both crates
    Tool: Bash
    Steps: ls llmdocs/ | grep -E 'whirlpool-node|whirlpool-node-simple'
    Expected: entries for both crates
    Evidence: .sisyphus/evidence/task-6-llmdocs.txt
  ```

  **Commit**: YES | Message: `docs(llmdocs): update docs for whirlpool-node split` | Files: llmdocs/**

## Final Verification Wave (4 parallel agents, ALL must APPROVE)

- [x] F1. Plan Compliance Audit — oracle
  Verify all tasks were executed per plan, no deviations.

- [x] F2. Code Quality Review — unspecified-high
  Review all modified/created files for code quality, correct imports, no dead code.

- [x] F3. Real Manual QA — unspecified-high
  Run both binaries briefly to confirm they start without panicking:
  ```bash
  timeout 5 nix develop --command cargo run -p whirlpool-node 2>&1 || true
  timeout 5 nix develop --command cargo run -p whirlpool-node-simple 2>&1 || true
  ```
  Both should show "Starting Whirlpool node" and begin consensus setup.

- [x] F4. Scope Fidelity Check — deep
  Verify: no feature gates remain, lib unchanged, no scope creep beyond the refactor.

## Commit Strategy
Two commits total:
1. `refactor(whirlpool-node): split into two binary crates, remove evm feature cfg` — all code changes
2. `docs(llmdocs): update docs for whirlpool-node split` — documentation only

## Success Criteria
- Zero `cfg(feature = "evm")` in any non-vendor crate
- Both binaries compile independently
- All existing tests pass with zero modifications
- Library code (lib.rs, app.rs, block.rs, config.rs) is byte-identical to pre-refactor
- llmdocs updated for affected crates
