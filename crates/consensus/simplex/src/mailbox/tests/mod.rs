use super::*;
use crate::tests::{MockApp, TestBlock};
use commonware_codec::Encode;
use commonware_consensus::simplex::Plan;
use commonware_consensus::types::{Epoch, Round, View};
use commonware_consensus::{Automaton, Block as CommonwareBlock, Relay};
use commonware_cryptography::ed25519::PrivateKey;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Signer;
use consensus::error::ConsensusError;
use futures::channel::mpsc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

fn empty_block_store() -> BlockStore<TestBlock> {
    Arc::new(RwLock::new(HashMap::new()))
}

fn context_for_parent(seed: u64, parent: &TestBlock) -> Context<Digest, PublicKey> {
    context_for_parent_digest(seed, compute_digest(parent))
}

fn context_for_parent_digest(seed: u64, parent_digest: Digest) -> Context<Digest, PublicKey> {
    let epoch = Epoch::new(1);
    let view = View::new(0);
    let round = Round::new(epoch, view);
    let parent_view = View::new(0);
    let private_key = PrivateKey::from_seed(seed);
    let leader = private_key.public_key();

    Context {
        round,
        leader,
        parent: (parent_view, parent_digest),
    }
}

fn start_mailbox<A>(app: Arc<A>, block_store: BlockStore<TestBlock>) -> Mailbox<TestBlock>
where
    A: consensus::app::ConsensusApp<Block = TestBlock>,
{
    let (tx, rx) = mpsc::channel(10);
    let mailbox = Mailbox::<TestBlock>::new(tx);
    let height = Arc::new(AtomicU64::new(0));
    tokio::spawn(MailboxActor::new(rx, height, app, block_store).run());
    mailbox
}

#[derive(Clone, Default)]
struct CountingApp {
    verify_calls: Arc<AtomicUsize>,
    propose_calls: Arc<AtomicUsize>,
}

impl consensus::app::ConsensusApp for CountingApp {
    type Block = TestBlock;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        async { TestBlock::genesis() }
    }

    fn propose(
        &self,
        parent: &Self::Block,
        _height: u64,
    ) -> impl std::future::Future<Output = Option<Self::Block>> + Send {
        self.propose_calls.fetch_add(1, Ordering::SeqCst);
        let child = TestBlock::child(parent);
        async move { Some(child) }
    }

    fn verify(
        &self,
        _parent: &Self::Block,
        _block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<(), ConsensusError>> + Send {
        self.verify_calls.fetch_add(1, Ordering::SeqCst);
        async { Ok(()) }
    }
}

#[derive(Clone, Default)]
struct AbstainingApp;

impl consensus::app::ConsensusApp for AbstainingApp {
    type Block = TestBlock;

    fn genesis(&self) -> impl std::future::Future<Output = Self::Block> + Send {
        async { TestBlock::genesis() }
    }

    fn propose(
        &self,
        _parent: &Self::Block,
        _height: u64,
    ) -> impl std::future::Future<Output = Option<Self::Block>> + Send {
        async { None }
    }

    fn verify(
        &self,
        _parent: &Self::Block,
        _block: &Self::Block,
    ) -> impl std::future::Future<Output = Result<(), ConsensusError>> + Send {
        async { Ok(()) }
    }
}

#[derive(Clone, Default)]
struct SlowVerifyApp {
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
}

impl consensus::app::ConsensusApp for SlowVerifyApp {
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
        let active = Arc::clone(&self.active);
        let max_active = Arc::clone(&self.max_active);
        async move {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            max_active.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(30)).await;
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        }
    }
}

#[tokio::test]
async fn test_genesis_returns_deterministic_digest() {
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(MockApp), block_store);

    let d1 = mailbox.genesis(Epoch::new(1)).await;
    let d2 = mailbox.genesis(Epoch::new(1)).await;
    assert_eq!(d1, d2);
}

#[tokio::test]
async fn test_propose_uses_context_parent_authority() {
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(MockApp), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let parent = TestBlock::child(&genesis);
    let parent_digest = compute_digest(&parent);
    block_store
        .write()
        .await
        .insert(parent_digest, parent.clone());

    let receiver = mailbox.propose(context_for_parent(1, &parent)).await;
    let digest = receiver.await.expect("proposal should produce digest");
    let proposed = block_store
        .read()
        .await
        .get(&digest)
        .cloned()
        .expect("proposed block is stored");

    assert_eq!(CommonwareBlock::parent(&proposed), parent_digest);
}

