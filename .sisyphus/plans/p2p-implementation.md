# P2P Networking Abstraction Layer

## TL;DR
> **Summary**: Create vendor-free `p2p` core crate (traits + mock) and `p2p-commonware` bridge crate, then integrate into `consensus-simplex` and `whirlpool-node` per `docs/design/p2p.md`.
> **Deliverables**: Two new crates (`p2p`, `p2p-commonware`), modified `consensus-simplex` engine, updated `whirlpool-node` wiring.
> **Effort**: Medium
> **Parallel**: YES - 3 waves
> **Critical Path**: Task 1 → Task 2 → Task 5,6 → Task 7 → Task 8 → Task 9

## Context
### Original Request
Implement `p2p` and `p2p-commonware` crates per the exhaustive design doc at `docs/design/p2p.md`.

### Interview Summary
- User confirmed TDD (Red-Green-Refactor) test strategy.
- Design doc specifies exact trait signatures, error types, crate layout, mock patterns, and integration approach.
- Scope: 5-phase migration from design doc.

### Metis Review (gaps addressed)
1. **PeerId `Copy` vs `Clone`**: Design doc says `Copy`, but commonware's `ed25519::PublicKey` is NOT `Copy` (contains `VerificationKey`). Resolution: `PeerId` requires `Clone` instead of `Copy`. `MockPeerId(u64)` can derive `Copy` for ergonomics but the trait only demands `Clone`. This is a **deviation from the design doc** — explicitly noted.
2. **Sender `Bytes` vs `impl Buf`**: Our `NetworkSender` uses `Bytes`. Commonware's `Sender` uses `impl Buf + Send`. Since `Bytes` implements `Buf`, this is a natural pass-through — no conversion needed.
3. **Quota handling**: Simulated network's `register()` requires a `governor::Quota`. `CommonwareNetworkProvider` takes quota config in its constructor. Deferred quota-per-channel to future work — all channels use the same default quota.
4. **`start()` sync vs async**: `ConsensusEngine::start()` is sync. `NetworkProvider::open_channel()` is async. Resolution: `start()` remains sync; it creates a tokio runtime handle and calls `handle.block_on()` to open channels inside the spawned thread, OR opens channels before spawning the thread using `tokio::runtime::Handle::current().block_on()`.

## Work Objectives
### Core Objective
Create a vendor-agnostic p2p networking abstraction that decouples consensus logic from specific networking implementations, enabling testability via mocks and future transport swaps.

### Deliverables
- `crates/p2p/` — Core traits (`PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider`), types (`Channel`, `Recipients`, `NetworkChannel`), error types (`NetworkError`), mock module
- `crates/p2p-commonware/` — Bridge types (`CommonwarePeerId`, `CommonwareSender`, `CommonwareReceiver`, `CommonwareNetworkProvider`)
- Modified `crates/consensus-simplex/src/engine.rs` — `CommonwareEngine<A,S,N>` with `NetworkProvider` generic
- Modified `crates/whirlpool-node/` — Wire `CommonwareNetworkProvider` into engine construction

### Definition of Done (verifiable conditions with commands)
- `cargo build --workspace` passes with zero errors
- `cargo test --workspace` passes with zero failures
- `cargo build --workspace --features mock` passes
- `crates/p2p/` compiles independently with no commonware dependencies
- `crates/p2p-commonware/` compiles with both `p2p` and commonware deps
- All existing tests in `consensus-simplex` and `whirlpool-node` pass (updated for new generic)

### Must Have
- PeerId trait with `Clone + Eq + Hash + Debug + Send + Sync + 'static` and `to_bytes()`
- Channel newtype over u64 with Display
- Recipients enum (All, Some, One)
- NetworkSender trait (async send with Recipients, Bytes, priority)
- NetworkReceiver trait (async recv returning (PeerId, Bytes))
- NetworkProvider trait (async open_channel returning NetworkChannel)
- NetworkError with thiserror
- Mock implementations behind `cfg(any(test, feature = "mock"))`
- CommonwarePeerId newtype wrapping `P: PublicKey`
- CommonwareSender/Receiver dual-trait impls
- CommonwareNetworkProvider wrapping control/register pattern
- Channel constants: VOTE_CHANNEL(0), CERTIFICATE_CHANNEL(1), RESOLVER_CHANNEL(2)
- Integration in CommonwareEngine and whirlpool-node

### Must NOT Have (guardrails, scope boundaries)
- DO NOT implement PeerManager, Blocker, or peer scoring — deferred
- DO NOT implement authenticated Network wiring — deferred
- DO NOT implement multi-transport support — deferred
- DO NOT add `Copy` bound to `PeerId` trait (incompatible with `ed25519::PublicKey`)
- DO NOT create workspace.dependencies section — follow existing per-crate dep management
- DO NOT modify any vendor/ files
- DO NOT change the `ConsensusEngine` trait itself — only `CommonwareEngine` impl
- DO NOT break the existing `ConsensusEngine::start()` sync signature

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: TDD (Red-Green-Refactor) + cargo test
- QA policy: Every task has agent-executed scenarios
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy
### Parallel Execution Waves

Wave 1 (Foundation — independent tasks):
- Task 1: `p2p` core crate (traits, types, errors) — `deep`
- Task 2: `p2p` mock module — `deep` (depends on Task 1 — SAME crate, sequential within wave)

Wave 2 (Bridge — depends on Wave 1):
- Task 3: `p2p-commonware` CommonwarePeerId + error mapping — `quick`
- Task 4: `p2p-commonware` CommonwareSender + CommonwareReceiver — `deep`
- Task 5: `p2p-commonware` CommonwareNetworkProvider — `deep`

