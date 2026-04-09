use alloy_primitives::Address;
use app::traits::TxSource;
use app::ApplicationAdapter;
use app_evm::{EvmApplication, WhirlpoolEvmConfig};
use chainspec::{
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators,
    try_simplex_validators_from_chain_spec, validate_genesis_alloc,
};
use commonware_codec::Read;
use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use commonware_runtime::{tokio, Metrics, Runner};
use consensus::traits::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use mempool_mdbx::PersistentTxPool;
use p2p_commonware::CommonwareNetworkProviderBuilder;
use reth_chainspec::ChainSpec;
use rpc_eth as eth_rpc;
use std::error::Error;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::info;
use validators::{ordered_consensus_pubkeys, ValidatorEntry};

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

fn decode_consensus_pubkey(bytes: [u8; 32]) -> NodeResult<ed25519::PublicKey> {
    let mut reader = bytes.as_slice();
    let public_key = ed25519::PublicKey::read_cfg(&mut reader, &())
        .map_err(|err| std::io::Error::other(format!("invalid simplex validator key: {err}")))?;
    if !reader.is_empty() {
        return Err(Box::new(std::io::Error::other(
            "invalid simplex validator key length",
        )));
    }
    Ok(public_key)
}

fn simplex_validators_from_chain_spec(
    chain_spec: &ChainSpec,
) -> NodeResult<Vec<ed25519::PublicKey>> {
    let entries = try_simplex_validators_from_chain_spec(chain_spec).map_err(|err| {
        Box::new(std::io::Error::other(format!(
            "failed to decode simplex validators registry: {err}"
        ))) as Box<dyn Error + Send + Sync>
    })?;

    ordered_consensus_pubkeys(&entries)
        .into_iter()
        .map(decode_consensus_pubkey)
        .collect()
}

fn resolve_validator_sets(
    config: &NodeConfig,
    local_signer: ed25519::PublicKey,
    genesis_simplex_validators: &[ed25519::PublicKey],
) -> NodeResult<(Vec<ed25519::PublicKey>, Vec<ed25519::PublicKey>)> {
    let bootstrap_validators = config
        .bootstrap_validators
        .clone()
        .unwrap_or_else(|| vec![local_signer.clone()]);
    if genesis_simplex_validators.is_empty() {
        return Err(Box::new(std::io::Error::other(
            "genesis-backed simplex validator registry is empty",
        )));
    }

    Ok((bootstrap_validators, genesis_simplex_validators.to_vec()))
}

