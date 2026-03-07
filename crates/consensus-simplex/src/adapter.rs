//! AppAdapter bridges consensus-core traits to commonware-consensus traits.

use std::marker::PhantomData;
use std::sync::Arc;

use futures::StreamExt;
use rand::Rng;
use tracing::{debug, warn};

use commonware_consensus::{
    marshal::ingress::mailbox::AncestorStream,
    simplex::types::{Activity, Context},
    Application, Heightable, Reporter, VerifyingApplication,
};
use commonware_cryptography::{certificate::Scheme, sha256::Digest, Committable};
use commonware_runtime::{Clock, Metrics, Spawner};
use consensus::{
    traits::{ConsensusApp, EventSink},
    ConsensusEvent,
};

use crate::traits::CommonwareBlock;
use crate::BlockStore;

/// Bridges `ConsensusApp` + `EventSink` (consensus-core) to
/// `Application` + `VerifyingApplication` + `Reporter` (commonware-consensus).
///
/// Generic parameters:
/// - `A`: The consensus application (implements `ConsensusApp`)
/// - `S`: The event sink for finalization notifications (implements `EventSink`)
/// - `B`: The block type (implements both core and commonware Block traits)
/// - `Sig`: The cryptographic signing scheme
pub struct AppAdapter<A, S, B, Sig>
where
    B: Committable<Commitment = Digest>,
{
    app: Arc<A>,
    sink: Arc<S>,
    /// Shared with [`MailboxActor`](crate::mailbox::MailboxActor) so that
    /// blocks created during propose/genesis are available when the reporter
    /// receives finalization activity.
    finalized_blocks: BlockStore<B>,
    _phantom: PhantomData<Sig>,
}

impl<A, S, B, Sig> Clone for AppAdapter<A, S, B, Sig>
where
    B: Committable<Commitment = Digest>,
{
    fn clone(&self) -> Self {
        Self {
            app: Arc::clone(&self.app),
            sink: Arc::clone(&self.sink),
            finalized_blocks: Arc::clone(&self.finalized_blocks),
            _phantom: PhantomData,
        }
    }
}

impl<A, S, B, Sig> AppAdapter<A, S, B, Sig>
where
    A: ConsensusApp<Block = B>,
    S: EventSink<Block = B>,
    B: CommonwareBlock + Committable<Commitment = Digest>,
    Sig: Scheme,
{
    /// Create a new adapter wrapping a consensus app, event sink, and shared block store.
    ///
    /// The `block_store` must be the **same** instance given to
    /// [`MailboxActor`](crate::mailbox::MailboxActor) so that blocks
    /// inserted during propose/genesis are visible to the reporter.
    pub fn new(app: Arc<A>, sink: Arc<S>, block_store: BlockStore<B>) -> Self {
        Self {
            app,
            sink,
            finalized_blocks: block_store,
            _phantom: PhantomData,
        }
    }

    async fn remember_block(&self, block: B) {
        self.finalized_blocks
            .write()
            .await
            .insert(block.commitment(), block);
    }
}

impl<E, A, S, B, Sig> Application<E> for AppAdapter<A, S, B, Sig>
where
    E: Rng + Spawner + Metrics + Clock,
    A: ConsensusApp<Block = B> + Clone + 'static,
    S: EventSink<Block = B> + 'static,
    B: CommonwareBlock + Committable<Commitment = Digest> + 'static,
    Sig: Scheme + 'static,
{
    type SigningScheme = Sig;
    type Context = Context<<B as Committable>::Commitment, <Sig as Scheme>::PublicKey>;
    type Block = B;

    async fn genesis(&mut self) -> Self::Block {
        let block = self.app.genesis().await;
        self.remember_block(block.clone()).await;
        block
    }

    async fn propose(
        &mut self,
        (_runtime, _context): (E, Self::Context),
        mut ancestry: AncestorStream<Self::SigningScheme, Self::Block>,
    ) -> Option<Self::Block> {
        // Marshaled passes [parent] in the ancestry stream for propose()
        let parent = ancestry.next().await?;
        self.remember_block(parent.clone()).await;
        let height = Heightable::height(&parent).next().get();
        let proposed = self.app.propose(&parent, height).await;
        if let Some(block) = &proposed {
            self.remember_block(block.clone()).await;
        }
        proposed
    }
}

impl<E, A, S, B, Sig> VerifyingApplication<E> for AppAdapter<A, S, B, Sig>
where
    E: Rng + Spawner + Metrics + Clock,
    A: ConsensusApp<Block = B> + Clone + 'static,
    S: EventSink<Block = B> + 'static,
    B: CommonwareBlock + Committable<Commitment = Digest> + 'static,
    Sig: Scheme + 'static,
{
    async fn verify(
        &mut self,
        (_runtime, _context): (E, Self::Context),
        mut ancestry: AncestorStream<Self::SigningScheme, Self::Block>,
    ) -> bool {
        // Marshaled passes [block, parent] in the ancestry stream for verify()
        let Some(block) = ancestry.next().await else {
            return false;
        };
        let Some(parent) = ancestry.next().await else {
            return false;
        };
        self.remember_block(parent.clone()).await;
        let verified = self.app.verify(&parent, &block).await.is_ok();
        if verified {
            self.remember_block(block).await;
        }
        verified
    }
}

impl<A, S, B, Sig> Reporter for AppAdapter<A, S, B, Sig>
where
    A: ConsensusApp<Block = B> + Clone + 'static,
    S: EventSink<Block = B> + 'static,
    B: CommonwareBlock + Committable<Commitment = Digest> + 'static,
    Sig: Scheme + 'static,
{
    type Activity = Activity<Sig, Digest>;

    async fn report(&mut self, activity: Self::Activity) {
        use Activity::*;

        match activity {
            Finalization(fin) => {
                // fin.proposal.payload is a Digest (the block's commitment)
                let commitment = fin.proposal.payload;
                if let Some(block) = self.finalized_blocks.write().await.remove(&commitment) {
                    let height = Heightable::height(&block).get();
                    self.sink
                        .handle(ConsensusEvent::Finalized {
                            block,
                            height,
                            proof: vec![],
                        })
                        .await;
                } else {
                    warn!(?commitment, "finalization received for unknown block");
                }
            }
            Certification(cert) => {
                // cert.proposal.payload is also a Digest
                let commitment = cert.proposal.payload;
                debug!(?commitment, "received certification activity");
            }
            other => {
                debug!(
                    ?other,
                    "ignoring simplex activity outside finalization path"
                );
            }
        }
    }
}
