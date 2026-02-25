# Consensus Wiring Refactor: Seal Infrastructure in consensus-simplex

## TL;DR
> **Summary**: Move consensus wiring infrastructure (mailbox, sink, wire) from whirlpool-node into consensus-simplex. Genericize hardcoded EmptyBlock types. Replace stub starter closure with real simplex engine wiring. Un-gate disabled code.
> **Deliverables**: Refactored consensus-simplex with sealed wiring API; cleaned-up whirlpool-node consuming it
> **Effort**: Medium
> **Parallel**: YES - 3 waves
> **Critical Path**: Task 1 (Mailbox) + Task 2 (Sink) → Task 3 (Engine Wiring) → Task 4 (whirlpool-node Integration) → Task 5 (Cleanup)

## Context
### Original Request
Bob wants consensus wiring moved from whirlpool-node into the consensus crate layer. Currently whirlpool-node contains generic consensus plumbing (mailbox bridging, event sink, wire starter) that should be sealed inside consensus-simplex.

### Interview Summary
- **Destination**: consensus-simplex crate (implementation layer, not consensus interface crate)
- **Un-gating**: Fully enable mailbox.rs and sink.rs (remove `never_enable_this` cfg gates)
- **Scope**: Relocate existing code + build real simplex engine wiring (not just stub)
- **Tests**: Tests-after approach
- **Stays in whirlpool-node**: EmptyBlock, EmptyBlockApp, config constants (business logic)

### Self-Review Gap Analysis (Metis timed out)
- **Genericization**: Mailbox/Sink hardcode EmptyBlock → must use associated types from ConsensusApp/EventSink traits
- **Real wiring**: Need builder/factory that accepts App + Sink + Config and handles full engine lifecycle
- **P2P bootstrapping**: Node-specific config (keys, peers, addresses) must be accepted as parameters
- **Dependency cleanup**: whirlpool-node can drop several commonware deps after move
- **Test migration**: Move tests alongside code, adapt to generic types

## Work Objectives
### Core Objective
Seal all consensus wiring infrastructure inside consensus-simplex so that whirlpool-node only provides business logic (App, Block) and calls a single high-level API to start consensus.

### Deliverables
- Genericized Mailbox + MailboxActor in consensus-simplex
- Genericized FinalizationSink in consensus-simplex
- Real engine wiring function/builder in consensus-simplex replacing CommonwareEngine's starter-closure pattern
- Simplified whirlpool-node consuming new API
- All tests passing: `cargo test --workspace`

### Definition of Done (verifiable conditions with commands)
- `cargo build --workspace` succeeds with zero errors
- `cargo test --workspace` succeeds with zero failures
- `cargo clippy --workspace -- -D warnings` passes
- whirlpool-node/src/ contains NO mailbox.rs, sink.rs, or wire.rs
- whirlpool-node/src/lib.rs has NO cfg-gated `never_enable_this` modules
- consensus-simplex/src/ contains mailbox.rs, sink.rs, and wiring logic
- grep for `EmptyBlock` in consensus-simplex returns zero matches (fully generic)

### Must Have
- All existing tests migrated and passing
- Generic types (no EmptyBlock references in consensus-simplex)
- Single entry-point API for starting consensus from whirlpool-node
- Backward-compatible consensus trait API (no changes to consensus crate traits)

### Must NOT Have (guardrails)
- NO changes to vendor/ directory
- NO changes to consensus crate trait definitions (Block, ConsensusApp, EventSink, ConsensusEngine)
- NO EmptyBlock or node-specific types leaking into consensus-simplex
- NO new feature flags or cfg gates replacing the old ones
- NO changes to EmptyBlock or EmptyBlockApp logic (only their import paths may change)
- DO NOT refactor AppAdapter — it's already generic and correct

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: Tests-after + framework: existing tokio + commonware deterministic::Runner
- QA policy: Every task has agent-executed build + test + clippy scenarios
- Evidence: .sisyphus/evidence/task-{N}-{slug}.{ext}

