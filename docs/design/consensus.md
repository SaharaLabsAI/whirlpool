# Whirlpool Consensus — Design Document

> **Status**: Draft  
> **Date**: 2026-02-24  
> **Scope**: `consensus-core` + `consensus-commonware` crates

---

## 1. Problem Statement

Whirlpool needs a consensus layer that:

1. Produces an ordered, finalized sequence of blocks.
2. Is backend-agnostic — the first implementation wraps **commonware-consensus** (Simplex BFT), but the system must support future engines without touching application code.
3. Keeps the core crate **dependency-free** from any specific consensus library or async runtime.

## 2. Crate Layout

```
crates/
  consensus-core/                   # Traits + types only. Zero heavy deps.
    src/
      lib.rs                        # Re-exports
      block.rs                      # Block trait
      engine.rs                     # ConsensusEngine trait + RunningEngine
      app.rs                        # ConsensusApp trait (propose/verify)
      event.rs                      # EventSink trait + ConsensusEvent enum
      error.rs                      # ConsensusError
  consensus-commonware/             # Adapter: core traits → commonware Simplex
    src/
      lib.rs                        # Re-exports
      adapter.rs                    # Maps core::ConsensusApp → commonware Application/VerifyingApplication
      engine.rs                     # Builds & runs Simplex + Marshal, implements core::ConsensusEngine
      config.rs                     # CommonwareConfig (scheme, timeouts, network channels)
      types.rs                      # Concrete block/digest/scheme bindings
```

### Dependency Graph

```
consensus-core
  └── (std only — no async runtime, no commonware crates)
consensus-commonware
  ├── consensus-core
  ├── commonware-consensus
  ├── commonware-broadcast
  ├── commonware-cryptography
  ├── commonware-p2p
  ├── commonware-runtime
  ├── commonware-storage
  └── commonware-codec
  ├── consensus-core               (compile-time trait bounds)
  └── consensus-commonware          (runtime, behind feature flag)
```

**Key rule**: Application code depends on `core` traits at compile time. The concrete adapter is injected at the binary/orchestration layer — never imported by business logic.

---

## 3. Core Traits

### 3.1 Block

Minimal, universal block identity. Does NOT prescribe codec, digest algorithm, or signature scheme.

```rust
// consensus-core/src/block.rs

/// A consensus block.
///
/// Intentionally minimal — only identity + ordering.
/// Serialization, digest computation, and proof attachment
/// belong to the adapter or the application crate.
pub trait Block: Send + Sync + 'static {
    /// Opaque block identifier (hash, commitment, etc.).
    type Id: Copy + Eq + core::hash::Hash + core::fmt::Debug + Send + Sync + 'static;

    /// This block's unique identifier.
    fn id(&self) -> Self::Id;

    /// Parent block's identifier. Genesis returns a well-known sentinel.
    fn parent_id(&self) -> Self::Id;

    /// Monotonically increasing block height. Genesis = 0.
    fn height(&self) -> u64;
}
```

**Rationale**: Commonware's `Block` super-trait chain (`Heightable + Codec + Digestible + Committable`) is implementation-specific. Our core only needs identity and ordering — the adapter maps the rest.

### 3.2 ConsensusApp

The application's interface to consensus — propose and verify blocks.

```rust
// consensus-core/src/app.rs

use crate::block::Block;
use crate::error::ConsensusError;

/// Application logic for block production and validation.
///
/// Implemented by the application. The consensus engine calls
/// these methods during its protocol rounds.
pub trait ConsensusApp: Send + Sync + 'static {
    type Block: Block;

    /// Produce the genesis block. Called once at chain init.
    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;

    /// Propose a new block extending `parent`.
    ///
    /// Returns `None` to skip this slot (engine will nullify/skip).
    /// `parent` is the most recently finalized/notarized ancestor.
    fn propose(
        &self,
        parent: &Self::Block,
        height: u64,
    ) -> impl Future<Output = Option<Self::Block>> + Send;

    /// Validate a block proposed by another participant.
    ///
    /// `parent` is the block's claimed parent.
    /// Must be deterministic for the same (parent, block) pair.
    fn verify(
        &self,
        parent: &Self::Block,
        block: &Self::Block,
    ) -> impl Future<Output = Result<(), ConsensusError>> + Send;
}
```

