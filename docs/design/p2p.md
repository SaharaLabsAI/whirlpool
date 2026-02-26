# P2P Network Layer Design

| Field   | Value                                               |
| ------- | --------------------------------------------------- |
| Status  | Draft                                               |
| Date    | 2026-02-26                                          |
| Scope   | `p2p` (core traits) + `p2p-commonware` (impl crate) |

---

## 1. Problem Statement

The consensus layer needs to send and receive messages between validators.
Today, `consensus-simplex` directly depends on commonware's p2p types
(`Sender`, `Receiver`, `Recipients`), coupling the consensus adapter to a
specific networking backend.

We need the same layering pattern used for consensus itself:

- **Core crate** (`p2p`) — vendor-free traits and types that any networking
  backend can implement.
- **Impl crate** (`p2p-commonware`) — bridges the core traits to commonware's
  `p2p`, `broadcast`, and `stream` crates.

This lets us:

1. Test consensus logic without a real network (mock provider).
2. Swap networking backends without touching consensus code.
3. Keep the `consensus` and `p2p` core crates dependency-free from vendor.

---

## 2. Crate Layout

```
crates/
├── p2p/                        # Core traits (vendor-free)
│   ├── Cargo.toml              # deps: thiserror, tokio, bytes
│   └── src/
│       ├── lib.rs              # re-exports
│       ├── peer.rs             # PeerId trait
│       ├── channel.rs          # Channel newtype, Recipients enum
│       ├── sender.rs           # NetworkSender trait
│       ├── receiver.rs         # NetworkReceiver trait
│       ├── provider.rs         # NetworkProvider trait, NetworkChannel struct
│       ├── error.rs            # NetworkError
│       └── mock/               # cfg(test) or cfg(feature = "mock")
│           ├── mod.rs
│           ├── channel.rs      # MockSender, MockReceiver
│           └── provider.rs     # MockNetworkProvider
│
├── p2p-commonware/             # Commonware implementation
│   ├── Cargo.toml              # deps: p2p (path), commonware-*, tokio, tracing
│   └── src/
│       ├── lib.rs              # re-exports
│       ├── sender.rs           # CommonwareSender wrapper
│       ├── receiver.rs         # CommonwareReceiver wrapper
│       ├── provider.rs         # CommonwareNetworkProvider
│       └── tests.rs            # integration tests with simulated network
```

### Dependency Graph

```
whirlpool-node
├── consensus-simplex
│   ├── consensus          (core traits)
│   ├── p2p-commonware     (network impl)
│   │   └── p2p            (core traits)
│   └── commonware-*       (vendor)
└── p2p-commonware
    └── p2p
```

**Key rule**: `p2p` never depends on any vendor crate.
`consensus` and `p2p` are peer core crates — neither depends on the other.

---

## 3. Core Traits (`p2p` crate)

### 3.1 PeerId

```rust
// src/peer.rs

/// Opaque peer identifier.
///
/// Implementations map this to their concrete key type
/// (e.g., ed25519 public key for commonware).
pub trait PeerId: Copy + Eq + Hash + Debug + Send + Sync + 'static {
    /// Serialized form of this peer identity.
    fn to_bytes(&self) -> Vec<u8>;
}
```

**Rationale**: The consensus layer should never know about specific
cryptographic key types. `PeerId` provides identity semantics (equality,
hashing, debug) without coupling to a signature scheme.

### 3.2 Channel & Recipients

```rust
// src/channel.rs
use crate::PeerId;

/// A logical communication channel identifier.
///
/// Channels partition network traffic by purpose
/// (e.g., votes = 0, certificates = 1, resolver = 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel(pub u64);

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Channel({})", self.0)
    }
}

/// Target recipients for an outbound message.
pub enum Recipients<P: PeerId> {
    /// Broadcast to all known peers on the channel.
    All,
    /// Send to a specific subset of peers.
    Some(Vec<P>),
    /// Send to exactly one peer.
    One(P),
}
```

**Rationale**: Direct 1:1 map with commonware's `Channel = u64` and
`Recipients<P>` enum. No unnecessary abstraction — these concepts are
universal in BFT networking.

### 3.3 NetworkSender

```rust
// src/sender.rs
use crate::{PeerId, Recipients, NetworkError};
use bytes::Bytes;

/// Send messages to peers on a specific channel.
///
/// Each sender is bound to one channel. Cloneable so multiple
/// components can share the same outbound channel.
pub trait NetworkSender: Clone + Send + Sync + 'static {
    type PeerId: PeerId;

    /// Send a message to the given recipients.
    ///
    /// Returns the list of peers the message was actually delivered to.
    /// `priority` hints that this message should be sent before
    /// non-priority messages in the outbound queue.
    fn send(
        &mut self,
        recipients: Recipients<Self::PeerId>,
        message: Bytes,
        priority: bool,
    ) -> impl Future<Output = Result<Vec<Self::PeerId>, NetworkError>> + Send;
}
```