Wave 3 (Integration — depends on Wave 2):
- Task 6: consensus-simplex engine modification — `deep`
- Task 7: whirlpool-node wiring — `quick`
- Task 8: Workspace Cargo.toml + final cleanup — `quick`

### Dependency Matrix
| Task | Depends On | Blocks |
|------|-----------|--------|
| 1    | —         | 2,3,4,5 |
| 2    | 1         | 6      |
| 3    | 1         | 4,5    |
| 4    | 1,3       | 5,6    |
| 5    | 1,3,4     | 6      |
| 6    | 2,4,5     | 7      |
| 7    | 6         | 8      |
| 8    | 7         | —      |

### Agent Dispatch Summary
| Wave | Tasks | Categories |
|------|-------|------------|
| 1    | 2     | deep, deep |
| 2    | 3     | quick, deep, deep |
| 3    | 3     | deep, quick, quick |

## TODOs

<!-- TASKS_START -->

- [x] 1. Create `p2p` core crate — traits, types, and errors (TDD)

  **What to do**:
  1. Create `crates/p2p/Cargo.toml` with deps: `thiserror = "2"`, `tokio = { version = "1", features = ["sync"] }`, `bytes = "1"`. Package name `p2p`, edition 2021, version 0.1.0. Add `[features] mock = []`.
  2. Write RED tests first in `crates/p2p/src/tests.rs`:
     - Test `Channel` Display format: `assert_eq!(format!("{}", Channel(42)), "Channel(42)")`
     - Test `Recipients::All`, `Recipients::Some(vec![...])`, `Recipients::One(...)` construction
     - Test `NetworkError` Display/Debug derive works
     - Test `NetworkChannel` struct field access
  3. Create `crates/p2p/src/lib.rs`:
     ```rust
     pub mod error;
     pub mod types;
     pub mod traits;
     #[cfg(any(test, feature = "mock"))]
     pub mod mock;
     #[cfg(test)]
     mod tests;
     // Re-exports
     pub use error::NetworkError;
     pub use types::{Channel, Recipients, NetworkChannel};
     pub use traits::{PeerId, NetworkSender, NetworkReceiver, NetworkProvider};
     ```
  4. Create `crates/p2p/src/error.rs`:
     ```rust
     use crate::types::Channel;
     use thiserror::Error;
     #[derive(Debug, Error)]
     pub enum NetworkError {
         #[error("connection closed")]
         ConnectionClosed,
         #[error("channel not found: {0}")]
         ChannelNotFound(Channel),
         #[error("channel already open: {0}")]
         ChannelAlreadyOpen(Channel),
         #[error("send failed")]
         SendFailed,
         #[error("recv failed")]
         RecvFailed,
         #[error("not ready")]
         NotReady,
         #[error(transparent)]
         Other(#[from] Box<dyn std::error::Error + Send + Sync>),
     }
     ```
  5. Create `crates/p2p/src/types.rs`:
     ```rust
     use std::fmt;
     #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
     pub struct Channel(pub u64);
     impl fmt::Display for Channel {
         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
             write!(f, "Channel({})", self.0)
         }
     }
     #[derive(Debug, Clone)]
     pub enum Recipients<P: Clone> {
         All,
         Some(Vec<P>),
         One(P),
     }
     pub struct NetworkChannel<S, R> {
         pub sender: S,
         pub receiver: R,
     }
     ```
  6. Create `crates/p2p/src/traits.rs`:
     ```rust
     use bytes::Bytes;
     use crate::error::NetworkError;
     use crate::types::{Channel, Recipients, NetworkChannel};
     use std::fmt::Debug;
     use std::future::Future;
     use std::hash::Hash;

     pub trait PeerId: Clone + Eq + Hash + Debug + Send + Sync + 'static {
         fn to_bytes(&self) -> Vec<u8>;
     }

     pub trait NetworkSender: Clone + Send + Sync + 'static {
         type PeerId: PeerId;
         fn send(
             &mut self,
             recipients: Recipients<Self::PeerId>,
             message: Bytes,
             priority: bool,
         ) -> impl Future<Output = Result<Vec<Self::PeerId>, NetworkError>> + Send;
     }

     pub trait NetworkReceiver: Send + 'static {
         type PeerId: PeerId;
         fn recv(&mut self) -> impl Future<Output = Result<(Self::PeerId, Bytes), NetworkError>> + Send;
     }

     pub trait NetworkProvider: Send + 'static {
         type Sender: NetworkSender<PeerId = <Self::Receiver as NetworkReceiver>::PeerId>;
         type Receiver: NetworkReceiver;
         fn open_channel(
             &mut self,
             channel: Channel,
         ) -> impl Future<Output = Result<NetworkChannel<Self::Sender, Self::Receiver>, NetworkError>> + Send;
     }
     ```
  7. Run tests (should go GREEN).

  **Must NOT do**:
  - DO NOT add any commonware dependencies
  - DO NOT add `Copy` bound to `PeerId` trait
  - DO NOT use `async_trait` — use RPITIT (`impl Future` in trait)

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Core trait design requires careful Rust generics, must follow design doc exactly
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: NO (Wave 1 foundation) | Wave 1 | Blocks: 2,3,4,5 | Blocked By: none

  **References**:
  - Design: `docs/design/p2p.md` — Primary source of truth for all type signatures
  - Error pattern: `crates/consensus/src/error.rs` — Follow thiserror derive pattern
  - Crate layout: `crates/consensus/Cargo.toml` — Follow dep declaration style (thiserror = "2", tokio features)
  - Lib structure: `crates/consensus/src/lib.rs` — Follow re-export and module organization pattern

  **Acceptance Criteria**:
  - [ ] `cargo build -p p2p` succeeds
  - [ ] `cargo test -p p2p` passes all tests
  - [ ] `PeerId` trait requires `Clone + Eq + Hash + Debug + Send + Sync + 'static`
  - [ ] `NetworkSender::send()` takes `Recipients`, `Bytes`, `bool` and returns `Result<Vec<PeerId>, NetworkError>`
  - [ ] `NetworkReceiver::recv()` returns `Result<(PeerId, Bytes), NetworkError>`
  - [ ] `NetworkProvider::open_channel()` takes `Channel` and returns `Result<NetworkChannel, NetworkError>`
  - [ ] No commonware dependencies in `crates/p2p/Cargo.toml`

  **QA Scenarios**:
  ```
  Scenario: Core types compile and work
    Tool: Bash
    Steps: cargo test -p p2p -- --nocapture
    Expected: All tests pass, output shows Channel Display, Recipients construction, NetworkError Display
    Evidence: .sisyphus/evidence/task-1-p2p-core.txt

  Scenario: No vendor leakage
    Tool: Bash
    Steps: grep -r "commonware" crates/p2p/ || echo "CLEAN"
    Expected: Output is "CLEAN" — no commonware references
    Evidence: .sisyphus/evidence/task-1-vendor-clean.txt
  ```

  **Commit**: YES | Message: `feat(p2p): add core traits, types, and error definitions` | Files: `crates/p2p/`