## Execution Strategy
### Parallel Execution Waves

Wave 1 (Foundation — independent moves): Tasks 1, 2
  - Move + genericize Mailbox (category: deep)
  - Move + genericize Sink (category: deep)

Wave 2 (Engine wiring — depends on Wave 1): Task 3
  - Build real engine wiring using moved Mailbox + Sink (category: deep)

Wave 3 (Integration + Cleanup — depends on Wave 2): Tasks 4, 5, 6
  - Update whirlpool-node to use new API (category: deep)
  - Dependency cleanup + final verification (category: quick)
  - Update llmdocs documentation (category: quick)

### Dependency Matrix
| Task | Depends On | Blocks |
|------|-----------|--------|
| 1. Move Mailbox | — | 3, 4 |
| 2. Move Sink | — | 3, 4 |
| 3. Engine Wiring | 1, 2 | 4 |
| 4. whirlpool-node Integration | 1, 2, 3 | 5, 6 |
| 5. Dependency Cleanup | 4 | 6 |
| 6. Update llmdocs | 4, 5 | — |

### Agent Dispatch Summary
| Wave | Tasks | Categories |
|------|-------|-----------|
| 1 | 2 | deep × 2 |
| 2 | 1 | deep × 1 |
| 3 | 3 | deep × 1, quick × 2 |

## TODOs

