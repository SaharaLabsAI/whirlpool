use commonware_consensus::simplex::types::Context;
use commonware_consensus::Block as CommonwareBlock;
use commonware_cryptography::ed25519::PublicKey;
use commonware_cryptography::sha256::Digest;
use commonware_cryptography::Digestible;
use commonware_utils::channel::oneshot;
use consensus::app::ConsensusApp;
use consensus::block::Block as CoreBlock;
use futures::channel::mpsc;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::mailbox::{compute_digest, Message};
use crate::BlockStore;

const PENDING_TASK_LIMIT: usize = 16;
const PAYLOAD_RETRY_DELAY: Duration = Duration::from_millis(5);

/// MailboxActor processes messages and delegates to ConsensusApp.
///
/// Stores every block it creates (genesis / proposed) into the shared
/// [`BlockStore`] so that the [`AppAdapter`](crate::adapter::AppAdapter)
/// reporter can later find them when finalization arrives. The store is an
/// availability cache only: proposals use `Context.parent.1`; verification
/// resolves only that context parent before accepting direct block linkage or
/// the height-1 local-genesis compatibility path for app parent ids.
pub struct MailboxActor<A: ConsensusApp>
where
    A::Block: Digestible<Digest = Digest>,
{
    receiver: mpsc::Receiver<Message>,
    _height: Arc<AtomicU64>,
    app: Arc<A>,
    block_store: BlockStore<A::Block>,
    genesis_block: Option<A::Block>,
    pending_permits: Arc<Semaphore>,
}

impl<A> MailboxActor<A>
where
    A: ConsensusApp,
    A::Block: Digestible<Digest = Digest> + CommonwareBlock<Digest = Digest> + Clone,
{
    pub fn new(
        receiver: mpsc::Receiver<Message>,
        height: Arc<AtomicU64>,
        app: Arc<A>,
        block_store: BlockStore<A::Block>,
    ) -> Self {
        Self {
            receiver,
            _height: height,
            app,
            block_store,
            genesis_block: None,
            pending_permits: Arc::new(Semaphore::new(PENDING_TASK_LIMIT)),
        }
    }

    /// Store a block in the shared block store, keyed by its recomputed digest.
    async fn remember_block(&self, block: &A::Block) {
        remember_block(&self.block_store, block).await;
    }

    async fn ensure_genesis(&mut self) -> A::Block {
        if let Some(block) = &self.genesis_block {
            return block.clone();
        }

        let block = self.app.genesis().await;
        self.remember_block(&block).await;
        self.genesis_block = Some(block.clone());
        block
    }

    async fn handle_genesis(&mut self, response: oneshot::Sender<Digest>) {
        let block = self.ensure_genesis().await;
        let _ = response.send(compute_digest(&block));
    }

    async fn handle_propose(
        &mut self,
        context: Context<Digest, PublicKey>,
        response: oneshot::Sender<Digest>,
    ) {
        let genesis = self.ensure_genesis().await;
        let Ok(permit) = Arc::clone(&self.pending_permits).acquire_owned().await else {
            return;
        };
        let app = Arc::clone(&self.app);
        let block_store = Arc::clone(&self.block_store);

        tokio::spawn(async move {
            let _permit = permit;
            run_propose(app, block_store, genesis, context, response).await;
        });
    }

    async fn handle_verify(
        &mut self,
        context: Context<Digest, PublicKey>,
        digest: Digest,
        response: oneshot::Sender<bool>,
    ) {
        let genesis = self.ensure_genesis().await;
        let Ok(permit) = Arc::clone(&self.pending_permits).acquire_owned().await else {
            return;
        };
        let app = Arc::clone(&self.app);
        let block_store = Arc::clone(&self.block_store);

        tokio::spawn(async move {
            let _permit = permit;
            run_verify(app, block_store, genesis, context, digest, response).await;
        });
    }

    pub async fn run(mut self) {
        while let Ok(msg) = self.receiver.recv().await {
            match msg {
                Message::Genesis { epoch: _, response } => self.handle_genesis(response).await,
                Message::Propose { context, response } => {
                    self.handle_propose(context, response).await;
                }
                Message::Verify {
                    context,
                    digest,
                    response,
                } => {
                    self.handle_verify(context, digest, response).await;
                }
            }
        }
    }
}

async fn run_propose<A>(
    app: Arc<A>,
    block_store: BlockStore<A::Block>,
    genesis: A::Block,
    context: Context<Digest, PublicKey>,
    response: oneshot::Sender<Digest>,
) where
    A: ConsensusApp,
    A::Block: Digestible<Digest = Digest> + CommonwareBlock<Digest = Digest> + Clone,
{
    loop {
        if response.is_closed() {
            return;
        }

        if let Some(parent) = resolve_parent(&block_store, &genesis, context.parent.1).await {
            let height = CoreBlock::height(&parent) + 1;
            let Some(block) = app.propose(&parent, height).await else {
                return;
            };

            let digest = compute_digest(&block);
            remember_block(&block_store, &block).await;
            let _ = response.send(digest);
            return;
        }

        tokio::time::sleep(PAYLOAD_RETRY_DELAY).await;
    }
}