**Design decisions**:
- `propose` takes the parent block directly, not a stream. The commonware adapter handles ancestry stream construction internally.
- `verify` returns `Result<(), ConsensusError>` instead of `bool` — errors carry diagnostic info.
- GAT-style `impl Future` return types avoid `async-trait` dependency.
- No runtime generic parameter `E`. Runtime bounds live in the adapter.

### 3.3 ConsensusEvent + EventSink

How the engine communicates state changes to the application.

```rust
// consensus-core/src/event.rs

use crate::block::Block;

/// Consensus lifecycle events delivered to the application.
#[derive(Debug)]
pub enum ConsensusEvent<B: Block> {
    /// Block has been finalized. Delivered in strict height order.
    /// `proof` is backend-specific opaque bytes (e.g., aggregated BLS sig).
    Finalized {
        block: B,
        height: u64,
        proof: Vec<u8>,
    },

    /// Block has been pre-finalized (notarized/prepared).
    /// Optional hint — not all backends emit this.
    /// Applications MUST NOT treat this as final.
    PreFinalized {
        block: B,
        height: u64,
    },

    /// A participant was detected as Byzantine.
    /// `evidence` is backend-specific opaque bytes.
    Fault {
        offender: Vec<u8>,
        evidence: Vec<u8>,
    },
}

/// Receives consensus events. Implemented by the application.
///
/// The engine guarantees:
/// - `Finalized` events are delivered in strictly ascending height order.
/// - Each height is finalized at most once.
/// - `PreFinalized` is best-effort and may be skipped.
pub trait EventSink: Send + Sync + 'static {
    type Block: Block;

    /// Handle a consensus event.
    ///
    /// Returning `Err` signals the engine to shut down gracefully.
    fn handle(
        &self,
        event: ConsensusEvent<Self::Block>,
    ) -> impl Future<Output = Result<(), ConsensusError>> + Send;
}
```

**Rationale**:
- Commonware has `Reporter` (report activity) + `Monitor` (subscribe to progress). We consolidate into a single push-based `EventSink`.
- `proof` is opaque `Vec<u8>` — Simplex finalization certificates, other engines provide their own justification format. Core doesn't parse it.
- `PreFinalized` models Simplex's notarization step. Single-step finality engines simply never emit it.
- `Fault` enables slashing/reputation systems without coupling to specific cryptographic evidence formats.

### 3.4 ConsensusEngine + RunningEngine

Engine lifecycle management.

```rust
// consensus-core/src/engine.rs

use crate::error::ConsensusError;

/// A consensus engine that can be started.
///
/// Each backend provides its own config and construction.
/// This trait only covers the lifecycle contract.
pub trait ConsensusEngine: Send + 'static {
    /// Start the engine. Consumes self.
    ///
    /// The engine begins participating in consensus rounds
    /// immediately. Events are delivered via the EventSink
    /// provided during construction.
    fn start(self) -> Result<RunningEngine, ConsensusError>;
}

/// Handle to a running consensus engine.
///
/// Dropping this handle initiates graceful shutdown.
pub struct RunningEngine {
    /// Signals shutdown when dropped.
    _shutdown: Box<dyn FnOnce() + Send>,
    /// Waitable future for engine completion.
    handle: tokio::task::JoinHandle<Result<(), ConsensusError>>,
}

impl RunningEngine {
    pub fn new(
        shutdown: impl FnOnce() + Send + 'static,
        handle: tokio::task::JoinHandle<Result<(), ConsensusError>>,
    ) -> Self {
        Self {
            _shutdown: Box::new(shutdown),
            handle,
        }
    }

    /// Wait for the engine to complete.
    /// Returns the engine's exit result.
    pub async fn wait(self) -> Result<(), ConsensusError> {
        self.handle.await.map_err(|e| ConsensusError::Runtime(e.to_string()))?
    }

    /// Request graceful shutdown and wait.
    pub async fn shutdown(self) -> Result<(), ConsensusError> {
        drop(self._shutdown);
        self.handle.await.map_err(|e| ConsensusError::Runtime(e.to_string()))?
    }
}
```