- [ ] 1. Move and Genericize Mailbox into consensus-simplex

  **What to do**:
  1. Create `crates/consensus-simplex/src/mailbox.rs`
  2. Move `Mailbox`, `MailboxActor`, `Message` enum, and helper functions (`compute_digest`, `digest_to_block_id`, `is_valid_digest`) from `crates/whirlpool-node/src/mailbox.rs`
  3. Genericize all types: Replace every `EmptyBlock` reference with generic `B: CommonwareBlock` (or appropriate trait bounds). The key changes:
     - `Message` enum: Currently uses `Digest` (sha256) — keep as-is since it's vendor-level, not block-specific
     - `MailboxActor`: Currently does `EmptyBlock::genesis()` and `EmptyBlock::new(height+1, ...)` in `run()`. Instead, hold an `Arc<A: ConsensusApp>` and delegate to `app.genesis()` and `app.propose()`. This requires MailboxActor to become `MailboxActor<A: ConsensusApp>` where `A::Block: CommonwareBlock`
     - `Mailbox` struct: Already generic over channel — stays mostly same. Add phantom type `PhantomData<B>` or make `Mailbox<B: CommonwareBlock>`
     - `compute_digest`: Currently takes `EmptyBlock` → genericize to `B: Digestible` (from commonware_cryptography). The function computes sha256 of codec-serialized block — make it `compute_digest<B: commonware_codec::Write + commonware_cryptography::Digestible>(block: &B) -> Digest`
  4. Update `Automaton` impl to use generic block for propose/genesis
  5. Move ALL 6 tests from whirlpool-node mailbox tests → consensus-simplex tests. Tests will need to use a test block type (can reuse `TestBlock` from existing `consensus-simplex/src/tests.rs` or create a `MockBlock` + `MockApp`)
  6. Add `pub mod mailbox;` to `consensus-simplex/src/lib.rs` and export `Mailbox`, `MailboxActor`
  7. Ensure `cargo build -p consensus-simplex` compiles

  **Must NOT do**:
  - Do NOT change the Automaton/CertifiableAutomaton/Relay trait implementations' semantics
  - Do NOT modify AppAdapter (it's already correct)
  - Do NOT reference EmptyBlock anywhere in consensus-simplex

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Complex genericization requiring understanding of trait bound cascading across Automaton, ConsensusApp, CommonwareBlock, codec, and cryptography traits
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: YES (with Task 2) | Wave 1 | Blocks: 3, 4 | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Source to move: `crates/whirlpool-node/src/mailbox.rs` (351 lines) — Contains Mailbox, MailboxActor, Message enum, helpers. Currently hardcodes `EmptyBlock` in MailboxActor::run() for genesis/propose. Uses `commonware_consensus::{Automaton, CertifiableAutomaton, Relay}`, `simplex::types::Context`, `ed25519::PublicKey`, `sha256::Digest`
  - Destination crate: `crates/consensus-simplex/src/` — Already has AppAdapter (adapter.rs), CommonwareEngine (engine.rs), CommonwareBlock trait (types.rs), CommonwareConfig (config.rs)
  - Generic block trait: `crates/consensus-simplex/src/types.rs` — `CommonwareBlock: CoreBlock + VendorBlock + Clone` with blanket impl
  - ConsensusApp trait: `crates/consensus/src/app.rs` — `ConsensusApp { type Block: Block; genesis(), propose(), verify() }`
  - Existing test patterns: `crates/consensus-simplex/src/tests.rs` — Has TestBlock, CollectorSink, MockApp types usable for mailbox tests
  - Codec traits: `vendor/commonware/codec/` — `Write`, `Read`, `EncodeSize` needed for digest computation
  - Crypto traits: `vendor/commonware/cryptography/` — `Digestible`, `Committable` for compute_digest

  **Acceptance Criteria** (agent-executable only):
  - [ ] `crates/consensus-simplex/src/mailbox.rs` exists and compiles
  - [ ] `grep -r "EmptyBlock" crates/consensus-simplex/` returns zero matches
  - [ ] `cargo build -p consensus-simplex` succeeds
  - [ ] `cargo test -p consensus-simplex` succeeds (all mailbox tests pass)

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Mailbox compiles with generic block type
    Tool: Bash
    Steps: cargo build -p consensus-simplex 2>&1
    Expected: Compilation succeeds with exit code 0
    Evidence: .sisyphus/evidence/task-1-mailbox-build.txt

  Scenario: All migrated mailbox tests pass
    Tool: Bash
    Steps: cargo test -p consensus-simplex mailbox 2>&1
    Expected: All 6+ tests pass, zero failures
    Evidence: .sisyphus/evidence/task-1-mailbox-tests.txt

  Scenario: No EmptyBlock references in consensus-simplex
    Tool: Bash
    Steps: grep -r "EmptyBlock" crates/consensus-simplex/src/ || echo "CLEAN"
    Expected: Output is "CLEAN"
    Evidence: .sisyphus/evidence/task-1-mailbox-no-emptyblock.txt
  ```

  **Commit**: YES | Message: `refactor(consensus-simplex): move and genericize Mailbox from whirlpool-node` | Files: `crates/consensus-simplex/src/mailbox.rs`, `crates/consensus-simplex/src/lib.rs`, `crates/consensus-simplex/Cargo.toml`

- [ ] 2. Move and Genericize FinalizationSink into consensus-simplex

  **What to do**:
  1. Create `crates/consensus-simplex/src/sink.rs`
  2. Move `FinalizationSink` from `crates/whirlpool-node/src/sink.rs`
  3. Genericize: Replace `EventSink<Block=EmptyBlock>` with `EventSink<Block=B>` where `B: consensus::Block`. Changes:
     - `FinalizationSink` → `FinalizationSink<B: consensus::Block>` with `PhantomData<B>`
     - `impl EventSink for FinalizationSink` → `impl<B: consensus::Block> EventSink for FinalizationSink<B>` with `type Block = B`
     - `handle()` method: Currently matches on `ConsensusEvent::Finalized{block,height,proof}` and uses `block.id()` for logging — this still works with generic `B: Block` since `Block` trait has `id()` method
  4. Move ALL 6 tests to consensus-simplex. Tests need a generic test block — reuse `TestBlock` from existing tests.rs or `MockBlock` from consensus crate
  5. Add `pub mod sink;` to `consensus-simplex/src/lib.rs` and export `FinalizationSink`
  6. Ensure `cargo build -p consensus-simplex` compiles

  **Must NOT do**:
  - Do NOT change EventSink trait semantics
  - Do NOT change the tracing log messages (preserve existing format)
  - Do NOT reference EmptyBlock anywhere

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Requires understanding Block trait bounds and EventSink generic impl, but simpler than Mailbox
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: YES (with Task 1) | Wave 1 | Blocks: 3, 4 | Blocked By: none

  **References**:
  - Source to move: `crates/whirlpool-node/src/sink.rs` (138 lines) — FinalizationSink with Arc<AtomicU64> height tracking. Implements EventSink<Block=EmptyBlock>. Uses tracing::{info,warn}
  - Destination: `crates/consensus-simplex/src/`
  - EventSink trait: `crates/consensus/src/event.rs` — `EventSink { type Block: Block; async fn handle(&self, event: ConsensusEvent<Self::Block>); }`
  - Block trait: `crates/consensus/src/block.rs` — `Block { type Id: Copy+Eq+Hash+Debug+Send+Sync; fn id(); fn parent_id(); fn height(); }`
  - Existing test helpers: `crates/consensus-simplex/src/tests.rs` — TestBlock and CollectorSink already exist (CollectorSink is a simpler EventSink that just collects heights)

  **Acceptance Criteria**:
  - [ ] `crates/consensus-simplex/src/sink.rs` exists and compiles
  - [ ] `grep -r "EmptyBlock" crates/consensus-simplex/` returns zero matches
  - [ ] `cargo build -p consensus-simplex` succeeds
  - [ ] `cargo test -p consensus-simplex` succeeds (all sink tests pass)

  **QA Scenarios**:
  ```
  Scenario: Sink compiles with generic block type
    Tool: Bash
    Steps: cargo build -p consensus-simplex 2>&1
    Expected: Compilation succeeds with exit code 0
    Evidence: .sisyphus/evidence/task-2-sink-build.txt

  Scenario: All migrated sink tests pass
    Tool: Bash
    Steps: cargo test -p consensus-simplex sink 2>&1
    Expected: All 6 tests pass, zero failures
    Evidence: .sisyphus/evidence/task-2-sink-tests.txt

  Scenario: No EmptyBlock references in consensus-simplex
    Tool: Bash
    Steps: grep -r "EmptyBlock" crates/consensus-simplex/src/ || echo "CLEAN"
    Expected: Output is "CLEAN"
    Evidence: .sisyphus/evidence/task-2-sink-no-emptyblock.txt
  ```

  **Commit**: YES | Message: `refactor(consensus-simplex): move and genericize FinalizationSink from whirlpool-node` | Files: `crates/consensus-simplex/src/sink.rs`, `crates/consensus-simplex/src/lib.rs`

- [ ] 3. Build Real Engine Wiring in consensus-simplex

  **What to do**:
  1. Refactor `CommonwareEngine` in `crates/consensus-simplex/src/engine.rs` to replace the starter-closure pattern with a proper builder/factory that internally wires all components:
     - **New API**: `CommonwareEngine::new(app, sink, config)` or a builder pattern `CommonwareEngine::builder().app(app).sink(sink).config(config).build()`
     - Constructor takes: `A: ConsensusApp`, `S: EventSink<Block=A::Block>`, `CommonwareConfig`, plus P2P parameters (validator key/signer, network config like bind address, peer addresses, namespace)
     - The `start()` method (ConsensusEngine impl) now internally:
       a. Creates mpsc channel for Mailbox↔MailboxActor
       b. Creates `Mailbox` (sender side) and `MailboxActor` (receiver side, with Arc of app)
       c. Creates `AppAdapter` wrapping app + sink
       d. Creates `FinalizationSink` wrapping height atomic
       e. Configures simplex `Config` from `CommonwareConfig`
       f. Spawns MailboxActor task
       g. Creates and starts the simplex engine (using commonware_consensus::simplex::Engine)
       h. Returns `RunningEngine` with shutdown handle
  2. Define a `SimplexEngineConfig` or extend `CommonwareConfig` to include P2P parameters needed for real wiring:
     - `namespace: &'static [u8]`
     - `bind_address: String` (or SocketAddr)
     - Validator identity (ed25519 keypair or signer)
     - Known validators list
  3. Remove the old `FnOnce` starter closure pattern from CommonwareEngine. The engine now owns the full construction.
  4. Update `crates/consensus-simplex/src/lib.rs` exports to include new public types
  5. Write tests for the new wiring: engine construction, start, status check, shutdown
  6. Ensure `cargo build -p consensus-simplex` and `cargo test -p consensus-simplex` pass

  **Must NOT do**:
  - Do NOT change the `ConsensusEngine` trait in consensus crate
  - Do NOT modify vendor code
  - Do NOT hardcode node-specific values (namespace, addresses) — accept as parameters
  - Do NOT break the existing AppAdapter — compose with it

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Core architectural change requiring deep understanding of simplex engine lifecycle, P2P setup, and component wiring
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 4 | Blocked By: 1, 2

  **References**:
  - Current engine: `crates/consensus-simplex/src/engine.rs` (103 lines) — CommonwareEngine wraps starter closure, ConsensusEngine::start() calls it
  - Mailbox (from Task 1): `crates/consensus-simplex/src/mailbox.rs` — Mailbox + MailboxActor to create
  - Sink (from Task 2): `crates/consensus-simplex/src/sink.rs` — FinalizationSink to create
  - AppAdapter: `crates/consensus-simplex/src/adapter.rs` (145 lines) — Bridges ConsensusApp+EventSink to vendor Application+Reporter. Already generic.
  - CommonwareConfig: `crates/consensus-simplex/src/config.rs` — Current config fields for simplex BFT timing/sizes
  - ConsensusEngine trait: `crates/consensus/src/engine.rs` — `trait ConsensusEngine { fn start(self) -> Result<RunningEngine, ConsensusError>; }`
  - RunningEngine: `crates/consensus/src/engine.rs` — Holds shutdown closure, JoinHandle, height atomic, running atomic
  - Vendor simplex engine: `vendor/commonware/consensus/src/simplex/` — The actual BFT engine being wrapped. Look at `engine.rs` for `Engine::new()` and `Engine::start()` signatures
  - Wire stub being replaced: `crates/whirlpool-node/src/wire.rs` (55 lines) — Current stub that polls flag every 100ms
  - Node config for reference: `crates/whirlpool-node/src/config.rs` — NAMESPACE, BLOCK_INTERVAL, BIND_ADDR, VALIDATOR_SEED constants (these are node-level values passed into the new API)

  **Acceptance Criteria**:
  - [ ] `CommonwareEngine` no longer accepts a starter closure — constructs internally
  - [ ] `CommonwareEngine::new()` or builder accepts App + Sink + Config + P2P params
  - [ ] `ConsensusEngine::start()` wires Mailbox, MailboxActor, AppAdapter, simplex engine
  - [ ] `cargo build -p consensus-simplex` succeeds
  - [ ] `cargo test -p consensus-simplex` succeeds
  - [ ] Old wire.rs stub pattern is no longer needed

  **QA Scenarios**:
  ```
  Scenario: Engine builds with new API
    Tool: Bash
    Steps: cargo build -p consensus-simplex 2>&1
    Expected: Compilation succeeds with exit code 0
    Evidence: .sisyphus/evidence/task-3-engine-build.txt

  Scenario: Engine can be constructed and started
    Tool: Bash
    Steps: cargo test -p consensus-simplex engine 2>&1
    Expected: Engine construction, start, status, shutdown tests pass
    Evidence: .sisyphus/evidence/task-3-engine-tests.txt

  Scenario: No starter closure pattern remains
    Tool: Bash
    Steps: grep -n "starter" crates/consensus-simplex/src/engine.rs || echo "CLEAN"
    Expected: Output is "CLEAN" (no starter references)
    Evidence: .sisyphus/evidence/task-3-no-starter.txt
  ```

  **Commit**: YES | Message: `refactor(consensus-simplex): replace starter closure with sealed engine wiring` | Files: `crates/consensus-simplex/src/engine.rs`, `crates/consensus-simplex/src/config.rs`, `crates/consensus-simplex/src/lib.rs`, `crates/consensus-simplex/Cargo.toml`

- [ ] 4. Update whirlpool-node to Use New consensus-simplex API

  **What to do**:
  1. Delete `crates/whirlpool-node/src/mailbox.rs` — code moved to consensus-simplex
  2. Delete `crates/whirlpool-node/src/sink.rs` — code moved to consensus-simplex
  3. Delete `crates/whirlpool-node/src/wire.rs` — replaced by sealed engine wiring
  4. Update `crates/whirlpool-node/src/lib.rs`:
     - Remove `#[cfg(any(test, feature = "never_enable_this"))] mod mailbox;`
     - Remove `#[cfg(any(test, feature = "never_enable_this"))] mod sink;`
     - Remove `pub mod wire;`
     - Keep: `pub mod app;`, `pub mod block;`, `pub mod config;`
  5. If main.rs or any integration point uses `wire::create_starter()`, update to use new `CommonwareEngine` API from consensus-simplex:
     - Import `consensus_simplex::{CommonwareEngine, CommonwareConfig, FinalizationSink}`
     - Construct engine with `EmptyBlockApp`, node config values, P2P params
     - Call `engine.start()` to get `RunningEngine`
  6. Ensure all remaining whirlpool-node tests still pass (EmptyBlock tests in block.rs, EmptyBlockApp tests in app.rs)
  7. Run `cargo build -p whirlpool-node` and `cargo test -p whirlpool-node`

  **Must NOT do**:
  - Do NOT change EmptyBlock or EmptyBlockApp implementations
  - Do NOT change config.rs constants
  - Do NOT add new dependencies to whirlpool-node (it should LOSE deps, not gain them)

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Needs to understand new API from Tasks 1-3 and wire it correctly
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 5, 6 | Blocked By: 1, 2, 3

  **References**:
  - Current lib.rs: `crates/whirlpool-node/src/lib.rs` (11 lines) — Module declarations with cfg gates
  - Current wire.rs to delete: `crates/whirlpool-node/src/wire.rs` (55 lines) — create_starter() stub
  - Current mailbox.rs to delete: `crates/whirlpool-node/src/mailbox.rs` (351 lines)
  - Current sink.rs to delete: `crates/whirlpool-node/src/sink.rs` (138 lines)
  - main.rs: `crates/whirlpool-node/src/main.rs` — Currently just prints "Sahara Chain Binary" (stub)
  - New API from consensus-simplex: `crates/consensus-simplex/src/engine.rs` (updated in Task 3)
  - EmptyBlockApp tests: `crates/whirlpool-node/src/app.rs` — 11 tests that must continue passing
  - EmptyBlock tests: `crates/whirlpool-node/src/block.rs` — 8 tests that must continue passing

  **Acceptance Criteria**:
  - [ ] `crates/whirlpool-node/src/mailbox.rs` does NOT exist
  - [ ] `crates/whirlpool-node/src/sink.rs` does NOT exist
  - [ ] `crates/whirlpool-node/src/wire.rs` does NOT exist
  - [ ] `grep -r "never_enable_this" crates/whirlpool-node/` returns zero matches
  - [ ] `cargo build -p whirlpool-node` succeeds
  - [ ] `cargo test -p whirlpool-node` succeeds (all app + block tests pass)

  **QA Scenarios**:
  ```
  Scenario: whirlpool-node compiles without old modules
    Tool: Bash
    Steps: cargo build -p whirlpool-node 2>&1
    Expected: Compilation succeeds with exit code 0
    Evidence: .sisyphus/evidence/task-4-node-build.txt

  Scenario: All remaining whirlpool-node tests pass
    Tool: Bash
    Steps: cargo test -p whirlpool-node 2>&1
    Expected: All app + block tests pass (19+ tests), zero failures
    Evidence: .sisyphus/evidence/task-4-node-tests.txt

  Scenario: Old files are deleted
    Tool: Bash
    Steps: ls crates/whirlpool-node/src/mailbox.rs crates/whirlpool-node/src/sink.rs crates/whirlpool-node/src/wire.rs 2>&1
    Expected: All three files report "No such file or directory"
    Evidence: .sisyphus/evidence/task-4-files-deleted.txt

  Scenario: No cfg gate remnants
    Tool: Bash
    Steps: grep -r "never_enable_this" crates/whirlpool-node/ || echo "CLEAN"
    Expected: Output is "CLEAN"
    Evidence: .sisyphus/evidence/task-4-no-cfg-gates.txt
  ```

  **Commit**: YES | Message: `refactor(whirlpool-node): remove consensus wiring, consume consensus-simplex API` | Files: `crates/whirlpool-node/src/lib.rs`, `crates/whirlpool-node/src/main.rs` (if updated), deleted: `mailbox.rs`, `sink.rs`, `wire.rs`

- [ ] 5. Dependency Cleanup and Final Workspace Verification

  **What to do**:
  1. Clean up `crates/whirlpool-node/Cargo.toml` — remove dependencies that were only needed by mailbox/sink/wire:
     - Likely removable: `commonware-consensus` (bridging now in consensus-simplex), `commonware-broadcast`, `commonware-runtime`, `commonware-p2p`, `commonware-storage`, `futures`
     - Keep: `consensus`, `consensus-simplex`, `commonware-cryptography` (for EmptyBlock Digestible/Committable), `commonware-codec` (for EmptyBlock Write/Read), `sha2` (for EmptyBlock id computation), `bytes`, `tracing`, `tracing-subscriber`, `tokio`
     - **IMPORTANT**: Before removing each dep, verify no remaining code in whirlpool-node uses it. Run `grep -r "commonware_broadcast" crates/whirlpool-node/src/` etc.
  2. Add any missing dependencies to `crates/consensus-simplex/Cargo.toml` that moved code needs (e.g., `sha2` for compute_digest, additional tokio features)
  3. Run full workspace verification:
     - `cargo build --workspace`
     - `cargo test --workspace`
     - `cargo clippy --workspace -- -D warnings`
  4. Verify no circular dependencies: `cargo tree -p consensus-simplex` should not show whirlpool-node

  **Must NOT do**:
  - Do NOT remove dependencies that are still used by remaining code
  - Do NOT add unnecessary new dependencies

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Straightforward Cargo.toml edits and verification commands
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 6 | Blocked By: 4

  **References**:
  - whirlpool-node Cargo.toml: `crates/whirlpool-node/Cargo.toml` — Current deps include 9 commonware crates + others
  - consensus-simplex Cargo.toml: `crates/consensus-simplex/Cargo.toml` — May need sha2, expanded tokio features
  - Remaining whirlpool-node source: `crates/whirlpool-node/src/{app,block,config,main,lib}.rs` — Check which deps they actually use

  **Acceptance Criteria**:
  - [ ] `cargo build --workspace` succeeds
  - [ ] `cargo test --workspace` succeeds
  - [ ] `cargo clippy --workspace -- -D warnings` passes
  - [ ] No unused dependencies in whirlpool-node (cargo clippy would catch via `unused_extern_crates` or similar)
  - [ ] `cargo tree -p consensus-simplex` shows no whirlpool-node dependency

  **QA Scenarios**:
  ```
  Scenario: Full workspace builds
    Tool: Bash
    Steps: cargo build --workspace 2>&1
    Expected: Compilation succeeds with exit code 0
    Evidence: .sisyphus/evidence/task-5-workspace-build.txt

  Scenario: Full workspace tests pass
    Tool: Bash
    Steps: cargo test --workspace 2>&1
    Expected: All tests pass across all 3 crates
    Evidence: .sisyphus/evidence/task-5-workspace-tests.txt

  Scenario: Clippy clean
    Tool: Bash
    Steps: cargo clippy --workspace -- -D warnings 2>&1
    Expected: Zero warnings, zero errors
    Evidence: .sisyphus/evidence/task-5-clippy.txt

  Scenario: No circular dependencies
    Tool: Bash
    Steps: cargo tree -p consensus-simplex 2>&1 | grep whirlpool || echo "NO_CIRCULAR"
    Expected: Output is "NO_CIRCULAR"
    Evidence: .sisyphus/evidence/task-5-no-circular.txt
  ```

  **Commit**: YES | Message: `chore(whirlpool-node): clean up dependencies after consensus wiring move` | Files: `crates/whirlpool-node/Cargo.toml`, `crates/consensus-simplex/Cargo.toml`

- [ ] 6. Update llmdocs Documentation

  **What to do**:
  1. Use the `ctx-update-doc` skill to regenerate/update llmdocs for both affected crates:
     - consensus-simplex (new modules: mailbox, sink, updated engine)
     - whirlpool-node (removed modules: mailbox, sink, wire)
  2. Verify the updated docs reflect the new architecture

  **Must NOT do**:
  - Do NOT manually write documentation — use the ctx-update-doc skill
  - Do NOT update vendor llmdocs

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Skill-driven doc generation
  - Skills: [`ctx-update-doc`] — Required for llmdocs generation
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: none | Blocked By: 4, 5

  **References**:
  - Existing llmdocs: Found at `llmdocs/guides/implementing-consensus-traits.md`, `llmdocs/guides/whirlpool-node-components.md`, `llmdocs/architecture/whirlpool-node.md`
  - AGENTS.md requirement: "After completing code changes, always use the ctx-update-doc skill to generate/update llmdocs for the affected crates."

  **Acceptance Criteria**:
  - [ ] llmdocs updated for consensus-simplex crate
  - [ ] llmdocs updated for whirlpool-node crate
  - [ ] Docs reflect new architecture (Mailbox/Sink in consensus-simplex, not whirlpool-node)

  **QA Scenarios**:
  ```
  Scenario: llmdocs exist and are current
    Tool: Bash
    Steps: find llmdocs/ -name "*.md" -newer crates/consensus-simplex/src/lib.rs | head -5
    Expected: At least one recently-updated doc file
    Evidence: .sisyphus/evidence/task-6-llmdocs.txt
  ```

  **Commit**: YES | Message: `docs: update llmdocs for consensus wiring refactor` | Files: `llmdocs/**/*.md`

## Final Verification Wave (4 parallel agents, ALL must APPROVE)
- [ ] F1. Plan Compliance Audit — oracle: Verify all tasks were executed as specified, no deviations
- [ ] F2. Code Quality Review — unspecified-high: Review all changed files for code quality, idiomatic Rust, proper error handling
- [ ] F3. Real Manual QA — unspecified-high: Run full build + test + clippy, verify file deletions, verify no EmptyBlock in consensus-simplex
- [ ] F4. Scope Fidelity Check — deep: Verify no scope creep (vendor untouched, consensus traits unchanged, EmptyBlock/App unchanged)

## Commit Strategy
- 5 atomic commits, one per task (except Task 6 docs)
- Each commit is independently buildable (workspace compiles after each)
- Final squash optional at Bob's discretion

## Success Criteria
1. `cargo build --workspace` ✓
2. `cargo test --workspace` ✓ (all existing tests pass)
3. `cargo clippy --workspace -- -D warnings` ✓
4. whirlpool-node has NO mailbox.rs, sink.rs, wire.rs
5. whirlpool-node has NO `never_enable_this` cfg gates
6. consensus-simplex has mailbox.rs, sink.rs with generic types (zero EmptyBlock refs)
7. consensus-simplex provides sealed engine wiring API (no starter closure)
8. Consensus crate traits UNCHANGED
9. Vendor directory UNCHANGED
10. llmdocs updated
