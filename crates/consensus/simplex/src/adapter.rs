//! AppAdapter bridges consensus-core traits to commonware-consensus traits.

use std::marker::PhantomData;
use std::sync::Arc;

use futures::StreamExt;
use rand::Rng;
use tracing::{debug, warn};

use commonware_actor::Feedback;
use commonware_consensus::{
    marshal::ancestry::Ancestry,
    simplex::types::{Activity, Context},
    Application, Heightable, Reporter,
};
use commonware_cryptography::{
    certificate::{Scheme, Verifier},
    sha256::Digest, Committable,
};
use commonware_runtime::{Clock, Metrics, Spawner};
use consensus::{
    traits::{ConsensusApp, EventSink},
    ConsensusEvent,
};

use crate::traits::CommonwareBlock;
use crate::BlockStore;

/// Bridges `ConsensusApp` + `EventSink` (consensus-core) to
/// `Application` + `Reporter` (commonware-consensus).
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
    /// blocks created during propose are available when the reporter
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
    /// inserted during propose are visible to the reporter.
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
    type Context = Context<<B as Committable>::Commitment, <Sig as Verifier>::PublicKey>;
    type Block = B;

    async fn propose(
        &mut self,
        (_runtime, _context): (E, Self::Context),
        mut ancestry: impl Ancestry<Self::Block>,
    ) -> Option<Self::Block> {
        // Marshaled passes [parent] in the ancestry stream for propose()
        let parent = ancestry.next().await?;
        let parent = parent.as_ref().clone();
        self.remember_block(parent.clone()).await;
        let height = Heightable::height(&parent).get();
        let proposed = self.app.propose(&parent, height).await;
        if let Some(block) = &proposed {
            self.remember_block(block.clone()).await;
        }
        proposed
    }

    async fn verify(
        &mut self,
        (_runtime, _context): (E, Self::Context),
        mut ancestry: impl Ancestry<Self::Block>,
    ) -> bool {
        // Marshaled passes [block, parent] in the ancestry stream for verify()
        let Some(block) = ancestry.next().await else {
            return false;
        };
        let Some(parent) = ancestry.next().await else {
            return false;
        };
        let parent = parent.as_ref().clone();
        self.remember_block(parent.clone()).await;
        let verified = self.app.verify(&parent, block.as_ref()).await.is_ok();
        if verified {
            self.remember_block(block.as_ref().clone()).await;
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

    fn report(&mut self, activity: Self::Activity) -> Feedback {
        // `Reporter::report` is synchronous; dispatch the (async) finalization
        // side effects to a spawned task and acknowledge immediately.
        let sink = Arc::clone(&self.sink);
        let store = Arc::clone(&self.finalized_blocks);
        tokio::spawn(async move {
            use Activity::*;

            match activity {
                Finalization(fin) => {
                    // fin.proposal.payload is a Digest (the block's commitment)
                    let commitment = fin.proposal.payload;
                    let block = {
                        let store = store.read().await;
                        store.get(&commitment).cloned()
                    };

                    if let Some(block) = block {
                        let height = Heightable::height(&block).get();
                        sink.handle(ConsensusEvent::Finalized {
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
        });
        Feedback::Ok
    }
}
