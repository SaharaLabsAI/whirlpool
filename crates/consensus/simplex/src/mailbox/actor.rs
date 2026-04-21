use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Digestible;
use consensus::app::ConsensusApp;
use futures::channel::mpsc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::BlockStore;

use super::{compute_digest, is_valid_digest, Message};

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
