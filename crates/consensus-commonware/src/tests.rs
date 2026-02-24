//! Unit tests for the consensus-commonware adapter crate.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite};
use commonware_consensus::{Block as VendorBlock, Heightable};
use commonware_cryptography::{Committable, Digestible};
use consensus_core::block::Block as CoreBlock;
use consensus_core::engine::ConsensusEngine;
use consensus_core::error::ConsensusError;
use consensus_core::event::{ConsensusEvent, EventSink};

use crate::config::CommonwareConfig;
use crate::engine::CommonwareEngine;
use crate::types::CommonwareBlock;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// A minimal digest type that satisfies `commonware_cryptography::Digest`.
/// We reuse the vendor's sha256::Digest which already implements all required
/// traits (Array, Copy, Random, etc.)
type TestDigest = commonware_cryptography::sha256::Digest;

/// Zero digest constant.
fn zero_digest() -> TestDigest {
    commonware_cryptography::sha256::Sha256::fill(0)
}

/// A test block that implements both `consensus_core::Block` and all vendor
/// traits required by `commonware_consensus::Block`.
#[derive(Clone, Debug)]
struct TestBlock {
    id: [u8; 32],
    parent: TestDigest,
    height: u64,
}

impl TestBlock {
    fn genesis() -> Self {
        Self {
            id: [0u8; 32],
            parent: zero_digest(),
            height: 0,
        }
    }

    fn child(parent: &Self) -> Self {
        let mut id = [0u8; 32];
        id[0] = (parent.height + 1) as u8;
        Self {
            id,
            parent: parent.commitment(),
            height: parent.height + 1,
        }
    }
}

// --- consensus_core::Block ---

impl CoreBlock for TestBlock {
    type Id = [u8; 32];

    fn id(&self) -> Self::Id {
        self.id
    }

    fn parent_id(&self) -> Self::Id {
        // Convert digest bytes to [u8; 32]
        let commitment = self.parent;
        let bytes: &[u8] = commitment.as_ref();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        arr
    }

    fn height(&self) -> u64 {
        self.height
    }
}

// --- commonware_codec traits ---

impl CodecWrite for TestBlock {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_slice(&self.id);
        buf.put_slice(self.parent.as_ref());
        buf.put_u64(self.height);
    }
}

impl EncodeSize for TestBlock {
    fn encode_size(&self) -> usize {
        32 + 32 + 8 // id + parent_digest + height
    }
}

impl CodecRead for TestBlock {
    type Cfg = ();

    fn read_cfg(reader: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, CodecError> {
        if reader.remaining() < 72 {
            return Err(CodecError::Invalid(
                "TestBlock",
                "not enough bytes",
            ));
        }
        let mut id = [0u8; 32];
        reader.copy_to_slice(&mut id);
        let mut digest_bytes = [0u8; 32];
        reader.copy_to_slice(&mut digest_bytes);
        let parent = TestDigest::from(digest_bytes);
        let height = reader.get_u64();
        Ok(Self { id, parent, height })
    }
}

// --- commonware_cryptography traits ---

impl Digestible for TestBlock {
    type Digest = TestDigest;

    fn digest(&self) -> Self::Digest {
        // Simple: hash the id bytes as digest
        TestDigest::from(self.id)
    }
}

impl Committable for TestBlock {
    type Commitment = TestDigest;

    fn commitment(&self) -> Self::Commitment {
        // Same as digest for test purposes
        self.digest()
    }
}

// --- commonware_consensus traits ---

impl Heightable for TestBlock {
    fn height(&self) -> commonware_consensus::types::Height {
        commonware_consensus::types::Height::new(self.height)
    }
}

impl VendorBlock for TestBlock {
    fn parent(&self) -> Self::Commitment {
        self.parent
    }
}

// --- EventSink for collecting events ---

struct CollectorSink {
    events: Arc<Mutex<Vec<u64>>>,
}

impl CollectorSink {
    fn new() -> (Arc<Self>, Arc<Mutex<Vec<u64>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::new(Self {
            events: Arc::clone(&events),
        });
        (sink, events)
    }
}

impl EventSink for CollectorSink {
    type Block = TestBlock;