async fn run_verify<A>(
    app: Arc<A>,
    block_store: BlockStore<A::Block>,
    genesis: A::Block,
    context: Context<Digest, PublicKey>,
    digest: Digest,
    response: oneshot::Sender<bool>,
) where
    A: ConsensusApp,
    A::Block: Digestible<Digest = Digest> + CommonwareBlock<Digest = Digest> + Clone,
{
    loop {
        if response.is_closed() {
            return;
        }

        let block = match load_cached_block_checked(&block_store, digest).await {
            CachedBlock::Found(block) => block,
            CachedBlock::DigestMismatch => {
                tracing::warn!(
                    ?digest,
                    "mailbox verify rejected digest-mismatched cache entry"
                );
                let _ = response.send(false);
                return;
            }
            CachedBlock::Missing => {
                tokio::time::sleep(PAYLOAD_RETRY_DELAY).await;
                continue;
            }
        };

        let parent =
            match resolve_verification_parent(&block_store, &genesis, &block, context.parent.1)
                .await
            {
                ParentResolution::Found(parent) => parent,
                ParentResolution::Pending => {
                    tokio::time::sleep(PAYLOAD_RETRY_DELAY).await;
                    continue;
                }
                ParentResolution::Rejected => {
                    let _ = response.send(false);
                    return;
                }
            };

        let verified = app.verify(&parent, &block).await.is_ok();
        if verified {
            remember_block(&block_store, &block).await;
        }
        let _ = response.send(verified);
        return;
    }
}

enum ParentResolution<B> {
    Found(B),
    Pending,
    Rejected,
}

async fn resolve_verification_parent<B>(
    block_store: &BlockStore<B>,
    genesis: &B,
    block: &B,
    context_parent: Digest,
) -> ParentResolution<B>
where
    B: CommonwareBlock<Digest = Digest> + CoreBlock + Digestible<Digest = Digest> + Clone,
{
    if let Some(parent) = resolve_parent(block_store, genesis, context_parent).await {
        if block_links_to_parent(block, &parent, context_parent)
            || is_height_one_child_of_context_genesis(genesis, &parent, block, context_parent)
        {
            return ParentResolution::Found(parent);
        }
        return ParentResolution::Rejected;
    }

    if CommonwareBlock::parent(block) != context_parent {
        return ParentResolution::Rejected;
    }

    find_declared_parent(block_store, genesis, block)
        .await
        .map_or(ParentResolution::Pending, ParentResolution::Found)
}

fn block_links_to_parent<B>(block: &B, parent: &B, parent_digest: Digest) -> bool
where
    B: CommonwareBlock<Digest = Digest> + CoreBlock,
{
    CommonwareBlock::parent(block) == parent_digest
        || CoreBlock::parent_id(block) == CoreBlock::id(parent)
}

fn is_height_one_child_of_context_genesis<B>(
    genesis: &B,
    parent: &B,
    block: &B,
    context_parent: Digest,
) -> bool
where
    B: CoreBlock + Digestible<Digest = Digest>,
{
    compute_digest(genesis) == context_parent
        && CoreBlock::id(parent) == CoreBlock::id(genesis)
        && CoreBlock::height(parent) == 0
        && CoreBlock::height(block) == 1
}

async fn find_declared_parent<B>(block_store: &BlockStore<B>, genesis: &B, block: &B) -> Option<B>
where
    B: CommonwareBlock<Digest = Digest> + CoreBlock + Digestible<Digest = Digest> + Clone,
{
    if block_links_to_parent(block, genesis, compute_digest(genesis)) {
        return Some(genesis.clone());
    }

    let candidates = {
        let store = block_store.read().await;
        store.values().cloned().collect::<Vec<_>>()
    };

    candidates
        .into_iter()
        .find(|candidate| block_links_to_parent(block, candidate, compute_digest(candidate)))
}

async fn remember_block<B>(block_store: &BlockStore<B>, block: &B)
where
    B: Digestible<Digest = Digest> + Clone,
{
    let digest = compute_digest(block);
    block_store.write().await.insert(digest, block.clone());
}

async fn resolve_parent<B>(
    block_store: &BlockStore<B>,
    genesis: &B,
    parent_digest: Digest,
) -> Option<B>
where
    B: Digestible<Digest = Digest> + Clone,
{
    if compute_digest(genesis) == parent_digest {
        return Some(genesis.clone());
    }

    match load_cached_block_checked(block_store, parent_digest).await {
        CachedBlock::Found(block) => Some(block),
        CachedBlock::Missing | CachedBlock::DigestMismatch => None,
    }
}

enum CachedBlock<B> {
    Found(B),
    Missing,
    DigestMismatch,
}

async fn load_cached_block_checked<B>(block_store: &BlockStore<B>, digest: Digest) -> CachedBlock<B>
where
    B: Digestible<Digest = Digest> + Clone,
{
    let Some(block) = ({
        let store = block_store.read().await;
        store.get(&digest).cloned()
    }) else {
        return CachedBlock::Missing;
    };

    if compute_digest(&block) == digest {
        CachedBlock::Found(block)
    } else {
        CachedBlock::DigestMismatch
    }
}
