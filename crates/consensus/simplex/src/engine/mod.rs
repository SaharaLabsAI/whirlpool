// CommonwareEngine — sealed wiring for commonware simplex BFT consensus
//!
//! This module provides sealed internal wiring that connects ConsensusApp and EventSink
//! to the commonware simplex BFT engine. Internal infrastructure (Mailbox, MailboxActor,
//! AppAdapter) is created and managed here, while the caller-provided EventSink is threaded
//! through so that finalization side-effects (e.g. block persistence) actually fire.

use std::collections::HashMap;
use std::num::{NonZeroU16, NonZeroUsize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use futures::channel::mpsc;
use futures::StreamExt;
use tokio::task::JoinHandle;

use commonware_consensus::simplex::{self, elector::RoundRobin};
use commonware_consensus::types::{Epoch, ViewDelta};
use commonware_cryptography::{bls12381::primitives::variant::MinSig, ed25519};
use commonware_cryptography::{
    sha256::{Digest, Sha256},
    Committable, Digestible,
};
use commonware_parallel::Sequential;
use commonware_runtime::buffer::paged::CacheRef;
use commonware_runtime::{BufferPooler, Clock, Metrics, Spawner, Storage};
use commonware_utils::ordered::Set;
use consensus::app::ConsensusApp;
use consensus::engine::{ConsensusEngine, RunningEngine};
use consensus::error::ConsensusError;
use consensus::event::EventSink;
use network::types::Channel;
use network_commonware::CommonwareReceiver;
use rand_core::CryptoRngCore;

use crate::adapter::AppAdapter;
use crate::config::{CommonwareConfig, SigningSchemeConfig};
use crate::mailbox::{Mailbox, MailboxActor};
use crate::receiver::payload_receive_loop;
use crate::traits::CommonwareBlock;
use crate::BlockStore;

type BlsThresholdVrfScheme =
    simplex::scheme::bls12381_threshold::vrf::Scheme<ed25519::PublicKey, MinSig>;

/// A consensus engine backed by the Commonware Simplex BFT protocol.
///
/// `CommonwareEngine` provides sealed internal wiring that connects your application
/// and event sink to the simplex consensus engine. All infrastructure components
/// (mailbox bridging, actor spawning, adapter wiring) are handled internally.
///
/// # Construction
///
/// ```ignore
/// let engine = CommonwareEngine::new(app, sink, config, network, context);
/// let running = engine.start()?;
/// ```
pub struct CommonwareEngine<A, S, E, C>
where
    A: ConsensusApp,
    S: EventSink<Block = A::Block>,
    A::Block: CommonwareBlock + Digestible<Digest = Digest>,
    E: Spawner
        + BufferPooler
        + Clock
        + CryptoRngCore
        + commonware_runtime::Network
        + commonware_runtime::Resolver
        + Metrics
        + Storage,
    C: commonware_cryptography::Signer + Send + Sync + 'static,
    C::PublicKey: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    app: Arc<A>,
    sink: Arc<S>,
    config: CommonwareConfig,
    network: network_commonware::CommonwareNetworkProvider<E, C>,
    context: E,
}

impl<A, S, E, C> CommonwareEngine<A, S, E, C>
where
    A: ConsensusApp + Send + Sync + 'static,
    S: EventSink<Block = A::Block> + Send + Sync + 'static,
    A::Block: CommonwareBlock + Digestible<Digest = Digest> + Send + Sync + 'static,
    E: Spawner
        + BufferPooler
        + Clock
        + CryptoRngCore
        + commonware_runtime::Network
        + commonware_runtime::Resolver
        + Metrics
        + Storage
        + Send
        + Sync
        + 'static,
    C: commonware_cryptography::Signer + Send + Sync + 'static,
    C::PublicKey: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static,
{
    /// Create a new `CommonwareEngine` with the given app, sink, config, network provider, and context.
    ///
    /// # Arguments
    /// - `app`: The consensus application (implements ConsensusApp)
    /// - `sink`: The event sink for finalization notifications
    /// - `config`: Configuration for the simplex consensus engine
    /// - `network`: Commonware network provider for P2P communication
    /// - `context`: Runtime context used for commonware engine operations
    pub fn new(
        app: Arc<A>,
        sink: Arc<S>,
        config: CommonwareConfig,
        network: network_commonware::CommonwareNetworkProvider<E, C>,
        context: E,
    ) -> Self {
        Self {
            app,
            sink,
            config,
            network,
            context,
        }
    }
}