- [x] 2. Add mock module to `p2p` crate (TDD)

  **What to do**:
  1. Write RED tests first in `crates/p2p/src/tests.rs` (append to existing):
     - Test `MockPeerId` implements `PeerId` (Clone, Eq, Hash, Debug, to_bytes)
     - Test `MockSender::send()` delivers message to linked `MockReceiver::recv()`
     - Test `MockNetworkProvider::open_channel()` returns working sender/receiver pair
     - Test `MockNetworkProvider::open_channel()` twice with same channel returns `ChannelAlreadyOpen`
     - Test `MockSender` is `Clone` and both clones deliver to same receiver
  2. Create `crates/p2p/src/mock/mod.rs`:
     ```rust
     mod peer_id;
     mod sender;
     mod receiver;
     mod provider;
     pub use peer_id::MockPeerId;
     pub use sender::MockSender;
     pub use receiver::MockReceiver;
     pub use provider::MockNetworkProvider;
     ```
  3. Create `crates/p2p/src/mock/peer_id.rs`:
     ```rust
     use crate::traits::PeerId;
     #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
     pub struct MockPeerId(pub u64);
     impl PeerId for MockPeerId {
         fn to_bytes(&self) -> Vec<u8> { self.0.to_le_bytes().to_vec() }
     }
     ```
  4. Create `crates/p2p/src/mock/sender.rs`:
     - `MockSender` wraps `tokio::sync::mpsc::UnboundedSender<(MockPeerId, Bytes)>`
     - Implements `NetworkSender` with `type PeerId = MockPeerId`
     - `send()` sends message to channel, ignores recipients/priority, returns `Ok(vec![])`
     - Derives `Clone`
  5. Create `crates/p2p/src/mock/receiver.rs`:
     - `MockReceiver` wraps `tokio::sync::mpsc::UnboundedReceiver<(MockPeerId, Bytes)>`
     - Implements `NetworkReceiver` with `type PeerId = MockPeerId`
     - `recv()` awaits on receiver, maps `None` to `NetworkError::ConnectionClosed`
  6. Create `crates/p2p/src/mock/provider.rs`:
     - `MockNetworkProvider` holds `HashMap<Channel, bool>` for tracking opened channels
     - Also holds a factory closure or stored sender/receiver pairs
     - `open_channel()` checks if already opened (returns `ChannelAlreadyOpen`), creates `mpsc::unbounded_channel()`, returns `NetworkChannel { sender: MockSender, receiver: MockReceiver }`
  7. Run tests (should go GREEN).

  **Must NOT do**:
  - DO NOT expose mock types when `mock` feature is disabled
  - DO NOT add any commonware dependencies
  - DO NOT use `async_trait`

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Mock wiring with mpsc channels needs careful async Rust
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: NO (sequential after Task 1, same crate) | Wave 1 | Blocks: 6 | Blocked By: 1

  **References**:
  - Design: `docs/design/p2p.md` — Mock section specifies MockSender/Receiver/Provider
  - Mock pattern: `crates/consensus/src/mock/mod.rs` — Follow module structure (mod.rs re-exporting submodules)
  - Mock pattern: `crates/consensus/src/mock/engine.rs` — Follow mock implementation style
  - Feature gating: `crates/consensus/src/lib.rs` — `#[cfg(any(test, feature = "mock"))]` pattern

  **Acceptance Criteria**:
  - [ ] `cargo test -p p2p` passes all mock tests
  - [ ] `cargo build -p p2p` succeeds (mock hidden without feature)
  - [ ] `cargo build -p p2p --features mock` succeeds (mock exposed)
  - [ ] MockSender is Clone
  - [ ] MockPeerId derives Copy + Clone + Eq + Hash + Debug
  - [ ] MockNetworkProvider returns ChannelAlreadyOpen for duplicate channels

  **QA Scenarios**:
  ```
  Scenario: Mock send/receive round-trip
    Tool: Bash
    Steps: cargo test -p p2p mock -- --nocapture
    Expected: Tests show message sent by MockSender is received by MockReceiver with correct peer_id and payload
    Evidence: .sisyphus/evidence/task-2-mock-roundtrip.txt

  Scenario: Feature gating works
    Tool: Bash
    Steps: cargo build -p p2p 2>&1 | grep -c "mock" || echo "0"
    Expected: Mock module not compiled without feature flag
    Evidence: .sisyphus/evidence/task-2-feature-gate.txt
  ```

  **Commit**: YES | Message: `feat(p2p): add mock sender, receiver, and provider for testing` | Files: `crates/p2p/src/mock/`