### 3.4 NetworkReceiver

```rust
// src/receiver.rs
use crate::{PeerId, NetworkError};
use bytes::Bytes;

/// Receive messages from peers on a specific channel.
///
/// Each receiver is bound to one channel. NOT cloneable —
/// only one consumer per channel (enforced at type level).
pub trait NetworkReceiver: Send + 'static {
    type PeerId: PeerId;

    /// Wait for the next inbound message.
    ///
    /// Returns the sender's identity and the raw message bytes.
    fn recv(
        &mut self,
    ) -> impl Future<Output = Result<(Self::PeerId, Bytes), NetworkError>> + Send;
}
```

### 3.5 NetworkProvider & NetworkChannel

```rust
// src/provider.rs
use crate::{Channel, NetworkSender, NetworkReceiver, NetworkError};

/// A paired sender + receiver for a single logical channel.
pub struct NetworkChannel<S: NetworkSender, R: NetworkReceiver> {
    pub sender: S,
    pub receiver: R,
}

/// Factory that creates network channels.
///
/// This is the main injection point: consensus engines receive a
/// `NetworkProvider` and call `open_channel` to get the sender/receiver
/// pairs they need.
pub trait NetworkProvider: Send + 'static {
    type Sender: NetworkSender;
    type Receiver: NetworkReceiver<PeerId = <Self::Sender as NetworkSender>::PeerId>;

    /// Register and open a channel with the given ID.
    ///
    /// Each channel ID should only be opened once. Opening the same
    /// channel twice is an error.
    fn open_channel(
        &mut self,
        channel: Channel,
    ) -> impl Future<Output = Result<
        NetworkChannel<Self::Sender, Self::Receiver>,
        NetworkError,
    >> + Send;
}
```

**Rationale**: The provider pattern matches how commonware's simulated
`Oracle` works: `oracle.register(channel_id, quota)` returns a
`(Sender, Receiver)` pair. In production, the authenticated `Network`
provides channel pairs similarly. `NetworkProvider` abstracts over both.

### 3.6 NetworkError

```rust
// src/error.rs
use crate::Channel;

#[derive(Debug, thiserror::Error)]
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

    #[error("provider not ready")]
    NotReady,

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
```

**Rationale**: Follows the `ConsensusError` pattern — specific variants for
common failure modes, transparent `Other` for backend-specific errors.

---

## 4. Mock Implementation (in `p2p` crate)

Behind `#[cfg(any(test, feature = "mock"))]`:

```rust
// src/mock/channel.rs

/// In-process sender backed by tokio::mpsc.
#[derive(Clone)]
pub struct MockSender {
    peer_id: MockPeerId,
    tx: mpsc::UnboundedSender<(MockPeerId, Bytes)>,
}

/// In-process receiver backed by tokio::mpsc.
pub struct MockReceiver {
    rx: mpsc::UnboundedReceiver<(MockPeerId, Bytes)>,
}

// src/mock/provider.rs

/// Creates mock channels using in-process mpsc.
/// Useful for testing consensus engines without a real network.
pub struct MockNetworkProvider { ... }
```

The mock mirrors how commonware's `simulated::Network` works but without
any vendor dependency.

---

## 5. Commonware Implementation (`p2p-commonware` crate)

### 5.1 PeerId Bridge

```rust
// Blanket or newtype — commonware PublicKey → PeerId
//
// Option A: Newtype wrapper (safer, no orphan rule issues)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CommonwarePeerId<P: PublicKey>(pub P);

impl<P: PublicKey> PeerId for CommonwarePeerId<P> {
    fn to_bytes(&self) -> Vec<u8> {
        // delegate to PublicKey serialization
    }
}
```

### 5.2 CommonwareSender

```rust
// src/sender.rs

/// Wraps a commonware p2p::Sender to implement NetworkSender.
/// Also implements commonware's p2p::Sender by forwarding, so it can
/// be passed directly to simplex::Engine::start() without unwrapping.
pub struct CommonwareSender<S> {
    inner: S,
}

// Our trait — used by generic code depending on `p2p` crate.
impl<S> NetworkSender for CommonwareSender<S>
where
    S: commonware_p2p::Sender,
    S::PublicKey: PublicKey,
{
    type PeerId = CommonwarePeerId<S::PublicKey>;

    fn send(
        &mut self,
        recipients: Recipients<Self::PeerId>,
        message: Bytes,
        priority: bool,
    ) -> impl Future<Output = Result<Vec<Self::PeerId>, NetworkError>> + Send {
        // Convert Recipients<CommonwarePeerId<P>> → commonware Recipients<P>
        // Delegate to self.inner.send(...)
        // Wrap result PK → CommonwarePeerId<PK>
    }
}

// Vendor trait — forwarding so wrappers can be passed to simplex engine directly.
impl<S> commonware_p2p::Sender for CommonwareSender<S>
where
    S: commonware_p2p::Sender,
{
    type PublicKey = S::PublicKey;

    fn send(
        &mut self,
        recipients: commonware_p2p::Recipients<S::PublicKey>,
        message: impl Buf,
        priority: bool,
    ) -> impl Future<Output = Result<Vec<S::PublicKey>, Error>> + Send {
        self.inner.send(recipients, message, priority)
    }
}
```