#[tokio::test]
async fn test_verify_valid_parent_and_block_calls_app_and_returns_true() {
    let block_store = empty_block_store();
    let app = Arc::new(CountingApp::default());
    let mut mailbox = start_mailbox(Arc::clone(&app), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let block = TestBlock::child(&genesis);
    let digest = compute_digest(&block);
    block_store.write().await.insert(digest, block);

    let receiver = mailbox
        .verify(context_for_parent(2, &genesis), digest)
        .await;
    assert!(receiver.await.expect("verify sends verdict"));
    assert_eq!(app.verify_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_verify_unknown_digest_remains_pending_until_payload_arrives() {
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(MockApp), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let block = TestBlock::child(&genesis);
    let digest = compute_digest(&block);
    let mut receiver = mailbox
        .verify(context_for_parent(3, &genesis), digest)
        .await;

    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut receiver)
            .await
            .is_err()
    );

    block_store.write().await.insert(digest, block);
    assert!(receiver
        .await
        .expect("verify resolves after payload arrives"));
}

#[tokio::test]
async fn test_verify_wrong_digest_cache_entry_returns_false() {
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(MockApp), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let block = TestBlock::child(&genesis);
    let wrong_digest = Digest::from([9u8; 32]);
    block_store.write().await.insert(wrong_digest, block);

    let receiver = mailbox
        .verify(context_for_parent(4, &genesis), wrong_digest)
        .await;
    assert!(!receiver.await.expect("verify sends false"));
}

#[tokio::test]
async fn test_verify_context_parent_mismatch_returns_false() {
    let block_store = empty_block_store();
    let app = Arc::new(CountingApp::default());
    let mut mailbox = start_mailbox(Arc::clone(&app), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let declared_parent = TestBlock::child(&genesis);
    let declared_parent_digest = compute_digest(&declared_parent);
    let block = TestBlock::child(&declared_parent);
    let digest = compute_digest(&block);
    block_store
        .write()
        .await
        .insert(declared_parent_digest, declared_parent);
    block_store.write().await.insert(digest, block);

    let wrong_parent = Digest::from([7u8; 32]);
    let wrong_parent_block =
        TestBlock::with_id_parent_digest([7u8; 32], compute_digest(&genesis), 1);
    block_store
        .write()
        .await
        .insert(wrong_parent, wrong_parent_block);

    let receiver = mailbox
        .verify(context_for_parent_digest(5, wrong_parent), digest)
        .await;
    assert!(!receiver.await.expect("verify sends verdict"));
    assert_eq!(app.verify_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_verify_height_one_wrong_context_parent_returns_false() {
    let block_store = empty_block_store();
    let app = Arc::new(CountingApp::default());
    let mut mailbox = start_mailbox(Arc::clone(&app), Arc::clone(&block_store));

    let wrong_parent = TestBlock::with_id_parent_digest([7u8; 32], Digest::from([0u8; 32]), 0);
    let wrong_parent_digest = compute_digest(&wrong_parent);
    let block = TestBlock::with_id_parent_digest([3u8; 32], Digest::from([2u8; 32]), 1);
    let digest = compute_digest(&block);
    block_store
        .write()
        .await
        .insert(wrong_parent_digest, wrong_parent);
    block_store.write().await.insert(digest, block);

    let receiver = mailbox
        .verify(context_for_parent_digest(6, wrong_parent_digest), digest)
        .await;
    assert!(!receiver.await.expect("verify sends verdict"));
    assert_eq!(app.verify_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_verify_missing_parent_does_not_use_genesis_fallback() {
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(MockApp), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let known_parent = TestBlock::child(&genesis);
    let missing_parent = Digest::from([8u8; 32]);
    let block = TestBlock::child_with_parent_digest(&known_parent, missing_parent, Vec::new());
    let digest = compute_digest(&block);
    block_store.write().await.insert(digest, block);

    let mut receiver = mailbox
        .verify(context_for_parent_digest(6, missing_parent), digest)
        .await;
    assert!(
        tokio::time::timeout(Duration::from_millis(25), &mut receiver)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_propose_abstain_produces_no_genesis_fallback_digest() {
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(AbstainingApp), block_store);

    let genesis = TestBlock::genesis();
    let receiver = mailbox.propose(context_for_parent(7, &genesis)).await;
    assert!(receiver.await.is_err(), "abstain closes without digest");
}

#[tokio::test]
async fn test_dropped_proposal_receiver_cancels_pending_parent_work() {
    let block_store = empty_block_store();
    let app = Arc::new(CountingApp::default());
    let mut mailbox = start_mailbox(Arc::clone(&app), Arc::clone(&block_store));

    let missing_parent = Digest::from([10u8; 32]);
    let receiver = mailbox
        .propose(context_for_parent_digest(8, missing_parent))
        .await;
    drop(receiver);

    let genesis = TestBlock::genesis();
    let parent =
        TestBlock::child_with_parent_digest(&genesis, compute_digest(&genesis), Vec::new());
    block_store.write().await.insert(missing_parent, parent);
    tokio::time::sleep(Duration::from_millis(25)).await;

    assert_eq!(app.propose_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn test_verify_concurrency_is_bounded() {
    let block_store = empty_block_store();
    let app = Arc::new(SlowVerifyApp::default());
    let mut mailbox = start_mailbox(Arc::clone(&app), Arc::clone(&block_store));

    let genesis = TestBlock::genesis();
    let mut receivers = Vec::new();
    for index in 0..32u8 {
        let block = TestBlock::with_id_parent_digest(
            [index.wrapping_add(1); 32],
            compute_digest(&genesis),
            1,
        );
        let digest = compute_digest(&block);
        block_store.write().await.insert(digest, block);
        receivers.push(
            mailbox
                .verify(context_for_parent(9, &genesis), digest)
                .await,
        );
    }

    for receiver in receivers {
        let _ = receiver.await;
    }

    assert!(
        app.max_active.load(Ordering::SeqCst) > 0,
        "verify requests must reach the app verifier"
    );
    assert!(
        app.max_active.load(Ordering::SeqCst) <= 16,
        "pending verify app calls must stay within semaphore cap"
    );
}

#[tokio::test]
async fn test_relay_broadcast_noop_without_wiring() {
    // Mailbox::new (no relay) — broadcast completes silently.
    let block_store = empty_block_store();
    let mut mailbox = start_mailbox(Arc::new(MockApp), block_store);

    mailbox
        .broadcast(Digest::from([1u8; 32]), Plan::Propose)
        .await;
    // No panic, no message sent — passes by definition.
}

#[tokio::test]
async fn test_relay_broadcast_sends_payload_message() {
    // with_relay mailbox — broadcast encodes and sends via payload_tx.
    let block_store = empty_block_store();
    let (payload_tx, mut payload_rx) = mpsc::unbounded::<Bytes>();
    let (mailbox_tx, mailbox_rx) = mpsc::channel(10);

    let mut mailbox =
        Mailbox::<TestBlock>::with_relay(mailbox_tx, Arc::clone(&block_store), payload_tx);

    let height = Arc::new(AtomicU64::new(0));
    let app = Arc::new(MockApp);
    tokio::spawn(MailboxActor::new(mailbox_rx, height, app, Arc::clone(&block_store)).run());

    // Populate block store with a known block.
    let block = TestBlock::genesis();
    let digest = compute_digest(&block);
    block_store.write().await.insert(digest, block.clone());

    // Broadcast the digest — should produce one outbound message.
    mailbox.broadcast(digest, Plan::Propose).await;

    // Verify a PayloadRelayMessage was enqueued.
    let wire = payload_rx.try_recv().expect("should have a message");
    let msg = PayloadRelayMessage::decode_wire(wire).expect("valid wire format");

    assert_eq!(msg.digest, digest);
    assert_eq!(msg.payload, block.encode());
}

#[tokio::test]
async fn test_relay_broadcast_missing_digest_is_silent() {
    // Block not in store — broadcast should log a warning and NOT send.
    let block_store = empty_block_store();
    let (payload_tx, mut payload_rx) = mpsc::unbounded::<Bytes>();
    let (mailbox_tx, mailbox_rx) = mpsc::channel(10);

    let mut mailbox =
        Mailbox::<TestBlock>::with_relay(mailbox_tx, Arc::clone(&block_store), payload_tx);

    let height = Arc::new(AtomicU64::new(0));
    let app = Arc::new(MockApp);
    tokio::spawn(MailboxActor::new(mailbox_rx, height, app, Arc::clone(&block_store)).run());

    // Broadcast a digest that has NO corresponding block.
    let unknown_digest = Digest::from([42u8; 32]);
    mailbox.broadcast(unknown_digest, Plan::Propose).await;

    // Channel should be empty — nothing was sent.
    assert!(payload_rx.try_recv().is_err(), "no message should be sent");
}

#[tokio::test]
async fn test_mailbox_clone_shares_channel() {
    let (tx, rx) = mpsc::channel::<Message>(10);
    let mailbox1 = Mailbox::<TestBlock>::new(tx);
    let mailbox2 = mailbox1.clone();

    let height = Arc::new(AtomicU64::new(0));
    let app = Arc::new(MockApp);
    tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

    drop(mailbox1);
    drop(mailbox2);
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
}

// -----------------------------------------------------------------------
// PayloadRelayMessage unit tests
// -----------------------------------------------------------------------

#[test]
fn test_payload_relay_message_roundtrip() {
    let digest = Digest::from([7u8; 32]);
    let payload = Bytes::from_static(b"hello block");
    let msg = PayloadRelayMessage::new(digest, payload.clone());

    let wire = msg.encode_wire();
    assert_eq!(wire.len(), DIGEST_SIZE + payload.len());

    let decoded = PayloadRelayMessage::decode_wire(wire).expect("decode should succeed");
    assert_eq!(decoded.digest, digest);
    assert_eq!(decoded.payload, payload);
}

#[test]
fn test_payload_relay_message_empty_payload() {
    let digest = Digest::from([0u8; 32]);
    let msg = PayloadRelayMessage::new(digest, Bytes::new());
    let wire = msg.encode_wire();
    assert_eq!(wire.len(), DIGEST_SIZE);

    let decoded = PayloadRelayMessage::decode_wire(wire).unwrap();
    assert_eq!(decoded.digest, digest);
    assert!(decoded.payload.is_empty());
}

#[test]
fn test_payload_relay_message_too_short() {
    let short = Bytes::from_static(&[0u8; 16]);
    assert!(PayloadRelayMessage::decode_wire(short).is_none());
}
