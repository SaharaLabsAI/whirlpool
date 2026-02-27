# EVM Integration: Implement app, app-evm, and state crates

## TL;DR
> **Summary**: Implement 3 new Rust crates (`state`, `app`, `app-evm`) that integrate EVM execution into Whirlpool consensus, plus wire them into `whirlpool-node`. Follows comprehensive design in `docs/design/evm-integration/`.
> **Deliverables**: 3 new crates with full test coverage, updated workspace config, node wiring with feature-gated app selection
> **Effort**: Large
> **Parallel**: YES - 5 waves
> **Critical Path**: Task 1 (workspace) → Tasks 2,3,4 (parallel: state, app, evm-config) → Task 5 (EvmApplication) → Task 6 (integration tests) → Task 7 (node wiring) → Task 8 (llmdocs) → F1-F4 (final verification)

## Context

### Original Request
Implement the EVM integration design according to the documents in `./docs/design/evm-integration/`. The design consists of 29 files covering 3 new crates, architecture flows, wiring matrices, and test contracts.

### Interview Summary
Design docs are decision-complete — all technical choices, type signatures, trait boundaries, and test contracts are specified. Key decisions from design:
- Adapter pattern (Option 2): `ApplicationAdapter<A>` bridges `Application` → `ConsensusApp` without modifying the consensus crate's trait
- `WhirlpoolEvmConfig` wraps `EthEvmConfig` pattern from reth
- `InMemoryStateDb` with flat keccak256 state roots (MVP — MPT is future work)
- `EvmBlock` implements 7 commonware codec traits following `EmptyBlock` pattern
- `NoopTxSource` for MVP transaction sourcing
- Feature-gated node wiring (`--app=evm` vs `--app=empty`)

### Metis Review (gaps addressed)
1. **CRITICAL — ConsensusError::Verification**: Design docs reference `ConsensusError::Verification(...)` but this variant doesn't exist. **Resolution**: Map to `ConsensusError::InvalidBlock(format!("verification failed: {}", e))` — avoids modifying consensus crate (out of scope per INTENT.md "ConsensusApp trait unchanged"). Tasks updated to use InvalidBlock mapping.
2. **CRITICAL — ExecutionResult inconsistency**: `app/README.md` defines `ExecutionResult` without `bundle_state`, but `app-evm/README.md` pseudo-code returns it with `BundleState`. **Resolution**: Keep `ExecutionResult` in `app` crate as the clean summary type (state_root, receipts_root, gas_used, receipt_count). `EvmApplication` caches `BundleState` internally keyed by block hash for finalization. The `ExecutionResult` is the cross-crate boundary type; `BundleState` stays internal to `app-evm`.
3. **CRITICAL — Type erasure for node wiring**: `CommonwareEngine` is generic over `A: ConsensusApp`. `EmptyBlockApp::Block = EmptyBlock` ≠ `ApplicationAdapter::Block = EvmBlock`. Cannot use both behind same engine instance. **Resolution**: Compile-time selection via Cargo features (`--features evm`). Default feature keeps `EmptyBlockApp`. `evm` feature compiles with `EvmApplication` + `ApplicationAdapter`. Clean separation, no runtime dispatch.
4. **HIGH — Genesis state root**: `genesis()` should use `state_db.state_root()` not hardcoded `EMPTY_ROOT_HASH` (works for empty genesis but incorrect pattern). **Resolution**: Task 6 explicitly computes genesis state root from state DB.
5. **HIGH — Reth workspace dep resolution**: Vendor reth uses `workspace = true` for all deps. **Resolution**: New crates declare reth deps with explicit `path = "../../vendor/reth/crates/..."` like existing crates do with commonware. No workspace.dependencies needed.
6. **MEDIUM — Test gaps identified**: EvmBlock serialization round-trip, concurrent access, double-commit, clone independence for adapter. **Resolution**: Added to relevant task QA scenarios.
7. **LOW — Scope creep risk**: B-003 (MPT) and B-004 (persistence) are future work. **Resolution**: Explicit "Must NOT Have" section enforces boundary.

## Work Objectives

### Core Objective
Implement EVM block execution capability within Whirlpool's consensus framework through 3 new crates following the abstract→concrete layering pattern.

### Deliverables
- `crates/state/` — In-memory state database implementing `revm::Database`
- `crates/app/` — Abstract `Application` trait, `ApplicationAdapter`, `EvmBlock`, `ExecutionResult`
- `crates/app-evm/` — `WhirlpoolEvmConfig`, `EvmApplication`, `build_sahara_chain_spec()`
- Updated `crates/whirlpool-node/` — Feature-gated EVM wiring
- Updated workspace `Cargo.toml` — New crate members
- All tests passing: `nix develop --command cargo test`

### Definition of Done (verifiable conditions with commands)
1. `nix develop --command cargo build` succeeds with no warnings for new crates
2. `nix develop --command cargo build --features evm` succeeds for whirlpool-node
3. `nix develop --command cargo test -p state` — all 18+ tests pass
4. `nix develop --command cargo test -p app` — all 5+ tests pass
5. `nix develop --command cargo test -p app-evm` — all 5+ tests pass
6. `nix develop --command cargo test -p whirlpool-node` — existing tests still pass
7. `nix develop --command cargo test` — full workspace green
8. No modifications to `vendor/` directory
9. No modifications to `crates/consensus/` (trait unchanged)

### Must Have
- `InMemoryStateDb` with `Database`, `DatabaseRef`, `Clone`, `commit()`, `state_root()`
- `Application` trait with `genesis()`, `propose()`, `verify()`
- `ApplicationAdapter<A>` implementing `ConsensusApp`
- `EvmBlock` with all 7 commonware trait impls
- `WhirlpoolEvmConfig` implementing `ConfigureEvm`
- `EvmApplication<DB>` implementing `Application`
- `build_sahara_chain_spec()` with chain_id=313371, Cancun at genesis
- `NoopTxSource` for MVP
- Feature-gated node wiring

### Must NOT Have (guardrails)
- NO persistent storage (RocksDB, MDBX) — in-memory only
- NO transaction pool implementation
- NO RPC endpoints
- NO network propagation changes
- NO vendor/ modifications
- NO consensus crate trait changes
- NO MPT state root computation (flat keccak256 only)
- NO runtime app selection dispatch — compile-time features only
- NO new external crate dependencies beyond what vendor reth already provides

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: Tests-with-implementation (each task includes its tests)
- Framework: `#[test]` for sync, `#[tokio::test]` for async, inline `#[cfg(test)]` modules
- QA policy: Every task has agent-executed scenarios with `nix develop --command cargo ...`
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Foundation): Task 1 — Workspace config + crate scaffolding
Wave 2 (Core crates, parallel): Tasks 2,3,4 — State crate, App crate, EVM Config (independent after Task 1)
Wave 3 (EVM Application): Task 5 — EvmApplication implementation (depends on 2,3,4)
Wave 4 (Integration + Wiring): Tasks 6,7,8 — Integration tests, Node wiring, Llmdocs (6 depends on 5; 7 depends on 5,6; 8 depends on 5,6)
Wave 5 (Final Verification): F1-F4 — Plan compliance, code review, manual QA, scope fidelity

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1. Workspace Config | — | 2,3,4,5,6,7,8 |
| 2. State Crate | 1 | 5,6 |
| 3. App Crate (EvmBlock+Adapter) | 1 | 5,6 |
| 4. EVM Config | 1 | 5 |
| 5. EvmApplication | 2,3,4 | 6,7,8 |
| 6. Integration Tests | 5 | 7 |
| 7. Node Wiring | 5,6 | F1-F4 |
| 8. Llmdocs | 5,6 | F1-F4 |
| F1-F4. Final Verification | 7,8 | — |

