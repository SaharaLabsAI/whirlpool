# Chain Binary — Single-Node Empty-Block Chain

## TL;DR
> **Summary**: Create a new Rust binary crate `crates/chain-binary` that wires `consensus-core` + `consensus-commonware` to produce a single-node chain finalizing empty blocks every 5 seconds.
> **Deliverables**: Working binary, 6 modules (block, app, sink, mailbox, wire, main), TDD test suites, integration test
> **Effort**: Large
> **Parallel**: YES — 3 waves
> **Critical Path**: Task 1 → Task 2 → Tasks 3,4,5 (parallel) → Task 6 → Task 7 → Task 8

## Context

### Original Request
Build the chain binary according to `docs/design/chain-binary/` — a single-node chain finalizing empty blocks every 5 seconds using the `consensus-core` and `consensus-commonware` crates.

### Interview Summary
- **Test strategy**: TDD (Red-Green-Refactor) — tests written first, then implementation
- **Config approach**: Hard-coded solo-validator config (no CLI flags, no config files for v0)
- **Non-goals v0**: tx execution, mempool, dynamic validator sets, RPC beyond health/height

### Metis Review (gaps addressed)
1. **Runner↔Engine bridge**: `Runner::start()` calls `block_on()` — must spawn dedicated OS thread via `std::thread::spawn` to avoid nested tokio panic. Addressed in Task 6.
2. **Automaton/Relay bridge**: `AppAdapter` does NOT implement `Automaton`/`CertifiableAutomaton`/`Relay` — Mailbox bridge pattern required (from vendor `examples/log`). Addressed in Task 5.
3. **Height tracking**: `CommonwareEngine` starter receives `Arc<AtomicU64>` for height but nobody updates it. `FinalizationSink` must share this atomic. Addressed in Task 4.
4. **Single-node BFT viability**: Simplex requires `n >= 3f+1`. With 1 validator, `f=0`, `n=1 >= 1` — valid. No issue.
5. **Block ID reconciliation**: `consensus-core::Block::Id` vs vendor `Digest`/`Commitment` — `EmptyBlock` must implement both. Addressed in Task 2.

## Work Objectives

### Core Objective
A running binary that starts a single-node simplex consensus engine, proposes empty blocks every 5 seconds, finalizes them, and logs finalization events.

### Deliverables
- `crates/chain-binary/Cargo.toml` — crate manifest
- `crates/chain-binary/src/lib.rs` — module declarations and re-exports
- `crates/chain-binary/src/config.rs` — hard-coded constants
- `crates/chain-binary/src/block.rs` — `EmptyBlock` type with dual-trait conformance
- `crates/chain-binary/src/app.rs` — `EmptyBlockApp` implementing `ConsensusApp`
- `crates/chain-binary/src/sink.rs` — `FinalizationSink` implementing `EventSink`
- `crates/chain-binary/src/mailbox.rs` — Mailbox bridge for `Automaton`/`CertifiableAutomaton`/`Relay`
- `crates/chain-binary/src/wire.rs` — Starter closure wiring all components
- `crates/chain-binary/src/main.rs` — Binary entrypoint
- `crates/chain-binary/tests/single_node.rs` — Integration test

### Definition of Done (verifiable conditions)
- `cargo build -p chain-binary` succeeds with zero warnings
- `cargo nextest run -p chain-binary` — all unit tests pass
- `cargo nextest run -p chain-binary --test single_node` — integration test passes
- Running `cargo run -p chain-binary` produces log output showing blocks finalized at ~5s intervals
- `cargo clippy -p chain-binary -- -D warnings` passes

### Must Have
- EmptyBlock implementing both `consensus_core::Block` AND vendor `Block` (Heightable + Codec + Digestible + Committable)
- EmptyBlockApp implementing `ConsensusApp` with 5 verify rules from design doc
- FinalizationSink implementing `EventSink` with height tracking via `Arc<AtomicU64>`
- Mailbox bridge implementing `Automaton`, `CertifiableAutomaton`, `Relay` for simplex engine
- Dedicated OS thread for commonware Runner (avoids nested tokio panic)
- Deterministic signer: `ed25519::PrivateKey::from_seed(0)`
- Graceful shutdown via Ctrl-C / SIGTERM

### Must NOT Have (guardrails)
- No transaction execution or mempool
- No dynamic validator sets — single hard-coded validator only
- No RPC server (beyond what design doc specifies for v0)
- No config files or CLI argument parsing
- No modifications to `vendor/**` code
- No modifications to existing `consensus-core` or `consensus-commonware` crates
- No persistent storage — use temp directory
- No custom networking — `127.0.0.1:0` with OS-assigned port

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: **TDD (Red-Green-Refactor)** using `cargo nextest`
- QA policy: Every task has agent-executed scenarios
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy

### Parallel Execution Waves

**Wave 1** (sequential foundation):
- Task 1: Crate scaffold (`quick`)
- Task 2: EmptyBlock type (`unspecified-high`)

**Wave 2** (parallel after Wave 1):
- Task 3: EmptyBlockApp (`unspecified-high`)
- Task 4: FinalizationSink (`unspecified-high`)
- Task 5: Mailbox bridge (`unspecified-high`)

**Wave 3** (sequential integration):
- Task 6: wire.rs starter closure (`deep`)
- Task 7: main.rs entrypoint (`quick`)
- Task 8: Integration test (`deep`)

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | — | 2,3,4,5,6,7,8 |
| 2 | 1 | 3,4,5,6,7,8 |
| 3 | 2 | 6,7,8 |
| 4 | 2 | 6,7,8 |
| 5 | 2 | 6,7,8 |
| 6 | 3,4,5 | 7,8 |
| 7 | 6 | 8 |
| 8 | 7 | — |

### Agent Dispatch Summary

| Wave | Tasks | Categories |
|------|-------|------------|
| 1 | 2 | quick, unspecified-high |
| 2 | 3 | unspecified-high × 3 |
| 3 | 3 | deep, quick, deep |

## TODOs


