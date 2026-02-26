# Wire Real Network in p2p-commonware

## TL;DR
> **Summary**: Replace the factory-closure-based `CommonwareNetworkProvider` with a concrete implementation that takes a pre-built commonware `discovery::Network`, registers the 3 known channels (VOTE, CERTIFICATE, RESOLVER), and multiplexes them through the single `NetworkSender`/`NetworkReceiver` trait interface. Wire this into `whirlpool-node/main.rs` replacing `MockNetworkProvider`.
> **Deliverables**: Redesigned provider, multiplexing sender/receiver, proper error mapping, updated main.rs, TDD test suite
> **Effort**: Medium
> **Parallel**: YES - 3 waves
> **Critical Path**: Task 1 (error mapping) → Task 2 (TDD tests) → Task 3 (multiplex sender) + Task 4 (multiplex receiver) → Task 5 (provider redesign) → Task 6 (wire main.rs) → Task 7 (integration test)

## Context
### Original Request
Wire real network in p2p-commonware instead of MockNetworkProvider in p2p.

### Interview Summary
- **Channel strategy**: Hardcode 3 channels (VOTE=0, CERTIFICATE=1, RESOLVER=2) — matches `p2p::types::Channel` constants
- **Provider ownership**: Take pre-built `discovery::Network` + `Oracle` — caller owns runtime/config lifecycle
- **Test strategy**: TDD — write failing tests first, implement to pass. Existing mock tests preserved behind `feature = "mock"`
- **Scope**: Provider redesign, adapter fixes, main.rs wiring, tests. NOT consensus engine wiring, CLI config, or deployment

### Metis Review (gaps addressed)
1. **Handle lifetime**: `network.start()` returns `Handle<()>` — must be stored or returned to caller to prevent network shutdown. **Decision**: Return handle from `start()` alongside sender/receiver.
2. **Oracle exposure**: Provider takes `Oracle` but doesn't own peer set management. **Decision**: Caller configures oracle before passing to provider; oracle is consumed by provider (stored for potential future use or dropped).
3. **Rate quota defaults**: Each `network.register()` call needs a `Quota` and `backlog`. **Decision**: Use sensible defaults (no rate limiting initially; backlog=1024). Accept as config parameter.
4. **Error mapping**: Current `map_error` maps everything to `InvalidRecipients`. **Decision**: Map to appropriate P2pError variants based on error type/context.
5. **Receiver fairness**: `tokio::select!` can starve channels. **Decision**: Use biased select with round-robin bias rotation, or `futures::select_all` on a stream. Simplest first: `tokio::select!` with TODO for fairness if needed.
6. **Shutdown**: Dropping the Handle shuts down network. **Decision**: Documented in return type, caller manages lifecycle.
7. **Generic bounds simplification**: Factory closure generic `F` approach is over-engineered. **Decision**: Make provider concrete over `discovery::Network` types.
8. **Thread safety**: `NetworkSender::send(&self)` but commonware sender needs `&mut self`. **Decision**: Each channel sender wrapped in clone-and-mutate pattern (already used in current `CommonwareSender`).

## Work Objectives
### Core Objective
Make `CommonwareNetworkProvider` use real commonware `discovery::Network` with proper multi-channel multiplexing, replacing the mock/stub factory approach.

### Deliverables
- Redesigned `CommonwareNetworkProvider` that accepts pre-built `discovery::Network`
- `MultiplexSender` that routes `send(channel, data, recipients)` to the correct per-channel commonware sender
- `MultiplexReceiver` that merges all per-channel commonware receivers into a single stream with channel tagging
- Proper error mapping from commonware errors to `P2pError` variants
- `whirlpool-node/main.rs` wired to use `CommonwareNetworkProvider` instead of `MockNetworkProvider`
- TDD test suite for all adapter components

### Definition of Done (verifiable conditions with commands)
- `nix develop --command cargo build` passes with no errors
- `nix develop --command cargo test -p p2p-commonware` passes all new tests
- `nix develop --command cargo test -p whirlpool-node` passes (existing + new tests)
- `nix develop --command cargo test` (workspace) passes
- `MockNetworkProvider` usage removed from `main.rs` (but preserved in test code behind feature flag)