### 5.3 CommonwareReceiver

```rust
// src/receiver.rs

/// Wraps a commonware p2p::Receiver to implement NetworkReceiver.
/// Also implements commonware's p2p::Receiver by forwarding.
pub struct CommonwareReceiver<R> {
    inner: R,
}

// Our trait
impl<R> NetworkReceiver for CommonwareReceiver<R>
where
    R: commonware_p2p::Receiver,
    R::PublicKey: PublicKey,
{
    type PeerId = CommonwarePeerId<R::PublicKey>;

    fn recv(
        &mut self,
    ) -> impl Future<Output = Result<(Self::PeerId, Bytes), NetworkError>> + Send {
        // Delegate to self.inner.recv()
        // Wrap (PK, Bytes) → (CommonwarePeerId<PK>, Bytes)
    }
}

// Vendor trait — forwarding
impl<R> commonware_p2p::Receiver for CommonwareReceiver<R>
where
    R: commonware_p2p::Receiver,
{
    type PublicKey = R::PublicKey;

    fn recv(
        &mut self,
    ) -> impl Future<Output = Result<commonware_p2p::Message<R::PublicKey>, Error>> + Send {
        self.inner.recv()
    }
}
```

### 5.4 CommonwareNetworkProvider

```rust
// src/provider.rs

/// Creates network channels from a commonware simulated Oracle
/// or authenticated Network.
///
/// Generic over the control handle that produces (Sender, Receiver) pairs.
pub struct CommonwareNetworkProvider<C> {
    control: C,
    opened: HashSet<Channel>,
}

impl<C> NetworkProvider for CommonwareNetworkProvider<C>
where
    C: ChannelFactory,  // internal trait abstracting Oracle.register()
{
    type Sender = CommonwareSender<...>;
    type Receiver = CommonwareReceiver<...>;

    fn open_channel(
        &mut self,
        channel: Channel,
    ) -> impl Future<Output = Result<NetworkChannel<...>, NetworkError>> + Send {
        if !self.opened.insert(channel) {
            return Err(NetworkError::ChannelAlreadyOpen(channel));
        }
        let (sender, receiver) = self.control.register(channel.0, quota);
        Ok(NetworkChannel {
            sender: CommonwareSender { inner: sender },
            receiver: CommonwareReceiver { inner: receiver },
        })
    }
}
```

---

## 6. Integration with consensus-simplex

### Current State (stub)

`CommonwareEngine::start()` creates a hardcoded background task that
simulates finalization. No real networking.

### Target State

```rust
// consensus-simplex/src/engine.rs

pub struct CommonwareEngine<A, S, N> {
    app: Arc<A>,
    sink: Arc<S>,
    config: CommonwareConfig,
    network: N,  // NEW: injected NetworkProvider
}

impl<A, S, N> CommonwareEngine<A, S, N>
where
    A: ConsensusApp + Send + Sync + 'static,
    S: EventSink<Block = A::Block> + Send + Sync + 'static,
    N: NetworkProvider,
{
    pub fn new(app: A, sink: S, config: CommonwareConfig, network: N) -> Self { ... }
}

impl<A, S, N> ConsensusEngine for CommonwareEngine<A, S, N>
where
    A: ConsensusApp + Send + Sync + 'static,
    S: EventSink<Block = A::Block> + Send + Sync + 'static,
    A::Block: CommonwareBlock + Digestible + Send + Sync + 'static,
    N: NetworkProvider,
{
    fn start(mut self) -> impl Future<Output = Result<RunningEngine, ConsensusError>> + Send {
        async move {
            // Open the 3 channels simplex needs (constants defined in consensus-simplex)
            let votes = self.network.open_channel(VOTE_CHANNEL)
                .await.map_err(|e| ConsensusError::Runtime(...))?;
            let certs = self.network.open_channel(CERTIFICATE_CHANNEL)
                .await.map_err(|e| ConsensusError::Runtime(...))?;
            let resolver = self.network.open_channel(RESOLVER_CHANNEL)
                .await.map_err(|e| ConsensusError::Runtime(...))?;

            // Wire into simplex engine — wrappers implement BOTH
            // our NetworkSender AND commonware's p2p::Sender, so they
            // can be passed directly without unwrapping.
            let handle = simplex::Engine::new(...).start(
                (votes.sender, votes.receiver),
                (certs.sender, certs.receiver),
                (resolver.sender, resolver.receiver),
            );

            // ... wrap in RunningEngine
        }
    }
}
```