fn ensure_signer_is_simplex_member(
    local_signer: &ed25519::PublicKey,
    simplex_validators: &[ed25519::PublicKey],
) -> NodeResult<()> {
    if simplex_validators
        .iter()
        .any(|validator| validator == local_signer)
    {
        return Ok(());
    }

    Err(Box::new(std::io::Error::other(
        "local signer is not present in the genesis-backed simplex validator set",
    )))
}

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
    let chain_spec = match chain_spec {
        Some(chain_spec) => {
            validate_genesis_alloc(&chain_spec.genesis.alloc)?;
            chain_spec
        }
        None => {
            let signer = ed25519::PrivateKey::from_seed(config.identity.seed);
            let validator_entry = ValidatorEntry {
                consensus_pubkey: signer
                    .public_key()
                    .as_ref()
                    .try_into()
                    .expect("ed25519 key length"),
                ethereum_address: Address::ZERO,
            };
            Arc::new(
                build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(
                    std::collections::BTreeMap::new(),
                    std::collections::BTreeMap::new(),
                    vec![validator_entry],
                ),
            )
        }
    };
    let genesis_simplex_validators = simplex_validators_from_chain_spec(&chain_spec)?;
    let (info_tx, info_rx) = mpsc::channel::<NodeResult<NodeInfo>>();

    let thread = thread::spawn(move || {
        let runtime_storage_dir = config.storage.runtime_dir();
        let runtime_cfg = tokio::Config::new().with_storage_directory(runtime_storage_dir);
        let executor = tokio::Runner::new(runtime_cfg);

        executor.start(|context| async move {
            info!(?config, "Commonware runtime started");

            let signer = ed25519::PrivateKey::from_seed(config.identity.seed);
            let signer_public_key = signer.public_key();
            let (bootstrap_validators, simplex_validators) = match resolve_validator_sets(
                &config,
                signer_public_key.clone(),
                &genesis_simplex_validators,
            ) {
                Ok(sets) => sets,
                Err(err) => {
                    let _ = info_tx.send(Err(err));
                    return;
                }
            };
            if let Err(err) =
                ensure_signer_is_simplex_member(&signer_public_key, &simplex_validators)
            {
                let _ = info_tx.send(Err(err));
                return;
            }

            let listen_addr = config.network.listen_addr;
            let dialable_addr = config.network.dialable_addr;
            let bootstrappers = config.network.bootstrap_peers.clone();

            let (network_provider, oracle_handle) = CommonwareNetworkProviderBuilder::new(
                signer.clone(),
                config.network.namespace.clone(),
            )
            .listen_addr(listen_addr)
            .dialable_addr(dialable_addr)
            .max_message_size(config.network.max_message_size)
            .initial_validators(0, bootstrap_validators.clone())
            .bootstrappers(bootstrappers)
            .build(context.with_label("network"))
            .await;

            let db_path = config.storage.state_dir();
            info!(?db_path, "Opening persistent state database");
            let reth_db =
                state_reth::open_state_db(&db_path).expect("failed to open state database");

            // Apply genesis allocations (pre-funded accounts) to the state DB if provided.
            let genesis_alloc =
                (!chain_spec.genesis.alloc.is_empty()).then_some(&chain_spec.genesis.alloc);
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
                validators: simplex_validators,
            };

            let mut proposer_public_key = [0u8; 32];
            proposer_public_key.copy_from_slice(public_key.as_ref());
            let evm_config = WhirlpoolEvmConfig::new(chain_spec.clone())
                .with_local_proposer_public_key(proposer_public_key);
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

            let eth_rpc_config = eth_rpc::RpcConfig {
                state_db: block_storage.clone(),
                chain_spec: chain_spec.clone(),
                tx_source: tx_pool.clone(),
                addr: config.rpc.bind_addr,
            };
            let (_eth_rpc_handle, rpc_addr) = eth_rpc::start_rpc_server(eth_rpc_config)
                .await
                .expect("failed to start Ethereum RPC server");
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

#[cfg(test)]
mod tests {
    use super::{ensure_signer_is_simplex_member, resolve_validator_sets};
    use crate::config::NodeConfig;
    use commonware_cryptography::ed25519;
    use commonware_cryptography::Signer;

    #[test]
    fn node_uses_genesis_registry_for_simplex_validators() {
        let local_signer = ed25519::PrivateKey::from_seed(1).public_key();
        let toml_bootstrap = vec![
            ed25519::PrivateKey::from_seed(9).public_key(),
            ed25519::PrivateKey::from_seed(10).public_key(),
        ];
        let genesis_simplex = vec![
            local_signer.clone(),
            ed25519::PrivateKey::from_seed(2).public_key(),
        ];
        let config = NodeConfig {
            bootstrap_validators: Some(toml_bootstrap),
            ..NodeConfig::default()
        };

        let (_bootstrap, simplex) = resolve_validator_sets(&config, local_signer, &genesis_simplex)
            .expect("validator sets should resolve");

        assert_eq!(simplex, genesis_simplex);
    }

    #[test]
    fn node_keeps_toml_validators_for_p2p_bootstrap_only() {
        let local_signer = ed25519::PrivateKey::from_seed(3).public_key();
        let bootstrap_subset = vec![local_signer.clone()];
        let genesis_simplex = vec![
            local_signer.clone(),
            ed25519::PrivateKey::from_seed(4).public_key(),
            ed25519::PrivateKey::from_seed(5).public_key(),
        ];
        let config = NodeConfig {
            bootstrap_validators: Some(bootstrap_subset.clone()),
            ..NodeConfig::default()
        };

        let (bootstrap, simplex) = resolve_validator_sets(&config, local_signer, &genesis_simplex)
            .expect("validator sets should resolve");

        assert_eq!(bootstrap, bootstrap_subset);
        assert_eq!(simplex, genesis_simplex);
    }

    #[test]
    fn node_rejects_empty_genesis_registry_for_simplex() {
        let local_signer = ed25519::PrivateKey::from_seed(6).public_key();
        let bootstrap_set = vec![
            local_signer.clone(),
            ed25519::PrivateKey::from_seed(7).public_key(),
        ];
        let config = NodeConfig {
            bootstrap_validators: Some(bootstrap_set.clone()),
            ..NodeConfig::default()
        };

        let err = resolve_validator_sets(&config, local_signer, &[])
            .expect_err("empty genesis registry should fail");
        assert!(
            err.to_string().contains("registry is empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn node_startup_fails_when_signer_missing_from_genesis_registry() {
        let local_signer = ed25519::PrivateKey::from_seed(8).public_key();
        let simplex_validators = vec![
            ed25519::PrivateKey::from_seed(11).public_key(),
            ed25519::PrivateKey::from_seed(12).public_key(),
        ];

        let err = ensure_signer_is_simplex_member(&local_signer, &simplex_validators)
            .expect_err("signer must be in simplex validator set");
        assert!(
            err.to_string().contains("local signer is not present"),
            "unexpected error: {err}"
        );
    }
}
