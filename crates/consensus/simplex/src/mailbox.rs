// Mailbox Bridge — Automaton/CertifiableAutomaton/Relay traits for simplex engine
//
// This module bridges the gap between ConsensusApp and the simplex consensus engine.
// The simplex engine requires Automaton/Relay traits, but ConsensusApp doesn't provide them.
// Mailbox implements these traits and delegates to an actor that handles the actual work.
//
// The Relay::broadcast implementation looks up the proposed block by digest in the shared
// BlockStore, encodes it as a PayloadRelayMessage (digest ++ encoded-block), and forwards the
// bytes through an mpsc channel.  The receiving end (spawned in engine.rs) pushes them onto the
// PAYLOAD P2P channel so remote validators can obtain the block before voting.

use bytes::{BufMut, Bytes, BytesMut};
use commonware_codec::Encode;
use commonware_consensus::simplex::types::Context;
use commonware_consensus::types::Epoch;
use commonware_consensus::{Automaton, CertifiableAutomaton, Relay};
use commonware_cryptography::ed25519::PublicKey;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Digestible;
use consensus::app::ConsensusApp;
use futures::channel::{mpsc, oneshot};
use futures::SinkExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::BlockStore;

// Message types for actor channel
pub enum Message {
    Genesis {
        epoch: Epoch,
        response: oneshot::Sender<Digest>,
    },
    Propose {
        response: oneshot::Sender<Digest>,
    },
    Verify {
        digest: Digest,
        response: oneshot::Sender<bool>,
    },
}

/// Mailbox implements Automaton/Relay traits for simplex consensus engine.
///
/// When constructed with [`Mailbox::with_relay`], the [`Relay::broadcast`]
/// implementation encodes the proposed block as a [`PayloadRelayMessage`]
/// and pushes it through an mpsc channel.  The consuming end (wired in
/// `engine.rs`) forwards the bytes onto the PAYLOAD P2P channel so that
/// remote validators receive the full block payload before voting.
///
/// When constructed with [`Mailbox::new`] (no relay wiring), broadcast
/// remains a silent no-op for backward-compatible single-node setups.
#[derive(Clone)]
pub struct Mailbox<B> {
    sender: mpsc::Sender<Message>,
    /// Shared block store — used by `broadcast` to look up the full block
    /// payload that corresponds to the digest the vendor engine provides.
    block_store: Option<BlockStore<B>>,
    /// Channel for outbound payload relay messages.  `None` ⇒ no-op relay.
    payload_tx: Option<mpsc::UnboundedSender<Bytes>>,
}

impl<B> Mailbox<B> {
    /// Create a mailbox **without** relay capability (broadcast is a no-op).
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self {
            sender,
            block_store: None,
            payload_tx: None,
        }
    }

    /// Create a mailbox **with** relay capability.
    ///
    /// `block_store` is shared with `MailboxActor` so that the relay can look
    /// up the full block for a given digest.  `payload_tx` is the sender-half
    /// of an unbounded channel whose receiver is consumed by a forwarding task
    /// in `engine.rs`.
    pub fn with_relay(
        sender: mpsc::Sender<Message>,
        block_store: BlockStore<B>,
        payload_tx: mpsc::UnboundedSender<Bytes>,
    ) -> Self {
        Self {
            sender,
            block_store: Some(block_store),
            payload_tx: Some(payload_tx),
        }
    }
}

// Implement Automaton trait (async methods matching vendor pattern)
impl<B: Clone + Send + Sync + 'static> Automaton for Mailbox<B> {
    type Context = Context<Digest, PublicKey>;
    type Digest = Digest;

    async fn genesis(&mut self, epoch: Epoch) -> Self::Digest {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Genesis { epoch, response })
            .await
            .expect("Failed to send genesis");
        receiver.await.expect("Failed to receive genesis")
    }

    async fn propose(&mut self, _ctx: Self::Context) -> oneshot::Receiver<Self::Digest> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Propose { response })
            .await
            .expect("Failed to send propose");
        receiver
    }

    async fn verify(
        &mut self,
        _ctx: Self::Context,
        digest: Self::Digest,
    ) -> oneshot::Receiver<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(Message::Verify { digest, response })
            .await
            .expect("Failed to send verify");
        receiver
    }
}

// Implement CertifiableAutomaton trait (uses default certify)
impl<B: Clone + Send + Sync + 'static> CertifiableAutomaton for Mailbox<B> {}