**Key insight**: `CommonwareSender<S>` implements *both* `NetworkSender`
(our trait) and commonware's `p2p::Sender` (vendor trait) by forwarding
to the inner `S`. Same for `CommonwareReceiver<R>`. This means the
wrappers pass directly into `simplex::Engine::start()` — no `into_inner()`
unwrapping needed, no leaky abstraction.
---

## 7. Key Flows

### Channel Setup Flow

```
whirlpool-node                 consensus-simplex          p2p-commonware
     │                              │                          │
     │  CommonwareEngine::new(      │                          │
     │    app, sink, config,        │                          │
     │    network_provider)         │                          │
     │─────────────────────────────>│                          │
     │                              │                          │
     │  engine.start()              │                          │
     │─────────────────────────────>│                          │
     │                              │  open_channel(0)         │
     │                              │─────────────────────────>│
     │                              │  NetworkChannel<S,R>     │
     │                              │<─────────────────────────│
     │                              │  open_channel(1)         │
     │                              │─────────────────────────>│
     │                              │  NetworkChannel<S,R>     │
     │                              │<─────────────────────────│
     │                              │  open_channel(2)         │
     │                              │─────────────────────────>│
     │                              │  NetworkChannel<S,R>     │
     │                              │<─────────────────────────│
     │                              │                          │
     │                              │  simplex::Engine.start(  │
     │                              │    vote, cert, resolver) │
     │                              │                          │
     │  RunningEngine               │                          │
     │<─────────────────────────────│                          │
```

### Message Send Flow

```
consensus-simplex (voter)    p2p NetworkSender    commonware p2p::Sender
        │                         │                       │
        │  send(All, msg, true)   │                       │
        │────────────────────────>│                       │
        │                         │  send(All, msg, true) │
        │                         │──────────────────────>│
        │                         │  Ok(vec![peer1, ...]) │
        │                         │<──────────────────────│
        │  Ok(vec![peer1, ...])   │                       │
        │<────────────────────────│                       │
```

---

## 8. What Core Does NOT Abstract

| Concern                     | Owned by              | Rationale                                    |
| --------------------------- | --------------------- | -------------------------------------------- |
| Peer discovery / bootstrap  | `p2p-commonware`      | Backend-specific (DNS, static list, DHT)     |
| Connection management       | `p2p-commonware`      | TLS, handshake, reconnection are backend     |
| Peer scoring / reputation   | Future crate          | Not needed for v0                            |
| Peer set management         | `p2p-commonware`      | Deferred; commonware Manager handles it      |
| Message serialization       | Consumer (consensus)  | Core passes opaque `Bytes`                   |
| Channel quota / rate limits | `p2p-commonware`      | Backend-specific tuning                      |

---

## 9. Deferred (Not in v0)

| Feature          | Notes                                                      |
| ---------------- | ---------------------------------------------------------- |
| `PeerManager`    | Trait for peer set updates/subscriptions. Add when needed.  |
| `Blocker`        | Trait for banning misbehaving peers. Add when needed.       |
| Authenticated    | Production `authenticated::Network` wiring. Simulated only for now. |
| Multi-transport  | TCP vs QUIC vs in-process selection.                        |

---

## 10. Resolved Questions

| #   | Question                                                                 | Resolution |
| --- | ------------------------------------------------------------------------ | ---------- |
| 1   | Should wrappers expose inner type via `into_inner()`?                    | **No.** Wrappers implement both our trait AND commonware's vendor trait by forwarding. No unwrapping needed — pass wrappers directly to `simplex::Engine::start()`. |
| 2   | Should `MockNetworkProvider` support simulating failures (partitions)?   | **Deferred.** Not in v0. |
| 3   | Channel constants (VOTES=0, CERTS=1, RESOLVER=2) — define where?        | **`consensus-simplex`.** These are consensus-protocol-specific, not network-generic. |
---

## 11. Migration Plan

| Phase | Action                                                | Crate(s)              |
| ----- | ----------------------------------------------------- | --------------------- |
| 1     | Create `p2p` crate with core traits + mock            | `p2p`                 |
| 2     | Create `p2p-commonware` with sender/receiver wrappers | `p2p-commonware`      |
| 3     | Add `NetworkProvider` to `CommonwareEngine` constructor | `consensus-simplex`  |
| 4     | Wire `p2p-commonware` in `whirlpool-node`             | `whirlpool-node`      |
| 5     | Remove direct commonware p2p deps from consensus-simplex (if possible) | `consensus-simplex` |