- [x] 1. Crate Scaffold

  **What to do**:
  1. Create `crates/chain-binary/Cargo.toml` with:
     ```toml
     [package]
     name = "chain-binary"
     version = "0.1.0"
     edition = "2021"

     [dependencies]
     consensus-core = { path = "../consensus-core" }
     consensus-commonware = { path = "../consensus-commonware" }
     commonware-consensus = { path = "../../vendor/commonware/consensus" }
     commonware-runtime = { path = "../../vendor/commonware/runtime" }
     commonware-p2p = { path = "../../vendor/commonware/p2p" }
     commonware-cryptography = { path = "../../vendor/commonware/cryptography" }
     commonware-codec = { path = "../../vendor/commonware/codec" }
     commonware-utils = { path = "../../vendor/commonware/utils" }
     sha2 = "0.10"
     bytes = "1"
     tracing = "0.1"
     tracing-subscriber = { version = "0.3", features = ["env-filter"] }
     tokio = { version = "1", features = ["full"] }
     futures = "0.3"
     ```
     Check `crates/consensus-commonware/Cargo.toml` for exact dependency versions and paths before writing — use the same versions. Add any missing deps that consensus-commonware uses (e.g., `commonware-storage` if needed).
  2. Create `crates/chain-binary/src/lib.rs` with module declarations:
     ```rust
     pub mod config;
     pub mod block;
     pub mod app;
     pub mod sink;
     pub mod mailbox;
     pub mod wire;
     ```
  3. Create `crates/chain-binary/src/main.rs` with a placeholder `fn main() { println!("chain-binary"); }`
  4. Create stub files for each module (`config.rs`, `block.rs`, `app.rs`, `sink.rs`, `mailbox.rs`, `wire.rs`) — empty or with a comment `// TODO`
  5. Register the crate in root `Cargo.toml` workspace members list — add `"crates/chain-binary"` to the `members` array
  6. Create `crates/chain-binary/src/config.rs` with hard-coded constants:
     ```rust
     use std::time::Duration;

     /// Namespace for the consensus engine (used in signing context)
     pub const NAMESPACE: &[u8] = b"sahara-chain-v0";

     /// Block proposal interval
     pub const BLOCK_INTERVAL: Duration = Duration::from_secs(5);

     /// Network bind address
     pub const BIND_ADDR: &str = "127.0.0.1:0";

     /// Seed for deterministic key generation (solo validator)
     pub const VALIDATOR_SEED: u64 = 0;
     ```
  7. Verify `cargo check -p chain-binary` compiles successfully

  **Must NOT do**:
  - Do not add CLI argument parsing
  - Do not add any runtime logic beyond stubs
  - Do not modify any existing crate

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Simple file scaffolding, no complex logic
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction needed

  **Parallelization**: Can Parallel: NO | Wave 1 (first) | Blocks: 2,3,4,5,6,7,8 | Blocked By: none

  **References**:
  - Pattern: `crates/consensus-commonware/Cargo.toml` — dependency paths and versions to mirror
  - Pattern: `crates/consensus-core/Cargo.toml` — workspace member pattern
  - Pattern: `Cargo.toml` (root) — workspace members list format (use `"crates/chain-binary"`)
  - Config: `docs/design/chain-binary/architecture.md` — module structure
  - Config: `docs/design/chain-binary/empty-block-cadence.md` — 5-second interval

  **Acceptance Criteria**:
  - [ ] `cargo check -p chain-binary` succeeds with zero errors
  - [ ] All 7 source files exist under `crates/chain-binary/src/`
  - [ ] `chain-binary` appears in root `Cargo.toml` members
  - [ ] `config.rs` contains NAMESPACE, BLOCK_INTERVAL, BIND_ADDR, VALIDATOR_SEED constants

  **QA Scenarios**:
  ```
  Scenario: Crate compiles clean
    Tool: Bash
    Steps: cargo check -p chain-binary 2>&1
    Expected: Exit code 0, no errors or warnings
    Evidence: .sisyphus/evidence/task-1-scaffold.txt

  Scenario: Workspace integration
    Tool: Bash
    Steps: cargo metadata --format-version=1 | jq '.packages[] | select(.name=="chain-binary") | .name'
    Expected: Output "chain-binary"
    Evidence: .sisyphus/evidence/task-1-workspace.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): scaffold crate with module stubs and config constants` | Files: `crates/chain-binary/`, `Cargo.toml`, `Cargo.lock`