### Must Have
- Multi-channel support (VOTE, CERTIFICATE, RESOLVER)
- Proper error type mapping
- `network.start()` Handle returned to caller
- Channel-tagged inbound messages (correct channel on `NetworkMessage`)
- Send routing by channel parameter

### Must NOT Have (guardrails)
- DO NOT modify `vendor/**` files
- DO NOT change the `p2p` crate trait definitions (`NetworkProvider`, `NetworkSender`, `NetworkReceiver`)
- DO NOT modify `consensus-simplex` engine code
- DO NOT add CLI argument parsing or config file loading
- DO NOT change `MockNetworkProvider` (it stays behind `feature = "mock"`)
- DO NOT implement peer set management / oracle.update() logic — that's the caller's responsibility
- DO NOT add production TLS/security configuration

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: TDD + `#[tokio::test]` with commonware deterministic runtime where applicable
- QA policy: Every task has agent-executed scenarios
- Evidence: .sisyphus/evidence/task-{N}-{slug}.{ext}

## Execution Strategy
### Parallel Execution Waves

Wave 1 (foundation): Tasks 1, 2 [error mapping + TDD test scaffolding]
Wave 2 (adapters): Tasks 3, 4 [multiplex sender + receiver — parallel, both depend on error mapping]
Wave 3 (integration): Tasks 5, 6, 7 [provider redesign → main.rs wiring → integration test — sequential]

### Dependency Matrix
| Task | Depends On | Blocks |
|------|-----------|--------|
| 1. Fix error mapping | — | 3, 4, 5 |
| 2. TDD test scaffolding | — | 3, 4, 5 |
| 3. MultiplexSender | 1, 2 | 5 |
| 4. MultiplexReceiver | 1, 2 | 5 |
| 5. Provider redesign | 3, 4 | 6 |
| 6. Wire main.rs | 5 | 7 |
| 7. Integration test | 6 | — |

### Agent Dispatch Summary
| Wave | Tasks | Categories |
|------|-------|-----------|
| 1 | 2 | deep, deep |
| 2 | 2 | deep, deep |
| 3 | 3 | deep, deep, deep |

## TODOs

