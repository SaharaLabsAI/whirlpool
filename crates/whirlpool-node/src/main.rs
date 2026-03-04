//! Whirlpool consensus node binary.

use commonware_cryptography::Signer;
use commonware_cryptography::ed25519;
use commonware_runtime::{tokio, Metrics, Runner};
use consensus::traits::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use p2p_commonware::CommonwareNetworkProviderBuilder;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicU64;
use std::num::NonZeroUsize;
use std::time::Duration;
use tracing::info;
use app::{ApplicationAdapter, InMemoryTxPool};
use app_evm::executor::EvmApplication;
use app_evm::traits::StateProvider;
use app_evm::{WhirlpoolEvmConfig, build_sahara_chain_spec};
use reth_revm::db::BundleState;
use state::InMemoryStateDb;
use whirlpool_node::config;

// Application namespace for network isolation
const APPLICATION_NAMESPACE: &[u8] = b"whirlpool-dev";
const MAX_MESSAGE_SIZE: u32 = 1024 * 1024; // 1 MB

#[derive(Clone, Debug)]
struct TestStateDb(InMemoryStateDb);

impl TestStateDb {
    fn new() -> Self {
        Self(InMemoryStateDb::new())
    }
}

impl StateProvider for TestStateDb {
    fn state_root(&self) -> revm::primitives::B256 {
        self.0.state_root()
    }

    fn commit(&mut self, bundle: &BundleState) {
        self.0.commit(bundle);
    }
}


impl revm::Database for TestStateDb {
    type Error = state::StateError;

    fn basic(
        &mut self,
        address: revm::primitives::Address,
    ) -> Result<Option<revm::state::AccountInfo>, Self::Error> {
        self.0.basic(address)
    }

    fn code_by_hash(
        &mut self,
        code_hash: revm::primitives::B256,
    ) -> Result<revm::state::Bytecode, Self::Error> {
        self.0.code_by_hash(code_hash)
    }

    fn storage(
        &mut self,
        address: revm::primitives::Address,
        index: revm::primitives::U256,
    ) -> Result<revm::primitives::U256, Self::Error> {
        self.0.storage(address, index)
    }

    fn block_hash(&mut self, number: u64) -> Result<revm::primitives::B256, Self::Error> {
        self.0.block_hash(number)
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Whirlpool node");

    let height = Arc::new(AtomicU64::new(0));
    let sink = Arc::new(FinalizationSink::new(Arc::clone(&height)));
    info!("Application and sink initialized");

    // Create commonware runtime and start async context
    let executor = tokio::Runner::default();

    executor.start(|context| async move {
        info!("Commonware runtime started");

        // Create ed25519 signer from deterministic seed (development only)
        let signer = ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED);
        let validators = vec![signer.public_key()]; // Single validator for development

        // Create local addresses for network setup
        let listen_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0); // OS assigns port
        let dialable_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let (network_provider, oracle_handle) = CommonwareNetworkProviderBuilder::new(signer.clone(), APPLICATION_NAMESPACE)
            .listen_addr(listen_addr)
            .dialable_addr(dialable_addr)
            .max_message_size(MAX_MESSAGE_SIZE)
            .build(context.with_label("network"));

        // Configure consensus engine config
        let engine_config = CommonwareConfig {
            namespace: String::from_utf8_lossy(config::NAMESPACE).to_string(),
            leader_timeout: Duration::from_secs(5),
            notarization_timeout: Duration::from_secs(5),
            nullify_retry: Duration::from_millis(500),
            activity_timeout: 10,
            skip_timeout: 5,
            mailbox_size: 100,
            replay_buffer: NonZeroUsize::new(100).unwrap(),
            write_buffer: NonZeroUsize::new(100).unwrap(),
            epoch: 0,
            fetch_timeout: Duration::from_secs(5),
            fetch_concurrent: 4,
            signer,
            validators,
        };

        let state_db = Arc::new(RwLock::new(TestStateDb::new()));
        let chain_spec = Arc::new(build_sahara_chain_spec());
        let evm_config = WhirlpoolEvmConfig::new(chain_spec);
        let tx_pool = Arc::new(InMemoryTxPool::new());
        let evm_app = EvmApplication::new(evm_config, state_db, tx_pool.clone());
        let app = Arc::new(ApplicationAdapter::new(evm_app));

        let engine = CommonwareEngine::new(app, sink, engine_config, network_provider, context.clone());
        let _running = engine.start().expect("failed to start consensus engine");
        info!("Consensus engine created and started successfully");

        // Keep oracle_handle alive for network health
        let _ = oracle_handle;

        // Wait indefinitely for the engine to run
        // In production, this would integrate with proper signal handling
        ::std::future::pending::<()>().await;
    });
}