- [ ] 2. EmptyBlock Type with Dual-Trait Conformance (TDD)

  **What to do**:

  This task uses TDD. Write ALL tests first, confirm they fail, then implement.

  **Phase 1 — RED (write tests first)**:
  Create `crates/chain-binary/src/block.rs` starting with `#[cfg(test)] mod tests { ... }` containing these 8 tests:

  1. `test_genesis_block_has_height_zero` — `EmptyBlock::genesis()` returns block with `height() == 0`
  2. `test_genesis_block_has_zero_parent` — genesis block's `parent_id()` returns `[0u8; 32]`
  3. `test_genesis_block_id_is_deterministic` — two calls to `genesis()` produce same `id()`
  4. `test_child_block_height_increments` — `EmptyBlock::new(1, parent_id)` has `height() == 1`
  5. `test_child_block_references_parent` — `EmptyBlock::new(1, parent_id)` has `parent_id() == parent_id`
  6. `test_codec_roundtrip` — encode then decode produces identical block (vendor `Codec` trait)
  7. `test_digest_deterministic` — same block produces same digest every time (vendor `Digestible`)
  8. `test_different_blocks_different_digests` — blocks at different heights produce different digests

  **Phase 2 — GREEN (implement)**:
  ```rust
  use consensus_core::Block as CoreBlock;
  use commonware_codec::{Codec, ReadBuffer, WriteBuffer, Error as CodecError};
  use commonware_consensus::Digestible;
  use commonware_consensus::Committable;
  use commonware_consensus::Heightable;
  use sha2::{Sha256, Digest as Sha2Digest};

  /// A 32-byte block identifier
  pub type BlockId = [u8; 32];

  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct EmptyBlock {
      height: u64,
      parent_id: BlockId,
  }

  impl EmptyBlock {
      pub fn genesis() -> Self {
          Self { height: 0, parent_id: [0u8; 32] }
      }

      pub fn new(height: u64, parent_id: BlockId) -> Self {
          Self { height, parent_id }
      }

      /// Compute the block's identity hash: SHA-256(height || parent_id)
      fn compute_id(&self) -> BlockId {
          let mut hasher = Sha256::new();
          hasher.update(self.height.to_le_bytes());
          hasher.update(&self.parent_id);
          let result = hasher.finalize();
          let mut id = [0u8; 32];
          id.copy_from_slice(&result);
          id
      }
  }
  ```

  Implement these traits on `EmptyBlock`:

  **`consensus_core::Block`**:
  ```rust
  impl CoreBlock for EmptyBlock {
      type Id = BlockId;
      fn id(&self) -> BlockId { self.compute_id() }
      fn parent_id(&self) -> BlockId { self.parent_id }
      fn height(&self) -> u64 { self.height }
  }
  ```

  **`commonware_codec::Codec`**:
  - `write(&self, buf: &mut impl WriteBuffer)` — write `height` as u64 then `parent_id` as 32 bytes
  - `read_from(buf: &mut impl ReadBuffer) -> Result<Self, CodecError>` — read u64 then 32 bytes
  - Check the exact trait signatures in `vendor/commonware/codec/src/lib.rs` — the method names may be `encode`/`decode` or `write`/`read_from`. Use whatever the trait actually defines.

  **`commonware_consensus::Heightable`**: `fn height(&self) -> u64 { self.height }`
  Note: This may conflict with CoreBlock::height(). Check if Heightable is defined in `vendor/commonware/consensus/src/lib.rs`. If there's a conflict, you may need a wrapper or explicit trait disambiguation. The TestBlock in `crates/consensus-commonware/src/tests.rs` shows how this is handled.

  **`commonware_consensus::Digestible`**: `fn digest(&self) -> Digest` — return `self.compute_id()`. Check the exact `Digest` type in `vendor/commonware/consensus/src/lib.rs` — likely `Bytes` or a newtype. The TestBlock in `crates/consensus-commonware/src/tests.rs` shows the exact pattern.

  **`commonware_consensus::Committable`**: `fn parent(&self) -> Commitment` — Check the exact `Commitment` type. Likely wraps the parent digest. Follow TestBlock pattern in `crates/consensus-commonware/src/tests.rs`.

  **Phase 3 — REFACTOR**: Ensure all 8 tests pass, clean up.

  **Must NOT do**:
  - Do not add any consensus logic (propose, verify) — that's Task 3
  - Do not implement `Automaton` or `Relay` on this type
  - Do not add serialization beyond `Codec`

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Complex trait implementation with dual conformance, TDD discipline, potential trait conflicts
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: NO | Wave 1 (after Task 1) | Blocks: 3,4,5,6,7,8 | Blocked By: 1

  **References**:
  - Pattern: `crates/consensus-core/src/block.rs` — `Block` trait definition (exact signatures: `type Id: Copy+Eq+Hash+Debug+Send+Sync+'static; fn id(&self)->Self::Id; fn parent_id(&self)->Self::Id; fn height(&self)->u64`)
  - Pattern: `crates/consensus-commonware/src/tests.rs` — `TestBlock` implementation showing dual-trait conformance (THIS IS THE PRIMARY REFERENCE — follow it exactly)
  - Pattern: `crates/consensus-commonware/src/types.rs` — `CommonwareBlock` blanket impl requires `T: CoreBlock + VendorBlock + Clone` where `VendorBlock` = `Heightable + Codec + Digestible + Committable`
  - API: `vendor/commonware/consensus/src/lib.rs` — vendor `Block`, `Heightable`, `Digestible`, `Committable` trait definitions
  - API: `vendor/commonware/codec/src/lib.rs` — `Codec` trait definition (exact method signatures)
  - Design: `docs/design/chain-binary/empty-block-cadence.md` — EmptyBlock spec

  **Acceptance Criteria**:
  - [ ] All 8 unit tests pass: `cargo nextest run -p chain-binary block::tests`
  - [ ] `EmptyBlock` implements `consensus_core::Block<Id=BlockId>`
  - [ ] `EmptyBlock` implements `Codec`, `Heightable`, `Digestible`, `Committable`
  - [ ] `EmptyBlock` satisfies `CommonwareBlock` blanket impl (compiles as `CommonwareBlock`)
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: TDD Red phase — tests fail before implementation
    Tool: Bash
    Steps: Write test module first, run cargo nextest run -p chain-binary block::tests 2>&1
    Expected: All 8 tests fail (compile error or assertion failure)
    Evidence: .sisyphus/evidence/task-2-red.txt

  Scenario: TDD Green phase — all tests pass after implementation
    Tool: Bash
    Steps: Implement EmptyBlock, run cargo nextest run -p chain-binary block::tests 2>&1
    Expected: All 8 tests pass, exit code 0
    Evidence: .sisyphus/evidence/task-2-green.txt

  Scenario: Dual-trait verification
    Tool: Bash
    Steps: Add a compile-time assertion in tests: `fn assert_commonware_block<T: consensus_commonware::types::CommonwareBlock>() {}; assert_commonware_block::<EmptyBlock>();`
    Expected: Compiles successfully
    Evidence: .sisyphus/evidence/task-2-dual-trait.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): implement EmptyBlock with dual-trait conformance (TDD)` | Files: `crates/chain-binary/src/block.rs`


- [ ] 3. EmptyBlockApp — ConsensusApp Implementation (TDD)

  **What to do**:

  TDD approach. Write tests first, confirm failure, then implement.

  **Phase 1 — RED (write 11 tests)**:
  Create `crates/chain-binary/src/app.rs` with test module containing:

  1. `test_genesis_returns_empty_block_at_height_zero` — `app.genesis().await.height() == 0`
  2. `test_propose_returns_block_at_correct_height` — `app.propose(&genesis, 1).await` returns `Some(block)` with `height == 1`
  3. `test_propose_references_parent` — proposed block's `parent_id()` equals parent's `id()`
  4. `test_verify_valid_block_succeeds` — `app.verify(&parent, &valid_child).await` returns `Ok(())`
  5. `test_verify_wrong_height_fails` — block with `height != expected` returns `Err(InvalidBlock("height mismatch..."))`
  6. `test_verify_wrong_parent_fails` — block with wrong `parent_id` returns `Err(InvalidBlock("parent mismatch..."))`
  7. `test_verify_genesis_height_nonzero_fails` — block at height 0 but not genesis returns `Err(InvalidBlock(...))`
  8. `test_verify_self_referencing_fails` — block whose `id() == parent_id()` returns error (except genesis)
  9. `test_verify_future_height_fails` — block with height far beyond parent+1 returns error
  10. `test_propose_after_propose_increments` — two sequential proposals yield heights 1, 2
  11. `test_genesis_is_valid_self_referentially` — `verify(&genesis, &genesis_child)` succeeds

  **Phase 2 — GREEN (implement)**:
  ```rust
  use consensus_core::{ConsensusApp, ConsensusError};
  use crate::block::{EmptyBlock, BlockId};

  pub struct EmptyBlockApp;

  impl EmptyBlockApp {
      pub fn new() -> Self { Self }
  }

  #[async_trait::async_trait]
  impl ConsensusApp for EmptyBlockApp {
      type Block = EmptyBlock;

      async fn genesis(&self) -> EmptyBlock {
          EmptyBlock::genesis()
      }

      async fn propose(&self, parent: &EmptyBlock, height: u64) -> Option<EmptyBlock> {
          Some(EmptyBlock::new(height, parent.id()))
      }

      async fn verify(&self, parent: &EmptyBlock, block: &EmptyBlock) -> Result<(), ConsensusError> {
          // Rule 1: Height must be parent.height() + 1
          if block.height() != parent.height() + 1 {
              return Err(ConsensusError::InvalidBlock(
                  format!("height mismatch: expected {}, got {}", parent.height() + 1, block.height())
              ));
          }
          // Rule 2: parent_id must match parent's id
          if block.parent_id() != parent.id() {
              return Err(ConsensusError::InvalidBlock(
                  format!("parent mismatch: expected {:?}, got {:?}", parent.id(), block.parent_id())
              ));
          }
          // Rule 3: Block cannot self-reference (id == parent_id) unless genesis
          if block.id() == block.parent_id() && block.height() != 0 {
              return Err(ConsensusError::InvalidBlock(
                  "non-genesis block self-references".to_string()
              ));
          }
          // Rule 4: Height must not be zero for non-genesis
          if block.height() == 0 && block.parent_id() != [0u8; 32] {
              return Err(ConsensusError::InvalidBlock(
                  "height 0 with non-zero parent".to_string()
              ));
          }
          // Rule 5: Block at height 0 must have zero parent_id (genesis check)
          // (covered by rules above)
          Ok(())
      }
  }
  ```

  Note: Check whether `ConsensusApp` uses `#[async_trait]` or native async trait syntax. Look at `crates/consensus-core/src/app.rs` for the exact definition. If it doesn't use `async_trait`, you don't need the `async_trait` dependency.

  **Phase 3 — REFACTOR**: All 11 tests green, clean up.

  **Must NOT do**:
  - Do not add any state to `EmptyBlockApp` (it is stateless)
  - Do not implement any block storage or caching
  - Do not add transaction-related logic

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: TDD with 11 tests, 5 verify rules, needs careful error handling
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: YES (with Tasks 4, 5) | Wave 2 | Blocks: 6,7,8 | Blocked By: 2

  **References**:
  - Pattern: `crates/consensus-core/src/app.rs` — `ConsensusApp` trait definition (exact method signatures, async trait syntax)
  - Pattern: `crates/consensus-core/src/error.rs` — `ConsensusError::InvalidBlock(String)` variant
  - Pattern: `crates/consensus-core/src/mock/block.rs` — `MockBlock` as simple reference
  - Pattern: `crates/consensus-commonware/src/tests.rs` — how `TestApp` implements `ConsensusApp`
  - Design: `docs/design/chain-binary/empty-block-cadence.md` — verification rules (section on block validation)
  - Type: `crate::block::EmptyBlock` — from Task 2

  **Acceptance Criteria**:
  - [ ] All 11 unit tests pass: `cargo nextest run -p chain-binary app::tests`
  - [ ] `EmptyBlockApp` implements `ConsensusApp<Block=EmptyBlock>`
  - [ ] All 5 verify rules produce correct `ConsensusError::InvalidBlock` messages
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: TDD Red phase
    Tool: Bash
    Steps: Write test module, run cargo nextest run -p chain-binary app::tests 2>&1
    Expected: All 11 tests fail
    Evidence: .sisyphus/evidence/task-3-red.txt

  Scenario: TDD Green phase
    Tool: Bash
    Steps: Implement EmptyBlockApp, run cargo nextest run -p chain-binary app::tests 2>&1
    Expected: All 11 tests pass, exit code 0
    Evidence: .sisyphus/evidence/task-3-green.txt

  Scenario: Verify rule coverage
    Tool: Bash
    Steps: Run tests 5-9 individually to confirm each verify rule triggers correct error
    Expected: Each test shows InvalidBlock with descriptive message
    Evidence: .sisyphus/evidence/task-3-verify-rules.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): implement EmptyBlockApp with 5 verify rules (TDD)` | Files: `crates/chain-binary/src/app.rs`

- [ ] 4. FinalizationSink — EventSink Implementation (TDD)

  **What to do**:

  TDD approach. Tests first.

  **Phase 1 — RED (write 6 tests)**:
  Create `crates/chain-binary/src/sink.rs` with test module:

  1. `test_handle_finalized_logs_height` — `handle(Finalized { block, height: 1, proof: vec![] })` succeeds
  2. `test_handle_finalized_updates_atomic_height` — After handling `Finalized` with height 5, the shared `Arc<AtomicU64>` reads 5
  3. `test_handle_prefinalized_is_noop` — `handle(PreFinalized { ... })` succeeds without updating height
  4. `test_handle_fault_logs_warning` — `handle(Fault { ... })` succeeds (logs but doesn't panic)
  5. `test_height_monotonically_increases` — Handle finalized at heights 1, 2, 3 — atomic reads 3
  6. `test_initial_height_is_zero` — Before any events, atomic reads 0

  **Phase 2 — GREEN (implement)**:
  ```rust
  use std::sync::Arc;
  use std::sync::atomic::{AtomicU64, Ordering};
  use consensus_core::{EventSink, ConsensusEvent};
  use crate::block::EmptyBlock;
  use tracing::{info, warn};

  pub struct FinalizationSink {
      height: Arc<AtomicU64>,
  }

  impl FinalizationSink {
      pub fn new(height: Arc<AtomicU64>) -> Self {
          Self { height }
      }

      pub fn current_height(&self) -> u64 {
          self.height.load(Ordering::SeqCst)
      }
  }

  #[async_trait::async_trait]  // or native async — check EventSink definition
  impl EventSink for FinalizationSink {
      type Block = EmptyBlock;

      async fn handle(&self, event: ConsensusEvent<EmptyBlock>) {
          match event {
              ConsensusEvent::Finalized { block, height, proof } => {
                  self.height.store(height, Ordering::SeqCst);
                  info!(height = height, block_id = ?block.id(), "block finalized");
              }
              ConsensusEvent::PreFinalized { block, height } => {
                  info!(height = height, "block pre-finalized");
              }
              ConsensusEvent::Fault { offender, evidence } => {
                  warn!(?offender, "consensus fault detected");
              }
          }
      }
  }
  ```

  Note: Check the exact `ConsensusEvent` variants and their field names in `crates/consensus-core/src/event.rs`. The proof field type may be specific (e.g., `Vec<u8>` or a generic). Adapt accordingly.

  Also check whether `EventSink` uses `async_trait` or native async — see `crates/consensus-core/src/app.rs` or `crates/consensus-core/src/event.rs` for the trait definition.

  **Phase 3 — REFACTOR**: All 6 tests green.

  **Must NOT do**:
  - Do not persist finalization data to disk
  - Do not add block storage
  - Do not panic on faults — log only

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: TDD, shared atomic state, async trait implementation
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: YES (with Tasks 3, 5) | Wave 2 | Blocks: 6,7,8 | Blocked By: 2

  **References**:
  - Pattern: `crates/consensus-core/src/event.rs` — `ConsensusEvent` enum (exact variants: `Finalized{block,height,proof}`, `PreFinalized{block,height}`, `Fault{offender,evidence}`)
  - Pattern: `crates/consensus-core/src/app.rs` — `EventSink` trait definition (exact: `type Block: Block; async fn handle(&self, event: ConsensusEvent<Self::Block>)`)
  - Pattern: `crates/consensus-commonware/src/engine.rs` — how `CommonwareEngine` starter receives `Arc<AtomicU64>` for height tracking
  - Type: `crate::block::EmptyBlock` — from Task 2

  **Acceptance Criteria**:
  - [ ] All 6 unit tests pass: `cargo nextest run -p chain-binary sink::tests`
  - [ ] `FinalizationSink` implements `EventSink<Block=EmptyBlock>`
  - [ ] Height atomic is updated on `Finalized` events
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: TDD Red phase
    Tool: Bash
    Steps: Write test module, run cargo nextest run -p chain-binary sink::tests 2>&1
    Expected: All 6 tests fail
    Evidence: .sisyphus/evidence/task-4-red.txt

  Scenario: TDD Green phase
    Tool: Bash
    Steps: Implement FinalizationSink, run cargo nextest run -p chain-binary sink::tests 2>&1
    Expected: All 6 tests pass
    Evidence: .sisyphus/evidence/task-4-green.txt

  Scenario: Atomic height tracking
    Tool: Bash
    Steps: Run test_height_monotonically_increases specifically
    Expected: After 3 finalized events, atomic reads 3
    Evidence: .sisyphus/evidence/task-4-atomic.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): implement FinalizationSink with height tracking (TDD)` | Files: `crates/chain-binary/src/sink.rs`


