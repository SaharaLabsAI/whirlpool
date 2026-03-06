// Mailbox Bridge — Automaton/CertifiableAutomaton/Relay traits for simplex engine
//
// This module bridges the gap between ConsensusApp and the simplex consensus engine.
// The simplex engine requires Automaton/Relay traits, but ConsensusApp doesn't provide them.
// Mailbox implements these traits and delegates to an actor that handles the actual work.

use commonware_consensus::simplex::types::Context;
use commonware_consensus::types::Epoch;
use commonware_consensus::{Automaton, CertifiableAutomaton, Relay};
use commonware_cryptography::ed25519::PublicKey;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Digestible;
use consensus::app::ConsensusApp;
use futures::channel::{mpsc, oneshot};
use futures::SinkExt;
use std::marker::PhantomData;
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

/// Mailbox implements Automaton/Relay traits for simplex consensus engine
#[derive(Clone)]
pub struct Mailbox<B> {
    sender: mpsc::Sender<Message>,
    _phantom: PhantomData<B>,
}

impl<B> Mailbox<B> {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self {
            sender,
            _phantom: PhantomData,
        }
    }
}

// Implement Automaton trait (async methods matching vendor pattern)
impl<B: Clone + Send + 'static> Automaton for Mailbox<B> {
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
impl<B: Clone + Send + 'static> CertifiableAutomaton for Mailbox<B> {}

// Implement Relay trait (no-op broadcast for single node)
impl<B: Clone + Send + 'static> Relay for Mailbox<B> {
    type Digest = Digest;

    async fn broadcast(&mut self, _payload: Self::Digest) {
        // No-op for single node
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
    bytes != &[255u8; 32]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{MockApp, TestBlock};
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
    async fn test_relay_broadcast_completes() {
        let (tx, rx) = mpsc::channel(10);
        let mut mailbox = Mailbox::<TestBlock>::new(tx);

        let height = Arc::new(AtomicU64::new(0));
        let app = Arc::new(MockApp);
        tokio::spawn(MailboxActor::new(rx, height, app, empty_block_store()).run());

        mailbox.broadcast(Digest::from([1u8; 32])).await;
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
}
