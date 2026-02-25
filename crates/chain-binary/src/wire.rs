//! Wiring for chain-binary: starter closure that launches the consensus stack.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use commonware_consensus::simplex::{self, elector::RoundRobin};
use commonware_consensus::types::{Epoch, ViewDelta};
use commonware_cryptography::{ed25519, Sha256, Signer as _};
use commonware_p2p::simulated;
use commonware_parallel::Sequential;
use commonware_runtime::{buffer::PoolRef, tokio, Metrics as _, Quota, Runner as _};
use commonware_utils::{ordered::Set, TryCollect as _, NZU16, NZU32, NZUsize};
use consensus_commonware::adapter::AppAdapter;
use consensus_core::error::ConsensusError;
use futures::channel::mpsc;

use crate::app::EmptyBlockApp;
use crate::config;
use crate::mailbox::{Mailbox, MailboxActor};
use crate::sink::FinalizationSink;

/// Creates the starter closure for `CommonwareEngine`.
///
/// The starter receives shared height and running atomics, wires all components
/// (signer, scheme, mailbox, adapter, simplex engine, runner), and returns
/// a shutdown function plus a thread handle.
pub fn create_starter(
) -> impl FnOnce(
    Arc<AtomicU64>,
    Arc<AtomicBool>,
) -> Result<
    (
        Box<dyn FnOnce() + Send>,
        tokio::task::JoinHandle<Result<(), ConsensusError>>,
    ),
    ConsensusError,
> + Send
       + 'static {
    move |height: Arc<AtomicU64>, running: Arc<AtomicBool>| {
        // 1. Signer & Validators
        let signer = ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED);
        let validators: Set<_> = vec![signer.public_key()]
            .into_iter()
            .try_collect()
            .map_err(|e| ConsensusError::Other(format!("Failed to create validator set: {}", e)))?;

        // 2. Scheme
        let scheme = simplex::Scheme::signer(config::NAMESPACE, validators.clone(), signer.clone())
            .map_err(|e| ConsensusError::Other(format!("Failed to create scheme: {}", e)))?;

        // 3. Mailbox channels
        const MAILBOX_BUFFER: usize = 256;
        let (sender, receiver) = mpsc::channel(MAILBOX_BUFFER);
        let mailbox = Mailbox::new(sender);
        let mailbox_actor = MailboxActor::new(receiver, Arc::clone(&height));

        // 4. AppAdapter
        let app = EmptyBlockApp::new();
        let sink = FinalizationSink::new(Arc::clone(&height));
        let adapter = AppAdapter::new(app, sink);

        // 5. Runtime configuration
        let storage_dir = "/tmp/chain-binary";
        let runtime_cfg = tokio::Config::new().with_storage_directory(storage_dir);
        let executor = tokio::Runner::new(runtime_cfg);

        // 6. Spawn runner in dedicated OS thread
        let running_clone = Arc::clone(&running);
        let handle = std::thread::spawn(move || {
            executor.start(|context| async move {
                // Initialize simulated network (single node - no real P2P)
                let (network, _oracle) = simulated::Network::new(
                    context.with_label("network"),
                    simulated::Config {
                        max_size: 1024 * 1024, // 1MB max message size
                    },
                );
                network.start();

                // Register consensus channels (vote, certificate, resolver)
                let (vote_sender, vote_receiver) = network.register(
                    0,
                    Quota::per_second(NZU32!(10)),
                    256, // 256 messages in flight
                );
                let (certificate_sender, certificate_receiver) = network.register(
                    1,
                    Quota::per_second(NZU32!(10)),
                    256,
                );
                let (resolver_sender, resolver_receiver) = network.register(
                    2,
                    Quota::per_second(NZU32!(10)),
                    256,
                );

                // Start mailbox actor
                context.spawn(|ctx| async move {
                    mailbox_actor.run(ctx).await;
                });

                // Reporter placeholder (no-op for now)
                let reporter = simplex::mocks::reporter::Reporter::new(
                    simplex::mocks::reporter::Config {
                        scheme: scheme.clone(),
                        elector: RoundRobin::<Sha256>::default(),
                        activity_timeout: ViewDelta::new(10),
                        skip_timeout: ViewDelta::new(5),
                    },
                );

                // Simplex engine configuration
                let simplex_cfg = simplex::Config {
                    scheme: scheme.clone(),
                    elector: RoundRobin::<Sha256>::default(),
                    blocker: network.oracle(),
                    automaton: mailbox.clone(),
                    relay: mailbox.clone(),
                    reporter: reporter.clone(),
                    partition: String::from("chain-binary"),
                    mailbox_size: 1024,
                    epoch: Epoch::zero(),
                    replay_buffer: NZUsize!(1024 * 1024),
                    write_buffer: NZUsize!(1024 * 1024),
                    leader_timeout: config::BLOCK_INTERVAL,
                    notarization_timeout: Duration::from_secs(2),
                    nullify_retry: Duration::from_secs(10),
                    fetch_timeout: Duration::from_secs(1),
                    activity_timeout: ViewDelta::new(10),
                    skip_timeout: ViewDelta::new(5),
                    fetch_concurrent: 32,
                    buffer_pool: PoolRef::new(NZU16!(16_384), NZUsize!(10_000)),
                    strategy: Sequential,
                };

                // Start simplex engine
                let engine = simplex::Engine::new(context.with_label("engine"), simplex_cfg);
                engine.start(
                    (vote_sender, vote_receiver),
                    (certificate_sender, certificate_receiver),
                    (resolver_sender, resolver_receiver),
                );

                // Wait for shutdown signal
                while running_clone.load(Ordering::SeqCst) {
                    context.sleep(Duration::from_millis(100)).await;
                }

                Ok(())
            })
        });

        // Wrap thread handle in tokio JoinHandle
        let join_handle = tokio::task::spawn_blocking(move || {
            handle
                .join()
                .map_err(|e| ConsensusError::Other(format!("Thread panicked: {:?}", e)))?
        });

        // 7. Shutdown function
        let stop_fn = Box::new(move || {
            running.store(false, Ordering::SeqCst);
        }) as Box<dyn FnOnce() + Send>;

        Ok((stop_fn, join_handle))
    }
}
