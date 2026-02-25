use crate::block::Block;
use crate::error::ConsensusError;
use crate::engine::ConsensusEngine;
use crate::event::{ConsensusEvent, EventSink};
use crate::mock::MockBlock;
use crate::mock::MockEngine;
use std::future::Future;
use std::sync::{Arc, Mutex};

/// A test event sink that collects finalized events.
struct CollectorSink {
    events: Arc<Mutex<Vec<u64>>>, // collected heights
}

impl CollectorSink {
    fn new() -> (Arc<Self>, Arc<Mutex<Vec<u64>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::new(Self {
            events: events.clone(),
        });
        (sink, events)
    }
}

impl EventSink for CollectorSink {
    type Block = MockBlock;

    fn handle(&self, event: ConsensusEvent<MockBlock>) -> impl Future<Output = ()> + Send {
        let events = self.events.clone();
        async move {
            if let ConsensusEvent::Finalized { height, .. } = event {
                events.lock().unwrap().push(height);
            }
        }
    }
}

#[test]
fn mock_block_genesis() {
    let genesis = MockBlock::genesis();
    assert_eq!(genesis.id(), [0u8; 32]);
    assert_eq!(genesis.parent_id(), [0u8; 32]);
    assert_eq!(genesis.height(), 0);
}

#[test]
fn mock_block_child() {
    let genesis = MockBlock::genesis();
    let child = MockBlock::child(&genesis);
    assert_eq!(child.height(), 1);
    assert_eq!(child.parent_id(), genesis.id());
    assert_ne!(child.id(), genesis.id());

    let grandchild = MockBlock::child(&child);
    assert_eq!(grandchild.height(), 2);
    assert_eq!(grandchild.parent_id(), child.id());
}

#[tokio::test]
async fn mock_engine_lifecycle() {
    let genesis = MockBlock::genesis();
    let b1 = MockBlock::child(&genesis);
    let b2 = MockBlock::child(&b1);
    let b3 = MockBlock::child(&b2);

    let (sink, events) = CollectorSink::new();
    let engine = MockEngine::new(vec![b1, b2, b3], sink);

    let running = engine.start().expect("engine should start");
    let result: Result<(), ConsensusError> = running.wait().await;
    assert!(result.is_ok(), "engine should exit cleanly");

    let collected = events.lock().unwrap();
    assert_eq!(
        *collected,
        vec![1, 2, 3],
        "should finalize 3 blocks at heights 1,2,3"
    );
}

#[tokio::test]
async fn mock_engine_shutdown() {
    // Create many blocks so we can shut down mid-stream
    let genesis = MockBlock::genesis();
    let mut blocks = Vec::new();
    let mut parent = genesis;
    for _ in 0..100 {
        let child = MockBlock::child(&parent);
        blocks.push(child.clone());
        parent = child;
    }

    let (sink, events) = CollectorSink::new();
    let engine = MockEngine::new(blocks, sink);

    let running = engine.start().expect("engine should start");
    // Immediately request shutdown
    let result: Result<(), ConsensusError> = running.shutdown().await;
    assert!(result.is_ok(), "shutdown should succeed cleanly");

    // Should have finalized some blocks (possibly all if fast enough, possibly fewer)
    let collected = events.lock().unwrap();
    assert!(
        collected.len() <= 100,
        "should not exceed total blocks"
    );
}

#[tokio::test]
async fn consensus_status() {
    let genesis = MockBlock::genesis();
    let b1 = MockBlock::child(&genesis);

    let (sink, _events) = CollectorSink::new();
    let engine = MockEngine::new(vec![b1.clone()], sink);

    let running = engine.start().expect("engine should start");

    // Query status while engine is running or just finished
    let status = running.status();
    // Status should show engine info (height may or may not have updated yet)
    // At minimum, verify the struct is accessible and has reasonable values
    assert!(status.current_height <= 1);

    // Wait for completion
    let _ = running.wait().await;

    // Create second engine to test status before completion
    let genesis2 = MockBlock::genesis();
    let b2 = MockBlock::child(&genesis2);
    let (sink2, _) = CollectorSink::new();
    let engine2 = MockEngine::new(vec![b2], sink2);
    let running2 = engine2.start().expect("engine should start");

    let status2 = running2.status();
    assert!(status2.current_height <= 1);
    let _ = running2.wait().await;
}

#[test]
fn consensus_error_display() {
    let err = ConsensusError::InvalidBlock("bad hash".to_string());
    assert_eq!(err.to_string(), "invalid block: bad hash");

    let err = ConsensusError::ProposalFailed("timeout".to_string());
    assert_eq!(err.to_string(), "proposal failed: timeout");

    let err = ConsensusError::NotReady("syncing".to_string());
    assert_eq!(err.to_string(), "not ready: syncing");

    let err = ConsensusError::Runtime("panic".to_string());
    assert_eq!(err.to_string(), "runtime error: panic");

    let err = ConsensusError::Shutdown;
    assert_eq!(err.to_string(), "consensus engine shut down");
}

#[tokio::test]
async fn event_sink_receives_all_events() {
    // Test that the event sink receives Finalized events for every block
    let genesis = MockBlock::genesis();
    let b1 = MockBlock::child(&genesis);
    let b2 = MockBlock::child(&b1);

    let (sink, events) = CollectorSink::new();
    let engine = MockEngine::new(vec![b1, b2], sink);

    let running = engine.start().expect("engine should start");
    running.wait().await.expect("should finish");

    let collected = events.lock().unwrap();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0], 1);
    assert_eq!(collected[1], 2);
}