**Note on runtime coupling**: `RunningEngine` uses `tokio::task::JoinHandle` — this is a pragmatic choice. Whirlpool uses tokio (via commonware-runtime's tokio backend). If a future backend needs a different runtime, `RunningEngine` can be generalized then. YAGNI for now.

### 3.5 ConsensusError

```rust
// consensus-core/src/error.rs

#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("proposal failed: {0}")]
    ProposalFailed(String),

    #[error("engine not ready: {0}")]
    NotReady(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("shutdown requested")]
    Shutdown,

    #[error("{0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
```

---

## 4. Commonware Adapter — Contracts

### 4.1 Adapter Layer (core::ConsensusApp → commonware traits)

The adapter bridges core's simple `ConsensusApp` to commonware's richer trait surface.

```
┌─────────────────────────────────┐
│      ConsensusApp (core)        │  ← Application implements this
└────────────┬────────────────────┘
             │
   ┌─────────▼──────────┐
   │   AppAdapter<A>     │  Implements:
   │                     │  • commonware Application<E>
   │                     │  • commonware VerifyingApplication<E>
   │                     │  • commonware Reporter
   └─────────┬──────────┘
             │
   ┌─────────▼──────────┐
   │     Marshaled       │  Commonware's own wrapper
   │  (wraps adapter)    │  Adds: storage, epoch handling,
   │                     │        ancestry streams
   └─────────┬──────────┘
             │
   ┌─────────▼──────────┐
   │  Simplex Engine     │  Calls Marshaled as automaton + relay
   └─────────┬──────────┘
             │
   ┌─────────▼──────────┐
   │   EventSink (core)  │  ← Application implements this
   └─────────────────────┘
```

**Key mapping**:

| Core trait | Commonware trait(s) | Adapter responsibility |
|---|---|---|
| `ConsensusApp::genesis` | `Application::genesis` | Direct passthrough |
| `ConsensusApp::propose` | `Application::propose` | Resolve parent from `AncestorStream`, then call core's propose |
| `ConsensusApp::verify` | `VerifyingApplication::verify` | Resolve parent from `AncestorStream`, then call core's verify |
| `EventSink::handle(Finalized{..})` | `Reporter::report` | Convert `Activity::Finalized` → `ConsensusEvent::Finalized`, serialize proof |
| `EventSink::handle(PreFinalized{..})` | `Reporter::report` | Convert `Activity::Notarized` → `ConsensusEvent::PreFinalized` |
| `EventSink::handle(Fault{..})` | `Reporter::report` | Convert fault activity → `ConsensusEvent::Fault` |
| (not exposed) | `Relay::broadcast` | Handled internally by Marshaled |
| (not exposed) | `Monitor::subscribe` | Handled internally by Marshal |

### 4.2 Block Type Mapping

```
Application's block type
  │
  │ must implement:
  │  • core::Block (id, parent_id, height)
  │  • commonware_consensus::Block (parent commitment)
  │  • Heightable, Codec, Digestible, Committable
  │
  ▼
The application's block type lives in the APPLICATION crate,
not in core or adapter. It implements traits from both.
```

Example (following Alto's pattern):

```rust
// In the application crate:
pub struct MyBlock {
    pub parent: [u8; 32],
    pub height: u64,
    pub timestamp: u64,
    pub payload: Vec<u8>,
    digest: [u8; 32],  // precomputed
}

// Implements core::Block
impl consensus_core::Block for MyBlock {
    type Id = [u8; 32];
    fn id(&self) -> [u8; 32] { self.digest }
    fn parent_id(&self) -> [u8; 32] { self.parent }
    fn height(&self) -> u64 { self.height }
}

// Implements commonware's trait surface
impl commonware_consensus::Block for MyBlock {
    fn parent(&self) -> Self::Commitment { /* ... */ }
}
// + Heightable, Codec, Digestible, Committable
```

### 4.3 CommonwareConfig

Engine configuration is backend-specific. The core does NOT define config.

```rust
// consensus-commonware/src/config.rs

pub struct CommonwareConfig<S, B> {
    // -- Identity --
    pub scheme: S,                      // Signing scheme (ed25519, bls12381, etc.)
    pub namespace: Vec<u8>,             // Protocol namespace for replay protection

    // -- Consensus tuning --
    pub leader_timeout: Duration,
    pub notarization_timeout: Duration,
    pub nullify_retry: Duration,
    pub activity_timeout: u64,          // ViewDelta
    pub skip_timeout: u64,              // ViewDelta

    // -- Networking --
    pub blocker: B,                     // P2P peer blocker
    pub mailbox_size: usize,

    // -- Storage --
    pub partition_prefix: String,
    pub replay_buffer: usize,
    pub write_buffer: usize,

    // -- Epoch --
    pub epoch: u64,
    pub epoch_length: u64,              // blocks per epoch (for FixedEpocher)

    // -- Fetch --
    pub fetch_timeout: Duration,
    pub fetch_concurrent: usize,
}
```

### 4.4 CommonwareEngine (implements core::ConsensusEngine)

```rust
// consensus-commonware/src/engine.rs

pub struct CommonwareEngine<A, E, S, B> {
    // Pre-built, ready to start:
    consensus: simplex::Engine<E, S, ...>,
    marshal: marshal::Actor<E, ...>,
    buffer: broadcast::buffered::Engine<...>,
    app_adapter: AppAdapter<A>,

    // Network channels (injected at construction)
    vote_channels: (Sender, Receiver),
    cert_channels: (Sender, Receiver),
    resolver_channels: (Sender, Receiver),
    broadcast_channels: (Sender, Receiver),
    marshal_channels: (Receiver, Resolver),
}

impl<...> ConsensusEngine for CommonwareEngine<...> {
    fn start(self) -> Result<RunningEngine, ConsensusError> {
        // 1. Start buffer (broadcast layer)
        // 2. Start marshal (storage/sync)
        // 3. Start consensus (Simplex)
        // 4. Return RunningEngine with joined handle
    }
}
```

**Construction** follows Alto's `engine::Engine::new()` pattern: open archives, create marshal, wrap app in Marshaled, create simplex engine. The adapter builder handles all of this.

---

## 5. Key Flows

### 5.1 Block Proposal

```
Simplex round starts, node is leader
  │
  ▼
Simplex calls Marshaled.propose(context)
  │
  ├─ Marshaled fetches parent from marshal storage
  ├─ Marshaled checks epoch boundary
  ▼
Marshaled calls AppAdapter.propose(context, ancestry_stream)
  │
  ├─ AppAdapter resolves parent block from ancestry stream
  ├─ AppAdapter calls ConsensusApp.propose(parent, height)
  │     │
  │     ▼
  │   Application builds block (timestamp, payload, etc.)
  │     │
  │     ▼
  │   Returns Some(block) or None
  │
  ├─ AppAdapter converts to commonware Block format
  ▼
Marshaled caches block, Simplex broadcasts Notarize vote
```

### 5.2 Block Verification

```
Simplex receives Notarize from leader
  │
  ▼
Simplex calls Marshaled.verify(context, payload)
  │
  ├─ Marshaled fetches parent + block from marshal
  ├─ Marshaled checks: parent commitment, height contiguity, epoch bounds
  ▼
Marshaled calls AppAdapter.verify(context, ancestry_stream)
  │
  ├─ AppAdapter resolves parent from stream
  ├─ AppAdapter calls ConsensusApp.verify(parent, block)
  │     │
  │     ▼
  │   Application validates (e.g., timestamp monotonicity, payload rules)
  │     │
  │     ▼
  │   Returns Ok(()) or Err(ConsensusError)
  │
  ▼
Marshaled returns bool to Simplex
Simplex votes Notarize (if valid) or Nullify (if invalid)
```

### 5.3 Block Finalization

```
Simplex achieves 2f+1 Finalize votes
  │
  ▼
Simplex produces Finalization certificate
  │
  ▼
Reporter.report(Activity::Finalized { block, cert })
  │
  ├─ AppAdapter serializes cert → opaque proof bytes
  ├─ AppAdapter calls EventSink.handle(ConsensusEvent::Finalized {
  │     block, height, proof
  │   })
  │     │
  │     ▼
  │   Application handles finalized block
  │   (persist state, update indices, notify subscribers)
  │
  ▼
Marshal stores finalized block + cert in archive
```

### 5.4 Engine Lifecycle

```
Application code:
  │
  ├─ Build CommonwareConfig
  ├─ Build ConsensusApp implementation
  ├─ Build EventSink implementation
  ├─ Setup P2P network, register channels
  │
  ▼
CommonwareEngine::builder()
  .config(cfg)
  .app(my_app)
  .sink(my_sink)
  .network_channels(vote, cert, resolver, broadcast, marshal)
  .build(runtime_context)      ← opens archives, creates marshal,
  │                               wraps app in Marshaled,
  │                               creates simplex engine
  ▼
engine.start()?               ← returns RunningEngine
  │
  ├─ running.wait().await      (block until engine exits)
  └─ running.shutdown().await   (graceful stop)
```

---

## 6. What Core Does NOT Abstract

These concerns are intentionally left to the adapter or application:

| Concern | Why not in core | Where it lives |
|---|---|---|
| **Networking / P2P** | Wildly different per engine (Simplex needs 3 typed channels; other engines may use gossip, libp2p, etc.) | Adapter config |
| **Cryptographic scheme** | Ed25519 vs BLS12-381 vs threshold — deeply engine-specific | Adapter config |
| **Leader election** | RoundRobin, VRF-random, external — engine-specific | Adapter config |
| **Block codec / serialization** | Application-defined format | Application crate |
| **Storage backend** | WAL, archive, leveldb — engine-specific | Adapter internals |
| **Epoch management** | FixedEpocher, custom — engine-specific | Adapter config |
| **Participant set / validator rotation** | Depends on staking, DKG, etc. | Application/adapter |

---

## 7. Future Backend Considerations

When adding a new consensus backend (e.g., HotStuff, Tendermint):

1. Create `consensus-{backend}/` crate.
2. Implement the `AppAdapter` pattern: map `ConsensusApp` ↔ backend's proposal/verification API.
3. Implement `ConsensusEngine` for the backend's engine type.
4. Map backend finality events → `ConsensusEvent` through `EventSink`.
5. Define backend-specific config.

The application code changes **zero lines** — only the binary's engine construction changes.

---

## 8. Open Questions

| # | Question | Leaning | Impact |
|---|---|---|---|
| 1 | Should `RunningEngine` use `tokio::JoinHandle` or be fully runtime-agnostic? | tokio — it's our runtime, avoid premature abstraction | Low — easy to generalize later |
| 2 | Should `ConsensusEvent::proof` be `Vec<u8>` or a typed `Box<dyn Any>`? | `Vec<u8>` — simpler, serializable, no downcasting | Medium — typed would be safer but couples core to backend |
| 3 | Should the core define a `ValidatorSet` trait for participant management? | Not yet — validator rotation is deeply app-specific | High if wrong — defer until second backend |
| 4 | Should `ConsensusApp::propose` receive timing hints (slot duration, deadline)? | Not yet — Simplex doesn't use them, add when needed | Low |
| 5 | Should there be a `ConsensusStatus` query API (current height, view, epoch)? | Yes, add `status() -> ConsensusStatus` to `RunningEngine` | Medium — useful for health checks/metrics |

---

## 9. Migration Plan

| Phase | Deliverable | Validates |
|---|---|---|
| **P0** | `consensus-core` crate with traits + types | Trait surface compiles, no runtime deps |
| **P1** | `consensus-commonware` adapter with Simplex | Full round-trip: propose → verify → finalize through core traits |
| **P2** | Integration test: multi-node consensus with core traits | End-to-end proof that abstraction holds |
| **P3** | Noop/mock engine for testing (implements core traits trivially) | Application tests don't need real consensus |