- [x] 3. Create `p2p-commonware` crate — CommonwarePeerId + error mapping (TDD)

  **What to do**:
  1. Create `crates/p2p-commonware/Cargo.toml` with deps: `p2p = { path = "../p2p" }`, `commonware-p2p = { path = "../../vendor/commonware/p2p" }`, `commonware-cryptography = { path = "../../vendor/commonware/cryptography" }`, `thiserror = "2"`, `bytes = "1"`, `tracing = "0.1"`. Edition 2021, version 0.1.0.
  2. Write RED tests first in `crates/p2p-commonware/src/tests.rs`:
     - Test `CommonwarePeerId<ed25519::PublicKey>` implements `PeerId`
     - Test `CommonwarePeerId` Clone, Eq, Hash, Debug
     - Test `CommonwarePeerId::to_bytes()` returns correct bytes
     - Test `map_error()` converts commonware errors to `NetworkError`
  3. Create `crates/p2p-commonware/src/lib.rs`:
     ```rust
     mod peer_id;
     mod error;
     #[cfg(test)]
     mod tests;
     pub use peer_id::CommonwarePeerId;
     pub use error::map_error;
     ```
  4. Create `crates/p2p-commonware/src/peer_id.rs`:
     ```rust
     use p2p::PeerId;
     use commonware_cryptography::PublicKey;
     use std::fmt;
     use std::hash::{Hash, Hasher};

     #[derive(Clone, Debug)]
     pub struct CommonwarePeerId<P: PublicKey>(pub P);

     impl<P: PublicKey + Clone + Eq + Hash + fmt::Debug + Send + Sync + 'static> PeerId for CommonwarePeerId<P> {
         fn to_bytes(&self) -> Vec<u8> { self.0.as_ref().to_vec() }
     }
     // Eq, PartialEq, Hash delegate to inner
     impl<P: PublicKey + PartialEq> PartialEq for CommonwarePeerId<P> { ... }
     impl<P: PublicKey + Eq> Eq for CommonwarePeerId<P> {}
     impl<P: PublicKey + Hash> Hash for CommonwarePeerId<P> { ... }
     ```
     Note: `P: PublicKey` already requires `PartialEq` from commonware's trait. Check if `Hash` is also in the supertrait chain. If not, the blanket PeerId impl restricts to `P: PublicKey + Hash`. Since `ed25519::PublicKey` derives `Hash`, this works.
  5. Create `crates/p2p-commonware/src/error.rs` — maps commonware p2p errors to `NetworkError`:
     ```rust
     pub fn map_error<E: std::fmt::Display + std::error::Error + Send + Sync + 'static>(err: E) -> p2p::NetworkError {
         p2p::NetworkError::Other(Box::new(err))
     }
     ```
  6. Run tests (should go GREEN).

  **Must NOT do**:
  - DO NOT implement sender/receiver wrappers (Task 4)
  - DO NOT implement provider (Task 5)

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Small crate setup with simple newtype
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: YES (with Task 2 if Task 1 done) | Wave 2 | Blocks: 4,5 | Blocked By: 1

  **References**:
  - Design: `docs/design/p2p.md` — CommonwarePeerId section
  - Crate layout: `crates/consensus-simplex/Cargo.toml` — Follow commonware dep path declarations
  - PublicKey trait: `vendor/commonware/cryptography/src/lib.rs` — `PublicKey: Verifier + Sized + ReadExt + Encode + PartialEq + Array`
  - ed25519 PublicKey: `vendor/commonware/cryptography/src/ed25519/scheme.rs` — derives Clone, Eq, PartialEq, Ord, PartialOrd, Hash (NOT Copy)

  **Acceptance Criteria**:
  - [ ] `cargo build -p p2p-commonware` succeeds
  - [ ] `cargo test -p p2p-commonware` passes all tests
  - [ ] `CommonwarePeerId<ed25519::PublicKey>` satisfies `PeerId` bounds
  - [ ] Error mapping converts any `Display + Error + Send + Sync` to `NetworkError::Other`

  **QA Scenarios**:
  ```
  Scenario: PeerId trait satisfaction
    Tool: Bash
    Steps: cargo test -p p2p-commonware -- --nocapture
    Expected: All peer_id and error tests pass
    Evidence: .sisyphus/evidence/task-3-peer-id.txt

  Scenario: Builds cleanly
    Tool: Bash
    Steps: cargo build -p p2p-commonware 2>&1
    Expected: Zero warnings, zero errors
    Evidence: .sisyphus/evidence/task-3-build.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): add CommonwarePeerId newtype and error mapping` | Files: `crates/p2p-commonware/`

