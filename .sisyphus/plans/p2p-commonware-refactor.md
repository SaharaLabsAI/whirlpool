# Refactor p2p-commonware: Encapsulate Discovery Layer

## TL;DR
> **Summary**: Add a builder to `CommonwareNetworkProvider` in `p2p-commonware` that encapsulates `commonware-p2p` discovery setup, then remove the direct `commonware-p2p` dependency from `whirlpool-node`.
> **Deliverables**: Builder struct in p2p-commonware, refactored whirlpool-node main.rs + integration tests, removed vendor deps
> **Effort**: Medium
> **Parallel**: YES - 3 waves
> **Critical Path**: Task 1 (builder tests) → Task 2 (builder impl) → Task 3 (lib.rs exports) → Task 4 (main.rs refactor) + Task 5 (integration tests refactor) → Task 6 (dep cleanup) → Task 7 (llmdocs) → Final Verification

## Context
### Original Request
Bob wants whirlpool-node to depend ONLY on `p2p-commonware` (our vendor bridge crate), NOT directly on `commonware-p2p` (the vendor). Currently, whirlpool-node manually constructs `commonware_p2p::authenticated::discovery::Config`, `discovery::Network`, and `Oracle` — these are vendor internals that should be hidden behind p2p-commonware's API.

### Interview Summary
- **Builder pattern**: Add `CommonwareNetworkProviderBuilder<E, C>` that accepts config params and internally constructs the discovery layer
- **Oracle handle**: Builder takes initial validator set; `build()` returns `(CommonwareNetworkProvider, OracleHandle)` since `start()` consumes the provider
- **Runtime wrapping**: OUT of scope — `commonware_runtime::tokio::Runner` stays in whirlpool-node
- **Test strategy**: TDD (red-green-refactor)
- **Unaffected files**: `block.rs` (vendor consensus traits), `single_node.rs` (uses MockNetworkProvider)

### Metis Review (gaps addressed)
1. **Bootstrapper type**: Actual type is `Vec<Bootstrapper<C::PublicKey>>` (= `Vec<(C::PublicKey, Ingress)>`), not `Vec<SocketAddr>`. Builder will accept `Vec<SocketAddr>` for bootstrapper socket addrs only since all current callers pass `vec![]` and full bootstrapper info (with public keys) is needed only later. **Decision**: Builder accepts `Vec<Bootstrapper<C::PublicKey>>` re-exported from p2p-commonware. This is acceptable since p2p-commonware IS the vendor bridge.
2. **Post-start ownership**: `start(self)` consumes the provider, so `update_validators()` can't be on the provider. Solution: `build()` returns `(Provider, OracleHandle)` tuple where `OracleHandle` wraps a cloned `Oracle`.
3. **Missing `commonware-utils` dep**: p2p-commonware needs `commonware-utils` added to Cargo.toml for `Set::from_iter_dedup`.
4. **Existing `oracle()` accessor**: Returns `&Oracle` (immutable), useless for updates. Remove it — replaced by the `OracleHandle` returned from `build()`.
5. **`context.with_label()`**: Builder takes `context: E` directly — caller responsible for labeling.
6. **`Config::local()` only**: This builder is for local/dev setups. A production builder (using `Config::recommended()`) can be added later.

## Work Objectives
### Core Objective
Encapsulate `commonware-p2p` discovery setup inside `p2p-commonware`, so `whirlpool-node` has zero direct `commonware-p2p` imports.

### Deliverables
- `CommonwareNetworkProviderBuilder<E, C>` struct in `p2p-commonware/src/provider.rs`
- `OracleHandle<PK>` newtype in `p2p-commonware/src/provider.rs`
- Updated `p2p-commonware/src/lib.rs` with new re-exports
- Refactored `whirlpool-node/src/main.rs` using builder
- Refactored `whirlpool-node/tests/network_integration.rs` using builder
- Updated `Cargo.toml` files (added dep in p2p-commonware, removed deps in whirlpool-node)
- Updated llmdocs for both crates