- [ ] 5. Mailbox Bridge — Automaton/CertifiableAutomaton/Relay (TDD)

  **What to do**:

  TDD approach. The simplex engine requires types implementing `Automaton`, `CertifiableAutomaton`, and `Relay`. The `AppAdapter` does NOT implement these. We need a Mailbox bridge, following the pattern from `vendor/commonware/examples/log/src/application/ingress.rs`.

  **Phase 1 — RED (write 6 tests)**:
  Create `crates/chain-binary/src/mailbox.rs` with test module:

  1. `test_propose_sends_message_and_receives_digest` — calling `propose(ctx)` returns a oneshot receiver that resolves to a digest
  2. `test_verify_valid_payload_returns_true` — `verify(ctx, payload)` returns true for a valid encoded block
  3. `test_verify_invalid_payload_returns_false` — `verify(ctx, garbage_bytes)` returns false
  4. `test_genesis_returns_deterministic_digest` — `genesis(epoch)` returns same digest every time
  5. `test_relay_broadcast_completes` — `broadcast(payload)` completes without error (no-op for single node)
  6. `test_mailbox_clone_shares_channel` — Two clones of Mailbox share the same sender

  **Phase 2 — GREEN (implement)**:

  Study `vendor/commonware/examples/log/src/application/ingress.rs` carefully. The pattern is:

  ```rust
  use commonware_consensus::{Automaton, CertifiableAutomaton, Relay};
  use commonware_consensus::types::{Digest, Epoch};
  use tokio::sync::{mpsc, oneshot};

  // Message enum for the actor channel
  enum Message {
      Genesis { epoch: Epoch, response: oneshot::Sender<Digest> },
      Propose { /* context fields */, response: oneshot::Sender<Digest> },
      Verify { /* payload */, response: oneshot::Sender<bool> },
  }

  #[derive(Clone)]
  pub struct Mailbox {
      sender: mpsc::Sender<Message>,
  }

  impl Mailbox {
      pub fn new(sender: mpsc::Sender<Message>) -> Self {
          Self { sender }
      }
  }
  ```

  Implement `Automaton` for `Mailbox`:
  - `type Context` — check vendor trait for exact associated type
  - `type Digest` — the digest type (likely `Bytes` or `[u8; 32]`)
  - `fn genesis(&mut self, epoch: Epoch) -> Digest` — send Genesis message, await response
  - `fn propose(&mut self, ctx: Context) -> oneshot::Receiver<Digest>` — send Propose, return receiver
  - `fn verify(&mut self, ctx: Context, payload: &[u8]) -> oneshot::Receiver<bool>` — send Verify, return receiver

  Implement `CertifiableAutomaton` for `Mailbox`:
  - Uses default `certify()` impl (always returns true) — just declare the impl

  Implement `Relay` for `Mailbox`:
  - `type Digest` — same as Automaton
  - `fn broadcast(&mut self, payload: &[u8]) -> impl Future<Output=()>` — no-op for single node, just return `async {}`

  **CRITICAL**: Check the EXACT trait definitions in `vendor/commonware/consensus/src/lib.rs`. The method signatures shown above are approximate. The actual signatures may differ (e.g., `&mut self` vs `&self`, return types, etc.). The `examples/log/src/application/ingress.rs` file is the authoritative implementation reference.

  Also create a `MailboxActor` that runs as a spawned task, receives messages from the channel, and delegates to `AppAdapter`:
  ```rust
  pub struct MailboxActor<A, S, B, Sig> {
      receiver: mpsc::Receiver<Message>,
      adapter: AppAdapter<A, S, B, Sig>,
  }
  ```

  The actor bridges between the low-level `Automaton`/`Relay` messages and the `AppAdapter` which implements the higher-level `Application`/`VerifyingApplication`/`Reporter` traits. Look at how `ingress.rs` delegates to the application.

  However, for v0 with empty blocks, the actor can be simpler:
  - `Genesis`: compute `EmptyBlock::genesis()`, encode it, return digest
  - `Propose`: create `EmptyBlock::new(height, parent_id)`, encode, return digest
  - `Verify`: decode payload as `EmptyBlock`, verify fields, return bool

  The actor needs access to the current height. It can receive this from the shared `Arc<AtomicU64>` or maintain its own state.

  **Phase 3 — REFACTOR**: All 6 tests green.

  **Must NOT do**:
  - Do not modify `AppAdapter` in `consensus-commonware`
  - Do not implement networking logic in the mailbox (Relay::broadcast is no-op)
  - Do not add persistence

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Complex trait bridging, actor pattern, async channels, must match vendor trait signatures exactly
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: YES (with Tasks 3, 4) | Wave 2 | Blocks: 6,7,8 | Blocked By: 2

  **References**:
  - Pattern: `vendor/commonware/examples/log/src/application/ingress.rs` — **PRIMARY REFERENCE** — Mailbox pattern with Automaton/CertifiableAutomaton/Relay implementations
  - API: `vendor/commonware/consensus/src/lib.rs` — `Automaton` trait (exact: `type Context; type Digest: Digest; fn genesis(&mut self, epoch) -> Self::Digest; fn propose(&mut self, ctx) -> Receiver<Self::Digest>; fn verify(&mut self, ctx, payload) -> Receiver<bool>`), `CertifiableAutomaton` trait, `Relay` trait
  - API: `vendor/commonware/consensus/src/types.rs` — `Digest`, `Epoch`, `Height` type definitions
  - Pattern: `crates/consensus-commonware/src/adapter.rs` — `AppAdapter` struct (what it implements vs what the mailbox must implement)
  - Type: `crate::block::EmptyBlock` — from Task 2

  **Acceptance Criteria**:
  - [ ] All 6 unit tests pass: `cargo nextest run -p chain-binary mailbox::tests`
  - [ ] `Mailbox` implements `Automaton`, `CertifiableAutomaton`, `Relay`
  - [ ] `MailboxActor` correctly bridges messages to block operations
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: TDD Red phase
    Tool: Bash
    Steps: Write test module, run cargo nextest run -p chain-binary mailbox::tests 2>&1
    Expected: All 6 tests fail
    Evidence: .sisyphus/evidence/task-5-red.txt

  Scenario: TDD Green phase
    Tool: Bash
    Steps: Implement Mailbox + MailboxActor, run cargo nextest run -p chain-binary mailbox::tests 2>&1
    Expected: All 6 tests pass
    Evidence: .sisyphus/evidence/task-5-green.txt

  Scenario: Trait conformance
    Tool: Bash
    Steps: Add compile-time check: `fn assert_automaton<T: commonware_consensus::Automaton>() {}; assert_automaton::<Mailbox>();`
    Expected: Compiles
    Evidence: .sisyphus/evidence/task-5-traits.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): implement Mailbox bridge for simplex Automaton/Relay (TDD)` | Files: `crates/chain-binary/src/mailbox.rs`

- [ ] 6. wire.rs — Starter Closure Wiring All Components

  **What to do**:

  Create `crates/chain-binary/src/wire.rs` containing a `create_starter()` function that returns the starter closure expected by `CommonwareEngine::new()`.

  The starter closure signature (from `crates/consensus-commonware/src/engine.rs`):
  ```rust
  FnOnce(Arc<AtomicU64>, Arc<AtomicBool>) -> Result<(Box<dyn FnOnce() + Send>, JoinHandle<Result<(), ConsensusError>>), ConsensusError>
  ```

  The closure receives:
  - `Arc<AtomicU64>` — current height (shared with FinalizationSink)
  - `Arc<AtomicBool>` — shutdown signal

  Inside the closure, wire everything together:

  1. **Create signer**: `ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED)` — check exact API in `vendor/commonware/cryptography/`
  2. **Create validators list**: `vec![signer.public_key()]` — single validator
  3. **Create scheme**: `Scheme::signer(config::NAMESPACE, validators.clone(), signer)` — check exact constructor in `vendor/commonware/consensus/src/simplex/`
  4. **Create P2P network**:
     ```rust
     let p2p_config = commonware_p2p::authenticated::Config {
         // Check exact fields in vendor/commonware/p2p/src/authenticated/
         // Key fields: bind address, bootstrappers (empty vec), allowed peers
     };
     ```
     The P2P setup is complex. Study `vendor/commonware/examples/log/src/main.rs` lines where P2P is configured. Key points:
     - Use `commonware_p2p::authenticated::Network` (or similar)
     - Bind to `127.0.0.1:0`
     - No bootstrappers (empty vec)
     - Register the single validator as allowed peer
  5. **Create channels**: `mpsc::channel` for Mailbox actor communication
  6. **Create Mailbox** and **MailboxActor** from Task 5
  7. **Create AppAdapter**:
     ```rust
     AppAdapter::new(app, sink)  // Check exact constructor
     ```
     Where `app = EmptyBlockApp::new()` and `sink = FinalizationSink::new(height.clone())`
  8. **Create simplex Config**:
     ```rust
     commonware_consensus::simplex::Config {
         // Check exact fields in vendor/commonware/consensus/src/simplex/
         // Key: namespace, leader strategy (RoundRobin), sequencing (Sequential)
     }
     ```
  9. **Create simplex Engine**:
     ```rust
     simplex::Engine::new(runtime_context, simplex_config, scheme, mailbox, relay, reporter)
     ```
     Check the exact generic params and constructor in `vendor/commonware/consensus/src/simplex/engine.rs`.
  10. **Create Runner and start**:
      ```rust
      let runner = commonware_runtime::tokio::Runner::new();
      // Start in dedicated OS thread
      let handle = std::thread::spawn(move || {
          runner.start(async move {
              // Start P2P network
              // Start mailbox actor
              // Start simplex engine
              // Await shutdown signal
          })
      });
      ```
  11. Return `(stop_fn, join_handle)` where:
      - `stop_fn`: Sets the `AtomicBool` shutdown flag to true
      - `join_handle`: The OS thread's `JoinHandle` (may need wrapping to match expected type)

  **CRITICAL NOTES**:
  - The `Runner::start()` method calls `block_on()` internally — it MUST run in a dedicated OS thread, not inside an existing tokio runtime.
  - Study `vendor/commonware/examples/log/src/main.rs` extensively — it shows the complete wiring pattern.
  - The P2P layer is the most complex part. If the vendor examples show a simpler networking option for single-node (e.g., in-memory channels), prefer that.
  - Check `CommonwareEngine::new()` in `crates/consensus-commonware/src/engine.rs` to understand exactly what `create_starter` must return.

  **Must NOT do**:
  - Do not add CLI argument parsing
  - Do not add persistent storage
  - Do not modify any vendor or existing crate code
  - Do not use tokio::spawn for the Runner — must be std::thread::spawn

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Most complex task, requires deep understanding of vendor internals, P2P setup, async runtime bridging, multi-component wiring
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: NO | Wave 3 (first) | Blocks: 7, 8 | Blocked By: 3, 4, 5

  **References**:
  - Pattern: `vendor/commonware/examples/log/src/main.rs` — **PRIMARY REFERENCE** — complete wiring of P2P + simplex + application
  - Pattern: `crates/consensus-commonware/src/engine.rs` — `CommonwareEngine::new(starter)` — what the starter closure must return
  - API: `vendor/commonware/consensus/src/simplex/engine.rs` — `Engine::new()` constructor, generic params
  - API: `vendor/commonware/runtime/src/tokio/runtime.rs` — `Runner::new()`, `Runner::start()` signatures
  - API: `vendor/commonware/p2p/src/authenticated/` — P2P config and network setup
  - API: `vendor/commonware/cryptography/` — ed25519 key generation
  - API: `vendor/commonware/consensus/src/simplex/` — `Config`, `Strategy::Sequential`, `Leader::RoundRobin` (verify exact names)
  - Config: `crate::config` — NAMESPACE, BLOCK_INTERVAL, BIND_ADDR, VALIDATOR_SEED
  - Type: `crate::block::EmptyBlock` — from Task 2
  - Type: `crate::app::EmptyBlockApp` — from Task 3
  - Type: `crate::sink::FinalizationSink` — from Task 4
  - Type: `crate::mailbox::{Mailbox, MailboxActor}` — from Task 5

  **Acceptance Criteria**:
  - [ ] `create_starter()` compiles and returns the correct closure type
  - [ ] `cargo check -p chain-binary` succeeds with zero errors
  - [ ] Closure correctly wires: signer, validators, scheme, P2P, channels, mailbox, adapter, simplex engine, runner
  - [ ] Runner spawns in dedicated OS thread (not tokio::spawn)
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: Compilation check
    Tool: Bash
    Steps: cargo check -p chain-binary 2>&1
    Expected: Zero errors, zero warnings
    Evidence: .sisyphus/evidence/task-6-check.txt

  Scenario: Type conformance
    Tool: Bash
    Steps: Verify create_starter return type matches CommonwareEngine::new() expectation by successful cargo check
    Expected: Compiles without type errors
    Evidence: .sisyphus/evidence/task-6-types.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): wire starter closure with P2P, simplex engine, and mailbox bridge` | Files: `crates/chain-binary/src/wire.rs`


