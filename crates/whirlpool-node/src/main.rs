//! Whirlpool consensus node binary.

use app::{ApplicationAdapter, InMemoryTxPool};
use app_evm::executor::EvmApplication;
use app_evm::{build_sahara_chain_spec, WhirlpoolEvmConfig, SAHARA_CHAIN_ID};
use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use commonware_runtime::{tokio, Metrics, Runner};
use consensus::traits::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use p2p_commonware::CommonwareNetworkProviderBuilder;
use rpc_eth as rpc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::info;
use state::BlockStorage;
use whirlpool_node::config;
use whirlpool_node::persisting_sink::PersistingFinalizationSink;

// Application namespace for network isolation
const APPLICATION_NAMESPACE: &[u8] = b"whirlpool-dev";
const MAX_MESSAGE_SIZE: u32 = 1024 * 1024; // 1 MB

/// Default path for the state database.
const DEFAULT_DB_PATH: &str = "data/state";

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Whirlpool node");

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

        let (network_provider, oracle_handle) =
            CommonwareNetworkProviderBuilder::new(signer.clone(), APPLICATION_NAMESPACE)
                .listen_addr(listen_addr)
                .dialable_addr(dialable_addr)
                .max_message_size(MAX_MESSAGE_SIZE)
                .build(context.with_label("network"));

        // Initialize state database
        let db_path = PathBuf::from(DEFAULT_DB_PATH);
        info!(?db_path, "Opening persistent state database");
        let reth_db =
            state_reth::open_state_db(&db_path).expect("failed to open state database");
        let state_db = Arc::new(RwLock::new(reth_db.clone()));
        let block_storage = Arc::new(reth_db);

        // Recover chain tip from persistent storage
        let recovered_height = block_storage
            .get_latest_block_number()
            .expect("failed to query chain tip")
            .unwrap_or(0);
        info!(recovered_height, "Chain tip recovered from database");

        let height = Arc::new(AtomicU64::new(recovered_height));
        let inner_sink = FinalizationSink::new(Arc::clone(&height));

        // Configure consensus engine config
        let engine_config = CommonwareConfig {
            namespace: String::from_utf8_lossy(config::NAMESPACE).to_string(),
            leader_timeout: Duration::from_secs(5),
            notarization_timeout: Duration::from_secs(5),
            nullify_retry: Duration::from_millis(500),
            activity_timeout: 10,
            skip_timeout: 5,
            mailbox_size: 100,
            replay_buffer: NonZeroUsize::new(1024 * 1024).unwrap(),
            write_buffer: NonZeroUsize::new(1024 * 1024).unwrap(),
            epoch: 0,
            initial_height: recovered_height,
            fetch_timeout: Duration::from_secs(5),
            fetch_concurrent: 4,
            signer,
            validators,
        };

        let chain_spec = Arc::new(build_sahara_chain_spec());
        let evm_config = WhirlpoolEvmConfig::new(chain_spec);
        let tx_pool = Arc::new(InMemoryTxPool::new());
        let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());

        // Wrap the finalization sink to persist blocks on finalization
        let sink = Arc::new(PersistingFinalizationSink::new(
            inner_sink,
            evm_app.clone(),
            block_storage.clone(),
        ));

        let app = Arc::new(ApplicationAdapter::new(evm_app));

        let engine =
            CommonwareEngine::new(app, sink, engine_config, network_provider, context.clone());
        let _running = engine.start().expect("failed to start consensus engine");
        info!("Consensus engine created and started successfully");

        // Start JSON-RPC server (share the height Arc for block number resolution)
        let mut rpc_ctx =
            rpc::context::EthRpcContext::new(tx_pool, state_db, block_storage, SAHARA_CHAIN_ID);
        rpc_ctx.block_height = height;
        let rpc_addr: SocketAddr = config::RPC_BIND_ADDR
            .parse()
            .expect("invalid RPC bind address");
        let _rpc_handle = rpc::server::start_rpc_server(rpc_ctx, rpc_addr)
            .await
            .expect("failed to start RPC server");
        info!("JSON-RPC server started");

        // Keep oracle_handle alive for network health
        let _ = oracle_handle;

        // Wait indefinitely for the engine to run
        // In production, this would integrate with proper signal handling
        ::std::future::pending::<()>().await;
    });
}
