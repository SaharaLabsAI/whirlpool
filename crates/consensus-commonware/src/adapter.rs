//! AppAdapter bridges consensus-core traits to commonware-consensus traits.

use std::marker::PhantomData;
use std::sync::Arc;

use futures::StreamExt;
use tracing::debug;
use rand::Rng;

use consensus_core::{ConsensusApp, ConsensusEvent, EventSink};
use commonware_consensus::{
    marshal::{ingress::mailbox::AncestorStream, Update},
    simplex::types::Context,
    Application, Heightable, Reporter, VerifyingApplication,
};
use commonware_cryptography::{certificate::Scheme, Committable};
use commonware_runtime::{Clock, Metrics, Spawner};
use commonware_utils::Acknowledgement;

use crate::types::CommonwareBlock;

/// Bridges `ConsensusApp` + `EventSink` (consensus-core) to
/// `Application` + `VerifyingApplication` + `Reporter` (commonware-consensus).
///
/// Generic parameters:
/// - `A`: The consensus application (implements `ConsensusApp`)
/// - `S`: The event sink for finalization notifications (implements `EventSink`)
/// - `B`: The block type (implements both core and commonware Block traits)
/// - `Sig`: The cryptographic signing scheme
pub struct AppAdapter<A, S, B, Sig> {
    app: Arc<A>,
    sink: Arc<S>,
    _phantom: PhantomData<(B, Sig)>,
}

impl<A, S, B, Sig> Clone for AppAdapter<A, S, B, Sig> {
    fn clone(&self) -> Self {
        Self {
            app: Arc::clone(&self.app),
            sink: Arc::clone(&self.sink),
            _phantom: PhantomData,
        }
    }
}

impl<A, S, B, Sig> AppAdapter<A, S, B, Sig>
where
    A: ConsensusApp<Block = B>,
    S: EventSink<Block = B>,
    B: CommonwareBlock,
    Sig: Scheme,
{
    /// Create a new adapter wrapping a consensus app and event sink.
    pub fn new(app: Arc<A>, sink: Arc<S>) -> Self {
        Self {
            app,
            sink,
            _phantom: PhantomData,
        }
    }
}

impl<E, A, S, B, Sig> Application<E> for AppAdapter<A, S, B, Sig>
where
    E: Rng + Spawner + Metrics + Clock,
    A: ConsensusApp<Block = B> + Clone + 'static,
    S: EventSink<Block = B> + Clone + 'static,
    B: CommonwareBlock + 'static,
    Sig: Scheme + 'static,
{
    type SigningScheme = Sig;
    type Context = Context<
        <B as Committable>::Commitment,
        <Sig as Scheme>::PublicKey,
    >;
    type Block = B;

    async fn genesis(&mut self) -> Self::Block {
        self.app.genesis().await
    }

    async fn propose(
        &mut self,
        (_runtime, _context): (E, Self::Context),
        mut ancestry: AncestorStream<Self::SigningScheme, Self::Block>,
    ) -> Option<Self::Block> {
        // Marshaled passes [parent] in the ancestry stream for propose()
        let parent = ancestry.next().await?;
        let height = Heightable::height(&parent).next().get();
        self.app.propose(&parent, height).await
    }
}

impl<E, A, S, B, Sig> VerifyingApplication<E> for AppAdapter<A, S, B, Sig>
where
    E: Rng + Spawner + Metrics + Clock,
    A: ConsensusApp<Block = B> + Clone + 'static,
    S: EventSink<Block = B> + Clone + 'static,
    B: CommonwareBlock + 'static,
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
        self.app.verify(&parent, &block).await.is_ok()
    }
}

impl<A, S, B, Sig> Reporter for AppAdapter<A, S, B, Sig>
where
    A: ConsensusApp<Block = B> + Clone + 'static,
    S: EventSink<Block = B> + Clone + 'static,
    B: CommonwareBlock + 'static,
    Sig: Scheme + 'static,
{
    type Activity = Update<B>;

    async fn report(&mut self, activity: Self::Activity) {
        match activity {
            Update::Block(block, ack) => {
                let height = Heightable::height(&block);
                self.sink
                    .handle(ConsensusEvent::Finalized {
                        block,
                        height: height.get(),
                        proof: vec![],
                    })
                    .await;
                ack.acknowledge();
            }
            Update::Tip(height, _commitment) => {
                debug!(height = height.get(), "received tip update");
            }
        }
    }
}
