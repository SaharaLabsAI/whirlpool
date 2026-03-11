use app::traits::TxSource;
use app::ApplicationAdapter;
use app_evm::executor::EvmApplication;
use app_evm::{build_sahara_chain_spec, WhirlpoolEvmConfig};
use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use commonware_runtime::{tokio, Metrics, Runner};
use consensus::traits::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use mempool_mdbx::PersistentTxPool;
use p2p_commonware::CommonwareNetworkProviderBuilder;
use reth_chainspec::ChainSpec;
use rpc_eth as rpc;
use std::error::Error;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::info;

use crate::config::NodeConfig;
use crate::persisting_sink::PersistingFinalizationSink;
use state::BlockStorage;

pub struct NodeHandle {
    pub rpc_addr: SocketAddr,
    pub p2p_addr: SocketAddr,
    pub public_key: ed25519::PublicKey,
    thread: Option<JoinHandle<()>>,
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            drop(thread);
        }
    }
}

struct NodeInfo {
    rpc_addr: SocketAddr,
    p2p_addr: SocketAddr,
    public_key: ed25519::PublicKey,
}

type NodeResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Start a node with the default Sahara chain specification.
pub fn start_node(config: NodeConfig) -> NodeResult<NodeHandle> {
    start_node_with_chain_spec(config, None)
}

/// Start a node with an optional custom chain specification.
///
/// If `chain_spec` is `None`, the default Sahara chain spec is used.
/// This is useful for tests that need pre-funded genesis accounts.
pub fn start_node_with_chain_spec(
    config: NodeConfig,
    chain_spec: Option<Arc<ChainSpec>>,
) -> NodeResult<NodeHandle> {
    let public_key = ed25519::PrivateKey::from_seed(config.identity.seed).public_key();
    let (info_tx, info_rx) = mpsc::channel::<NodeResult<NodeInfo>>();

    let thread = thread::spawn(move || {
        let runtime_storage_dir = config.storage.runtime_dir();
        let runtime_cfg = tokio::Config::new().with_storage_directory(runtime_storage_dir);
        let executor = tokio::Runner::new(runtime_cfg);

        executor.start(|context| async move {
            info!(?config, "Commonware runtime started");

            let signer = ed25519::PrivateKey::from_seed(config.identity.seed);
            let validators = config
                .validators
                .clone()
                .unwrap_or_else(|| vec![signer.public_key()]);

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

            let db_path = config.storage.state_dir();
            info!(?db_path, "Opening persistent state database");
            let reth_db =
                state_reth::open_state_db(&db_path).expect("failed to open state database");

            // Apply genesis allocations (pre-funded accounts) to the state DB if provided.
            let genesis_alloc = chain_spec
                .as_ref()
                .map(|cs| &cs.genesis.alloc)
                .filter(|a| !a.is_empty());
            if let Some(alloc) = genesis_alloc {
                let alloc_map: std::collections::HashMap<_, _> =
                    alloc.iter().map(|(k, v)| (*k, v.clone())).collect();
                reth_db
                    .apply_genesis(&alloc_map)
                    .expect("failed to apply genesis allocations");
                info!(
                    num_accounts = alloc.len(),
                    "Applied genesis account allocations"
                );
            }

            let state_db = Arc::new(RwLock::new(reth_db.clone()));
            let block_storage = Arc::new(reth_db);

            let recovered_height = block_storage
                .get_latest_block_number()
                .expect("failed to query chain tip")
                .unwrap_or(0);
            info!(recovered_height, "Chain tip recovered from database");

            let height = Arc::new(AtomicU64::new(recovered_height));
            let inner_sink = FinalizationSink::new(Arc::clone(&height));

            let engine_config = CommonwareConfig {
                namespace: String::from_utf8_lossy(&config.consensus.namespace).to_string(),
                leader_timeout: config.consensus.block_interval,
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

            let chain_spec = chain_spec.unwrap_or_else(|| Arc::new(build_sahara_chain_spec()));
            let evm_config = WhirlpoolEvmConfig::new(chain_spec.clone());
            let mempool_path = config.storage.mempool_dir();
            info!(?mempool_path, "Opening persistent mempool database");
            let tx_pool: Arc<dyn TxSource> = Arc::new(
                PersistentTxPool::open(&mempool_path).expect("failed to open mempool database"),
            );
            let evm_app = EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone());

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

            let rpc_config = rpc::RpcConfig {
                state_db: block_storage.clone(),
                chain_spec,
                tx_source: tx_pool.clone(),
                addr: config.rpc.bind_addr,
            };
            let (_rpc_handle, rpc_addr) = rpc::start_rpc_server(rpc_config)
                .await
                .expect("failed to start RPC server");
            info!(%rpc_addr, %dialable_addr, "JSON-RPC server started");

            let _ = info_tx.send(Ok(NodeInfo {
                rpc_addr,
                p2p_addr: dialable_addr,
                public_key: public_key.clone(),
            }));

            let _ = oracle_handle;
            ::std::future::pending::<()>().await;
        });
    });

    let node_info = info_rx
        .recv()
        .map_err(|err| -> Box<dyn Error + Send + Sync> { Box::new(err) })??;

    Ok(NodeHandle {
        rpc_addr: node_info.rpc_addr,
        p2p_addr: node_info.p2p_addr,
        public_key: node_info.public_key,
        thread: Some(thread),
    })
}
