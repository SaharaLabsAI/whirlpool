# Implementing Consensus Traits

This guide explains how to implement the Whirlpool consensus traits for a custom blockchain application.

## Prerequisites
Ensure your project has access to the `consensus` crate and its core types.

## Step 1: Implement the Block Trait
The `Block` trait defines the data structure for your blockchain's blocks. It requires an associated `Id` type.

```rust
use consensus::Block;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MyBlock {
    pub id: [u8; 32],
    pub parent_id: [u8; 32],
    pub height: u64,
}

impl Block for MyBlock {
    type Id = [u8; 32];

    fn id(&self) -> Self::Id { self.id }
    fn parent_id(&self) -> Self::Id { self.parent_id }
    fn height(&self) -> u64 { self.height }
}
```

## Step 2: Implement the ConsensusApp Trait
The `ConsensusApp` trait defines the logic for generating and verifying blocks. It uses `impl Future` for asynchronous operations.

```rust
use consensus::{ConsensusApp, ConsensusError, Block};
use std::future::Future;

pub struct MyApp;

impl ConsensusApp for MyApp {
    type Block = MyBlock;

    fn genesis(&self) -> impl Future<Output = Self::Block> + Send {
        async {
            MyBlock { id: [0; 32], parent_id: [0; 32], height: 0 }
        }
    }

    fn propose(&self, parent: &Self::Block, height: u64) -> impl Future<Output = Option<Self::Block>> + Send {
        async move {
            let mut id = parent.id;
            id[0] = id[0].wrapping_add(1);
            Some(MyBlock { id, parent_id: parent.id, height })
        }
    }

    fn verify(&self, _parent: &Self::Block, _block: &Self::Block) -> impl Future<Output = Result<(), ConsensusError>> + Send {
        async {
            Ok(())
        }
    }
}
```

## Step 3: Implement the EventSink Trait
The `EventSink` trait handles consensus events like finalization or faults.

```rust
use consensus::{EventSink, ConsensusEvent, Block};
use std::future::Future;

pub struct MySink;

impl EventSink for MySink {
    type Block = MyBlock;

    fn handle(&self, event: ConsensusEvent<Self::Block>) -> impl Future<Output = ()> + Send {
        async move {
            match event {
                ConsensusEvent::Finalized { block, height, .. } => {
                    println!("Block {} finalized at height {}", hex::encode(block.id()), height);
                }
                _ => {}
            }
        }
    }
}
```

## Step 4: Implement the ConsensusEngine Trait
The `ConsensusEngine` starts the background consensus process.

```rust
use consensus::{ConsensusEngine, RunningEngine, ConsensusError};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool};
use tokio::task;

pub struct MyEngine;

impl ConsensusEngine for MyEngine {
    fn start(self) -> Result<RunningEngine, ConsensusError> {
        let height = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));
        
        let h_clone = height.clone();
        let r_clone = running.clone();

        let shutdown_flag = r_clone.clone();
        let shutdown = Box::new(move || {
            shutdown_flag.store(false, std::sync::atomic::Ordering::SeqCst);
        });

        let handle = task::spawn(async move {
            while r_clone.load(std::sync::atomic::Ordering::SeqCst) {
                // Simulate consensus work
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                h_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            Ok(())
        });

        Ok(RunningEngine::new(shutdown, handle, height, running))
    }
}
```

## Reference Mock Implementations
The `consensus` crate provides mock implementations that demonstrate how these traits work together.

### MockBlock
A deterministic block used for testing.
```rust
#[derive(Debug, Clone)]
pub struct MockBlock { pub id: [u8; 32], pub parent_id: [u8; 32], pub height: u64 }

impl MockBlock {
    pub fn genesis() -> Self {
        MockBlock { id: [0; 32], parent_id: [0; 32], height: 0 }
    }
    pub fn child(parent: &MockBlock) -> Self {
        let mut id = parent.id;
        id[0] = id[0].wrapping_add(1);
        MockBlock { id, parent_id: parent.id, height: parent.height + 1 }
    }
}

impl Block for MockBlock {
    type Id = [u8; 32];
    fn id(&self) -> Self::Id { self.id }
    fn parent_id(&self) -> Self::Id { self.parent_id }
    fn height(&self) -> u64 { self.height }
}
```

### MockEngine
A simple engine that feeds pre-defined blocks to an event sink.
```rust
pub struct MockEngine<S: EventSink<Block = MockBlock>> {
    blocks: Vec<MockBlock>,
    sink: Arc<S>,
}

impl<S: EventSink<Block = MockBlock>> MockEngine<S> {
    pub fn new(blocks: Vec<MockBlock>, sink: Arc<S>) -> Self {
        Self { blocks, sink }
    }
}

impl<S: EventSink<Block = MockBlock>> ConsensusEngine for MockEngine<S> {
    fn start(self) -> Result<RunningEngine, ConsensusError> {
        // Mock implementation logic
        // Spawns task that handles events for provided blocks
        unimplemented!()
    }
}
```