- [x] 1. Fix error mapping in `p2p-commonware`

  **What to do**:
  Replace the catch-all `map_error` function in `crates/p2p-commonware/src/error.rs` with proper error mapping from commonware errors to appropriate `P2pError` variants.

  Current code maps ALL errors to `P2pError::InvalidRecipients` — this is semantically wrong. Implement context-aware error mapping:
  - Send failures → `P2pError::SendFailed(msg)`
  - Receive failures → `P2pError::ReceiveFailed(msg)`
  - Channel-related → `P2pError::InvalidChannel(id)`
  - Network shutdown → `P2pError::NetworkShutdown`
  - Invalid recipients → `P2pError::InvalidRecipients(msg)` (keep for actual recipient errors)

  Replace the single `map_error` function with multiple context-specific helpers:
  ```rust
  pub fn map_send_error<E: Display>(e: E) -> P2pError { P2pError::SendFailed(e.to_string()) }
  pub fn map_recv_error<E: Display>(e: E) -> P2pError { P2pError::ReceiveFailed(e.to_string()) }
  ```

  **Must NOT do**: Change `P2pError` enum variants (that's in the `p2p` crate).

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Requires understanding error semantics across crate boundaries
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No browser interaction

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [3, 4, 5] | Blocked By: []

  **References**:
  - Current error mapping: `crates/p2p-commonware/src/error.rs` — single `map_error` function that maps all errors to `InvalidRecipients`
  - P2pError definition: `crates/p2p/src/errors.rs` — `P2pError` enum with variants: `ChannelFull`, `SendFailed(String)`, `ReceiveFailed(String)`, `NetworkShutdown`, `InvalidChannel(u64)`, `InvalidRecipients(String)`
  - Usage in sender.rs: `crates/p2p-commonware/src/sender.rs:52` — `.map_err(map_error)` on send result
  - Usage to add in receiver.rs: `crates/p2p-commonware/src/receiver.rs:40` — `Err(_) => None` should log/map error

  **Acceptance Criteria** (agent-executable only):
  - [ ] `map_error` replaced with context-specific `map_send_error`, `map_recv_error` helpers
  - [ ] `sender.rs` uses `map_send_error` instead of `map_error`
  - [ ] All existing tests pass: `nix develop --command cargo test -p p2p-commonware`
  - [ ] `nix develop --command cargo build -p p2p-commonware` compiles

  **QA Scenarios** (MANDATORY):
  ```
  Scenario: Send error maps to SendFailed
    Tool: Bash
    Steps: Write a test that calls map_send_error with a std::io::Error, asserts result is P2pError::SendFailed
    Expected: Test passes, error message preserved in SendFailed variant
    Evidence: .sisyphus/evidence/task-1-error-mapping.txt

  Scenario: Recv error maps to ReceiveFailed
    Tool: Bash
    Steps: Write a test that calls map_recv_error, asserts result is P2pError::ReceiveFailed
    Expected: Test passes, error message preserved
    Evidence: .sisyphus/evidence/task-1-error-mapping-recv.txt
  ```

  **Commit**: YES | Message: `fix(p2p-commonware): use context-specific error mapping instead of catch-all InvalidRecipients` | Files: `crates/p2p-commonware/src/error.rs`, `crates/p2p-commonware/src/sender.rs`

- [x] 2. TDD: Write failing tests for MultiplexSender, MultiplexReceiver, and provider

  **What to do**:
  Write comprehensive test scaffolding in `crates/p2p-commonware/src/tests.rs` that defines the expected behavior of the new multiplexing types. These tests MUST fail initially (RED phase of TDD).

  Tests to write:
  1. **MultiplexSender tests**:
     - `test_multiplex_sender_routes_vote_channel` — send on Channel::VOTE routes to channel 0 sender
     - `test_multiplex_sender_routes_certificate_channel` — send on Channel::CERTIFICATE routes to channel 1 sender
     - `test_multiplex_sender_routes_resolver_channel` — send on Channel::RESOLVER routes to channel 2 sender
     - `test_multiplex_sender_invalid_channel` — send on unknown channel returns `P2pError::InvalidChannel`
     - `test_multiplex_sender_clone` — cloned sender works independently

  2. **MultiplexReceiver tests**:
     - `test_multiplex_receiver_tags_channel` — message from channel 0 receiver has `Channel::VOTE`
     - `test_multiplex_receiver_merges_channels` — messages from all 3 channels are received
     - `test_multiplex_receiver_returns_none_on_shutdown` — when all senders drop, recv returns None

  3. **Provider tests**:
     - `test_provider_start_returns_sender_receiver` — start() succeeds
     - `test_provider_registers_three_channels` — verify 3 channels registered

  Use commonware's simulated network or mock channels (tokio::mpsc) to create test doubles for the commonware Sender/Receiver types. Import `commonware_p2p::Sender as CwSender` and `commonware_p2p::Receiver as CwReceiver` traits.

  **Must NOT do**: Implement the actual MultiplexSender/MultiplexReceiver types yet (tests should fail). DO define the struct signatures as empty stubs so tests compile but fail.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Requires careful test design with proper mock setup for commonware types
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [3, 4, 5] | Blocked By: []

  **References**:
  - Existing test patterns: `crates/p2p-commonware/src/tests.rs` — uses `ed25519::PrivateKey::from_seed()` for test keys
  - Mock pattern: `crates/p2p/src/mock.rs` — MockNetworkProvider uses tokio::mpsc
  - Channel constants: `crates/p2p/src/types.rs` — `Channel::VOTE(0)`, `Channel::CERTIFICATE(1)`, `Channel::RESOLVER(2)`
  - Commonware sender trait: `vendor/commonware/p2p/src/lib.rs` — `trait UnlimitedSender { fn send(&mut self, recipients, message: impl Buf, priority: bool) }`
  - Commonware receiver trait: `vendor/commonware/p2p/src/lib.rs` — `trait Receiver { fn recv(&mut self) -> Result<(PublicKey, Bytes), Error> }`
  - Simulated network: `vendor/commonware/p2p/src/simulated/` — test harness with configurable latency

  **Acceptance Criteria** (agent-executable only):
  - [ ] Test file compiles: `nix develop --command cargo test -p p2p-commonware --no-run`
  - [ ] Tests FAIL (RED phase): `nix develop --command cargo test -p p2p-commonware` shows test failures
  - [ ] At least 8 test functions defined covering sender, receiver, and provider
  - [ ] Stub types (MultiplexSender, MultiplexReceiver) exist but are unimplemented

  **QA Scenarios**:
  ```
  Scenario: Tests compile but fail
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware 2>&1
    Expected: Compilation succeeds, test execution shows FAILED for new tests, existing tests still pass
    Evidence: .sisyphus/evidence/task-2-tdd-red.txt
  ```

  **Commit**: YES | Message: `test(p2p-commonware): add TDD test scaffolding for multiplex sender/receiver/provider` | Files: `crates/p2p-commonware/src/tests.rs`, `crates/p2p-commonware/src/lib.rs`

- [x] 3. Implement MultiplexSender

  **What to do**:
  Create `crates/p2p-commonware/src/sender.rs` with a `MultiplexSender` that wraps a `HashMap<Channel, CommonwareSender<S>>` and routes `send()` calls to the correct per-channel sender.

  Implementation:
  ```rust
  use std::collections::HashMap;
  use std::sync::Arc;

  #[derive(Clone)]
  pub struct MultiplexSender<S> {
      senders: Arc<HashMap<Channel, CommonwareSender<S>>>,
  }

  impl<S> MultiplexSender<S> {
      pub fn new(senders: HashMap<Channel, CommonwareSender<S>>) -> Self {
          Self { senders: Arc::new(senders) }
      }
  }

  impl<S> NetworkSender for MultiplexSender<S>
  where S: CwSender + Clone + Send + Sync + 'static,
        S::PublicKey: Clone + Eq + Hash + Debug + Send + Sync + 'static,
  {
      type PeerId = CommonwarePeerId<S::PublicKey>;

      async fn send(&self, channel: Channel, data: Bytes, recipients: Recipients<Self::PeerId>) -> Result<(), P2pError> {
          let sender = self.senders.get(&channel)
              .ok_or(P2pError::InvalidChannel(channel.0))?;
          sender.send(channel, data, recipients).await
      }
  }
  ```

  Keep the existing `CommonwareSender<S>` (single-channel adapter) as an internal building block — `MultiplexSender` delegates to it.

  **Must NOT do**: Remove `CommonwareSender` — it's still useful as the per-channel adapter. Don't change the `NetworkSender` trait.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Requires careful generic bounds management and understanding of clone/Arc patterns
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: YES (with Task 4) | Wave 2 | Blocks: [5] | Blocked By: [1, 2]

  **References**:
  - Current sender: `crates/p2p-commonware/src/sender.rs` — `CommonwareSender<S>` wraps CwSender, ignores `_channel` param
  - Channel type: `crates/p2p/src/types.rs:8-18` — `Channel(pub u64)` with VOTE=0, CERTIFICATE=1, RESOLVER=2; implements Eq, Hash
  - NetworkSender trait: `crates/p2p/src/traits.rs:22-44` — `send(&self, channel, data, recipients)`
  - Error mapping: `crates/p2p-commonware/src/error.rs` — use `map_send_error` (after Task 1)

  **Acceptance Criteria** (agent-executable only):
  - [ ] `MultiplexSender` struct defined with `HashMap<Channel, CommonwareSender<S>>` wrapped in `Arc`
  - [ ] `MultiplexSender` implements `NetworkSender`
  - [ ] `MultiplexSender` implements `Clone`
  - [ ] Send on known channel delegates to correct inner sender
  - [ ] Send on unknown channel returns `P2pError::InvalidChannel`
  - [ ] TDD tests from Task 2 for sender now PASS: `nix develop --command cargo test -p p2p-commonware multiplex_sender`
  - [ ] `nix develop --command cargo build -p p2p-commonware` compiles

  **QA Scenarios**:
  ```
  Scenario: Route to correct channel sender
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware test_multiplex_sender_routes
    Expected: All 3 channel routing tests pass
    Evidence: .sisyphus/evidence/task-3-sender-routes.txt

  Scenario: Invalid channel rejected
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware test_multiplex_sender_invalid
    Expected: Test passes, returns P2pError::InvalidChannel
    Evidence: .sisyphus/evidence/task-3-sender-invalid.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): implement MultiplexSender for multi-channel routing` | Files: `crates/p2p-commonware/src/sender.rs`, `crates/p2p-commonware/src/lib.rs`

- [x] 4. Implement MultiplexReceiver

  **What to do**:
  Create `crates/p2p-commonware/src/receiver.rs` with a `MultiplexReceiver` that merges multiple per-channel commonware receivers into a single stream, tagging each message with its source channel.

  Implementation approach:
  ```rust
  pub struct MultiplexReceiver<R> {
      receivers: Vec<(Channel, CommonwareReceiver<R>)>,
  }

  impl<R> MultiplexReceiver<R> {
      pub fn new(receivers: Vec<(Channel, CommonwareReceiver<R>)>) -> Self {
          Self { receivers }
      }
  }
  ```

  For `NetworkReceiver::recv()`, use `tokio::select!` macro to poll all receivers concurrently. Since we have exactly 3 channels (fixed), a manual 3-branch select is acceptable:
  ```rust
  async fn recv(&mut self) -> Option<NetworkMessage<Self::PeerId>> {
      // Use select! across all 3 channel receivers
      // When a message arrives, tag it with the correct channel
      // When a receiver returns None, remove it from polling
      // When all receivers are done, return None
  }
  ```

  Fix the current bug: `CommonwareReceiver` hardcodes `Channel(0)` — the `MultiplexReceiver` must tag with the correct channel for each receiver.

  Keep the existing `CommonwareReceiver<R>` (single-channel adapter) as an internal building block — it handles the commonware-to-p2p message conversion per channel.

  **Must NOT do**: Remove `CommonwareReceiver` — it's the per-channel adapter. Don't use `unsafe`. Don't change `NetworkReceiver` trait.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Async select! patterns with ownership management are tricky
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: YES (with Task 3) | Wave 2 | Blocks: [5] | Blocked By: [1, 2]

  **References**:
  - Current receiver: `crates/p2p-commonware/src/receiver.rs` — `CommonwareReceiver<R>` wraps CwReceiver, hardcodes `Channel(0)`
  - NetworkReceiver trait: `crates/p2p/src/traits.rs:51-65` — `recv(&mut self) -> Option<NetworkMessage<PeerId>>`
  - NetworkMessage: `crates/p2p/src/types.rs:50-56` — struct with `channel`, `data`, `peer_id` fields
  - Channel constants: `crates/p2p/src/types.rs:8-18` — VOTE=0, CERTIFICATE=1, RESOLVER=2

  **Acceptance Criteria** (agent-executable only):
  - [ ] `MultiplexReceiver` struct defined with `Vec<(Channel, CommonwareReceiver<R>)>`
  - [ ] `MultiplexReceiver` implements `NetworkReceiver`
  - [ ] Messages are tagged with correct channel (not hardcoded Channel(0))
  - [ ] Returns `None` when all channel receivers are exhausted
  - [ ] TDD tests from Task 2 for receiver now PASS: `nix develop --command cargo test -p p2p-commonware multiplex_receiver`
  - [ ] `nix develop --command cargo build -p p2p-commonware` compiles

  **QA Scenarios**:
  ```
  Scenario: Messages tagged with correct channel
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware test_multiplex_receiver_tags
    Expected: Test passes, each message has correct channel field
    Evidence: .sisyphus/evidence/task-4-receiver-tags.txt

  Scenario: All channels merged
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware test_multiplex_receiver_merges
    Expected: Test passes, messages from all 3 channels received
    Evidence: .sisyphus/evidence/task-4-receiver-merges.txt

  Scenario: Shutdown propagation
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware test_multiplex_receiver_returns_none
    Expected: Test passes, recv returns None when all inner receivers close
    Evidence: .sisyphus/evidence/task-4-receiver-shutdown.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): implement MultiplexReceiver for multi-channel message merging` | Files: `crates/p2p-commonware/src/receiver.rs`, `crates/p2p-commonware/src/lib.rs`

- [ ] 5. Redesign CommonwareNetworkProvider to use discovery::Network

  **What to do**:
  Replace the factory-closure-based `CommonwareNetworkProvider<F>` with a concrete provider that takes a pre-built commonware `discovery::Network` and `Oracle`, registers the 3 hardcoded channels, and returns `MultiplexSender`/`MultiplexReceiver` from `start()`.

  New provider design:
  ```rust
  use commonware_p2p::authenticated::discovery;

  pub struct CommonwareNetworkProvider<E, C>
  where
      E: Spawner + Clock + ...,
      C: Signer,
  {
      network: discovery::Network<E, C>,
      oracle: discovery::Oracle<C::PublicKey>,
      channel_config: ChannelConfig,
  }

  pub struct ChannelConfig {
      pub backlog: usize,    // default: 1024
      // Rate quota can be added later
  }

  impl<E, C> NetworkProvider for CommonwareNetworkProvider<E, C> { ... }
  ```

  The `start(self)` method:
  1. Calls `self.network.register(Channel::VOTE.0, quota, backlog)` → `(vote_sender, vote_receiver)`
  2. Calls `self.network.register(Channel::CERTIFICATE.0, quota, backlog)` → `(cert_sender, cert_receiver)`
  3. Calls `self.network.register(Channel::RESOLVER.0, quota, backlog)` → `(res_sender, res_receiver)`
  4. Calls `self.network.start()` → `handle`
  5. Builds `MultiplexSender` from the 3 senders
  6. Builds `MultiplexReceiver` from the 3 receivers
  7. Returns `Ok((multiplex_sender, multiplex_receiver))` — note: handle must be stored or returned

  **Handle management**: The `NetworkProvider` trait returns `(Sender, Receiver)` only. The Handle must be stored somewhere. Options:
  - Wrap handle in the MultiplexReceiver (drop receiver = shutdown network)
  - Change return type (but we can't change the trait). 
  - **Best option**: Store handle in MultiplexReceiver via a `_handle: Handle<()>` field. When receiver is dropped, handle drops, network shuts down. This is natural since the receiver is the last to be dropped (it runs in the recv loop).

  Also update `Cargo.toml` to add the `commonware-runtime` dependency (needed for trait bounds on E).

  **Must NOT do**: Change the `NetworkProvider` trait. Own the runtime lifecycle. Add CLI argument parsing.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Complex generic bounds, trait satisfaction, and lifecycle management
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: [6] | Blocked By: [3, 4]

  **References**:
  - Current provider: `crates/p2p-commonware/src/provider.rs` — factory closure approach, 52 lines
  - discovery::Network API: `vendor/commonware/p2p/src/authenticated/discovery/mod.rs` — `register(channel, rate, backlog)`, `start()`
  - discovery config: `vendor/commonware/p2p/src/authenticated/discovery/config.rs` — `Config::test()`, `Config::local()`
  - NetworkProvider trait: `crates/p2p/src/traits.rs:72-96` — `start(self) -> Result<(Sender, Receiver), P2pError>`
  - Channel constants: `crates/p2p/src/types.rs:8-18` — VOTE=0, CERT=1, RESOLVER=2
  - Runtime context types: commonware_runtime::tokio::Context satisfies E bounds (Spawner + Clock + Network + Resolver + Metrics)
  - Signer: `vendor/commonware/cryptography/src/lib.rs` — ed25519::PrivateKey implements Signer

  **Acceptance Criteria** (agent-executable only):
  - [ ] `CommonwareNetworkProvider` no longer generic over factory closure `F`
  - [ ] `CommonwareNetworkProvider::new()` takes `discovery::Network` and `Oracle`
  - [ ] `start()` registers 3 channels and returns `(MultiplexSender, MultiplexReceiver)`
  - [ ] Handle stored in MultiplexReceiver (network stays alive while receiver lives)
  - [ ] TDD tests from Task 2 for provider now PASS
  - [ ] `nix develop --command cargo build -p p2p-commonware` compiles
  - [ ] `nix develop --command cargo test -p p2p-commonware` all tests pass (GREEN phase)

  **QA Scenarios**:
  ```
  Scenario: All TDD tests pass (GREEN)
    Tool: Bash
    Steps: nix develop --command cargo test -p p2p-commonware 2>&1
    Expected: ALL tests pass — zero failures
    Evidence: .sisyphus/evidence/task-5-all-green.txt

  Scenario: Build succeeds
    Tool: Bash
    Steps: nix develop --command cargo build -p p2p-commonware 2>&1
    Expected: Compilation succeeds with no errors
    Evidence: .sisyphus/evidence/task-5-build.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): redesign provider to use discovery::Network with multi-channel registration` | Files: `crates/p2p-commonware/src/provider.rs`, `crates/p2p-commonware/src/receiver.rs`, `crates/p2p-commonware/src/lib.rs`, `crates/p2p-commonware/Cargo.toml`

- [ ] 6. Wire CommonwareNetworkProvider into whirlpool-node main.rs

  **What to do**:
  Replace `MockNetworkProvider` with `CommonwareNetworkProvider` in `crates/whirlpool-node/src/main.rs`.

  Steps:
  1. Add `p2p-commonware` dependency to `crates/whirlpool-node/Cargo.toml`
  2. Add `commonware-runtime` dependency (for `tokio::Runner`)
  3. Add `commonware-cryptography` dependency (for `ed25519::PrivateKey`)
  4. In `main.rs`:
     - Remove `use p2p::mock::{MockNetworkProvider, MockPeerId};`
     - Import `p2p_commonware::CommonwareNetworkProvider` and commonware types
     - Create an ed25519 key pair (from seed or random for dev)
     - Create `discovery::Config::local()` with appropriate settings
     - Create `commonware_runtime::tokio::Runner`
     - Inside runner: create `discovery::Network::new(context, config)`, get `(network, oracle)`
     - Create `CommonwareNetworkProvider::new(network, oracle)`
     - Pass provider to `CommonwareEngine::new()`
  5. Remove the `p2p` feature `mock` from `whirlpool-node/Cargo.toml` dependencies (currently `p2p = { path = "../p2p", features = ["mock"] }`)
  6. Keep the TODO about proper key management — use deterministic seed for now

  **Must NOT do**: Add production key management. Change engine code. Add CLI args for network config.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Requires careful wiring of runtime lifecycle with async main
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: [7] | Blocked By: [5]

  **References**:
  - Current main.rs: `crates/whirlpool-node/src/main.rs` — 67 lines, uses MockNetworkProvider, has TODO on line 44
  - Current Cargo.toml: `crates/whirlpool-node/Cargo.toml` — `p2p = { path = "../p2p", features = ["mock"] }`
  - Engine constructor: `crates/consensus-simplex/src/engine.rs` — `CommonwareEngine::new(app, sink, config, network)` where `N: NetworkProvider`
  - Provider API (after Task 5): `CommonwareNetworkProvider::new(network, oracle)`
  - Runtime: `commonware_runtime::tokio::Runner::default().start(|ctx| async { ... })`
  - Config: `discovery::Config::local()` — suitable for development
  - Crypto: `ed25519::PrivateKey::from_seed(seed)` — deterministic key for dev

  **Acceptance Criteria** (agent-executable only):
  - [ ] `MockNetworkProvider` no longer used in main.rs
  - [ ] `CommonwareNetworkProvider` used with real `discovery::Network`
  - [ ] `p2p` dependency no longer requires `features = ["mock"]` in whirlpool-node
  - [ ] `p2p-commonware` added as dependency in whirlpool-node/Cargo.toml
  - [ ] `nix develop --command cargo build -p whirlpool-node` compiles
  - [ ] Binary starts without panic (may fail to connect if no peers, but should not crash)

  **QA Scenarios**:
  ```
  Scenario: Build succeeds
    Tool: Bash
    Steps: nix develop --command cargo build -p whirlpool-node 2>&1
    Expected: Compilation succeeds with no errors
    Evidence: .sisyphus/evidence/task-6-build.txt

  Scenario: Binary starts cleanly
    Tool: interactive_bash
    Steps: Start binary in background, wait 3s, check it's running, send SIGTERM
    Expected: Process starts without panic, logs network initialization, shuts down cleanly on SIGTERM
    Evidence: .sisyphus/evidence/task-6-startup.txt
  ```

  **Commit**: YES | Message: `feat(whirlpool-node): wire CommonwareNetworkProvider replacing MockNetworkProvider` | Files: `crates/whirlpool-node/src/main.rs`, `crates/whirlpool-node/Cargo.toml`

- [ ] 7. Integration test: real network provider end-to-end

  **What to do**:
  Update `crates/whirlpool-node/tests/single_node.rs` to use `CommonwareNetworkProvider` instead of `MockNetworkProvider`. Also add a new test that verifies the network provider can start and create sender/receiver handles.

  Test scenarios:
  1. **Update existing test**: `test_single_node_finalizes_blocks` — replace MockNetworkProvider with CommonwareNetworkProvider using `Config::test()` and commonware deterministic/tokio runtime
  2. **New test**: `test_network_provider_starts` — create provider with test config, call start(), verify sender/receiver are usable
  3. **New test**: `test_network_provider_shutdown` — start provider, drop handles, verify clean shutdown

  Use `discovery::Config::test()` for test configuration. Use localhost binding (127.0.0.1:0 for OS-assigned port).

  **Must NOT do**: Test actual multi-node networking (that requires multiple processes). Change engine behavior.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Integration test setup with async runtime and network binding
  - Skills: [] — No special skills needed

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: [] | Blocked By: [6]

  **References**:
  - Current test: `crates/whirlpool-node/tests/single_node.rs` — uses MockNetworkProvider, checks block finalization
  - Test config: `discovery::Config::test()` — pre-configured for testing
  - Engine test: `crates/consensus-simplex/src/engine.rs` — engine test patterns
  - Provider API: `CommonwareNetworkProvider::new(network, oracle)`

  **Acceptance Criteria** (agent-executable only):
  - [ ] `single_node.rs` uses `CommonwareNetworkProvider` instead of `MockNetworkProvider`
  - [ ] At least 2 new test functions for provider lifecycle
  - [ ] `nix develop --command cargo test -p whirlpool-node` all tests pass
  - [ ] `nix develop --command cargo test` workspace-wide passes

  **QA Scenarios**:
  ```
  Scenario: Integration tests pass
    Tool: Bash
    Steps: nix develop --command cargo test -p whirlpool-node 2>&1
    Expected: All tests pass including updated single_node test
    Evidence: .sisyphus/evidence/task-7-integration.txt

  Scenario: Workspace tests pass
    Tool: Bash
    Steps: nix develop --command cargo test 2>&1
    Expected: All workspace tests pass — zero failures
    Evidence: .sisyphus/evidence/task-7-workspace.txt
  ```

  **Commit**: YES | Message: `test(whirlpool-node): update integration tests to use real CommonwareNetworkProvider` | Files: `crates/whirlpool-node/tests/single_node.rs`

## Final Verification Wave (4 parallel agents, ALL must APPROVE)
- [ ] F1. Plan Compliance Audit — oracle: Verify all tasks completed per acceptance criteria
- [ ] F2. Code Quality Review — unspecified-high: Review code for idiomatic Rust, proper error handling, no unwrap() in non-test code
- [ ] F3. Real Manual QA — unspecified-high: Run `nix develop --command cargo build && nix develop --command cargo test` end-to-end
- [ ] F4. Scope Fidelity Check — deep: Verify no scope creep (no vendor changes, no trait changes, no engine changes)

## Commit Strategy
Sequential commits per task (7 commits). Each task produces one atomic commit. Final squash optional.

## Success Criteria
1. `nix develop --command cargo build` — zero errors
2. `nix develop --command cargo test` — zero failures
3. `MockNetworkProvider` removed from main.rs (still available behind feature flag)
4. `CommonwareNetworkProvider` uses real `discovery::Network` with 3 registered channels
5. Messages correctly routed by channel (VOTE, CERTIFICATE, RESOLVER)
6. Network Handle lifecycle managed (kept alive while receiver lives)
7. All TDD tests GREEN