    fn handle(&self, event: ConsensusEvent<Self::Block>) -> impl std::future::Future<Output = ()> + Send {
        async move {
            if let ConsensusEvent::Finalized { height, .. } = event {
                self.events.lock().unwrap().push(height);
            }
        }
    }
}

// --- Mock ConsensusApp ---

struct MockApp;

impl consensus_core::app::ConsensusApp for MockApp {
    type Block = TestBlock;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        async { TestBlock::genesis() }
    }

    fn propose(
        &self,
        parent: &Self::Block,
        _height: u64,
    ) -> impl std::future::Future<Output = Option<Self::Block>> + Send {
        let child = TestBlock::child(parent);
        async move { Some(child) }
    }

    fn verify(
        &self,
        _parent: &Self::Block,
        _block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<(), ConsensusError>> + Send {
        async { Ok(()) }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Verify that TestBlock satisfies the CommonwareBlock blanket impl.
#[test]
fn test_commonware_block_blanket_impl() {
    fn assert_commonware_block<T: CommonwareBlock>() {}
    assert_commonware_block::<TestBlock>();
}

/// Verify CommonwareConfig can be constructed with all 12 fields.
#[test]
fn test_config_construction() {
    let config = CommonwareConfig {
        namespace: "test-consensus".to_string(),
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

    assert_eq!(config.namespace, "test-consensus");
    assert_eq!(config.leader_timeout, Duration::from_millis(500));
    assert_eq!(config.notarization_timeout, Duration::from_millis(1000));
    assert_eq!(config.nullify_retry, Duration::from_millis(200));
    assert_eq!(config.activity_timeout, 10);
    assert_eq!(config.skip_timeout, 5);
    assert_eq!(config.mailbox_size, 128);
    assert_eq!(config.replay_buffer.get(), 64);
    assert_eq!(config.write_buffer.get(), 32);
    assert_eq!(config.epoch, 1);
    assert_eq!(config.fetch_timeout, Duration::from_secs(5));
    assert_eq!(config.fetch_concurrent, 4);
}

/// Verify AppAdapter type bounds compile correctly.
/// We use a type-level assertion because constructing a concrete Scheme
/// (e.g., ed25519::certificate::Scheme) requires complex crypto setup.
#[test]
fn test_adapter_type_bounds_compile() {
    use crate::adapter::AppAdapter;
    use commonware_cryptography::certificate::Scheme;

    // This function asserts that AppAdapter<MockApp, CollectorSink, TestBlock, S>
    // satisfies Clone + Send for any Scheme S.
    fn _assert_adapter_bounds<S: Scheme>() {
        fn _assert_clone<T: Clone>() {}
        fn _assert_send<T: Send>() {}
        _assert_clone::<AppAdapter<MockApp, CollectorSink, TestBlock, S>>();
        _assert_send::<AppAdapter<MockApp, CollectorSink, TestBlock, S>>();
    }
}

/// Verify CommonwareEngine can start and report correct status.
#[tokio::test]
async fn test_engine_start_and_status() {
    let engine = CommonwareEngine::new(|height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
        // Verify initial state
        assert_eq!(height.load(Ordering::SeqCst), 0);
        assert!(!running.load(Ordering::SeqCst));

        let running_for_shutdown = Arc::clone(&running);
        let handle = tokio::spawn(async move {
            // Simulate a long-running consensus task that waits for shutdown
            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Ok(())
        });

        let shutdown: Box<dyn FnOnce() + Send> = Box::new(move || {
            running_for_shutdown.store(false, Ordering::SeqCst);
        });

        Ok((shutdown, handle))
    });

    let running_engine = engine.start().expect("engine should start");

    let status = running_engine.status();
    assert!(status.is_running);
    assert_eq!(status.current_height, 0);

    running_engine.shutdown().await.expect("shutdown should succeed");
}

/// Verify CommonwareEngine returns error when starter fails.
#[test]
fn test_engine_start_failure() {
    let engine = CommonwareEngine::new(|_height: Arc<AtomicU64>, _running: Arc<AtomicBool>| {
        Err(ConsensusError::NotReady("test failure".into()))
    });

    let result = engine.start();
    assert!(result.is_err());

    match result {
        Err(ConsensusError::NotReady(msg)) => assert_eq!(msg, "test failure"),
        Err(other) => panic!("expected NotReady, got: {other}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}

/// Verify engine shutdown completes cleanly.
#[tokio::test]
async fn test_engine_shutdown() {
    let engine = CommonwareEngine::new(|_height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
        let running_clone = Arc::clone(&running);
        let handle = tokio::spawn(async move {
            while running_clone.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok(())
        });

        let shutdown: Box<dyn FnOnce() + Send> = Box::new(move || { running.store(false, Ordering::SeqCst); });

        Ok((shutdown, handle))
    });

    let running_engine = engine.start().expect("engine should start");
    assert!(running_engine.status().is_running);

    let result = running_engine.shutdown().await;
    assert!(result.is_ok());
}

/// Verify engine height tracking via the atomic counter.
#[tokio::test]
async fn test_engine_height_tracking() {
    let engine = CommonwareEngine::new(|height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
        let height_clone = Arc::clone(&height);
        let running_clone = Arc::clone(&running);

        let handle = tokio::spawn(async move {
            // Simulate processing blocks
            for h in 1..=5 {
                height_clone.store(h, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            while running_clone.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Ok(())
        });

        let shutdown: Box<dyn FnOnce() + Send> = Box::new(move || { running.store(false, Ordering::SeqCst); });
        Ok((shutdown, handle))
    });

    let running_engine = engine.start().expect("engine should start");

    // Wait for blocks to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    let status = running_engine.status();
    assert!(status.is_running);
    assert_eq!(status.current_height, 5);

    running_engine.shutdown().await.expect("shutdown should succeed");
}

/// Verify that TestBlock correctly implements both core and vendor block traits.
#[test]
fn test_block_dual_trait_impl() {
    let genesis = TestBlock::genesis();

    // Core trait
    assert_eq!(CoreBlock::height(&genesis), 0);
    assert_eq!(genesis.id(), [0u8; 32]);

    // Vendor trait
    let vendor_height: commonware_consensus::types::Height = Heightable::height(&genesis);
    assert_eq!(vendor_height.get(), 0);

    // Child block
    let child = TestBlock::child(&genesis);
    assert_eq!(CoreBlock::height(&child), 1);
    assert_eq!(Heightable::height(&child).get(), 1);

    // Parent references match
    let core_parent_id = child.parent_id();
    let vendor_parent: TestDigest = VendorBlock::parent(&child);
    // Both should reference the genesis block
    let genesis_commitment = genesis.commitment();
    assert_eq!(vendor_parent, genesis_commitment);
    // Core parent_id is derived from genesis commitment bytes
    let expected: [u8; 32] = <[u8; 32]>::try_from(genesis_commitment.as_ref()).unwrap();
    assert_eq!(core_parent_id, expected);
}

/// Verify CollectorSink captures finalized events.
#[tokio::test]
async fn test_collector_sink_captures_events() {
    let (sink, events) = CollectorSink::new();

    let block = TestBlock::genesis();
    sink.handle(ConsensusEvent::Finalized {
        block: block.clone(),
        height: 42,
        proof: vec![],
    })
    .await;

    sink.handle(ConsensusEvent::Finalized {
        block,
        height: 43,
        proof: vec![1, 2, 3],
    })
    .await;

    let collected = events.lock().unwrap();
    assert_eq!(*collected, vec![42, 43]);
}
