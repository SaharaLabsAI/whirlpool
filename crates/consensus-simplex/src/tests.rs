//! Unit tests for the consensus-simplex crate.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, Read as CodecRead, Write as CodecWrite};
use commonware_consensus::{Block as VendorBlock, Heightable};
use commonware_cryptography::{Committable, Digestible};
use consensus::block::Block as CoreBlock;
use consensus::engine::ConsensusEngine;
use consensus::error::ConsensusError;
use consensus::event::{ConsensusEvent, EventSink};

use crate::config::CommonwareConfig;
use crate::engine::CommonwareEngine;
use crate::sink::FinalizationSink;
use crate::types::CommonwareBlock;

type TestDigest = commonware_cryptography::sha256::Digest;

fn zero_digest() -> TestDigest {
    commonware_cryptography::sha256::Sha256::fill(0)
}

#[derive(Clone, Debug)]
pub(crate) struct TestBlock {
    id: [u8; 32],
    parent: TestDigest,
    height: u64,
}

impl TestBlock {
    pub(crate) fn genesis() -> Self {
        Self {
            id: [0u8; 32],
            parent: zero_digest(),
            height: 0,
        }
    }

    pub(crate) fn child(parent: &Self) -> Self {
        let mut id = [0u8; 32];
        id[0] = (parent.height + 1) as u8;
        Self {
            id,
            parent: parent.commitment(),
            height: parent.height + 1,
        }
    }
}

impl CoreBlock for TestBlock {
    type Id = [u8; 32];

    fn id(&self) -> Self::Id {
        self.id
    }

    fn parent_id(&self) -> Self::Id {
        let bytes: &[u8] = self.parent.as_ref();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32]);
        arr
    }

    fn height(&self) -> u64 {
        self.height
    }
}

impl CodecWrite for TestBlock {
    fn write(&self, buf: &mut impl BufMut) {
        buf.put_slice(&self.id);
        buf.put_slice(self.parent.as_ref());
        buf.put_u64(self.height);
    }
}

impl EncodeSize for TestBlock {
    fn encode_size(&self) -> usize {
        32 + 32 + 8
    }
}

impl CodecRead for TestBlock {
    type Cfg = ();

    fn read_cfg(reader: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, CodecError> {
        if reader.remaining() < 72 {
            return Err(CodecError::Invalid("TestBlock", "not enough bytes"));
        }

        let mut id = [0u8; 32];
        reader.copy_to_slice(&mut id);

        let mut digest_bytes = [0u8; 32];
        reader.copy_to_slice(&mut digest_bytes);

        let height = reader.get_u64();

        Ok(Self {
            id,
            parent: TestDigest::from(digest_bytes),
            height,
        })
    }
}

impl Digestible for TestBlock {
    type Digest = TestDigest;

    fn digest(&self) -> Self::Digest {
        TestDigest::from(self.id)
    }
}

impl Committable for TestBlock {
    type Commitment = TestDigest;

    fn commitment(&self) -> Self::Commitment {
        self.digest()
    }
}

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

struct CollectorSink {
    events: Arc<Mutex<Vec<u64>>>,
}

impl CollectorSink {
    fn new() -> (Arc<Self>, Arc<Mutex<Vec<u64>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        (
            Arc::new(Self {
                events: Arc::clone(&events),
            }),
            events,
        )
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

pub(crate) struct MockApp;

impl consensus::app::ConsensusApp for MockApp {
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

fn test_config(namespace: &str) -> CommonwareConfig {
    CommonwareConfig {
        namespace: namespace.to_string(),
        leader_timeout: Duration::from_secs(1),
        notarization_timeout: Duration::from_secs(1),
        nullify_retry: Duration::from_millis(100),
        activity_timeout: 10,
        skip_timeout: 5,
        mailbox_size: 16,
        replay_buffer: NonZeroUsize::new(16).unwrap(),
        write_buffer: NonZeroUsize::new(16).unwrap(),
        epoch: 0,
        fetch_timeout: Duration::from_secs(1),
        fetch_concurrent: 4,
    }
}

#[test]
fn test_commonware_block_blanket_impl() {
    fn assert_commonware_block<T: CommonwareBlock>() {}
    assert_commonware_block::<TestBlock>();
}

#[test]
fn test_config_construction() {
    let config = test_config("test-consensus");
    assert_eq!(config.namespace, "test-consensus");
    assert_eq!(config.mailbox_size, 16);
    assert_eq!(config.replay_buffer.get(), 16);
    assert_eq!(config.write_buffer.get(), 16);
}

#[test]
fn test_adapter_type_bounds_compile() {
    use crate::adapter::AppAdapter;
    use commonware_cryptography::certificate::Scheme;

    fn _assert_adapter_bounds<S: Scheme>() {
        fn _assert_clone<T: Clone>() {}
        fn _assert_send<T: Send>() {}
        _assert_clone::<AppAdapter<MockApp, CollectorSink, TestBlock, S>>();
        _assert_send::<AppAdapter<MockApp, CollectorSink, TestBlock, S>>();
    }
}

#[tokio::test]
async fn test_engine_start_and_status() {
    let app = Arc::new(MockApp);
    let height = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(FinalizationSink::<TestBlock>::new(height));
    let engine = CommonwareEngine::new(app, sink, test_config("engine-start"));

    let running_engine = engine.start().expect("engine should start");
    let status = running_engine.status();
    assert!(status.is_running);
    assert_eq!(status.current_height, 0);

    running_engine.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn test_engine_shutdown() {
    let app = Arc::new(MockApp);
    let height = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(FinalizationSink::<TestBlock>::new(height));
    let engine = CommonwareEngine::new(app, sink, test_config("engine-shutdown"));

    let running_engine = engine.start().expect("engine should start");
    assert!(running_engine.status().is_running);
    running_engine.shutdown().await.expect("shutdown should succeed");
}

#[tokio::test]
async fn test_engine_height_tracking() {
    let app = Arc::new(MockApp);
    let _height = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(FinalizationSink::<TestBlock>::new(Arc::clone(&_height)));
    let engine = CommonwareEngine::new(app, sink, test_config("engine-height"));

    let running_engine = engine.start().expect("engine should start");

    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    let mut observed_height = 0u64;
    let mut reached_height = false;
    while tokio::time::Instant::now() < deadline {
        observed_height = running_engine.status().current_height;
        if observed_height >= 1 {
            reached_height = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    running_engine.shutdown().await.expect("shutdown should succeed");
    assert!(
        reached_height,
        "expected height >= 1, observed {}",
        observed_height
    );
}

#[test]
fn test_block_dual_trait_impl() {
    let genesis = TestBlock::genesis();

    assert_eq!(CoreBlock::height(&genesis), 0);
    assert_eq!(genesis.id(), [0u8; 32]);

    let vendor_height: commonware_consensus::types::Height = Heightable::height(&genesis);
    assert_eq!(vendor_height.get(), 0);

    let child = TestBlock::child(&genesis);
    assert_eq!(CoreBlock::height(&child), 1);
    assert_eq!(Heightable::height(&child).get(), 1);

    let core_parent_id = child.parent_id();
    let vendor_parent: TestDigest = VendorBlock::parent(&child);
    let genesis_commitment = genesis.commitment();
    assert_eq!(vendor_parent, genesis_commitment);

    let expected: [u8; 32] = <[u8; 32]>::try_from(genesis_commitment.as_ref()).unwrap();
    assert_eq!(core_parent_id, expected);
}

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
