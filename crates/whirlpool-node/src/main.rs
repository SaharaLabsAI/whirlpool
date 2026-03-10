//! Whirlpool consensus node binary.

use app::traits::TxSource;
use app::ApplicationAdapter;
use app_evm::executor::EvmApplication;
use app_evm::{build_sahara_chain_spec, WhirlpoolEvmConfig, SAHARA_CHAIN_ID};
use clap::Parser;
use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use commonware_runtime::{tokio, Metrics, Runner};
use consensus::traits::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use mempool_mdbx::PersistentTxPool;
use p2p_commonware::CommonwareNetworkProviderBuilder;
use rpc_eth as rpc;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::info;
use state::BlockStorage;
use whirlpool_node::config::{NodeArgs, NodeConfig};
use whirlpool_node::persisting_sink::PersistingFinalizationSink;

fn main() {
    let args = NodeArgs::parse();
    let config = NodeConfig::from(args);
    let runtime_storage_dir = config.storage.runtime_dir();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!(?config, "Starting Whirlpool node");

    // Create commonware runtime with persistent storage so the consensus
    // journal survives restarts (the default uses a random temp directory).
    let runtime_cfg = tokio::Config::new().with_storage_directory(runtime_storage_dir);
    let executor = tokio::Runner::new(runtime_cfg);

    executor.start(|context| async move {
        info!(?config, "Commonware runtime started");

        // Create ed25519 signer from deterministic seed (development only)
        let signer = ed25519::PrivateKey::from_seed(config.identity.seed);
        let validators = vec![signer.public_key()]; // Single validator for development

        // Create local addresses for network setup
        let listen_addr = config.network.listen_addr;
        let dialable_addr = config.network.dialable_addr;
        let bootstrappers = config.network.bootstrap_peers.clone();

        let (network_provider, oracle_handle) =
            CommonwareNetworkProviderBuilder::new(signer.clone(), config.network.namespace.clone())
                .listen_addr(listen_addr)
                .dialable_addr(dialable_addr)
                .max_message_size(config.network.max_message_size)
                .initial_validators(0, validators.clone())
                .bootstrappers(bootstrappers)
                .build(context.with_label("network"))
                .await;

        // Initialize state database
        let db_path = config.storage.state_dir();
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
            namespace: String::from_utf8_lossy(&config.consensus.namespace).to_string(),
            leader_timeout: Duration::from_secs(5),
            notarization_timeout: Duration::from_secs(5),
            nullify_retry: Duration::from_millis(500),
            activity_timeout: 10,
            skip_timeout: 5,
            mailbox_size: 100,
            replay_buffer: NonZeroUsize::new(1024 * 1024).unwrap(),
            write_buffer: NonZeroUsize::new(1024 * 1024).unwrap(),
            epoch: 0,
            height: Arc::clone(&height),
            fetch_timeout: Duration::from_secs(5),
            fetch_concurrent: 4,
            signer,
            validators,
        };

        let chain_spec = Arc::new(build_sahara_chain_spec());
        let evm_config = WhirlpoolEvmConfig::new(chain_spec);
        let mempool_path = config.storage.mempool_dir();
        info!(?mempool_path, "Opening persistent mempool database");
        let tx_pool: Arc<dyn TxSource> = Arc::new(
            PersistentTxPool::open(&mempool_path).expect("failed to open mempool database"),
        );
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
        let _rpc_handle = rpc::server::start_rpc_server(rpc_ctx, config.rpc.bind_addr)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use whirlpool_node::config;

    #[test]
    fn tst_req2_002_node_startup_wiring_populates_builder_bootstrappers_and_validators() {
        // Verify node wiring can pass validators and bootstrappers in sequence.
        let signer = ed25519::PrivateKey::from_seed(config::VALIDATOR_SEED);
        let validators = vec![signer.public_key()];
        let bootstrappers = vec![];

        let builder = CommonwareNetworkProviderBuilder::new(
            signer.clone(),
            config::APPLICATION_NAMESPACE,
        )
        .listen_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .dialable_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .max_message_size(config::DEFAULT_MAX_MESSAGE_SIZE)
        .initial_validators(0, validators.clone())
        .bootstrappers(bootstrappers.clone());

        drop(builder);
    }
}