- [x] 4. Add CommonwareSender and CommonwareReceiver to `p2p-commonware` (TDD)

  **What to do**:
  1. Write RED tests in `crates/p2p-commonware/src/tests.rs` (append):
     - Test that `CommonwareSender<S>` implements `NetworkSender` for an `S: commonware_p2p::Sender`
     - Test that `CommonwareReceiver<R>` implements `NetworkReceiver` for an `R: commonware_p2p::Receiver`
     - Test Recipients conversion: our `Recipients::All` → commonware `Recipients::All`, etc.
     - Integration test: use simulated network to create real sender/receiver, wrap in CommonwareSender/CommonwareReceiver, send message through our traits, verify receipt
  2. Create `crates/p2p-commonware/src/sender.rs`:
     ```rust
     use p2p::{NetworkSender, NetworkError, Recipients as OurRecipients};
     use commonware_p2p::{Sender as CwSender, Recipients as CwRecipients};
     use bytes::Bytes;
     use crate::CommonwarePeerId;

     #[derive(Clone)]
     pub struct CommonwareSender<S> { inner: S }

     impl<S> CommonwareSender<S> {
         pub fn new(inner: S) -> Self { Self { inner } }
         pub fn into_inner(self) -> S { self.inner }
     }

     // Implement NetworkSender (our trait)
     impl<S, P> NetworkSender for CommonwareSender<S>
     where
         S: CwSender<PublicKey = P> + Clone + Send + Sync + 'static,
         P: commonware_cryptography::PublicKey + Clone + Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static,
     {
         type PeerId = CommonwarePeerId<P>;
         async fn send(&mut self, recipients: OurRecipients<Self::PeerId>, message: Bytes, priority: bool) -> Result<Vec<Self::PeerId>, NetworkError> {
             let cw_recipients = convert_recipients(recipients);
             self.inner.send(cw_recipients, message, priority)
                 .await
                 .map(|pks| pks.into_iter().map(CommonwarePeerId).collect())
                 .map_err(|e| crate::error::map_error(e))
         }
     }

     fn convert_recipients<P>(ours: OurRecipients<CommonwarePeerId<P>>) -> CwRecipients<P>
     where P: commonware_cryptography::PublicKey
     {
         match ours {
             OurRecipients::All => CwRecipients::All,
             OurRecipients::Some(peers) => CwRecipients::Some(peers.into_iter().map(|p| p.0).collect()),
             OurRecipients::One(peer) => CwRecipients::One(peer.0),
         }
     }
     ```
     Note: `Bytes` implements `Buf + Send`, so it passes directly to commonware's `send(recipients, impl Buf + Send, priority)`.

  3. Create `crates/p2p-commonware/src/receiver.rs`:
     ```rust
     use p2p::{NetworkReceiver, NetworkError};
     use commonware_p2p::Receiver as CwReceiver;
     use bytes::Bytes;
     use crate::CommonwarePeerId;

     pub struct CommonwareReceiver<R> { inner: R }

     impl<R> CommonwareReceiver<R> {
         pub fn new(inner: R) -> Self { Self { inner } }
         pub fn into_inner(self) -> R { self.inner }
     }

     impl<R, P> NetworkReceiver for CommonwareReceiver<R>
     where
         R: CwReceiver<PublicKey = P> + Send + 'static,
         P: commonware_cryptography::PublicKey + Clone + Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static,
     {
         type PeerId = CommonwarePeerId<P>;
         async fn recv(&mut self) -> Result<(Self::PeerId, Bytes), NetworkError> {
             self.inner.recv().await
                 .map(|(pk, bytes)| (CommonwarePeerId(pk), bytes))
                 .map_err(|e| crate::error::map_error(e))
         }
     }
     ```
  4. Update `crates/p2p-commonware/src/lib.rs` to add:
     ```rust
     mod sender;
     mod receiver;
     pub use sender::CommonwareSender;
     pub use receiver::CommonwareReceiver;
     ```
  5. Add necessary deps to `Cargo.toml` if not already present: `commonware-runtime` (for simulated network tests), `commonware-p2p` (already added in Task 3).
  6. Run tests (should go GREEN).

  **Must NOT do**:
  - DO NOT implement `commonware_p2p::Sender` on `CommonwareSender` (the design doc's dual-trait pattern) — this is actually NOT needed because simplex::Engine::start() takes `impl Sender` and our wrapper holds the original sender as `inner` which already impls `Sender`. The wrapper adds `NetworkSender`. If dual-trait is needed for some reason, add a forwarding impl, but verify the need first.
  - DO NOT implement provider (Task 5)

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Complex generic bounds bridging two trait systems
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: YES (after Task 1+3) | Wave 2 | Blocks: 5,6 | Blocked By: 1,3

  **References**:
  - Design: `docs/design/p2p.md` — CommonwareSender/Receiver sections
  - Sender trait: `vendor/commonware/p2p/src/lib.rs` — `Sender: LimitedSender`, `LimitedSender: Clone+Send+Sync+'static`
  - Receiver trait: `vendor/commonware/p2p/src/lib.rs` — `Receiver: Debug+Send+'static`
  - Recipients: `vendor/commonware/p2p/src/lib.rs` — `Recipients<P: PublicKey>` enum
  - Simulated network: `vendor/commonware/p2p/src/simulated/mod.rs` — For integration test setup

  **Acceptance Criteria**:
  - [ ] `cargo test -p p2p-commonware` passes all sender/receiver tests
  - [ ] `CommonwareSender` implements `NetworkSender`
  - [ ] `CommonwareReceiver` implements `NetworkReceiver`
  - [ ] `CommonwareSender` is `Clone`
  - [ ] Recipients conversion works for All, Some, One variants
  - [ ] Error mapping works for send/recv failures

  **QA Scenarios**:
  ```
  Scenario: Sender/Receiver round-trip via simulated network
    Tool: Bash
    Steps: cargo test -p p2p-commonware sender_receiver -- --nocapture
    Expected: Message sent via CommonwareSender is received by CommonwareReceiver with correct PeerId wrapping
    Evidence: .sisyphus/evidence/task-4-roundtrip.txt

  Scenario: Recipients conversion
    Tool: Bash
    Steps: cargo test -p p2p-commonware recipients -- --nocapture
    Expected: All three Recipients variants convert correctly between our types and commonware types
    Evidence: .sisyphus/evidence/task-4-recipients.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): add CommonwareSender and CommonwareReceiver wrappers` | Files: `crates/p2p-commonware/src/sender.rs`, `crates/p2p-commonware/src/receiver.rs`, `crates/p2p-commonware/src/lib.rs`

- [x] 5. Add CommonwareNetworkProvider to `p2p-commonware` (TDD)

  **What to do**:
  1. Write RED tests in `crates/p2p-commonware/src/tests.rs`:
     - Test `CommonwareNetworkProvider` implements `NetworkProvider`
     - Test `open_channel()` returns working `NetworkChannel` with `CommonwareSender`/`CommonwareReceiver`
     - Test `open_channel()` twice with same channel returns `ChannelAlreadyOpen` error
     - Integration test with simulated network: create provider, open 3 channels (VOTE=0, CERT=1, RESOLVER=2), send/receive on each
  2. Create `crates/p2p-commonware/src/provider.rs`:
     The design doc specifies an internal `ChannelFactory` trait to abstract over `oracle.control(pk).register()`. However, the simpler approach is:
     ```rust
     use p2p::{NetworkProvider, NetworkError, Channel, NetworkChannel};
     use std::collections::HashSet;
     use crate::{CommonwareSender, CommonwareReceiver};

     pub struct CommonwareNetworkProvider<F> {
         factory: F,
         opened: HashSet<Channel>,
     }

     impl<F, S, R> CommonwareNetworkProvider<F>
     where
         F: FnMut(u64) -> Result<(S, R), Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
     {
         pub fn new(factory: F) -> Self {
             Self { factory, opened: HashSet::new() }
         }
     }

     impl<F, S, R, P> NetworkProvider for CommonwareNetworkProvider<F>
     where
         F: FnMut(u64) -> Result<(S, R), Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
         S: commonware_p2p::Sender<PublicKey = P> + Clone + Send + Sync + 'static,
         R: commonware_p2p::Receiver<PublicKey = P> + Send + 'static,
         P: commonware_cryptography::PublicKey + Clone + Eq + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static,
     {
         type Sender = CommonwareSender<S>;
         type Receiver = CommonwareReceiver<R>;

         async fn open_channel(&mut self, channel: Channel) -> Result<NetworkChannel<Self::Sender, Self::Receiver>, NetworkError> {
             if !self.opened.insert(channel) {
                 return Err(NetworkError::ChannelAlreadyOpen(channel));
             }
             let (sender, receiver) = (self.factory)(channel.0)
                 .map_err(|e| NetworkError::Other(e))?;
             Ok(NetworkChannel {
                 sender: CommonwareSender::new(sender),
                 receiver: CommonwareReceiver::new(receiver),
             })
         }
     }
     ```
     Key insight: The factory closure captures the oracle control handle and quota. The caller (whirlpool-node) creates it like:
     ```rust
     let control = oracle.control(my_pk);
     let quota = Quota::per_second(nonzero!(100u32));
     let provider = CommonwareNetworkProvider::new(move |channel_id| {
         control.register(channel_id, quota).map_err(|e| Box::new(e) as _)
     });
     ```
  3. Update `crates/p2p-commonware/src/lib.rs`:
     ```rust
     mod provider;
     pub use provider::CommonwareNetworkProvider;
     ```
  4. Ensure `Channel` derives/implements `Hash` + `Eq` (needed for `HashSet<Channel>`) — verify in Task 1's types.rs. (It derives `Hash + Eq` already per Task 1.)
  5. Run tests (should go GREEN).

  **Must NOT do**:
  - DO NOT implement ChannelFactory as a separate trait if the closure approach works — simpler is better
  - DO NOT handle quota per-channel — single quota for all channels (deferred)
  - DO NOT add authenticated network support

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Complex generic bounds with closure factory pattern
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: YES (after Tasks 1,3,4) | Wave 2 | Blocks: 6 | Blocked By: 1,3,4

  **References**:
  - Design: `docs/design/p2p.md` — CommonwareNetworkProvider + ChannelFactory section
  - Simulated network: `vendor/commonware/p2p/src/simulated/mod.rs` — `oracle.control(pk).register(channel_id, quota)` pattern
  - Channel type: `crates/p2p/src/types.rs` (from Task 1) — must derive Hash+Eq for HashSet

  **Acceptance Criteria**:
  - [ ] `cargo test -p p2p-commonware` passes all provider tests
  - [ ] `CommonwareNetworkProvider` implements `NetworkProvider`
  - [ ] Duplicate channel open returns `ChannelAlreadyOpen`
  - [ ] Factory closure pattern works with simulated network

  **QA Scenarios**:
  ```
  Scenario: Open 3 channels and communicate
    Tool: Bash
    Steps: cargo test -p p2p-commonware provider_three_channels -- --nocapture
    Expected: VOTE(0), CERT(1), RESOLVER(2) channels all open successfully and can send/receive
    Evidence: .sisyphus/evidence/task-5-three-channels.txt

  Scenario: Duplicate channel rejection
    Tool: Bash
    Steps: cargo test -p p2p-commonware provider_duplicate -- --nocapture
    Expected: Second open_channel(Channel(0)) returns Err(ChannelAlreadyOpen(Channel(0)))
    Evidence: .sisyphus/evidence/task-5-duplicate.txt
  ```

  **Commit**: YES | Message: `feat(p2p-commonware): add CommonwareNetworkProvider with channel factory` | Files: `crates/p2p-commonware/src/provider.rs`, `crates/p2p-commonware/src/lib.rs`

- [ ] 6. Modify `consensus-simplex` CommonwareEngine to accept NetworkProvider (TDD)

  **What to do**:
  1. Write RED tests first in `crates/consensus-simplex/src/tests.rs` (update existing):
     - Update `test_engine_can_be_constructed` to pass `MockNetworkProvider`
     - Update `test_engine_can_start_and_shutdown` to pass `MockNetworkProvider`
     - Update `test_engine_simulates_block_finalization` to pass `MockNetworkProvider`
     - Add test: `test_engine_opens_three_channels` — verify provider's open_channel is called 3 times
  2. Add `p2p` dependency to `crates/consensus-simplex/Cargo.toml`:
     ```toml
     p2p = { path = "../p2p", features = ["mock"] }
     ```
     Note: needs `mock` feature for tests.
  3. Define channel constants in `crates/consensus-simplex/src/config.rs` or a new `crates/consensus-simplex/src/channels.rs`:
     ```rust
     use p2p::Channel;
     pub const VOTE_CHANNEL: Channel = Channel(0);
     pub const CERTIFICATE_CHANNEL: Channel = Channel(1);
     pub const RESOLVER_CHANNEL: Channel = Channel(2);
     ```
  4. Modify `CommonwareEngine<A, S>` to `CommonwareEngine<A, S, N>`:
     ```rust
     pub struct CommonwareEngine<A, S, N> {
         app: Arc<A>,
         sink: Arc<S>,
         config: CommonwareConfig,
         network: N,
     }
     ```
  5. Update `CommonwareEngine::new()`:
     ```rust
     pub fn new(app: Arc<A>, sink: Arc<S>, config: CommonwareConfig, network: N) -> Self
     where N: NetworkProvider
     ```
  6. Update `ConsensusEngine` impl for `CommonwareEngine<A, S, N>`:
     - Add `N: NetworkProvider` bound
     - In `start()`, open 3 channels using the network provider before spawning the consensus thread
     - Since `start()` is sync but `open_channel()` is async, use `tokio::runtime::Handle::current().block_on()` to bridge
     - The opened channels' senders/receivers are moved into the spawned thread
     - The stub simulation loop remains for now (real simplex engine wiring is out of scope — the stub still simulates blocks)
  7. Update all existing tests to pass a `MockNetworkProvider` to `new()`.
  8. Run tests (should go GREEN).

  **Must NOT do**:
  - DO NOT change the `ConsensusEngine` trait — only the `CommonwareEngine` impl
  - DO NOT wire the real simplex engine yet — keep the stub simulation loop
  - DO NOT break the sync `start()` return type
  - DO NOT remove any existing test functionality

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: Modifying existing engine with new generic, must maintain backward compat
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 7 | Blocked By: 2,4,5

  **References**:
  - Current engine: `crates/consensus-simplex/src/engine.rs` — Current CommonwareEngine<A,S> stub with simulated blocks
  - Current tests: `crates/consensus-simplex/src/tests.rs` — TestBlock, MockApp, CollectorSink test fixtures
  - ConsensusEngine trait: `crates/consensus/src/engine.rs` — `fn start(self) -> Result<RunningEngine, ConsensusError>`
  - Mock types: `crates/p2p/src/mock/` (from Tasks 1-2) — MockNetworkProvider, MockPeerId
  - Channel constants: `docs/design/p2p.md` — VOTE=0, CERT=1, RESOLVER=2
  - Config: `crates/consensus-simplex/src/config.rs` — CommonwareConfig fields

  **Acceptance Criteria**:
  - [ ] `cargo test -p consensus-simplex` passes all tests (existing + new)
  - [ ] `CommonwareEngine` has 3 type parameters: `A`, `S`, `N`
  - [ ] `new()` takes `network: N` parameter
  - [ ] `start()` opens VOTE(0), CERTIFICATE(1), RESOLVER(2) channels
  - [ ] Channel constants are public in consensus-simplex
  - [ ] Existing tests compile and pass with MockNetworkProvider

  **QA Scenarios**:
  ```
  Scenario: Engine starts with mock network
    Tool: Bash
    Steps: cargo test -p consensus-simplex test_engine -- --nocapture
    Expected: All engine tests pass, channels opened via MockNetworkProvider
    Evidence: .sisyphus/evidence/task-6-engine-mock.txt

  Scenario: Full workspace builds
    Tool: Bash
    Steps: cargo build --workspace
    Expected: Zero errors — all crates compile together
    Evidence: .sisyphus/evidence/task-6-workspace-build.txt
  ```

  **Commit**: YES | Message: `feat(consensus-simplex): add NetworkProvider generic to CommonwareEngine` | Files: `crates/consensus-simplex/src/engine.rs`, `crates/consensus-simplex/src/tests.rs`, `crates/consensus-simplex/Cargo.toml`

- [ ] 7. Update `whirlpool-node` to wire CommonwareNetworkProvider

  **What to do**:
  1. Add dependencies to `crates/whirlpool-node/Cargo.toml`:
     ```toml
     p2p = { path = "../p2p" }
     p2p-commonware = { path = "../p2p-commonware" }
     ```
  2. Update `crates/whirlpool-node/src/main.rs`:
     - Import `p2p_commonware::CommonwareNetworkProvider` (or `p2p::mock::MockNetworkProvider` for now since there's no real network setup yet)
     - Since the current whirlpool-node doesn't set up a real commonware simulated/authenticated network, use `MockNetworkProvider` for now:
       ```rust
       use p2p::mock::MockNetworkProvider;
       let network = MockNetworkProvider::new();
       let engine = CommonwareEngine::new(app, sink, config, network);
       ```
     - Add a TODO comment: `// TODO: Replace MockNetworkProvider with CommonwareNetworkProvider once network infrastructure is set up`
  3. Ensure `p2p` dep has `features = ["mock"]` in whirlpool-node's Cargo.toml (needed for MockNetworkProvider).
  4. Verify `cargo build -p whirlpool-node` and `cargo run -p whirlpool-node -- --help` still work.

  **Must NOT do**:
  - DO NOT set up real authenticated/simulated network — deferred
  - DO NOT change whirlpool-node's CLI interface
  - DO NOT add runtime configuration for network — deferred

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Simple wiring change in main.rs
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 8 | Blocked By: 6

  **References**:
  - Current main: `crates/whirlpool-node/src/main.rs` — Current engine construction without network param
  - Current deps: `crates/whirlpool-node/Cargo.toml` — Existing dependency structure
  - MockNetworkProvider: `crates/p2p/src/mock/provider.rs` (from Task 2) — Import path and usage
  - CommonwareNetworkProvider: `crates/p2p-commonware/src/provider.rs` (from Task 5) — Future replacement

  **Acceptance Criteria**:
  - [ ] `cargo build -p whirlpool-node` succeeds
  - [ ] `cargo run -p whirlpool-node` starts without error (may shut down quickly — that's fine)
  - [ ] TODO comment present for future CommonwareNetworkProvider replacement

  **QA Scenarios**:
  ```
  Scenario: Node builds and starts
    Tool: Bash
    Steps: cargo build -p whirlpool-node 2>&1
    Expected: Build succeeds with zero errors
    Evidence: .sisyphus/evidence/task-7-node-build.txt

  Scenario: Node binary runs
    Tool: Bash
    Steps: timeout 5 cargo run -p whirlpool-node 2>&1 || true
    Expected: Node starts (may exit due to no real network, but no panic/compile error)
    Evidence: .sisyphus/evidence/task-7-node-run.txt
  ```

  **Commit**: YES | Message: `feat(whirlpool-node): wire MockNetworkProvider into engine construction` | Files: `crates/whirlpool-node/src/main.rs`, `crates/whirlpool-node/Cargo.toml`

- [ ] 8. Update workspace Cargo.toml and final cleanup

  **What to do**:
  1. Add new crates to workspace members in `Cargo.toml`:
     ```toml
     members = [
         "crates/consensus",
         "crates/consensus-simplex",
         "crates/p2p",
         "crates/p2p-commonware",
         "crates/whirlpool-node",
     ]
     ```
  2. Run full workspace verification:
     ```bash
     cargo build --workspace
     cargo test --workspace
     cargo build --workspace --features mock  # if workspace-level feature exists
     ```
  3. Fix any compilation errors or test failures across the workspace.
  4. Verify no vendor/ files were modified: `git diff --name-only vendor/`
  5. Run `cargo build -p p2p` independently to verify no vendor leakage.

  **Must NOT do**:
  - DO NOT add workspace.dependencies section — follow existing pattern
  - DO NOT modify vendor/ files
  - DO NOT add new features beyond what's specified

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: Simple Cargo.toml edit + verification
  - Skills: [] — No special skills needed
  - Omitted: [`playwright`] — No UI work

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: none | Blocked By: 7

  **References**:
  - Workspace: `Cargo.toml` (root) — Current members list
  - All new crates: `crates/p2p/Cargo.toml`, `crates/p2p-commonware/Cargo.toml`

  **Acceptance Criteria**:
  - [ ] `cargo build --workspace` succeeds
  - [ ] `cargo test --workspace` passes all tests
  - [ ] Workspace members list includes all 5 crates
  - [ ] No vendor/ modifications: `git diff --name-only vendor/` shows nothing
  - [ ] `cargo build -p p2p` succeeds independently with zero commonware deps

  **QA Scenarios**:
  ```
  Scenario: Full workspace builds and tests
    Tool: Bash
    Steps: cargo build --workspace && cargo test --workspace
    Expected: All crates build, all tests pass
    Evidence: .sisyphus/evidence/task-8-workspace.txt

  Scenario: Vendor untouched
    Tool: Bash
    Steps: git diff --name-only vendor/ | wc -l
    Expected: Output is "0"
    Evidence: .sisyphus/evidence/task-8-vendor-clean.txt
  ```

  **Commit**: YES | Message: `chore: add p2p and p2p-commonware to workspace members` | Files: `Cargo.toml`

<!-- TASKS_END -->

## Final Verification Wave (4 parallel agents, ALL must APPROVE)
- [ ] F1. Plan Compliance Audit — oracle
  - Verify all design doc requirements from `docs/design/p2p.md` are addressed
  - Check trait signatures match design doc (with documented Clone vs Copy deviation)
  - Verify all 5 migration phases are covered
- [ ] F2. Code Quality Review — unspecified-high
  - Review all new code for Rust best practices
  - Check generic bounds are minimal and correct
  - Verify error handling is comprehensive
- [ ] F3. Real Manual QA — unspecified-high
  - Run `cargo build --workspace && cargo test --workspace`
  - Verify `p2p` crate has zero commonware deps
  - Test mock round-trip end-to-end
- [ ] F4. Scope Fidelity Check — deep
  - Verify no scope creep beyond design doc
  - Check that deferred items (PeerManager, Blocker, authenticated network) are NOT implemented
  - Verify no vendor/ modifications

## Commit Strategy
Atomic commits per task (8 commits total). Each commit must leave the workspace in a buildable state (`cargo build --workspace` passes after each).

## Success Criteria
1. `cargo build --workspace` passes
2. `cargo test --workspace` passes
3. `p2p` crate is vendor-free (zero commonware deps)
4. `p2p-commonware` bridges our traits to commonware's traits correctly
5. `consensus-simplex` CommonwareEngine accepts NetworkProvider
6. `whirlpool-node` compiles and runs with MockNetworkProvider
7. All deferred items (PeerManager, Blocker, authenticated network, multi-transport) are NOT implemented
8. No vendor/ files modified