### Definition of Done (verifiable conditions with commands)
```bash
# No vendor p2p imports in whirlpool-node
grep -r 'commonware_p2p' crates/whirlpool-node/src/ crates/whirlpool-node/tests/  # 0 matches
grep -r 'commonware_utils' crates/whirlpool-node/src/ crates/whirlpool-node/tests/  # 0 matches
grep -c 'commonware-p2p' crates/whirlpool-node/Cargo.toml  # 0
grep -c 'commonware-utils' crates/whirlpool-node/Cargo.toml  # 0
# All builds and tests pass
nix develop --command cargo build --workspace
nix develop --command cargo test --workspace
```

### Must Have
- Builder that hides discovery::Config, discovery::Network, Oracle construction
- OracleHandle for post-build validator set updates
- TDD: tests written before implementation
- All existing tests continue to pass with identical behavior

### Must NOT Have (guardrails)
- DO NOT modify the `NetworkProvider` trait in `crates/p2p/src/traits.rs`
- DO NOT modify any files in `vendor/**`
- DO NOT modify `crates/whirlpool-node/src/block.rs`
- DO NOT modify `crates/whirlpool-node/tests/single_node.rs`
- DO NOT change channel registration logic (VOTE=0, CERT=1, RESOLVER=2) or quota settings
- DO NOT refactor `MultiplexSender`, `MultiplexReceiver`, `CommonwareSender`, or `CommonwareReceiver`
- DO NOT add `commonware_runtime::tokio::Runner` wrapping — stays in whirlpool-node
- DO NOT add config validation in builder for network-level params (port ranges, etc.)
- DO NOT remove the existing `new()` / `with_config()` constructors — they can stay for direct construction use cases

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: **TDD** (red-green-refactor) with existing test framework (Rust built-in `#[cfg(test)]`)
- QA policy: Every task has agent-executed scenarios
- Build/test command prefix: `nix develop --command` (cargo is not on PATH)
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy
### Parallel Execution Waves

Wave 1: Foundation (2 tasks — sequential dependency within wave)
- Task 1: [TDD RED] Write builder + OracleHandle unit tests (p2p-commonware) — `deep`
- Task 2: [TDD GREEN] Implement builder + OracleHandle (p2p-commonware) — `deep`

Wave 2: Exports + Deps (2 tasks — parallel)
- Task 3: Update p2p-commonware lib.rs exports + Cargo.toml deps — `quick`
- Task 4: Update p2p-commonware Cargo.toml (add commonware-utils) — `quick` (MERGE into Task 3)

Wave 2 (actual, 1 task):
- Task 3: Update p2p-commonware lib.rs exports + Cargo.toml deps — `quick`

Wave 3: Consumer refactor (2 tasks — parallel)
- Task 4: Refactor whirlpool-node main.rs to use builder — `quick`
- Task 5: Refactor whirlpool-node integration tests to use builder — `quick`

Wave 4: Cleanup (2 tasks — parallel)
- Task 6: Remove vendor deps from whirlpool-node Cargo.toml — `quick`
- Task 7: Update llmdocs for p2p-commonware and whirlpool-node — `writing`

### Dependency Matrix
| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 (TDD RED) | — | 2 |
| 2 (TDD GREEN) | 1 | 3 |
| 3 (exports+deps) | 2 | 4, 5 |
| 4 (main.rs) | 3 | 6 |
| 5 (integration tests) | 3 | 6 |
| 6 (dep cleanup) | 4, 5 | 7 |
| 7 (llmdocs) | 6 | F1-F4 |

### Agent Dispatch Summary
| Wave | Tasks | Categories |
|------|-------|-----------|
| 1 | 2 | deep, deep |
| 2 | 1 | quick |
| 3 | 2 | quick, quick |
| 4 | 2 | quick, writing |
| Final | 4 | oracle, unspecified-high ×2, deep |

## TODOs