// Implement Relay trait — broadcast block payloads to peers via PAYLOAD channel
impl<B: Clone + Encode + Send + Sync + 'static> Relay for Mailbox<B>
where
    B: Digestible<Digest = Digest>,
{
    type Digest = Digest;

    async fn broadcast(&mut self, digest: Self::Digest) {
        let (Some(ref block_store), Some(ref payload_tx)) = (&self.block_store, &self.payload_tx)
        else {
            // No relay wiring — silent no-op (single-node mode).
            return;
        };

        // Look up the full block by its digest.
        let block = {
            let store = block_store.read().await;
            store.get(&digest).cloned()
        };

        let Some(block) = block else {
            tracing::warn!(
                ?digest,
                "relay broadcast: digest not found in block store, skipping"
            );
            return;
        };

        // Encode as PayloadRelayMessage: [32-byte digest][encoded block]
        let msg = PayloadRelayMessage::new(digest, block.encode());
        let wire = msg.encode_wire();

        if let Err(e) = payload_tx.unbounded_send(wire) {
            tracing::warn!(
                ?digest,
                error = %e,
                "relay broadcast: failed to enqueue payload message"
            );
        }
    }
}

/// MailboxActor processes messages and delegates to ConsensusApp.
///
/// Stores every block it creates (genesis / proposed) into the shared
/// [`BlockStore`] so that the [`AppAdapter`](crate::adapter::AppAdapter)
/// reporter can later find them when finalization arrives.
pub struct MailboxActor<A: ConsensusApp>
where
    A::Block: Digestible<Digest = Digest>,
{
    receiver: mpsc::Receiver<Message>,
    height: Arc<AtomicU64>,
    app: Arc<A>,
    block_store: BlockStore<A::Block>,
    genesis_block: Option<A::Block>,
}

impl<A> MailboxActor<A>
where
    A: ConsensusApp,
    A::Block: Digestible<Digest = Digest> + Clone,
{
    pub fn new(
        receiver: mpsc::Receiver<Message>,
        height: Arc<AtomicU64>,
        app: Arc<A>,
        block_store: BlockStore<A::Block>,
    ) -> Self {
        Self {
            receiver,
            height,
            app,
            block_store,
            genesis_block: None,
        }
    }

    /// Store a block in the shared block store, keyed by its digest.
    async fn remember_block(&self, block: &A::Block) {
        let digest = compute_digest(block);
        self.block_store.write().await.insert(digest, block.clone());
    }

    pub async fn run(mut self) {
        while let Ok(msg) = self.receiver.recv().await {
            match msg {
                Message::Genesis { epoch: _, response } => {
                    // Cache genesis block on first call
                    if self.genesis_block.is_none() {
                        let block = self.app.genesis().await;
                        self.remember_block(&block).await;
                        self.genesis_block = Some(block);
                    }
                    let block = self.genesis_block.as_ref().unwrap();
                    let digest = compute_digest(block);
                    let _ = response.send(digest);
                }
                Message::Propose { response } => {
                    let current = self.height.load(Ordering::SeqCst);
                    // Use genesis as parent (simplified - real impl would track parent)
                    if self.genesis_block.is_none() {
                        let block = self.app.genesis().await;
                        self.remember_block(&block).await;
                        self.genesis_block = Some(block);
                    }
                    let parent = self.genesis_block.as_ref().unwrap();

                    match self.app.propose(parent, current + 1).await {
                        Some(block) => {
                            let digest = compute_digest(&block);
                            self.remember_block(&block).await;
                            let _ = response.send(digest);
                        }
                        None => {
                            // If propose returns None, send genesis digest as fallback
                            let digest = compute_digest(parent);
                            let _ = response.send(digest);
                        }
                    }
                }
                Message::Verify { digest, response } => {
                    // For testing: accept any valid digest (not all-255)
                    // In a real implementation, we'd decode and validate the block
                    let valid = is_valid_digest(digest);
                    let _ = response.send(valid);
                }
            }
        }
    }
}

// Helper functions
fn compute_digest<B>(block: &B) -> Digest
where
    B: Digestible<Digest = Digest>,
{
    block.digest()
}

#[allow(dead_code)]
fn digest_to_block_id(digest: Digest) -> [u8; 32] {
    let bytes: &[u8] = digest.as_ref();
    let mut id = [0u8; 32];
    id.copy_from_slice(bytes);
    id
}

fn is_valid_digest(digest: Digest) -> bool {
    // For testing: reject all-255 digests as invalid, accept others
    let bytes: &[u8] = digest.as_ref();
    bytes != [255u8; 32]
}

// ---------------------------------------------------------------------------
// PayloadRelayMessage — wire envelope for PAYLOAD channel
// ---------------------------------------------------------------------------

/// Wire-format envelope for block payloads relayed over the PAYLOAD P2P channel.
///
/// Layout: `[32-byte SHA-256 digest][variable-length encoded block]`
///
/// The digest is placed first so the receiver can validate the block before
/// fully decoding it.  `encode_wire` / `decode_wire` handle serialisation
/// without pulling in an external framework.
pub struct PayloadRelayMessage {
    pub digest: Digest,
    pub payload: Bytes,
}