- [ ] 7. main.rs — Binary Entrypoint

  **What to do**:

  Replace the placeholder `main.rs` with the actual binary entrypoint.

  ```rust
  use std::sync::Arc;
  use std::sync::atomic::{AtomicU64, AtomicBool};
  use consensus_core::ConsensusEngine;
  use consensus_commonware::CommonwareEngine;
  use tracing_subscriber::EnvFilter;

  fn main() {
      // 1. Initialize tracing
      tracing_subscriber::fmt()
          .with_env_filter(
              EnvFilter::try_from_default_env()
                  .unwrap_or_else(|_| EnvFilter::new("info"))
          )
          .init();

      tracing::info!("chain-binary starting");

      // 2. Create CommonwareEngine with starter from wire.rs
      let engine = CommonwareEngine::new(chain_binary::wire::create_starter());

      // 3. Start the engine
      let running = engine.start().expect("failed to start consensus engine");

      tracing::info!("consensus engine started, press Ctrl-C to stop");

      // 4. Wait for Ctrl-C
      // Use std::sync channels or signal handling
      // Check if CommonwareEngine/RunningEngine has a wait/join method
      // Or use ctrlc crate if available
      // Simplest: block on a signal
      let (tx, rx) = std::sync::mpsc::channel();
      ctrlc::set_handler(move || {
          let _ = tx.send(());
      }).expect("failed to set Ctrl-C handler");

      rx.recv().expect("failed to receive shutdown signal");

      tracing::info!("shutting down...");
      // running.stop() or drop(running) — check RunningEngine API
  }
  ```

  **CRITICAL NOTES**:
  - Check if `ctrlc` crate is needed — if so, add it to `Cargo.toml` dependencies
  - Check `RunningEngine` API in `crates/consensus-core/src/engine.rs` — how to stop/wait
  - The `CommonwareEngine::new()` takes the starter closure, and `start()` returns `Result<RunningEngine, ConsensusError>`
  - Check `RunningEngine` struct — it likely has `stop()` or implements `Drop`
  - If `ctrlc` is not available, use `tokio::signal::ctrl_c()` with a tokio runtime for signal handling only (the consensus engine runs in its own thread)

  **Must NOT do**:
  - Do not add CLI argument parsing
  - Do not add RPC server
  - Do not add config file loading

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Simple main function wiring, most logic is in wire.rs
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: NO | Wave 3 (after Task 6) | Blocks: 8 | Blocked By: 6

  **References**:
  - Pattern: `vendor/commonware/examples/log/src/main.rs` — how the example binary starts and stops
  - API: `crates/consensus-core/src/engine.rs` — `ConsensusEngine::start()`, `RunningEngine` struct
  - API: `crates/consensus-commonware/src/engine.rs` — `CommonwareEngine::new(starter)` constructor
  - Type: `crate::wire::create_starter` — from Task 6

  **Acceptance Criteria**:
  - [ ] `cargo build -p chain-binary` succeeds
  - [ ] Binary starts and logs "chain-binary starting"
  - [ ] Binary handles Ctrl-C gracefully
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: Binary builds
    Tool: Bash
    Steps: cargo build -p chain-binary 2>&1
    Expected: Successful build, exit code 0
    Evidence: .sisyphus/evidence/task-7-build.txt

  Scenario: Binary starts and stops
    Tool: Bash
    Steps: timeout 10 cargo run -p chain-binary 2>&1 || true
    Expected: Output contains "chain-binary starting" and "consensus engine started"
    Evidence: .sisyphus/evidence/task-7-run.txt
  ```

  **Commit**: YES | Message: `feat(chain-binary): implement main entrypoint with signal handling` | Files: `crates/chain-binary/src/main.rs`

- [ ] 8. Integration Test — Single-Node Finalization

  **What to do**:

  Create `crates/chain-binary/tests/single_node.rs` — an integration test that starts the full engine, waits for at least 2 blocks to finalize, then shuts down.

  ```rust
  use std::sync::Arc;
  use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
  use std::time::Duration;
  use consensus_core::ConsensusEngine;
  use consensus_commonware::CommonwareEngine;

  #[test]
  fn test_single_node_finalizes_blocks() {
      // 1. Initialize tracing for test output
      let _ = tracing_subscriber::fmt()
          .with_test_writer()
          .try_init();

      // 2. Create and start engine
      let engine = CommonwareEngine::new(chain_binary::wire::create_starter());
      let running = engine.start().expect("engine should start");

      // 3. Get reference to height atomic (need to expose this somehow)
      // Option A: create_starter returns Arc<AtomicU64> along with the closure
      // Option B: RunningEngine exposes height
      // Check what CommonwareEngine/RunningEngine provides

      // 4. Wait for at least 2 blocks (with timeout)
      let start = std::time::Instant::now();
      let timeout = Duration::from_secs(30); // 5s interval × 2 blocks + margin

      loop {
          if start.elapsed() > timeout {
              panic!("timeout waiting for block finalization");
          }
          // Check height somehow — this depends on the API
          // If height >= 2, break
          std::thread::sleep(Duration::from_millis(500));
      }

      // 5. Shutdown
      // running.stop() or equivalent

      // 6. Assert height >= 2
  }
  ```

  **CRITICAL DESIGN DECISION**: The test needs to observe the finalized height. Options:
  - **Option A (preferred)**: Modify `create_starter()` to accept and return a shared `Arc<AtomicU64>` that the test can poll. This means `create_starter` should take `Arc<AtomicU64>` as parameter rather than creating it internally.
  - **Option B**: Expose height through `RunningEngine` — check if this API exists.
  - **Option C**: Parse log output for finalization messages — fragile, avoid.

  The implementer should use **Option A**: modify `wire::create_starter()` to accept an external `Arc<AtomicU64>` that gets shared with `FinalizationSink`. This way the test can poll it. If Task 6's implementation already takes this as a parameter, no change needed. If not, adjust `wire.rs` to accept it.

  **Must NOT do**:
  - Do not use sleep-based timing assertions (use polling with timeout)
  - Do not test more than 2-3 blocks (keep test fast)
  - Do not modify any code outside `chain-binary` crate

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Full system integration, async runtime management, timing-sensitive assertions, may need to adjust wire.rs API
  - Skills: [] — no special skills needed
  - Omitted: [`playwright`] — no browser interaction

  **Parallelization**: Can Parallel: NO | Wave 3 (last) | Blocks: none | Blocked By: 7

  **References**:
  - Pattern: `crates/consensus-commonware/src/tests.rs` — how CommonwareEngine is tested (integration test patterns)
  - API: `crates/consensus-core/src/engine.rs` — `RunningEngine` API (stop/join)
  - API: `crate::wire::create_starter` — from Task 6 (may need API adjustment)
  - Design: `docs/design/chain-binary/empty-block-cadence.md` — 5-second finalization interval

  **Acceptance Criteria**:
  - [ ] `cargo nextest run -p chain-binary --test single_node` passes
  - [ ] Test verifies at least 2 blocks finalized
  - [ ] Test completes within 30 seconds
  - [ ] No flaky timing issues (uses polling, not fixed sleep)
  - [ ] `cargo clippy -p chain-binary -- -D warnings` passes

  **QA Scenarios**:
  ```
  Scenario: Integration test passes
    Tool: Bash
    Steps: cargo nextest run -p chain-binary --test single_node 2>&1
    Expected: 1 test passed, exit code 0
    Evidence: .sisyphus/evidence/task-8-integration.txt

  Scenario: Blocks finalize at expected cadence
    Tool: Bash
    Steps: Run test with RUST_LOG=info, capture output
    Expected: Log shows "block finalized" at height 1 and height 2 with ~5s interval
    Evidence: .sisyphus/evidence/task-8-cadence.txt
  ```

  **Commit**: YES | Message: `test(chain-binary): add single-node finalization integration test` | Files: `crates/chain-binary/tests/single_node.rs`, possibly `crates/chain-binary/src/wire.rs` (if API adjusted)

## Final Verification Wave (4 parallel agents, ALL must APPROVE)

- [ ] F1. Plan Compliance Audit
  - Agent: `oracle`
  - Check: All 8 tasks follow TDD where specified, all references exist, all acceptance criteria are agent-executable
  - Evidence: `.sisyphus/evidence/f1-compliance.txt`

- [ ] F2. Code Quality Review
  - Agent: `unspecified-high`
  - Check: `cargo clippy -p chain-binary -- -D warnings`, `cargo fmt -p chain-binary -- --check`, no unused imports/variables
  - Evidence: `.sisyphus/evidence/f2-quality.txt`

- [ ] F3. Real Manual QA
  - Agent: `unspecified-high`
  - Check: Start binary, observe 3+ blocks finalized via log output, Ctrl-C graceful shutdown
  - Steps:
    1. `cargo build -p chain-binary`
    2. Start `cargo run -p chain-binary` in background
    3. Wait 20 seconds, capture logs
    4. Send SIGTERM
    5. Verify logs show finalized blocks at heights 1, 2, 3+
    6. Verify clean shutdown (no panics, exit code 0)
  - Evidence: `.sisyphus/evidence/f3-manual-qa.txt`

- [ ] F4. Scope Fidelity Check
  - Agent: `deep`
  - Check: No modifications to `vendor/`, `consensus-core/`, or `consensus-commonware/`. Only `chain-binary/` crate + workspace `Cargo.toml` changed.
  - Evidence: `.sisyphus/evidence/f4-scope.txt`

## Commit Strategy

| Task | Commit Message | Files |
|------|---------------|-------|
| 1 | `feat(chain-binary): scaffold crate with module stubs and config constants` | `crates/chain-binary/`, `Cargo.toml`, `Cargo.lock` |
| 2 | `feat(chain-binary): implement EmptyBlock with dual-trait conformance (TDD)` | `crates/chain-binary/src/block.rs` |
| 3 | `feat(chain-binary): implement EmptyBlockApp with 5 verify rules (TDD)` | `crates/chain-binary/src/app.rs` |
| 4 | `feat(chain-binary): implement FinalizationSink with height tracking (TDD)` | `crates/chain-binary/src/sink.rs` |
| 5 | `feat(chain-binary): implement Mailbox bridge for simplex Automaton/Relay (TDD)` | `crates/chain-binary/src/mailbox.rs` |
| 6 | `feat(chain-binary): wire starter closure with P2P, simplex engine, and mailbox bridge` | `crates/chain-binary/src/wire.rs` |
| 7 | `feat(chain-binary): implement main entrypoint with signal handling` | `crates/chain-binary/src/main.rs` |
| 8 | `test(chain-binary): add single-node finalization integration test` | `crates/chain-binary/tests/single_node.rs` |

## Success Criteria

1. `cargo build -p chain-binary` — zero errors, zero warnings
2. `cargo nextest run -p chain-binary` — all unit tests pass (31 tests: 8+11+6+6)
3. `cargo nextest run -p chain-binary --test single_node` — integration test passes
4. `cargo clippy -p chain-binary -- -D warnings` — clean
5. Running binary finalizes empty blocks at ~5-second intervals
6. Graceful shutdown on Ctrl-C/SIGTERM
7. No modifications to vendor, consensus-core, or consensus-commonware code