- [x] 1. [TDD RED] Write Builder + OracleHandle Unit Tests

  **What to do**: Create failing unit tests in `crates/p2p-commonware/src/tests.rs` (existing test file) for the new builder API. Tests should exercise:
  1. `CommonwareNetworkProviderBuilder::new(signer, namespace)` — basic construction
  2. `.listen_addr(addr)` / `.dialable_addr(addr)` / `.max_message_size(size)` — config setters
  3. `.bootstrappers(vec![])` — bootstrapper configuration
  4. `.initial_validators(epoch, validators)` — validator set seeding
  5. `.build(context)` — returns `(CommonwareNetworkProvider<E, C>, OracleHandle<C::PublicKey>)`
  6. `OracleHandle::update_validators(&mut self, epoch, validators)` — delegates to oracle.update()
  7. Edge case: empty validator set (should not panic)
  8. Edge case: validator set including self (signer's own pubkey)

  Write tests that **compile but fail** (red phase). The test should use `commonware_runtime::tokio::Runner` as executor and `ed25519::PrivateKey::from_seed()` for signers (same patterns as existing tests in `crates/p2p-commonware/src/tests.rs`).

  **Must NOT do**: Implement the builder yet — only write tests. Do not modify existing tests.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: TDD requires careful API design through test-first thinking
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [2] | Blocked By: []

  **References**:
  - Pattern: `crates/p2p-commonware/src/tests.rs` — existing test patterns, MockCwReceiver, use of commonware_runtime
  - Type: `crates/p2p-commonware/src/provider.rs:28-40` — current CommonwareNetworkProvider struct definition
  - Type: `crates/p2p-commonware/src/provider.rs:42-76` — current constructors (new, with_config, oracle)
  - Trait bounds: `E: Spawner + Clock + CryptoRngCore + Network + Resolver + Metrics, C: Signer` (from provider.rs line 42)
  - Vendor API: `vendor/commonware/p2p/src/authenticated/discovery/config.rs:170+` — `Config::local()` signature
  - External: `commonware_utils::ordered::Set::from_iter_dedup` for validator set construction

  **Acceptance Criteria**:
  - [ ] New tests exist in `crates/p2p-commonware/src/tests.rs`
  - [ ] Tests reference `CommonwareNetworkProviderBuilder` and `OracleHandle` types
  - [ ] `nix develop --command cargo test -p p2p-commonware` fails with compilation errors (types don't exist yet)
  - [ ] Existing tests in the file are NOT modified

  **QA Scenarios**:
  ```
  Scenario: Tests compile-fail correctly
    Tool: Bash
    Steps: Run `nix develop --command cargo test -p p2p-commonware 2>&1`
    Expected: Compilation error mentioning `CommonwareNetworkProviderBuilder` not found (proves RED phase)
    Evidence: .sisyphus/evidence/task-1-tdd-red.txt

  Scenario: Existing tests unaffected
    Tool: Bash
    Steps: Comment out new tests temporarily, run `nix develop --command cargo test -p p2p-commonware`
    Expected: All existing tests pass
    Evidence: .sisyphus/evidence/task-1-existing-tests.txt
  ```

  **Commit**: YES | Message: `test(p2p-commonware): add failing tests for network provider builder and oracle handle` | Files: `crates/p2p-commonware/src/tests.rs`

- [x] 2. [TDD GREEN] Implement CommonwareNetworkProviderBuilder + OracleHandle

  **What to do**: Implement the builder and oracle handle in `crates/p2p-commonware/src/provider.rs` to make the tests from Task 1 pass.

  **Implementation details**:

  1. **`OracleHandle<PK>`** (newtype):
     ```rust
     pub struct OracleHandle<PK: PublicKey>(Oracle<PK>);
     impl<PK: PublicKey + Clone + Hash + Eq + Debug + Send + Sync + 'static> OracleHandle<PK> {
         pub async fn update_validators(&mut self, epoch: u64, validators: impl IntoIterator<Item = PK>) {
             use commonware_utils::ordered::Set;
             self.0.update(epoch, Set::from_iter_dedup(validators.into_iter().collect::<Vec<_>>())).await;
         }
     }
     ```
     Note: `Oracle` is `Clone` (it wraps an UnboundedMailbox sender). Cloning gives another handle to the same mailbox.

  2. **`CommonwareNetworkProviderBuilder<E, C>`**:
     ```rust
     pub struct CommonwareNetworkProviderBuilder<E, C: Signer> {
         signer: C,
         namespace: Vec<u8>,
         listen_addr: SocketAddr,
         dialable_addr: SocketAddr,  // defaults to listen_addr
         bootstrappers: Vec<Bootstrapper<C::PublicKey>>,
         max_message_size: u32,
         initial_validators: Option<(u64, Vec<C::PublicKey>)>,
         channel_config: ChannelConfig,
         _phantom: PhantomData<E>,
     }
     ```
     Methods:
     - `new(signer: C, namespace: impl Into<Vec<u8>>) -> Self` — required params, rest default
     - `listen_addr(mut self, addr: SocketAddr) -> Self`
     - `dialable_addr(mut self, addr: SocketAddr) -> Self`
     - `bootstrappers(mut self, b: Vec<Bootstrapper<C::PublicKey>>) -> Self`
     - `max_message_size(mut self, size: u32) -> Self`
     - `initial_validators(mut self, epoch: u64, validators: Vec<C::PublicKey>) -> Self`
     - `channel_config(mut self, config: ChannelConfig) -> Self`
     - `build(self, context: E) -> (CommonwareNetworkProvider<E, C>, OracleHandle<C::PublicKey>)` — constructs `Config::local()`, `Network::new()`, calls `oracle.update()` if initial_validators set, clones oracle for handle, returns `(provider, handle)`

  3. **Remove `pub fn oracle(&self)`** — replaced by `OracleHandle` return from `build()`.

  4. **Keep `new()` and `with_config()`** constructors — they remain as low-level escape hatches.

  5. **Imports needed**: Add `use commonware_p2p::authenticated::discovery::config::Bootstrapper;` and `use commonware_utils::ordered::Set;` and `use std::marker::PhantomData;` and `use std::net::SocketAddr;`. Import `Manager` trait for `oracle.update()`.

  **Must NOT do**: Do not modify existing tests. Do not change `start()` implementation. Do not modify `MultiplexSender`/`MultiplexReceiver`. Do not modify `NetworkProvider` trait.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Core implementation with careful generic bounds and ownership semantics
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [3] | Blocked By: [1]

  **References**:
  - Pattern: `crates/p2p-commonware/src/provider.rs:42-76` — existing constructors to follow naming/style
  - Pattern: `crates/p2p-commonware/src/provider.rs:78-135` — start() impl showing how network/oracle are used
  - Vendor API: `vendor/commonware/p2p/src/authenticated/discovery/config.rs` — `Config::local()`, `Bootstrapper`, `Ingress` types
  - Vendor API: `vendor/commonware/p2p/src/authenticated/discovery/mod.rs` — `Network::new()` signature
  - Import: `commonware_p2p::Manager` trait — provides the `.update()` method on Oracle
  - Import: `commonware_utils::ordered::Set` — for `Set::from_iter_dedup()`

  **Acceptance Criteria**:
  - [ ] `CommonwareNetworkProviderBuilder` struct exists with all methods listed above
  - [ ] `OracleHandle` struct exists with `update_validators()` method
  - [ ] `nix develop --command cargo test -p p2p-commonware` passes (all tests including new ones from Task 1)
  - [ ] `nix develop --command cargo build -p p2p-commonware` passes
  - [ ] Existing `new()` and `with_config()` constructors still exist and work

  **QA Scenarios**:
  ```
  Scenario: All p2p-commonware tests pass (GREEN)
    Tool: Bash
    Steps: Run `nix develop --command cargo test -p p2p-commonware`
    Expected: All tests pass, including new builder tests from Task 1
    Evidence: .sisyphus/evidence/task-2-tdd-green.txt

  Scenario: Build succeeds with no warnings
    Tool: Bash
    Steps: Run `nix develop --command cargo build -p p2p-commonware 2>&1`
    Expected: Build succeeds. No errors.
    Evidence: .sisyphus/evidence/task-2-build.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): implement network provider builder and oracle handle` | Files: `crates/p2p-commonware/src/provider.rs`

- [x] 3. Update p2p-commonware Exports and Dependencies

  **What to do**:
  1. **`crates/p2p-commonware/Cargo.toml`**: Add `commonware-utils` dependency:
     ```toml
     commonware-utils = { path = "../../vendor/commonware/utils" }
     ```
  2. **`crates/p2p-commonware/src/lib.rs`**: Add re-exports for the new types and vendor types needed by consumers:
     - `pub use provider::{CommonwareNetworkProviderBuilder, OracleHandle};`
     - Re-export `commonware_p2p::authenticated::discovery::config::Bootstrapper;` (if used in builder's public API)
     - Re-export `commonware_p2p::authenticated::discovery::config::Ingress;` (if Bootstrapper needs it)
     Verify exact re-export needs by checking what types appear in the builder's public method signatures.

  **Must NOT do**: Do not change any existing re-exports. Do not modify sender/receiver/peer_id modules.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Small config and export changes
  - Skills: []

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: [4, 5] | Blocked By: [2]

  **References**:
  - File: `crates/p2p-commonware/Cargo.toml` — add commonware-utils dep
  - File: `crates/p2p-commonware/src/lib.rs` — existing re-exports pattern (lines 1-15)
  - File: `crates/p2p-commonware/src/provider.rs` — check which types are in public API signatures

  **Acceptance Criteria**:
  - [ ] `commonware-utils` is in `crates/p2p-commonware/Cargo.toml` `[dependencies]`
  - [ ] `CommonwareNetworkProviderBuilder` and `OracleHandle` are publicly re-exported from `crates/p2p-commonware/src/lib.rs`
  - [ ] Any vendor types used in builder's public API (e.g. `Bootstrapper`, `Ingress`) are re-exported
  - [ ] `nix develop --command cargo build -p p2p-commonware` succeeds
  - [ ] `nix develop --command cargo test -p p2p-commonware` passes

  **QA Scenarios**:
  ```
  Scenario: Exports accessible from external crate
    Tool: Bash
    Steps: Run `nix develop --command cargo build -p whirlpool-node` (should still compile since we haven't changed whirlpool-node yet)
    Expected: Build succeeds
    Evidence: .sisyphus/evidence/task-3-exports.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): export builder types and add commonware-utils dependency` | Files: `crates/p2p-commonware/Cargo.toml`, `crates/p2p-commonware/src/lib.rs`

- [x] 4. Refactor whirlpool-node main.rs to Use Builder

  **What to do**: Replace the manual discovery setup in `crates/whirlpool-node/src/main.rs` with the new builder API.

  **Current code to replace** (lines ~49-74):
  ```rust
  let signer = ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED);
  // ... discovery::Config::local(), discovery::Network::new(), oracle.update()
  let network_provider = CommonwareNetworkProvider::new(network, oracle);
  ```

  **New code**:
  ```rust
  let signer = ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED);
  let (network_provider, _oracle_handle) = CommonwareNetworkProviderBuilder::new(signer, APPLICATION_NAMESPACE)
      .listen_addr(listen_addr)
      .dialable_addr(dialable_addr)
      .max_message_size(MAX_MESSAGE_SIZE)
      // .initial_validators(0, vec![])  // empty set, can omit
      .build(context.with_label("network"));
  ```

  **Import changes**:
  - REMOVE: `use commonware_p2p::{Manager, authenticated::discovery};`
  - REMOVE: `use commonware_utils::ordered::Set;`
  - ADD: `use p2p_commonware::CommonwareNetworkProviderBuilder;`
  - KEEP: `use commonware_cryptography::{Signer, ed25519};` (needed for signer creation)
  - KEEP: `use commonware_runtime::{tokio, Runner, Metrics};` (needed for executor — out of scope)

  **Must NOT do**: Do not change block.rs. Do not change config constants. Do not change the engine start logic (lines 78+). Do not wrap the runtime executor.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Straightforward code replacement
  - Skills: []

  **Parallelization**: Can Parallel: YES (with Task 5) | Wave 3 | Blocks: [6] | Blocked By: [3]

  **References**:
  - File: `crates/whirlpool-node/src/main.rs:3-8` — current imports to modify
  - File: `crates/whirlpool-node/src/main.rs:49-74` — current discovery setup to replace
  - API: `CommonwareNetworkProviderBuilder::new(signer, namespace).listen_addr().dialable_addr().max_message_size().build(context)` — new builder API from Task 2

  **Acceptance Criteria**:
  - [ ] `grep -c 'commonware_p2p' crates/whirlpool-node/src/main.rs` outputs `0`
  - [ ] `grep -c 'commonware_utils' crates/whirlpool-node/src/main.rs` outputs `0`
  - [ ] `nix develop --command cargo build -p whirlpool-node` succeeds
  - [ ] `nix develop --command cargo run -p whirlpool-node` starts without panic (can be killed after startup)

  **QA Scenarios**:
  ```
  Scenario: No vendor p2p imports remain
    Tool: Bash
    Steps: Run `grep -E 'commonware_p2p|commonware_utils' crates/whirlpool-node/src/main.rs`
    Expected: No output (zero matches)
    Evidence: .sisyphus/evidence/task-4-no-vendor-imports.txt

  Scenario: Node starts successfully
    Tool: Bash
    Steps: Run `timeout 5 nix develop --command cargo run -p whirlpool-node 2>&1 || true`
    Expected: Process starts (may timeout, but no panic or compilation error)
    Evidence: .sisyphus/evidence/task-4-node-starts.txt
  ```

  **Commit**: YES | Message: `refactor(whirlpool-node): use p2p-commonware builder instead of direct discovery setup` | Files: `crates/whirlpool-node/src/main.rs`

- [x] 5. Refactor whirlpool-node Integration Tests to Use Builder

  **What to do**: Replace manual discovery setup in all 3 tests in `crates/whirlpool-node/tests/network_integration.rs` with the builder API.

  **Current pattern in each test** (repeated 3 times):
  ```rust
  let signer = ed25519::PrivateKey::from_seed(N);
  let p2p_cfg = discovery::Config::local(signer, namespace, listen, listen, bootstrappers, max_msg_size);
  let (network, mut oracle) = discovery::Network::new(context.with_label("..."), p2p_cfg);
  oracle.update(epoch, Set::from_iter_dedup(validators)).await;
  let network_provider = CommonwareNetworkProvider::new(network, oracle);
  ```

  **New pattern**:
  ```rust
  let signer = ed25519::PrivateKey::from_seed(N);
  let (network_provider, mut oracle_handle) = CommonwareNetworkProviderBuilder::new(signer, namespace)
      .listen_addr(listen)
      .max_message_size(max_msg_size)
      .initial_validators(epoch, validators)
      .build(context.with_label("..."));
  ```

  For the two-node test, also add `.bootstrappers(bootstrapper_list)` where applicable.

  **Import changes**:
  - REMOVE: `use commonware_p2p::authenticated::discovery;`
  - REMOVE: `use commonware_p2p::Manager;`
  - REMOVE: `use commonware_utils::ordered::Set;`
  - ADD: `use p2p_commonware::CommonwareNetworkProviderBuilder;`
  - KEEP: `use commonware_cryptography::{ed25519, Signer};` (needed for signer creation)
  - KEEP: `use commonware_runtime::{tokio as cw_tokio, Metrics, Runner};` (needed for test executor)

  Note: The two-node test (test 2) uses `oracle_handle.update_validators()` mid-test. Verify the `OracleHandle` supports this. Also, the two-node test may need bootstrapper info — currently passes `vec![]` for bootstrappers then does `oracle.update()` with both validators. Check if the builder's `initial_validators` is sufficient or if bootstrappers with public keys are needed.

  **Must NOT do**: Do not modify `tests/single_node.rs`. Do not change test assertions or expected behaviors. Do not change helper functions (`localhost_ephemeral`, `test_engine_config`).

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Mechanical code replacement across 3 tests
  - Skills: []

  **Parallelization**: Can Parallel: YES (with Task 4) | Wave 3 | Blocks: [6] | Blocked By: [3]

  **References**:
  - File: `crates/whirlpool-node/tests/network_integration.rs:1-30` — current imports
  - File: `crates/whirlpool-node/tests/network_integration.rs:63-140` — test 1 (single_node_real_network_lifecycle)
  - File: `crates/whirlpool-node/tests/network_integration.rs:148-267` — test 2 (two_nodes_discover_and_run)
  - File: `crates/whirlpool-node/tests/network_integration.rs:269-326` — test 3 (real_network_graceful_shutdown)
  - API: Builder from Task 2, OracleHandle from Task 2

  **Acceptance Criteria**:
  - [ ] `grep -c 'commonware_p2p' crates/whirlpool-node/tests/network_integration.rs` outputs `0`
  - [ ] `grep -c 'commonware_utils' crates/whirlpool-node/tests/network_integration.rs` outputs `0`
  - [ ] `nix develop --command cargo test -p whirlpool-node -- network_integration` passes (all 3 tests)
  - [ ] Test behavior is identical (same assertions, same timeouts)

  **QA Scenarios**:
  ```
  Scenario: All 3 integration tests pass
    Tool: Bash
    Steps: Run `nix develop --command cargo test -p whirlpool-node -- network_integration --nocapture 2>&1`
    Expected: 3 tests pass: single_node_real_network_lifecycle, two_nodes_discover_and_run, real_network_graceful_shutdown
    Evidence: .sisyphus/evidence/task-5-integration-tests.txt

  Scenario: No vendor imports remain in test file
    Tool: Bash
    Steps: Run `grep -E 'commonware_p2p|commonware_utils' crates/whirlpool-node/tests/network_integration.rs`
    Expected: No output (zero matches)
    Evidence: .sisyphus/evidence/task-5-no-vendor-imports.txt
  ```

  **Commit**: YES | Message: `refactor(whirlpool-node): use p2p-commonware builder in integration tests` | Files: `crates/whirlpool-node/tests/network_integration.rs`

- [x] 6. Remove Vendor Dependencies from whirlpool-node Cargo.toml

  **What to do**: Remove `commonware-p2p` and `commonware-utils` from `crates/whirlpool-node/Cargo.toml` dependencies.

  **Lines to remove**:
  ```toml
  commonware-p2p = { path = "../../vendor/commonware/p2p" }
  commonware-utils = { path = "../../vendor/commonware/utils" }
  ```

  Verify with `cargo build` that no transitive dependency requires these.

  **Must NOT do**: Do not remove other vendor deps (commonware-consensus, commonware-codec, commonware-cryptography, commonware-runtime — all needed by block.rs and main.rs).

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Two-line Cargo.toml edit
  - Skills: []

  **Parallelization**: Can Parallel: YES (with Task 7) | Wave 4 | Blocks: [7] | Blocked By: [4, 5]

  **References**:
  - File: `crates/whirlpool-node/Cargo.toml` — lines containing `commonware-p2p` and `commonware-utils`

  **Acceptance Criteria**:
  - [ ] `grep -c 'commonware-p2p' crates/whirlpool-node/Cargo.toml` outputs `0`
  - [ ] `grep -c 'commonware-utils' crates/whirlpool-node/Cargo.toml` outputs `0`
  - [ ] `nix develop --command cargo build -p whirlpool-node` succeeds
  - [ ] `nix develop --command cargo test -p whirlpool-node` passes
  - [ ] `nix develop --command cargo build --workspace` succeeds
  - [ ] `nix develop --command cargo test --workspace` succeeds

  **QA Scenarios**:
  ```
  Scenario: Full workspace builds and tests after dep removal
    Tool: Bash
    Steps: Run `nix develop --command cargo build --workspace && nix develop --command cargo test --workspace`
    Expected: All builds and tests pass
    Evidence: .sisyphus/evidence/task-6-workspace-clean.txt

  Scenario: Verify removed deps are gone
    Tool: Bash
    Steps: Run `grep -E 'commonware-p2p|commonware-utils' crates/whirlpool-node/Cargo.toml`
    Expected: No output
    Evidence: .sisyphus/evidence/task-6-deps-removed.txt
  ```

  **Commit**: YES | Message: `refactor(whirlpool-node): remove direct commonware-p2p and commonware-utils dependencies` | Files: `crates/whirlpool-node/Cargo.toml`

- [ ] 7. Update llmdocs for Affected Crates

  **What to do**: Use the `ctx-update-doc` skill to update llmdocs for both `p2p-commonware` and `whirlpool-node` crates. The documentation should reflect:
  - New `CommonwareNetworkProviderBuilder` and `OracleHandle` types in p2p-commonware
  - Removed vendor dependency in whirlpool-node
  - Updated usage patterns showing builder instead of manual discovery setup

  **Must NOT do**: Do not manually write markdown — use the `ctx-update-doc` skill.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: Documentation update
  - Skills: [`ctx-update-doc`] — Required for llmdocs workflow

  **Parallelization**: Can Parallel: YES (with Task 6 if deps already removed) | Wave 4 | Blocks: [F1-F4] | Blocked By: [6]

  **References**:
  - File: `llmdocs/crates/p2p-commonware.md` — existing docs to update
  - File: `llmdocs/architecture/whirlpool-node.md` — existing architecture docs
  - File: `llmdocs/guides/whirlpool-node-components.md` — existing component guide
  - Skill: `ctx-update-doc`

  **Acceptance Criteria**:
  - [ ] `llmdocs/crates/p2p-commonware.md` mentions `CommonwareNetworkProviderBuilder` and `OracleHandle`
  - [ ] `llmdocs/architecture/whirlpool-node.md` reflects that whirlpool-node no longer depends on commonware-p2p directly
  - [ ] Documentation accurately describes the builder API

  **QA Scenarios**:
  ```
  Scenario: Docs mention new types
    Tool: Bash
    Steps: Run `grep -l 'CommonwareNetworkProviderBuilder' llmdocs/`
    Expected: At least one file matches
    Evidence: .sisyphus/evidence/task-7-docs-updated.txt
  ```

  **Commit**: YES | Message: `docs: update llmdocs for p2p-commonware builder refactor` | Files: `llmdocs/crates/p2p-commonware.md`, `llmdocs/architecture/whirlpool-node.md`, `llmdocs/guides/whirlpool-node-components.md`

## Final Verification Wave (4 parallel agents, ALL must APPROVE)

- [ ] F1. Plan Compliance Audit — oracle
  Verify all tasks were executed per plan. Check that no guardrails were violated. Confirm all acceptance criteria met.

- [ ] F2. Code Quality Review — unspecified-high
  Review all changed files for code quality: proper error handling, no unwraps on fallible operations, proper generic bounds, no dead code, idiomatic Rust patterns.

- [ ] F3. Real Manual QA — unspecified-high
  Execute full workspace build and test suite. Verify zero `commonware_p2p` references in whirlpool-node. Run each integration test individually.

- [ ] F4. Scope Fidelity Check — deep
  Verify no scope creep: `vendor/**` untouched, `NetworkProvider` trait untouched, `block.rs` untouched, `single_node.rs` untouched, no runtime wrapping added.

## Commit Strategy
Sequential commits per task (7 commits + final squash optional):
1. `test(p2p-commonware): add failing tests for network provider builder and oracle handle`
2. `feat(p2p-commonware): implement network provider builder and oracle handle`
3. `feat(p2p-commonware): export builder types and add commonware-utils dependency`
4. `refactor(whirlpool-node): use p2p-commonware builder instead of direct discovery setup`
5. `refactor(whirlpool-node): use p2p-commonware builder in integration tests`
6. `refactor(whirlpool-node): remove direct commonware-p2p and commonware-utils dependencies`
7. `docs: update llmdocs for p2p-commonware builder refactor`

## Success Criteria
1. `grep -r 'commonware_p2p' crates/whirlpool-node/src/ crates/whirlpool-node/tests/` → zero matches
2. `grep -r 'commonware_utils' crates/whirlpool-node/src/ crates/whirlpool-node/tests/` → zero matches
3. `commonware-p2p` NOT in `crates/whirlpool-node/Cargo.toml`
4. `commonware-utils` NOT in `crates/whirlpool-node/Cargo.toml`
5. `nix develop --command cargo build --workspace` succeeds
6. `nix develop --command cargo test --workspace` succeeds
7. New `CommonwareNetworkProviderBuilder` and `OracleHandle` types available in p2p-commonware
8. All existing tests pass with identical behavior
