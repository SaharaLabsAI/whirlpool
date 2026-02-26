# Wiring the Simplex Adapter

The Simplex Adapter Bridge is now sealed inside CommonwareEngine. This guide explains how to use the new API after the consensus wiring refactor.

## 1. Define your Block type

Your block must implement both the internal `CoreBlock` trait and the vendor's `VendorBlock` trait, plus codec and cryptography traits from Commonware. A simple approach is ensuring your type satisfies `CommonwareBlock` bounds.

### Example: TestBlock Reference

The `TestBlock` implementation in consensus-simplex/src/tests.rs demonstrates required trait implementations:
- `Block` trait: id(), parent_id(), height()
- `VendorBlock` trait: parent() → commitment
- `Heightable`: Maps height to vendor Height type
- `Digestible`: Computes Digest from block data
- `Committable`: Returns commitment for consensus
- `Write`/`Read`: Codec serialization (40 bytes typical)

## 2. Implement ConsensusApp

Your app must implement the `ConsensusApp` trait with three methods:

```rust
pub struct MyApp;
impl ConsensusApp for MyApp {
    type Block = MyBlock;
    
    async fn genesis(&self) -> Self::Block { ... }
    async fn propose(&self, parent: &Self::Block, height: u64) -> Option<Self::Block> { ... }
    async fn verify(&self, parent: &Self::Block, block: &Self::Block) -> Result<(), ConsensusError> { ... }
}
```

## 3. Implement EventSink

Create an event handler for finalized blocks:

```rust
pub struct MyEventSink;
impl EventSink for MyEventSink {
    type Block = MyBlock;
    async fn handle(&self, event: ConsensusEvent<Self::Block>) { ... }
}
```

## 4. Create CommonwareEngine (NEW API)

The engine constructor now takes app, sink, and config directly. It internally wires Mailbox, MailboxActor, AppAdapter, FinalizationSink, and simplex engine:

```rust
let app = Arc::new(MyApp);
let sink = Arc::new(MyEventSink);
let config = CommonwareConfig { ... };

let engine = CommonwareEngine::new(app, sink, config);
let running = engine.start()?;

// Query height
let height = running.height();

// Shutdown
running.shutdown();
```

**Key change**: No more starter closure. The CommonwareEngine owns the full construction and startup logic internally.

## 5. Configure CommonwareConfig

The config struct holds Simplex protocol parameters:

```rust
let config = CommonwareConfig {
    namespace: "my-blockchain".to_string(),
    leader_timeout: Duration::from_millis(500),
    notarization_timeout: Duration::from_millis(1000),
    nullify_retry: Duration::from_millis(200),
    activity_timeout: 10,
    skip_timeout: 5,
    mailbox_size: 128,
    replay_buffer: NonZeroUsize::new(64).unwrap(),
    write_buffer: NonZeroUsize::new(32).unwrap(),
    epoch: 1,
    fetch_timeout: Duration::from_secs(5),
    fetch_concurrent: 4,
};
```

These settings control timing and buffer sizes for the underlying consensus protocol.

## Component Wiring (Internal)

CommonwareEngine internally:
1. Creates mpsc channel for Mailbox↔MailboxActor
2. Spawns MailboxActor task delegating to app methods
3. Wraps app+sink in AppAdapter for vendor traits
4. Creates FinalizationSink for event handling
5. Configures and starts simplex engine
6. Returns RunningEngine with shutdown closure

This sealed design eliminates manual orchestration complexity.
