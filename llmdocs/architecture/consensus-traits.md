# Consensus Trait Layer Architecture

The Consensus Trait Layer abstracts consensus mechanics from specific engine implementations, allowing the Whirlpool project to swap consensus algorithms while maintaining a consistent application interface.

## Purpose
Decouple block production, verification, and finalization from the underlying transport and consensus logic.

## Public Types and Signatures

### Traits

#### Block
Core unit of data in the consensus system.
```rust
pub trait Block: Send + Sync + 'static {
    type Id: Copy + Eq + Hash + Debug + Send + Sync + 'static;
    fn id(&self) -> Self::Id;
    fn parent_id(&self) -> Self::Id;
    fn height(&self) -> u64;
}
```

#### ConsensusApp
Defines application-level consensus logic.
```rust
pub trait ConsensusApp: Send + Sync + 'static {
    type Block: Block;
    fn genesis(&self) -> impl Future<Output = Self::Block> + Send;
    fn propose(&self, parent: &Self::Block, height: u64) -> impl Future<Output = Option<Self::Block>> + Send;
    fn verify(&self, parent: &Self::Block, block: &Self::Block) -> impl Future<Output = Result<(), ConsensusError>> + Send;
}
```

#### EventSink
Receiver for consensus-related events.
```rust
pub trait EventSink: Send + Sync + 'static {
    type Block: Block;
    fn handle(&self, event: ConsensusEvent<Self::Block>) -> impl Future<Output = ()> + Send;
}
```

#### ConsensusEngine
Entry point for starting a consensus process.
```rust
pub trait ConsensusEngine {
    fn start(self) -> Result<RunningEngine, ConsensusError>;
}
```

### Structs and Enums

#### ConsensusEvent
Events emitted by the consensus engine.
```rust
#[derive(Debug)]
pub enum ConsensusEvent<B: Block> {
    Finalized { block: B, height: u64, proof: Vec<u8> },
    PreFinalized { block: B, height: u64 },
    Fault { offender: Vec<u8>, evidence: Vec<u8> },
}
```

#### ConsensusError
Error types for consensus operations.
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("invalid block: {0}")] InvalidBlock(String),
    #[error("proposal failed: {0}")] ProposalFailed(String),
    #[error("not ready: {0}")] NotReady(String),
    #[error("runtime error: {0}")] Runtime(String),
    #[error("consensus engine shut down")] Shutdown,
    #[error(transparent)] Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
```

#### ConsensusStatus
Snapshot of the engine's current state.
```rust
#[derive(Debug, Clone, Copy)]
pub struct ConsensusStatus { pub current_height: u64, pub is_running: bool }
```

#### RunningEngine
Handle to a consensus process running in the background.
```rust
pub struct RunningEngine {
    _shutdown: Box<dyn FnOnce() + Send>,
    handle: JoinHandle<Result<(), ConsensusError>>,
    height: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
}

impl RunningEngine {
    pub fn new(shutdown: Box<dyn FnOnce() + Send>, handle: JoinHandle<Result<(), ConsensusError>>, height: Arc<AtomicU64>, running: Arc<AtomicBool>) -> Self;
    pub fn status(&self) -> ConsensusStatus;
    pub async fn wait(self) -> Result<(), ConsensusError>;
    pub async fn shutdown(self) -> Result<(), ConsensusError>;
}
```

## Trait Relationships

```
Block <── ConsensusApp::Block (associated type)
Block <── EventSink::Block (associated type)
Block <── ConsensusEvent<B: Block> (generic)
ConsensusEngine::start() -> RunningEngine
RunningEngine::status() -> ConsensusStatus
Engine runtime -> EventSink::handle(ConsensusEvent)
```

## Design Decisions
- **Zero-cost Async**: Uses `impl Future` return types for trait methods instead of `async_trait` macro overhead.
- **Thread Safety**: All core traits enforce `Send + Sync + 'static` to ensure compatibility with multi-threaded runtimes.
- **Background Engine Model**: The `ConsensusEngine::start()` method returns a `RunningEngine` handle, spawning the actual consensus task in the background for decoupled execution.
- **Ergonomic Errors**: Uses `thiserror` for precise error variants with a transparent `Other` catch-all for easy integration of external error types.
- **Minimal Surface**: Only 8 public types define the entire consensus interface.

## File Locations
- `crates/consensus/src/lib.rs`
- `crates/consensus/src/block.rs`
- `crates/consensus/src/app.rs`
- `crates/consensus/src/event.rs`
- `crates/consensus/src/error.rs`
- `crates/consensus/src/engine.rs`
- `crates/consensus/src/mock/mod.rs`
- `crates/consensus/src/mock/block.rs`
- `crates/consensus/src/mock/engine.rs`