impl<A, S, E, C> ConsensusEngine for CommonwareEngine<A, S, E, C>
where
    A: ConsensusApp + Clone + Send + Sync + 'static,
    S: EventSink<Block = A::Block> + Send + Sync + 'static,
    A::Block: CommonwareBlock
        + Digestible<Digest = Digest>
        + Committable<Commitment = Digest>
        + commonware_codec::Encode
        + commonware_codec::Decode<Cfg = ()>
        + Send
        + Sync
        + 'static,
    E: Spawner
        + BufferPooler
        + Clock
        + CryptoRngCore
        + commonware_runtime::Network
        + commonware_runtime::Resolver
        + Metrics
        + Storage
        + Clone
        + Send
        + 'static,
    C: commonware_cryptography::Signer<PublicKey = ed25519::PublicKey> + Send + Sync + 'static,
    C::Signature: Send + Sync + 'static,
{
    fn start(self) -> Result<RunningEngine, ConsensusError> {
        // Step 1: Clone oracle before consuming network
        let oracle = self.network.oracle().clone();

        // Step 2: Start network to get four channel pairs (raw vendor types)
        let per_channel = self
            .network
            .start_per_channel()
            .map_err(|e| ConsensusError::Other(format!("Failed to start network: {}", e).into()))?;

        // Destructure the PAYLOAD channel pair — used for relay wiring.
        let (mut payload_sender, payload_receiver) = per_channel.payload;

        // Step 3: Create shared height tracker and block store
        //
        // Block store is created early so it can be shared with:
        //   - MailboxActor   (writes on propose/genesis)
        //   - Mailbox Relay  (reads on broadcast)
        //   - payload_receive_loop (writes on inbound payloads)
        //   - AppAdapter     (reads on finalization)
        let height = Arc::clone(&self.config.height);
        let running = Arc::new(AtomicBool::new(true));
        let block_store: BlockStore<A::Block> = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // Step 4: Create mailbox channel and outbound relay channel
        let (mailbox_tx, mailbox_rx) = mpsc::channel(self.config.mailbox_size);
        let (relay_tx, mut relay_rx) = mpsc::unbounded::<Bytes>();

        // Step 5: Create Mailbox with relay wiring (Automaton + Relay)
        //
        // Mailbox::with_relay gives the Relay::broadcast implementation access
        // to the block store (for digest→block lookup) and the relay_tx channel
        // (for forwarding encoded PayloadRelayMessages to the outbound task).
        let mailbox =
            Mailbox::<A::Block>::with_relay(mailbox_tx, Arc::clone(&block_store), relay_tx);

        // Step 6: Spawn MailboxActor using commonware spawn API
        let actor = MailboxActor::new(
            mailbox_rx,
            Arc::clone(&height),
            Arc::clone(&self.app),
            Arc::clone(&block_store),
        );
        let height_for_actor = Arc::clone(&height);
        let _actor_handle = self.context.clone().spawn(|_ctx| async move {
            actor.run().await;
            tracing::info!(
                "MailboxActor completed, final height: {}",
                height_for_actor.load(Ordering::SeqCst)
            );
        });

        // Step 7: Spawn outbound payload relay forwarder
        //
        // Reads encoded PayloadRelayMessages from relay_rx and sends them to
        // all peers via the vendor PAYLOAD sender.  Runs until relay_rx is
        // closed (which happens when all Mailbox clones are dropped).
        tokio::spawn(async move {
            while let Some(wire) = relay_rx.next().await {
                use commonware_p2p::Sender as _;
                if let Err(e) = payload_sender
                    .send(
                        commonware_p2p::Recipients::All,
                        wire,
                        false, // not priority — payload relay is best-effort
                    )
                    .await
                {
                    tracing::warn!(error = %e, "outbound payload relay send failed");
                }
            }
            tracing::debug!("outbound payload relay task exited");
        });

        // Step 8: Spawn inbound payload receiver loop
        //
        // Wraps the raw vendor receiver in a CommonwareReceiver adapter and
        // feeds it to payload_receive_loop which decodes, validates, and
        // stores inbound block payloads in the shared block store.
        let payload_cw_receiver = CommonwareReceiver::new(Channel::PAYLOAD, payload_receiver);
        let block_store_for_receiver = Arc::clone(&block_store);
        tokio::spawn(async move {
            payload_receive_loop(payload_cw_receiver, block_store_for_receiver).await;
            tracing::debug!("inbound payload receiver task exited");
        });

        match self.config.signing_scheme.clone() {
            SigningSchemeConfig::Ed25519 { signer, validators } => {
                // Step 9: Create AppAdapter (Reporter) using the caller-provided sink
                let reporter = AppAdapter::new(
                    Arc::clone(&self.app),
                    Arc::clone(&self.sink),
                    Arc::clone(&block_store),
                );

                // Step 10: Create ed25519 Scheme from signer and validators
                let participants = Set::from_iter_dedup(validators);
                let scheme = simplex::scheme::ed25519::Scheme::signer(
                    self.config.namespace.as_bytes(),
                    participants.clone(),
                    signer,
                )
                .ok_or_else(|| ConsensusError::Other("signer not in validator set".into()))?;

                // Step 11: Build simplex::Config
                let simplex_config = simplex::Config {
                    scheme,
                    elector: RoundRobin::<Sha256>::default(),
                    blocker: oracle,
                    automaton: mailbox.clone(),
                    relay: mailbox,
                    reporter,
                    strategy: Sequential,
                    partition: self.config.namespace.clone(),
                    mailbox_size: self.config.mailbox_size,
                    epoch: Epoch::new(self.config.epoch),
                    replay_buffer: self.config.replay_buffer,
                    write_buffer: self.config.write_buffer,
                    page_cache: CacheRef::from_pooler(
                        &self.context,
                        NonZeroU16::new(4096).unwrap(),  // page_size
                        NonZeroUsize::new(100).unwrap(), // capacity
                    ),
                    leader_timeout: self.config.leader_timeout,
                    certification_timeout: self.config.notarization_timeout,
                    timeout_retry: self.config.nullify_retry,
                    activity_timeout: ViewDelta::new(self.config.activity_timeout),
                    skip_timeout: ViewDelta::new(self.config.skip_timeout),
                    fetch_timeout: self.config.fetch_timeout,
                    fetch_concurrent: self.config.fetch_concurrent,
                    forwarding: simplex::ForwardingPolicy::Disabled,
                };

                // Step 12: Validate config
                simplex_config.assert();

                // Step 13: Create vendor Engine
                let engine = simplex::Engine::new(self.context, simplex_config);

                // Step 14: Start vendor engine with three channel pairs (vote/cert/resolver only)
                let vendor_handle =
                    engine.start(per_channel.vote, per_channel.cert, per_channel.resolver);

                // Step 15: Convert vendor Handle to tokio JoinHandle
                let join_handle: JoinHandle<Result<(), ConsensusError>> =
                    tokio::task::spawn(async move {
                        let _ = vendor_handle.await;
                        Ok(())
                    });

                // Step 16: Create shutdown function
                let running_for_shutdown = Arc::clone(&running);
                let stop_fn = Box::new(move || {
                    running_for_shutdown.store(false, Ordering::SeqCst);
                    tracing::info!("Shutdown signal sent to consensus engine");
                }) as Box<dyn FnOnce() + Send>;

                // Step 17: Return RunningEngine
                Ok(RunningEngine::new(stop_fn, join_handle, height, running))
            }
            SigningSchemeConfig::BlsThresholdVrf {
                participants,
                polynomial,
                share,
            } => {
                // Step 9: Create AppAdapter (Reporter) using the caller-provided sink
                let reporter = AppAdapter::new(
                    Arc::clone(&self.app),
                    Arc::clone(&self.sink),
                    Arc::clone(&block_store),
                );

                // Step 10: Create BLS threshold VRF scheme from participants + share
                let participants = Set::from_iter_dedup(participants);
                let scheme = BlsThresholdVrfScheme::signer(
                    self.config.namespace.as_bytes(),
                    participants,
                    polynomial,
                    share,
                )
                .ok_or_else(|| {
                    ConsensusError::Other(
                        "threshold share does not match BLS participant configuration".into(),
                    )
                })?;

                // Step 11: Build simplex::Config
                let simplex_config = simplex::Config {
                    scheme,
                    elector: RoundRobin::<Sha256>::default(),
                    blocker: oracle,
                    automaton: mailbox.clone(),
                    relay: mailbox,
                    reporter,
                    strategy: Sequential,
                    partition: self.config.namespace.clone(),
                    mailbox_size: self.config.mailbox_size,
                    epoch: Epoch::new(self.config.epoch),
                    replay_buffer: self.config.replay_buffer,
                    write_buffer: self.config.write_buffer,
                    page_cache: CacheRef::from_pooler(
                        &self.context,
                        NonZeroU16::new(4096).unwrap(),  // page_size
                        NonZeroUsize::new(100).unwrap(), // capacity
                    ),
                    leader_timeout: self.config.leader_timeout,
                    certification_timeout: self.config.notarization_timeout,
                    timeout_retry: self.config.nullify_retry,
                    activity_timeout: ViewDelta::new(self.config.activity_timeout),
                    skip_timeout: ViewDelta::new(self.config.skip_timeout),
                    fetch_timeout: self.config.fetch_timeout,
                    fetch_concurrent: self.config.fetch_concurrent,
                    forwarding: simplex::ForwardingPolicy::Disabled,
                };

                // Step 12: Validate config
                simplex_config.assert();

                // Step 13: Create vendor Engine
                let engine = simplex::Engine::new(self.context, simplex_config);

                // Step 14: Start vendor engine with three channel pairs (vote/cert/resolver only)
                let vendor_handle =
                    engine.start(per_channel.vote, per_channel.cert, per_channel.resolver);

                // Step 15: Convert vendor Handle to tokio JoinHandle
                let join_handle: JoinHandle<Result<(), ConsensusError>> =
                    tokio::task::spawn(async move {
                        let _ = vendor_handle.await;
                        Ok(())
                    });

                // Step 16: Create shutdown function
                let running_for_shutdown = Arc::clone(&running);
                let stop_fn = Box::new(move || {
                    running_for_shutdown.store(false, Ordering::SeqCst);
                    tracing::info!("Shutdown signal sent to consensus engine");
                }) as Box<dyn FnOnce() + Send>;

                // Step 17: Return RunningEngine
                Ok(RunningEngine::new(stop_fn, join_handle, height, running))
            }
        }
    }
}

#[cfg(test)]
mod tests;
