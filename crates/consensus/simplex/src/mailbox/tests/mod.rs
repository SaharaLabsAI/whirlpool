
use super::*;
use crate::tests::{MockApp, TestBlock};
use commonware_codec::Encode;
use commonware_consensus::simplex::Plan;
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