### Agent Dispatch Summary

| Wave | Tasks | Count | Categories |
|------|-------|-------|-----------|
| 1 | 1 | 1 | deep |
| 2 | 2, 3, 4 | 3 | deep, deep, deep |
| 3 | 5 | 1 | ultrabrain |
| 4 | 6, 7, 8 | 3 | deep, deep, quick |
| 5 | F1-F4 | 4 | oracle, unspecified-high, unspecified-high, deep |

## TODOs

<!-- TASKS_START -->

- [x] 1. Workspace Configuration + Crate Scaffolding

  **What to do**:
  1. Add `state`, `app`, `app-evm` to workspace `members` in root `Cargo.toml`
  2. Create `crates/state/Cargo.toml` with:
     ```toml
     [package]
     name = "state"
     version.workspace = true
     edition.workspace = true

     [dependencies]
     revm = { path = "../../vendor/reth/crates/revm" }
     alloy-primitives = { path = "../../vendor/reth/crates/primitives-traits/../../alloy/crates/primitives" }
     sha2 = "0.10"
     thiserror = "2"
     ```
     NOTE: Exact alloy-primitives path needs validation. Check `vendor/reth/Cargo.toml` workspace deps for the actual alloy path. The revm dep likely re-exports alloy-primitives. If so, use `revm` re-exports instead of direct dep.
  3. Create `crates/state/src/lib.rs` with `pub mod db; pub mod error;` and empty module files
  4. Create `crates/app/Cargo.toml` with:
     ```toml
     [package]
     name = "app"
     version.workspace = true
     edition.workspace = true

     [dependencies]
     consensus = { path = "../consensus" }
     sha2 = "0.10"
     thiserror = "2"
     async-trait = "0.1"
     ```
     NOTE: Check if `async-trait` is needed — ConsensusApp uses `impl Future` syntax (RPITIT), not `#[async_trait]`. The Application trait should also use RPITIT to match.
  5. Create `crates/app/src/lib.rs` with `pub mod types; pub mod error; pub mod traits; pub mod adapter;` and empty module files
  6. Create `crates/app-evm/Cargo.toml` with:
     ```toml
     [package]
     name = "app-evm"
     version.workspace = true
     edition.workspace = true

     [dependencies]
     app = { path = "../app" }
     state = { path = "../state" }
     consensus = { path = "../consensus" }
     reth-evm = { path = "../../vendor/reth/crates/evm/evm" }
     reth-evm-ethereum = { path = "../../vendor/reth/crates/ethereum/evm" }
     reth-chainspec = { path = "../../vendor/reth/crates/chainspec" }
     reth-execution-types = { path = "../../vendor/reth/crates/evm/execution-types" }
     reth-execution-errors = { path = "../../vendor/reth/crates/evm/execution-errors" }
     reth-primitives-traits = { path = "../../vendor/reth/crates/primitives-traits" }
     reth-ethereum-primitives = { path = "../../vendor/reth/crates/ethereum/primitives" }
     revm = { path = "../../vendor/reth/crates/revm" }
     alloy-primitives = { path = "../../vendor/reth/crates/primitives-traits/../../alloy/crates/primitives" }
     alloy-consensus = { path = "../../vendor/reth/crates/primitives-traits/../../alloy/crates/consensus" }
     alloy-evm = { path = "../../vendor/reth/crates/evm/evm/../../alloy/crates/evm" }
     thiserror = "2"
     ```
     NOTE: All alloy-* paths must be resolved from the reth workspace Cargo.toml. The implementer MUST check `vendor/reth/Cargo.toml` `[workspace.dependencies]` section for actual alloy crate paths. Many alloy crates are `git` deps in reth — if so, they need to be resolved via reth's own path structure or re-exports.
  7. Create `crates/app-evm/src/lib.rs` with `pub mod config; pub mod executor; pub mod error;` and empty module files
  8. Verify: `nix develop --command cargo check` succeeds (may have warnings for unused imports, that's OK at this stage)
  **Must NOT do**: Write any implementation code beyond empty module stubs. Do NOT modify vendor/.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Cargo dependency resolution with vendor paths is tricky; needs research capability
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [2,3,4,5,6,7] | Blocked By: none

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `Cargo.toml` (workspace root, lines 1-13) — workspace member list pattern, `version = "0.1.0"`, `edition = "2021"` in `[workspace.package]`
  - Pattern: `crates/whirlpool-node/Cargo.toml` — path dep syntax: `consensus = { path = "../consensus" }`, vendor paths: `commonware-consensus = { path = "../../vendor/commonware/consensus" }`
  - Pattern: `crates/consensus/Cargo.toml` — minimal crate Cargo.toml pattern with `version.workspace = true`, `edition.workspace = true`
  - Vendor: `vendor/reth/Cargo.toml` — MUST READ this to resolve alloy-* crate paths from reth's workspace deps
  - Vendor: `vendor/reth/crates/evm/evm/Cargo.toml` — reth-evm deps list
  - Vendor: `vendor/reth/crates/ethereum/evm/Cargo.toml` — reth-evm-ethereum deps list
  - Design: `docs/design/evm-integration/CRATES.md` — proposed crate structure and dependencies
  - Design: `docs/design/evm-integration/WORKSPACE.md` — dependency graph

  **Acceptance Criteria** (agent-executable only):
  - [ ] Root `Cargo.toml` lists `state`, `app`, `app-evm` in members
  - [ ] `crates/state/Cargo.toml` exists with correct deps
  - [ ] `crates/app/Cargo.toml` exists with correct deps
  - [ ] `crates/app-evm/Cargo.toml` exists with correct deps
  - [ ] `nix develop --command cargo check` succeeds (empty stubs compile)
  - [ ] `nix develop --command cargo build` succeeds

  **QA Scenarios**:
  ```
  Scenario: Workspace resolves all dependencies
    Tool: Bash
    Steps: nix develop --command cargo check 2>&1
    Expected: exit 0, all 3 new crates compile (may have dead_code warnings)
    Evidence: .sisyphus/evidence/task-1-cargo-check.txt

  Scenario: No vendor modifications
    Tool: Bash
    Steps: git diff --name-only vendor/
    Expected: Empty output
    Evidence: .sisyphus/evidence/task-1-vendor-clean.txt
  ```

  **Commit**: YES | Message: `feat(workspace): scaffold state, app, and app-evm crates` | Files: `Cargo.toml`, `crates/state/**`, `crates/app/**`, `crates/app-evm/**`

---

- [x] 2. State Crate — InMemoryStateDb Implementation

  **What to do**:
  1. Implement `crates/state/src/error.rs`:
     - `StateError` enum with `Internal(String)` variant, derive `thiserror::Error`, `Debug`
  2. Implement `crates/state/src/db.rs`:
     - `DbAccount` struct: `info: AccountInfo`, `storage: HashMap<U256, U256>`. Derive `Clone`, `Debug`, `Default`
     - `InMemoryStateDb` struct: `accounts: HashMap<Address, DbAccount>`, `bytecodes: HashMap<B256, Bytecode>`, `block_hashes: HashMap<u64, B256>`. Derive `Clone`, `Debug`
     - `impl InMemoryStateDb`:
       - `new() -> Self` — empty hashmaps
       - `with_genesis(alloc: HashMap<Address, GenesisAccount>) -> Self` — populate accounts from genesis allocations (balance, nonce, code, storage)
       - `commit(&mut self, bundle: &BundleState)` — iterate `bundle.state` entries: Created/Changed → upsert account+storage, Destroyed → remove. Iterate `bundle.contracts` → insert bytecodes.
       - `state_root(&self) -> B256` — deterministic flat hash: sort accounts by Address, for each account sort storage by key, keccak256 over concatenated (addr, nonce, balance, code_hash, [(key,value)...]). Return `KECCAK_EMPTY` for empty DB.
       - `insert_block_hash(&mut self, number: u64, hash: B256)`
     - `impl Database for InMemoryStateDb`:
       - `type Error = StateError`
       - `basic_ref(&self, addr) -> Ok(self.accounts.get(&addr).map(|a| a.info.clone()))`
       - `code_by_hash_ref(&self, hash) -> Ok(self.bytecodes.get(&hash).cloned().unwrap_or_default())`
       - `storage_ref(&self, addr, key) -> Ok(self.accounts.get(&addr).and_then(|a| a.storage.get(&key).copied()).unwrap_or(U256::ZERO))`
       - `block_hash_ref(&self, number) -> Ok(self.block_hashes.get(&number).copied().unwrap_or(B256::ZERO))`
     - `impl DatabaseRef for InMemoryStateDb` — same semantics via `&self`
  3. Update `crates/state/src/lib.rs` to export public types
  4. Write all 18 unit tests inline in `db.rs` under `#[cfg(test)] mod tests`:
     - Database tests: `test_basic_none`, `test_basic_returns_info`, `test_storage_zero`, `test_storage_value`, `test_code_by_hash_default`, `test_block_hash_zero`, `test_block_hash_inserted`
     - Commit tests: `test_commit_create_account`, `test_commit_update_account`, `test_commit_destroy_account`, `test_commit_storage_changes`, `test_commit_new_bytecode`
     - State root tests: `test_state_root_deterministic`, `test_state_root_changes_after_commit`, `test_state_root_empty_db`
     - Clone test: `test_independent_snapshot`
     - Genesis test: `test_with_genesis_populates`
     - EXTRA (Metis): `test_state_root_account_ordering` — verify different insertion orders produce same root
  **Must NOT do**: Implement MPT. Do NOT add persistence. Do NOT add RwLock here (that's the consumer's responsibility).

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Complex revm type integration requires research into exact revm types
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: NO | Wave 1 (after Task 1) | Blocks: [3,5,6] | Blocked By: [1]

  **References** (executor has NO interview context — be exhaustive):
  - Design: `docs/design/evm-integration/state/README.md` — complete type definitions, method signatures, invariants
  - Design: `docs/design/evm-integration/domains/state-storage.md` — domain model and invariants
  - Design: `docs/design/evm-integration/tests/state-unit.md` — all 18 test contracts with pseudo-code
  - Design: `docs/design/evm-integration/wiring/state-storage.md` — wiring requirements
  - Vendor: `vendor/reth/crates/revm/` — revm re-exports, `Database` trait, `DatabaseRef` trait, `BundleState` type, `AccountInfo`, `Bytecode`
  - Vendor: Check `revm::db` or `revm::database` module for `Database` trait definition — the exact import path matters
  - Pattern: Use `alloy_primitives::{Address, B256, U256}` and `revm::primitives::{AccountInfo, Bytecode, KECCAK_EMPTY}`
  - CRITICAL: `BundleState` is in `revm::db::states::bundle_state` or `reth-execution-types`. Check exact import path from reth-revm.

  **Acceptance Criteria** (agent-executable only):
  - [ ] `nix develop --command cargo test -p state` — all 18+ tests pass
  - [ ] `nix develop --command cargo build -p state` — no warnings
  - [ ] `InMemoryStateDb` implements `Database`, `DatabaseRef`, `Clone`
  - [ ] `state_root()` is deterministic (same state → same hash regardless of insertion order)
  - [ ] `commit()` handles Created, Changed, Destroyed account states

  **QA Scenarios**:
  ```
  Scenario: All state unit tests pass
    Tool: Bash
    Steps: nix develop --command cargo test -p state -- --nocapture 2>&1
    Expected: test result: ok. 18+ passed; 0 failed
    Evidence: .sisyphus/evidence/task-2-state-tests.txt

  Scenario: State root determinism
    Tool: Bash
    Steps: nix develop --command cargo test -p state test_state_root_deterministic -- --nocapture 2>&1
    Expected: Two identical state DBs produce identical roots
    Evidence: .sisyphus/evidence/task-2-determinism.txt

  Scenario: Clone isolation
    Tool: Bash
    Steps: nix develop --command cargo test -p state test_independent_snapshot -- --nocapture 2>&1
    Expected: Mutations to clone don't affect original
    Evidence: .sisyphus/evidence/task-2-clone.txt
  ```

  **Commit**: YES | Message: `feat(state): implement InMemoryStateDb with revm Database` | Files: `crates/state/src/**`


---

- [x] 3. App Crate — Application Trait + EvmBlock + ApplicationAdapter

  **What to do**:
  1. Implement `crates/app/src/error.rs`:
     - `ApplicationError` enum: `Execution(String)`, `Verification(String)`, `State(String)`. Derive `thiserror::Error`, `Debug`.
  2. Implement `crates/app/src/types.rs`:
     - `ExecutionResult` struct: `state_root: [u8; 32]`, `receipts_root: [u8; 32]`, `gas_used: u64`, `receipt_count: usize`. Derive `Clone`, `Debug`.
     - `EvmBlock` struct: `height: u64`, `parent_id: [u8; 32]`, `state_root: [u8; 32]`, `transactions_root: [u8; 32]`, `receipts_root: [u8; 32]`, `gas_used: u64`, `timestamp: u64`, `transactions: Vec<Vec<u8>>`. Derive `Clone`, `Debug`.
     - `impl consensus::Block for EvmBlock`:
       - `type Id = [u8; 32]`
       - `id()` → SHA-256 of (height, parent_id, state_root, transactions_root) — deterministic
       - `parent_id()` → self.parent_id
       - `height()` → self.height
     - Implement 7 commonware traits EXACTLY like `EmptyBlock` in `crates/whirlpool-node/src/block.rs`:
       - `CodecWrite`: serialize all fields to writer
       - `CodecRead`: deserialize from reader
       - `EncodeSize`: return encoded byte size
       - `Digestible`: `type Digest = sha256::Digest`; `fn digest() → sha256 of all fields`
       - `Committable`: delegate to Digestible
       - `Heightable`: `Height::new(self.height)`
       - `VendorBlock`: `fn parent() → Digest::from(self.parent_id)`
     - Import commonware traits from `commonware-codec` and `commonware-cryptography`
  3. Implement `crates/app/src/traits.rs`:
     - `Application` trait (async, using RPITIT like ConsensusApp):
       ```rust
       pub trait Application: Send + Sync + Clone + 'static {
           type Block: consensus::Block;
           type Result: Clone + Send;
           type Error: std::error::Error + Send + Sync;
           fn genesis(&self) -> impl Future<Output = Self::Block> + Send;
           fn propose(&self, parent: &Self::Block, height: u64) -> impl Future<Output = Result<(Self::Block, Self::Result), Self::Error>> + Send;
           fn verify(&self, parent: &Self::Block, block: &Self::Block) -> impl Future<Output = Result<Self::Result, Self::Error>> + Send;
       }
       ```
     - `TxSource` trait: `fn pending(&self) -> Vec<Vec<u8>>`. With `NoopTxSource` struct that returns empty vec.
  4. Implement `crates/app/src/adapter.rs`:
     - `ApplicationAdapter<A: Application<Block = EvmBlock>>` struct: `inner: A`
     - `new(app: A) -> Self`, `inner(&self) -> &A`
     - Derive `Clone`
     - `impl ConsensusApp for ApplicationAdapter<A>`:
       - `type Block = EvmBlock`
       - `genesis()` → delegates to `self.inner.genesis().await`
       - `propose()` → match `self.inner.propose().await { Ok((block, _)) => Some(block), Err(_) => None }`
       - `verify()` → match `self.inner.verify().await { Ok(_) => Ok(()), Err(e) => Err(ConsensusError::InvalidBlock(e.to_string())) }`
     - NOTE: Design docs reference `ConsensusError::Verification` which doesn't exist. Map to `ConsensusError::InvalidBlock` instead — it has the right semantics for "the block failed application-level verification".
  5. Update `crates/app/src/lib.rs` to re-export all public types
  6. Add commonware codec/crypto deps to `crates/app/Cargo.toml`:
     ```toml
     commonware-codec = { path = "../../vendor/commonware/codec" }
     commonware-cryptography = { path = "../../vendor/commonware/cryptography" }
     sha2 = "0.10"
     ```
  7. Write unit tests in relevant modules under `#[cfg(test)]`:
     - In types.rs: `test_evm_block_trait_impl`, `test_evm_block_codec_roundtrip`, `test_execution_result_fields`
     - In adapter.rs: `test_adapter_wrapping`, `test_adapter_genesis_passthrough`
     - In error.rs: `test_application_error_display`
  **Must NOT do**: Do NOT add `ConsensusError::Verification` variant to consensus crate. Map to `InvalidBlock`. Do NOT depend on reth crates from the app crate.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: 7 commonware trait impls require careful pattern matching from EmptyBlock
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: YES (with Task 2) | Wave 2 | Blocks: [4,5,6] | Blocked By: [1]

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `crates/whirlpool-node/src/block.rs` — EmptyBlock with ALL 7 commonware trait impls. Copy this pattern EXACTLY for EvmBlock. Lines 1-200+.
  - Pattern: `crates/whirlpool-node/src/app.rs` — EmptyBlockApp implementing ConsensusApp. Shows verify() error mapping pattern.
  - Design: `docs/design/evm-integration/app/README.md` — complete type definitions, Application trait, ApplicationAdapter
  - Design: `docs/design/evm-integration/domains/application.md` — Application domain model
  - Design: `docs/design/evm-integration/tests/app-unit.md` — 5 test contracts
  - Design: `docs/design/evm-integration/architecture/consensus-app-bridge.md` — adapter bridge pattern
  - Trait: `crates/consensus/src/app.rs` — ConsensusApp trait signature (RPITIT async)
  - Trait: `crates/consensus/src/block.rs` — Block trait (Id, id(), parent_id(), height())
  - Error: `crates/consensus/src/error.rs` — ConsensusError variants (use InvalidBlock for verify failures)
  - Vendor: `vendor/commonware/codec/` — CodecWrite, CodecRead, EncodeSize traits
  - Vendor: `vendor/commonware/cryptography/` — Digestible, Committable traits
  - Vendor: `vendor/commonware/consensus/` — Heightable, VendorBlock traits
  - CRITICAL: The exact import paths for commonware traits vary. Check `crates/whirlpool-node/src/block.rs` use statements for correct imports.

  **Acceptance Criteria** (agent-executable only):
  - [ ] `nix develop --command cargo test -p app` — all 6+ tests pass
  - [ ] `nix develop --command cargo build -p app` — no errors
  - [ ] `EvmBlock` implements `consensus::Block`, `CodecWrite`, `CodecRead`, `EncodeSize`, `Digestible`, `Committable`, `Heightable`, `VendorBlock`
  - [ ] `ApplicationAdapter` implements `ConsensusApp`
  - [ ] Application trait uses RPITIT (no async-trait macro)

  **QA Scenarios**:
  ```
  Scenario: All app unit tests pass
    Tool: Bash
    Steps: nix develop --command cargo test -p app -- --nocapture 2>&1
    Expected: test result: ok. 6+ passed; 0 failed
    Evidence: .sisyphus/evidence/task-3-app-tests.txt

  Scenario: EvmBlock codec roundtrip
    Tool: Bash
    Steps: nix develop --command cargo test -p app test_evm_block_codec_roundtrip -- --nocapture 2>&1
    Expected: Encode → decode produces identical block
    Evidence: .sisyphus/evidence/task-3-codec-roundtrip.txt

  Scenario: Adapter maps errors correctly
    Tool: Bash
    Steps: nix develop --command cargo test -p app test_adapter -- --nocapture 2>&1
    Expected: Application errors map to ConsensusError::InvalidBlock
    Evidence: .sisyphus/evidence/task-3-adapter.txt
  ```

  **Commit**: YES | Message: `feat(app): implement Application trait, EvmBlock, and ApplicationAdapter` | Files: `crates/app/src/**`, `crates/app/Cargo.toml`

---

- [x] 4. App-EVM Crate — WhirlpoolEvmConfig + build_sahara_chain_spec()

  **What to do**:
  1. Implement `crates/app-evm/src/config.rs`:
     - `pub const SAHARA_CHAIN_ID: u64 = 313_371;`
     - `pub fn build_sahara_chain_spec() -> ChainSpec`:
       ```rust
       ChainSpecBuilder::default()
           .chain(Chain::from_id(SAHARA_CHAIN_ID))
           .genesis(Genesis { gas_limit: 30_000_000, difficulty: U256::ZERO, ..Default::default() })
           .cancun_activated()
           .build()
       ```
     - `WhirlpoolEvmConfig` struct — a newtype wrapper around `EthEvmConfig`:
       ```rust
       #[derive(Debug, Clone)]
       pub struct WhirlpoolEvmConfig {
           inner: EthEvmConfig,
       }
       ```
     - `impl WhirlpoolEvmConfig`:
       - `new(chain_spec: Arc<ChainSpec>) -> Self { Self { inner: EthEvmConfig::new(chain_spec) } }`
       - `chain_spec() -> &Arc<ChainSpec>` — delegate to inner
     - `impl ConfigureEvm for WhirlpoolEvmConfig` — delegate ALL methods to `self.inner`:
       - `type Primitives = EthPrimitives`
       - `type Error = Infallible`
       - `type NextBlockEnvCtx = NextBlockEnvAttributes`
       - `type BlockExecutorFactory = <EthEvmConfig as ConfigureEvm>::BlockExecutorFactory`
       - `type BlockAssembler = <EthEvmConfig as ConfigureEvm>::BlockAssembler`
       - All methods: `fn evm_env(..) { self.inner.evm_env(..) }`, etc.
     - ALTERNATIVE (simpler): If `EthEvmConfig` derives `Clone + Debug + Send + Sync + Unpin`, just use `pub type WhirlpoolEvmConfig = EthEvmConfig;` as a type alias. But a newtype is preferred for future Sahara-specific customization.
  2. Implement `crates/app-evm/src/error.rs`:
     - `EvmAppError` enum:
       - `Execution(String)` (wraps BlockExecutionError stringified)
       - `StateRootMismatch { expected: [u8; 32], computed: [u8; 32] }`
       - `State(String)`
       - `InvalidBlock(String)`
     - `impl From<EvmAppError> for ApplicationError`
  3. Update `crates/app-evm/src/lib.rs` to re-export public types
  4. Write unit tests in config.rs under `#[cfg(test)]`:
     - `test_evm_config_chain_spec`: verify chain_id, gas_limit, Cancun hardfork active
     - `test_evm_config_exposes_factory_and_assembler`: compile-time check that accessors exist
     - `test_build_sahara_chain_spec_values`: verify chain_id=313371, gas_limit=30M
  **Must NOT do**: Do NOT implement EvmApplication yet (that's Task 5). Do NOT modify vendor crates.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: ConfigureEvm delegation requires understanding reth trait bounds
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: YES (with Tasks 2, 3) | Wave 2 | Blocks: [5] | Blocked By: [1]

  **References** (executor has NO interview context — be exhaustive):
  - Pattern: `vendor/reth/crates/ethereum/evm/src/lib.rs` (lines 79-214) — EthEvmConfig struct + ConfigureEvm impl. Copy delegation pattern.
  - Design: `docs/design/evm-integration/app-evm/README.md` — WhirlpoolEvmConfig spec, build_sahara_chain_spec()
  - Design: `docs/design/evm-integration/domains/evm-execution.md` — EVM execution domain model
  - Design: `docs/design/evm-integration/wiring/evm-execution.md` — wiring requirements
  - Design: `docs/design/evm-integration/tests/app-evm-unit.md` — 5 test contracts
  - Design: `docs/design/evm-integration/BLOCKERS.md` — B-001 ChainSpec resolution
  - Vendor: `vendor/reth/crates/chainspec/src/spec.rs` — ChainSpec, ChainSpecBuilder, cancun_activated()
  - Vendor: `vendor/reth/crates/evm/evm/src/lib.rs` — ConfigureEvm trait definition (lines 184-456)
  - Vendor: `vendor/reth/crates/ethereum/evm/src/lib.rs` — EthEvmConfig (lines 79-214), constructor, impl ConfigureEvm
  - Types: `reth_evm::{ConfigureEvm, NextBlockEnvAttributes, EvmEnv}`, `reth_evm_ethereum::{EthEvmConfig, EthEvmFactory, EthBlockExecutorFactory, EthBlockAssembler, RethReceiptBuilder}`
  - Types: `reth_chainspec::{ChainSpec, ChainSpecBuilder, Chain}`, `reth_ethereum_primitives::EthPrimitives`

  **Acceptance Criteria** (agent-executable only):
  - [ ] `nix develop --command cargo test -p app-evm` — all 3+ tests pass
  - [ ] `nix develop --command cargo build -p app-evm` — no errors
  - [ ] `build_sahara_chain_spec()` returns ChainSpec with chain_id=313371, Cancun hardforks active
  - [ ] `WhirlpoolEvmConfig` implements `ConfigureEvm`

  **QA Scenarios**:
  ```
  Scenario: ChainSpec values correct
    Tool: Bash
    Steps: nix develop --command cargo test -p app-evm test_build_sahara_chain_spec -- --nocapture 2>&1
    Expected: chain_id=313371, gas_limit=30000000, Cancun active at genesis
    Evidence: .sisyphus/evidence/task-4-chainspec.txt

  Scenario: ConfigureEvm compiles
    Tool: Bash
    Steps: nix develop --command cargo build -p app-evm 2>&1
    Expected: exit 0, WhirlpoolEvmConfig satisfies ConfigureEvm bounds
    Evidence: .sisyphus/evidence/task-4-build.txt
  ```

  **Commit**: YES | Message: `feat(app-evm): implement WhirlpoolEvmConfig and build_sahara_chain_spec` | Files: `crates/app-evm/src/**`

---

- [x] 5. App-EVM Crate — EvmApplication Implementation

  **What to do**:
  1. Implement `crates/app-evm/src/executor.rs`:
     - `EvmApplication<DB: Database<Error: Into<EvmAppError>> + DatabaseRef<Error: Into<EvmAppError>> + Clone>` struct:
       ```rust
       pub struct EvmApplication<DB> {
           evm_config: WhirlpoolEvmConfig,
           state_db: Arc<RwLock<DB>>,
           tx_source: Arc<dyn TxSource + Send + Sync>,
       }
       ```
     - Derive `Clone` (all fields are Clone: WhirlpoolEvmConfig is Clone, Arc is Clone)
     - Constructor: `new(evm_config: WhirlpoolEvmConfig, state_db: Arc<RwLock<DB>>, tx_source: Arc<dyn TxSource + Send + Sync>) -> Self`
     - `impl Application for EvmApplication<DB>`:
       - `type Block = EvmBlock`
       - `type Result = ExecutionResult`
       - `type Error = EvmAppError`
       - `genesis()`: return EvmBlock with height=0, all roots=`EMPTY_ROOT_HASH` or compute from state_db.state_root(). Use `state_db.read().unwrap().state_root()` for the state_root field to handle non-empty genesis.
       - `propose(parent, height)`:
         1. Clone state: `let snapshot = self.state_db.read().unwrap().clone()`
         2. Get pending txs from tx_source
         3. Create reth `State::builder().with_database(snapshot).with_bundle_update().build()`
         4. Construct `NextBlockEnvAttributes` with `timestamp: parent_timestamp + 12`, `suggested_fee_recipient: Address::ZERO`, `prev_randao: B256::ZERO`, `gas_limit: 30_000_000`
         5. Get evm_env: `self.evm_config.next_evm_env(parent_header, &attrs)?`
         6. Get context: `self.evm_config.context_for_next_block(parent_sealed_header, attrs)?`
         7. Create executor from factory, execute transactions
         8. Finish: `executor.finish()` → `BlockExecutionOutput { state, result }`
         9. Commit bundle to snapshot: `snapshot.commit(output.state)`
         10. Compute `state_root = snapshot.state_root()`
         11. Assemble EvmBlock with computed roots
         12. Return `Ok((block, ExecutionResult { state_root, receipts_root, gas_used, receipt_count }))`
       - `verify(parent, block)`:
         1. Clone state snapshot
         2. Re-execute block transactions against snapshot
         3. Compute state_root from snapshot
         4. Compare: if `computed != block.state_root` → `Err(EvmAppError::StateRootMismatch { expected: block.state_root, computed })`
         5. Return `Ok(ExecutionResult { .. })`
     - CRITICAL DESIGN NOTE (Metis finding): The propose/verify pseudo-code requires constructing reth `Header`/`SealedHeader` types from EvmBlock fields. This is the hardest part — EvmBlock is a custom type, not a reth Block. You'll need to create a `Header` from scratch: `Header { number: block.height, parent_hash: B256::from(block.parent_id), state_root: ..., gas_limit: 30_000_000, timestamp: block.timestamp, .. }`. Check `reth_primitives_traits::Header` or `alloy_consensus::Header` for the struct definition.
     - CRITICAL DESIGN NOTE (Metis finding): For genesis, use `state_db.read().unwrap().state_root()` instead of hardcoded `EMPTY_ROOT_HASH`. If with_genesis() added accounts, the root won't be empty.
  2. Write unit tests:
     - `test_evm_app_genesis`: genesis returns height=0 block with correct state root
     - `test_evm_app_error_mapping`: EvmAppError → ApplicationError conversion
  3. Wire TxSource: Accept `Arc<dyn TxSource>` in constructor. For MVP, callers pass `Arc::new(NoopTxSource)` from the app crate.
  **Must NOT do**: Do NOT implement state finalization/caching (that's Task 7). Do NOT persist execution results. Keep propose/verify stateless w.r.t. canonical state.

  **Recommended Agent Profile**:
  - Category: `ultrabrain` — Reason: This is the most complex task — bridging custom types with reth's type system requires deep understanding of both codebases
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: [6,7] | Blocked By: [2,3,4]

  **References** (executor has NO interview context — be exhaustive):
  - Design: `docs/design/evm-integration/app-evm/README.md` — EvmApplication spec with full propose/verify pseudo-code
  - Design: `docs/design/evm-integration/architecture/block-proposal.md` — 7-stage proposal flow
  - Design: `docs/design/evm-integration/architecture/block-verification.md` — 7-stage verification flow
  - Design: `docs/design/evm-integration/tests/app-evm-unit.md` — test contracts
  - Design: `docs/design/evm-integration/BLOCKERS.md` — B-R02 TxSource resolution
  - Vendor: `vendor/reth/crates/evm/evm/src/lib.rs` — ConfigureEvm method signatures (evm_env, next_evm_env, context_for_block, context_for_next_block)
  - Vendor: `vendor/reth/crates/ethereum/evm/src/lib.rs` — EthEvmConfig ConfigureEvm impl for method delegation examples
  - Vendor: `vendor/reth/crates/revm/src/` — `State` builder pattern, `Database` trait, `BundleState`
  - Vendor: `vendor/reth/crates/evm/execution-types/src/execute.rs` — `BlockExecutionOutput` struct (has `state: BundleState` + `result: BlockExecutionResult`)
  - Vendor: `vendor/reth/crates/evm/evm/src/execute.rs` — `BlockExecutorFactory`, `BlockAssembler`, `BasicBlockExecutor`
  - Types: `alloy_consensus::Header` or `reth_primitives_traits::Header` — for constructing headers from EvmBlock fields
  - Types: `alloy_primitives::{Address, B256, Bytes, EMPTY_ROOT_HASH}`
  - Pattern: `crates/app/src/traits.rs` — Application trait (from Task 3)
  - Pattern: `crates/state/src/db.rs` — InMemoryStateDb (from Task 2)
  - Pattern: `crates/app-evm/src/config.rs` — WhirlpoolEvmConfig (from Task 4)

  **Acceptance Criteria** (agent-executable only):
  - [ ] `nix develop --command cargo test -p app-evm` — all tests pass
  - [ ] `nix develop --command cargo build -p app-evm` — no errors
  - [ ] `EvmApplication<InMemoryStateDb>` implements `Application`
  - [ ] Genesis returns height-0 block with state_root matching state_db.state_root()
  - [ ] Propose with no txs succeeds (empty block execution)

  **QA Scenarios**:
  ```
  Scenario: Genesis block creation
    Tool: Bash
    Steps: nix develop --command cargo test -p app-evm test_evm_app_genesis -- --nocapture 2>&1
    Expected: Genesis block has height=0, valid state root
    Evidence: .sisyphus/evidence/task-5-genesis.txt

  Scenario: Full workspace builds
    Tool: Bash
    Steps: nix develop --command cargo build 2>&1
    Expected: exit 0, entire workspace compiles
    Evidence: .sisyphus/evidence/task-5-build.txt
  ```

  **Commit**: YES | Message: `feat(app-evm): implement EvmApplication with propose/verify` | Files: `crates/app-evm/src/executor.rs`

---

- [ ] 6. Integration Tests — Application + EVM Execution

  **What to do**:
  1. Create `crates/app-evm/tests/` directory for integration tests
  2. Implement `crates/app-evm/tests/application_integration.rs`:
     - Tests using `ApplicationAdapter<EvmApplication<InMemoryStateDb>>`:
     - `test_adapter_propose_success`: Create real EvmApplication + ApplicationAdapter. Call propose() via ConsensusApp trait. Assert returns Some(EvmBlock).
     - `test_adapter_propose_returns_some`: Verify adapter wraps Application::propose Ok result as Some.
     - `test_adapter_verify_success`: Create EvmApplication, propose a block, then verify it. Assert Ok(()).
     - `test_adapter_verify_failure`: Tamper with block state_root, verify fails with ConsensusError::InvalidBlock.
  3. Implement `crates/app-evm/tests/evm_execution_integration.rs`:
     - `test_execute_empty_block`: Propose with NoopTxSource, verify 0 receipts, gas_used=0.
     - `test_state_root_computation`: Propose block, verify state_root in block matches result.state_root.
     - `test_reconstruct_header_for_verify`: Propose then verify — verify succeeds with matching state_root.
  4. Implement `crates/app-evm/tests/cross_crate_flows.rs`:
     - `test_propose_verify_success`: Full genesis→propose→verify lifecycle.
     - `test_state_root_mismatch`: Tamper state_root → verify returns StateRootMismatch mapped to ConsensusError::InvalidBlock.
     - `test_genesis_to_verify`: Genesis block → propose child → verify child.
     - `test_error_propagation_through_adapter`: Invalid parent → ConsensusError::InvalidBlock through adapter.
     - `test_propose_verify_state_root_consistency`: Two sequential blocks, verify both have valid state roots.
     - `test_multi_block_state_accumulation`: Genesis → block1 → block2. State roots differ between blocks (even empty blocks advance state).
     - `test_failed_verify_does_not_corrupt_state`: Bad verify → propose still works with clean state.
  5. Add necessary dev-dependencies to `crates/app-evm/Cargo.toml`:
     ```toml
     [dev-dependencies]
     tokio = { version = "1", features = ["rt", "macros"] }
     ```
  **Must NOT do**: Do NOT test persistent state. Do NOT mock reth — use real execution against InMemoryStateDb.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Integration tests require wiring real EVM execution, understanding error flows
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: NO | Wave 4 | Blocks: [7] | Blocked By: [2,3,4,5]

  **References** (executor has NO interview context — be exhaustive):
  - Design: `docs/design/evm-integration/tests/application-integration.md` — 4 test contracts with pseudo-code
  - Design: `docs/design/evm-integration/tests/evm-execution-integration.md` — 4 test contracts
  - Design: `docs/design/evm-integration/tests/cross-crate-flows.md` — 7 test contracts
  - Design: `docs/design/evm-integration/tests/overview.md` — test strategy overview
  - Pattern: `crates/whirlpool-node/tests/single_node.rs` — existing integration test pattern (if applicable)
  - Pattern: `crates/whirlpool-node/src/app.rs` tests section — EmptyBlockApp unit tests for ConsensusApp pattern
  - All crate source: `crates/app/src/**`, `crates/app-evm/src/**`, `crates/state/src/**` — the types being tested

  **Acceptance Criteria** (agent-executable only):
  - [ ] `nix develop --command cargo test -p app-evm` — all unit + integration tests pass
  - [ ] At least 13 integration tests exist across 3 test files
  - [ ] Cross-crate flow tests exercise full genesis→propose→verify lifecycle
  - [ ] Error propagation test confirms tampered state_root is caught

  **QA Scenarios**:
  ```
  Scenario: All integration tests pass
    Tool: Bash
    Steps: nix develop --command cargo test -p app-evm -- --nocapture 2>&1
    Expected: test result: ok. 13+ passed; 0 failed
    Evidence: .sisyphus/evidence/task-6-integration-tests.txt

  Scenario: State corruption test
    Tool: Bash
    Steps: nix develop --command cargo test -p app-evm test_failed_verify_does_not_corrupt_state -- --nocapture 2>&1
    Expected: After failed verify, subsequent propose succeeds with clean state
    Evidence: .sisyphus/evidence/task-6-no-corruption.txt
  ```

  **Commit**: YES | Message: `test(app-evm): add application, evm execution, and cross-crate integration tests` | Files: `crates/app-evm/tests/**`, `crates/app-evm/Cargo.toml`

---

- [ ] 7. Node Wiring — EvmApplication in whirlpool-node

  **What to do**:
  1. Add dependencies to `crates/whirlpool-node/Cargo.toml`:
     ```toml
     app = { path = "../app" }
     app-evm = { path = "../app-evm" }
     state = { path = "../state" }
     ```
  2. Add a feature flag to `crates/whirlpool-node/Cargo.toml`:
     ```toml
     [features]
     default = ["evm"]
     evm = ["dep:app", "dep:app-evm", "dep:state"]
     ```
     Make the new deps optional: `app = { path = "../app", optional = true }`, etc.
  3. Update `crates/whirlpool-node/src/main.rs`:
     - KEEP the existing `EmptyBlockApp` wiring as the non-evm path
     - Under `#[cfg(feature = "evm")]`, add the EVM wiring:
       ```rust
       let chain_spec = Arc::new(build_sahara_chain_spec());
       let state_db = Arc::new(RwLock::new(InMemoryStateDb::new()));
       let evm_config = WhirlpoolEvmConfig::new(chain_spec);
       let tx_source = Arc::new(NoopTxSource);
       let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_source);
       let app = ApplicationAdapter::new(evm_app);
       // Pass `app` to CommonwareEngine instead of EmptyBlockApp
       ```
     - Handle the type erasure problem: Since `CommonwareEngine` is generic over `A: ConsensusApp` and `EmptyBlockApp::Block ≠ EvmBlock`, use compile-time selection:
       ```rust
       #[cfg(feature = "evm")]
       {
           let app = /* evm wiring above */;
           let engine = CommonwareEngine::new(Arc::new(app), sink, config, network_provider);
           engine.start().await;
       }
       #[cfg(not(feature = "evm"))]
       {
           let app = Arc::new(EmptyBlockApp);
           let engine = CommonwareEngine::new(app, sink, config, network_provider);
           engine.start().await;
       }
       ```
     - NOTE: `ApplicationAdapter` must be wrapped in `Arc` since `CommonwareEngine::new` expects `Arc<A>`. Check `consensus-simplex/src/engine.rs` for exact signature.
  4. Ensure `cargo build` with default features (evm) works
  5. Ensure `cargo build --no-default-features` still compiles (EmptyBlockApp path)
  **Must NOT do**: Do NOT remove EmptyBlockApp or EmptyBlock. Do NOT modify consensus or consensus-simplex. Keep backward compatibility.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Feature-gated wiring with type-level dispatch requires careful Rust knowledge
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: NO | Wave 5 | Blocks: [8] | Blocked By: [5,6]

  **References** (executor has NO interview context — be exhaustive):
  - Design: `docs/design/evm-integration/architecture/node-startup.md` — node startup wiring flow
  - Design: `docs/design/evm-integration/wiring/application.md` — ApplicationAdapter wiring
  - Pattern: `crates/whirlpool-node/src/main.rs` — current wiring: `Arc<EmptyBlockApp>` → `CommonwareEngine::new(app, sink, config, network_provider)`
  - Pattern: `crates/consensus-simplex/src/engine.rs` — CommonwareEngine constructor signature and type bounds
  - Pattern: `crates/consensus-simplex/src/sink.rs` — FinalizationSink for event handling
  - Types: `app::adapter::ApplicationAdapter`, `app_evm::config::{build_sahara_chain_spec, WhirlpoolEvmConfig}`, `app_evm::executor::EvmApplication`
  - Types: `state::db::InMemoryStateDb`
  - Types: `app::traits::NoopTxSource`
  - CRITICAL: Check if `ConsensusApp` bound requires `Clone`. If so, `ApplicationAdapter` must impl Clone (it should, since inner EvmApplication is Clone).
  - CRITICAL: Check if `CommonwareEngine` expects `Arc<A>` or `A` directly. Current main.rs uses `Arc<EmptyBlockApp>`.

  **Acceptance Criteria** (agent-executable only):
  - [ ] `nix develop --command cargo build` — default features (evm) builds successfully
  - [ ] `nix develop --command cargo build -p whirlpool-node --no-default-features` — EmptyBlockApp path still compiles
  - [ ] `nix develop --command cargo test` — all workspace tests pass
  - [ ] EmptyBlockApp tests in whirlpool-node still pass
  - [ ] No modifications to consensus/ or consensus-simplex/

  **QA Scenarios**:
  ```
  Scenario: EVM feature builds
    Tool: Bash
    Steps: nix develop --command cargo build 2>&1
    Expected: exit 0, full workspace builds with evm feature
    Evidence: .sisyphus/evidence/task-7-build-evm.txt

  Scenario: Non-EVM still builds
    Tool: Bash
    Steps: nix develop --command cargo build -p whirlpool-node --no-default-features 2>&1
    Expected: exit 0, EmptyBlockApp path compiles
    Evidence: .sisyphus/evidence/task-7-build-empty.txt

  Scenario: Existing tests unbroken
    Tool: Bash
    Steps: nix develop --command cargo test -p whirlpool-node -- --nocapture 2>&1
    Expected: All existing EmptyBlockApp tests still pass
    Evidence: .sisyphus/evidence/task-7-existing-tests.txt

  - [x] 8. Documentation — llmdocs Generation with ctx-update-doc Skill
    Tool: Bash
    Steps: git diff --name-only vendor/
    Expected: Empty output
    Evidence: .sisyphus/evidence/task-7-vendor-clean.txt
  ```

  **Commit**: YES | Message: `feat(whirlpool-node): wire EvmApplication via feature-gated evm path` | Files: `crates/whirlpool-node/src/main.rs`, `crates/whirlpool-node/Cargo.toml`

---

- [ ] 8. Documentation — Update llmdocs for New Crates

  **What to do**:
  1. Run `ctx-update-doc` skill for each new crate: state, app, app-evm
  2. Verify llmdocs are generated/updated in `llmdocs/` directory
  3. Review generated docs for accuracy against design docs
  **Must NOT do**: Do NOT hand-write documentation. Use the ctx-update-doc skill as required by AGENTS.md.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Skill invocation, minimal code changes
  - Skills: [`ctx-update-doc`] — Required by AGENTS.md
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: NO | Wave 5 (with Task 7) | Blocks: none | Blocked By: [5,6]

  **References**:
  - Rule: `AGENTS.md` — "After completing code changes, always use the ctx-update-doc skill"
  - Existing: `llmdocs/` directory — existing llmdocs structure
  - All source: `crates/state/src/**`, `crates/app/src/**`, `crates/app-evm/src/**`

  **Acceptance Criteria** (agent-executable only):
  - [ ] llmdocs updated for state, app, app-evm crates
  - [ ] `nix develop --command cargo build` still passes after doc updates

  **QA Scenarios**:
  ```
  Scenario: Llmdocs generated
    Tool: Bash
    Steps: ls llmdocs/ | grep -E '(state|app|app-evm)'
    Expected: Documentation files exist for all 3 new crates
    Evidence: .sisyphus/evidence/task-8-llmdocs.txt
  ```

  **Commit**: YES | Message: `docs: update llmdocs for state, app, and app-evm crates` | Files: `llmdocs/**`
## Final Verification Wave (4 parallel agents, ALL must APPROVE)

- [ ] 9. Plan Compliance Audit

  **What to do**: Verify every deliverable from INTENT.md is implemented. Cross-reference the 9 success criteria against actual code.
  **Must NOT do**: Make any code changes.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Requires thorough cross-referencing of design docs against implementation
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: YES | Wave 5 | Blocks: none | Blocked By: [8]

  **References**:
  - Design: `docs/design/evm-integration/INTENT.md` — 9 success criteria to verify
  - Design: `docs/design/evm-integration/CRATES.md` — crate structure to verify
  - Design: `docs/design/evm-integration/BLOCKERS.md` — resolved blockers to confirm

  **Acceptance Criteria**:
  - [ ] All 9 INTENT.md success criteria verified with evidence
  - [ ] All resolved blockers (B-001, B-002, B-R01, B-R02) confirmed implemented
  - [ ] No active blockers (B-003, B-004) accidentally in scope

  **QA Scenarios**:
  ```
  Scenario: Full compliance check
    Tool: Bash
    Steps: Read INTENT.md success criteria, grep for each deliverable in crate sources
    Expected: Each criterion maps to concrete implementation
    Evidence: .sisyphus/evidence/task-9-compliance.md

  Scenario: Scope boundary verification
    Tool: Bash
    Steps: git diff --stat to verify no vendor/ changes, no consensus/ trait changes
    Expected: Zero changes outside allowed scope
    Evidence: .sisyphus/evidence/task-9-scope.md
  ```

  **Commit**: NO

- [ ] 10. Code Quality Review

  **What to do**: Review all new code for Rust idioms, error handling, unsafe usage, documentation, clippy compliance. Run `nix develop --command cargo clippy -p state -p app -p app-evm`.
  **Must NOT do**: Make code changes (report only). Do NOT add new dependencies.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Code review requires broad Rust expertise
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: YES | Wave 5 | Blocks: none | Blocked By: [8]

  **References**:
  - Pattern: `crates/whirlpool-node/src/block.rs` — Reference Rust style
  - Pattern: `crates/consensus/src/app.rs` — Reference trait patterns

  **Acceptance Criteria**:
  - [ ] `nix develop --command cargo clippy -p state -p app -p app-evm -- -D warnings` passes
  - [ ] No `unsafe` blocks in new crates
  - [ ] All public items have doc comments

  **QA Scenarios**:
  ```
  Scenario: Clippy clean
    Tool: Bash
    Steps: nix develop --command cargo clippy -p state -p app -p app-evm -- -D warnings
    Expected: Exit code 0, no warnings
    Evidence: .sisyphus/evidence/task-10-clippy.txt

  Scenario: No unsafe code
    Tool: Bash
    Steps: grep -r "unsafe" crates/state/src crates/app/src crates/app-evm/src
    Expected: Zero matches
    Evidence: .sisyphus/evidence/task-10-unsafe.txt
  ```

  **Commit**: NO

- [ ] 11. Real Manual QA — Full Build + Test

  **What to do**: Clean build from scratch and run full test suite. Verify both default and `evm` feature configurations.
  **Must NOT do**: Skip any test. Do NOT modify code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: Comprehensive build+test verification
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: YES | Wave 5 | Blocks: none | Blocked By: [8]

  **References**:
  - Config: `Cargo.toml` (workspace root) — member list
  - Config: `flake.nix` — Nix dev shell

  **Acceptance Criteria**:
  - [ ] `nix develop --command cargo build` succeeds (default features)
  - [ ] `nix develop --command cargo build -p whirlpool-node --features evm` succeeds
  - [ ] `nix develop --command cargo test` — all tests pass (default)
  - [ ] `nix develop --command cargo test -p whirlpool-node --features evm` — all tests pass

  **QA Scenarios**:
  ```
  Scenario: Clean build default features
    Tool: Bash
    Steps: nix develop --command cargo build 2>&1
    Expected: Compiling... Finished, exit 0
    Evidence: .sisyphus/evidence/task-11-build-default.txt

  Scenario: Clean build EVM features
    Tool: Bash
    Steps: nix develop --command cargo build -p whirlpool-node --features evm 2>&1
    Expected: Compiling... Finished, exit 0
    Evidence: .sisyphus/evidence/task-11-build-evm.txt

  Scenario: Full test suite
    Tool: Bash
    Steps: nix develop --command cargo test 2>&1
    Expected: test result: ok. 0 failures
    Evidence: .sisyphus/evidence/task-11-tests.txt
  ```

  **Commit**: NO

- [ ] 12. Scope Fidelity Check

  **What to do**: Verify implementation matches design docs exactly. Check that no scope creep occurred (no persistence, no tx pool, no RPC, no MPT). Verify vendor/ untouched.
  **Must NOT do**: Make any changes.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Deep cross-referencing of design vs implementation
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser needed

  **Parallelization**: Can Parallel: YES | Wave 5 | Blocks: none | Blocked By: [8]

  **References**:
  - Design: `docs/design/evm-integration/INTENT.md` — scope boundaries
  - Design: `docs/design/evm-integration/BLOCKERS.md` — B-003, B-004 must NOT be implemented

  **Acceptance Criteria**:
  - [ ] No files in `vendor/` modified (git diff clean)
  - [ ] No RocksDB/MDBX references in new crates
  - [ ] No tx pool implementation
  - [ ] No RPC endpoints
  - [ ] State root uses flat keccak256 only (no MPT)

  **QA Scenarios**:
  ```
  Scenario: Vendor untouched
    Tool: Bash
    Steps: git diff --name-only vendor/
    Expected: Empty output (no changes)
    Evidence: .sisyphus/evidence/task-12-vendor.txt

  Scenario: No scope creep
    Tool: Bash
    Steps: grep -r "rocksdb\|mdbx\|txpool\|rpc_server\|merkle_patricia" crates/state/src crates/app/src crates/app-evm/src
    Expected: Zero matches
    Evidence: .sisyphus/evidence/task-12-scope.txt
  ```

  **Commit**: NO

## Commit Strategy
- Each task (1-8) creates its own atomic commit
- Final verification tasks (9-12) do NOT commit
- Commit messages follow conventional format: `feat(crate): description`
- After all tasks: one final commit for any doc updates (llmdocs)

## Success Criteria
1. All 3 new crates compile and pass tests independently
2. Full workspace `cargo build` and `cargo test` green
3. Feature-gated `evm` build for whirlpool-node works
4. Zero vendor modifications
5. Zero consensus crate modifications
6. Design doc success criteria 1-9 from INTENT.md all met
7. llmdocs updated for new crates