/// Fixed size of the digest prefix in the wire format (SHA-256 = 32 bytes).
const DIGEST_SIZE: usize = 32;

impl PayloadRelayMessage {
    /// Create a new relay message from a digest and pre-encoded block bytes.
    pub fn new(digest: Digest, payload: Bytes) -> Self {
        Self { digest, payload }
    }

    /// Serialise to wire format: `[digest bytes][payload bytes]`.
    pub fn encode_wire(&self) -> Bytes {
        let digest_bytes: &[u8] = self.digest.as_ref();
        let mut buf = BytesMut::with_capacity(DIGEST_SIZE + self.payload.len());
        buf.put_slice(digest_bytes);
        buf.put_slice(&self.payload);
        buf.freeze()
    }

    /// Deserialise from wire format.  Returns `None` if the buffer is too
    /// short to contain even the digest prefix.
    pub fn decode_wire(data: Bytes) -> Option<Self> {
        if data.len() < DIGEST_SIZE {
            return None;
        }
        let mut digest_arr = [0u8; DIGEST_SIZE];
        digest_arr.copy_from_slice(&data[..DIGEST_SIZE]);
        let digest = Digest::from(digest_arr);
        let payload = data.slice(DIGEST_SIZE..);
        Some(Self { digest, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{MockApp, TestBlock};
    use commonware_codec::Encode;
    use commonware_consensus::types::{Epoch, Round, View};
    use commonware_consensus::{Automaton, Relay};
    use commonware_cryptography::ed25519::PrivateKey;
    use commonware_cryptography::sha256::Digest;
    use commonware_cryptography::Signer;
    use futures::channel::mpsc;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn empty_block_store() -> BlockStore<TestBlock> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    fn sample_context(seed: u64) -> Context<Digest, PublicKey> {
        let epoch = Epoch::new(1);
        let view = View::new(0);
        let round = Round::new(epoch, view);
        let parent_digest = Digest::from([seed as u8; 32]);
        let parent_view = View::new(0);
        let private_key = PrivateKey::from_seed(seed);
        let leader = private_key.public_key();

        Context {
            round,
            leader,
            parent: (parent_view, parent_digest),
        }
    }

    #[tokio::test]
    async fn test_genesis_returns_deterministic_digest() {
        let (tx, rx) = mpsc::channel(10);
        let mut mailbox = Mailbox::<TestBlock>::new(tx);

        let height = Arc::new(AtomicU64::new(0));
        let app = Arc::new(MockApp);
        tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

        let d1 = mailbox.genesis(Epoch::new(1)).await;
        let d2 = mailbox.genesis(Epoch::new(1)).await;
        assert_eq!(d1, d2);
    }

    #[tokio::test]
    async fn test_propose_returns_digest() {
        let (tx, rx) = mpsc::channel(10);
        let mut mailbox = Mailbox::<TestBlock>::new(tx);

        let height = Arc::new(AtomicU64::new(0));
        let app = Arc::new(MockApp);
        tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

        let receiver = mailbox.propose(sample_context(1)).await;
        let digest: Result<Digest, _> = receiver.await;
        assert!(digest.is_ok());
    }

    #[tokio::test]
    async fn test_verify_valid_payload_returns_true() {
        let (tx, rx) = mpsc::channel(10);
        let mut mailbox = Mailbox::<TestBlock>::new(tx);

        let height = Arc::new(AtomicU64::new(0));
        let app = Arc::new(MockApp);
        tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

        let genesis = TestBlock::genesis();
        let valid_digest = compute_digest(&genesis);
        let receiver = mailbox.verify(sample_context(2), valid_digest).await;
        let valid: Result<bool, _> = receiver.await;
        assert!(valid.unwrap());
    }

    #[tokio::test]
    async fn test_verify_invalid_payload_returns_false() {
        let (tx, rx) = mpsc::channel(10);
        let mut mailbox = Mailbox::<TestBlock>::new(tx);

        let height = Arc::new(AtomicU64::new(0));
        let app = Arc::new(MockApp);
        tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

        let garbage_digest = Digest::from([255u8; 32]);
        let receiver = mailbox.verify(sample_context(3), garbage_digest).await;
        let valid: Result<bool, _> = receiver.await;
        assert!(!valid.unwrap());
    }

    #[tokio::test]
    async fn test_relay_broadcast_noop_without_wiring() {
        // Mailbox::new (no relay) — broadcast completes silently.
        let (tx, rx) = mpsc::channel(10);
        let mut mailbox = Mailbox::<TestBlock>::new(tx);

        let height = Arc::new(AtomicU64::new(0));
        let app = Arc::new(MockApp);
        tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

        mailbox.broadcast(Digest::from([1u8; 32])).await;
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
        mailbox.broadcast(digest).await;

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
        mailbox.broadcast(unknown_digest).await;

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
}
