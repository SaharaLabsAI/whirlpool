use alloy_primitives::Address;
use app::traits::TxSource;
use app::{ApplicationAdapter, FullDkgOutputV1};
use app_evm_execution::{EvmApplication, WhirlpoolEvmConfig};
use chainspec::{
    build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators,
    try_simplex_validators_from_chain_spec, validate_genesis_alloc,
};
use commonware_codec::{Encode, Read};
use commonware_cryptography::ed25519;
use commonware_cryptography::Signer;
use commonware_runtime::{tokio, Metrics, Runner};
use consensus::traits::ConsensusEngine;
use consensus_manager::{
    load_local_bundle, run_trusted_dealer_bootstrap, LoadLocalBundleConfig, LocalBundleMaterial,
    TrustedDealerBootstrapConfig,
};
use consensus_simplex::{
    CommonwareConfig, CommonwareEngine, FinalizationSink, SigningSchemeConfig,
};
use mempool_mdbx::PersistentTxPool;
use network_commonware::CommonwareNetworkProviderBuilder;
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

fn bootstrap_session_id() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn bootstrap_participants(config: &NodeConfig) -> NodeResult<Vec<ed25519::PublicKey>> {
    if let Some(validators) = config.bootstrap_validators.clone() {
        if let Some(expected_count) = config.bootstrap.genesis_bootstrap_validator_count {
            if validators.len() != expected_count as usize {
                return Err(Box::new(std::io::Error::other(format!(
                    "genesis bootstrap validator count mismatch: expected {expected_count}, got {}",
                    validators.len()
                ))));
            }
        }
        return Ok(validators);
    }

    Err(Box::new(std::io::Error::other(
        "genesis bootstrap requires explicit --validator values; validator-count can only validate explicit input",
    )))
}

/// Run trusted-dealer genesis bootstrap and write validator bundles.
pub fn run_genesis_bootstrap(config: &NodeConfig) -> NodeResult<()> {
    let participants = bootstrap_participants(config)?;
    let output_root = config
        .bootstrap
        .genesis_dkg_session_dir
        .clone()
        .unwrap_or_else(|| config.storage.data_dir.join("bootstrap"));

    let result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
        session_id: bootstrap_session_id(),
        output_dir: output_root,
        participants,
    })
    .map_err(|err| {
        Box::new(std::io::Error::other(format!(
            "trusted-dealer bootstrap failed: {err}"
        ))) as Box<dyn Error + Send + Sync>
    })?;

    info!(
        session_dir = %result.session_dir.display(),
        manifest = %result.manifest_path.display(),
        dealer_pubkey = %commonware_utils::hex(result.dealer_public_key.as_ref()),
        bundles = result.bundle_paths.len(),
        "genesis bootstrap completed"
    );
    Ok(())
}

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

fn consensus_pubkey_to_bytes(key: &ed25519::PublicKey) -> [u8; 32] {
    key.as_ref()
        .try_into()
        .expect("ed25519 public key length should be 32 bytes")
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

fn load_bls_material_for_signer(
    config: &NodeConfig,
    signer_public_key: &ed25519::PublicKey,
    simplex_validators: &[ed25519::PublicKey],
) -> NodeResult<Option<LocalBundleMaterial>> {
    let session_dir = config.bootstrap.genesis_dkg_session_dir.clone();
    let expected_dealer = config.bootstrap.genesis_dkg_dealer_pubkey.clone();

    let Some(session_dir) = session_dir else {
        if expected_dealer.is_some() {
            return Err(Box::new(std::io::Error::other(
                "--genesis-dkg-dealer-pubkey requires --genesis-dkg-session-dir",
            )));
        }
        return Ok(None);
    };

    let Some(expected_dealer) = expected_dealer else {
        return Err(Box::new(std::io::Error::other(
            "BLS session directory requires --genesis-dkg-dealer-pubkey for trusted manifest verification",
        )));
    };

    let material = load_local_bundle(LoadLocalBundleConfig {
        session_dir,
        local_validator: signer_public_key.clone(),
        expected_dealer,
    })
    .map_err(|err| {
        Box::new(std::io::Error::other(format!(
            "failed to load BLS bundle material: {err}"
        ))) as Box<dyn Error + Send + Sync>
    })?;

    if material.participants != simplex_validators {
        return Err(Box::new(std::io::Error::other(
            "BLS bundle participant list does not match genesis validator registry",
        )));
    }

    Ok(Some(material))
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
            info!(
                rpc_addr = %config.rpc.bind_addr,
                p2p_listen_addr = %config.network.listen_addr,
                bootstrap_mode = config.bootstrap.genesis_bootstrap_dkg,
                has_bls_session_dir = config.bootstrap.genesis_dkg_session_dir.is_some(),
                "Commonware runtime started"
            );

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

            let bls_material = match load_bls_material_for_signer(
                &config,
                &signer_public_key,
                &simplex_validators,
            ) {
                Ok(material) => material,
                Err(err) => {
                    let _ = info_tx.send(Err(err));
                    return;
                }
            };
            let full_dkg_output = bls_material.as_ref().map(|material| FullDkgOutputV1 {
                dealers: material
                    .dealers
                    .iter()
                    .map(consensus_pubkey_to_bytes)
                    .collect(),
                players: material
                    .participants
                    .iter()
                    .map(consensus_pubkey_to_bytes)
                    .collect(),
                public_polynomial: material.polynomial.encode().to_vec(),
            });

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
                app_evm_state::open_state_db(&db_path).expect("failed to open state database");

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
                signing_scheme: match bls_material.as_ref() {
                    Some(material) => SigningSchemeConfig::BlsThresholdVrf {
                        participants: material.participants.clone(),
                        polynomial: material.polynomial.clone(),
                        share: material.share.clone(),
                    },
                    None => SigningSchemeConfig::Ed25519 {
                        signer,
                        validators: simplex_validators,
                    },
                },
            };

            let mut proposer_public_key = [0u8; 32];
            proposer_public_key.copy_from_slice(public_key.as_ref());
            let mut evm_config = WhirlpoolEvmConfig::new(chain_spec.clone())
                .with_local_proposer_public_key(proposer_public_key)
                .with_full_dkg_strict_height(config.consensus.full_dkg_strict_height);
            if let Some(full_dkg_output) = full_dkg_output {
                evm_config = evm_config.with_current_full_dkg_output(full_dkg_output);
            }
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
    use super::{
        bootstrap_participants, ensure_signer_is_simplex_member, load_bls_material_for_signer,
        resolve_validator_sets,
    };
    use crate::config::NodeConfig;
    use commonware_cryptography::ed25519;
    use commonware_cryptography::Signer;
    use consensus_manager::{run_trusted_dealer_bootstrap, TrustedDealerBootstrapConfig};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::{fs, path::PathBuf};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(1);

    fn temp_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("whirlpool-node-{label}-{id}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

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

    #[test]
    fn bootstrap_participants_generates_count_when_validators_missing() {
        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_bootstrap_validator_count: Some(3),
                ..Default::default()
            },
            ..NodeConfig::default()
        };

        let err = bootstrap_participants(&config)
            .expect_err("count-only bootstrap without explicit validators must fail");
        assert!(err
            .to_string()
            .contains("requires explicit --validator values"));
    }

    #[test]
    fn bootstrap_participants_prefers_explicit_validators() {
        let explicit = vec![
            ed25519::PrivateKey::from_seed(21).public_key(),
            ed25519::PrivateKey::from_seed(22).public_key(),
        ];
        let config = NodeConfig {
            bootstrap_validators: Some(explicit.clone()),
            bootstrap: crate::config::BootstrapConfig {
                genesis_bootstrap_validator_count: Some(2),
                ..Default::default()
            },
            ..NodeConfig::default()
        };

        let participants = bootstrap_participants(&config).expect("explicit validators should win");
        assert_eq!(participants, explicit);
    }

    #[test]
    fn bootstrap_participants_rejects_count_mismatch() {
        let explicit = vec![
            ed25519::PrivateKey::from_seed(31).public_key(),
            ed25519::PrivateKey::from_seed(32).public_key(),
        ];
        let config = NodeConfig {
            bootstrap_validators: Some(explicit),
            bootstrap: crate::config::BootstrapConfig {
                genesis_bootstrap_validator_count: Some(3),
                ..Default::default()
            },
            ..NodeConfig::default()
        };

        let err =
            bootstrap_participants(&config).expect_err("validator-count mismatch must be rejected");
        assert!(err.to_string().contains("count mismatch"));
    }

    #[test]
    fn bls_material_load_rejects_missing_bundle() {
        let participants = vec![
            ed25519::PrivateKey::from_seed(41).public_key(),
            ed25519::PrivateKey::from_seed(42).public_key(),
        ];
        let local = participants[0].clone();
        let session_root = temp_dir("missing-bundle");
        let bootstrap_result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
            session_id: 100,
            output_dir: session_root,
            participants: participants.clone(),
        })
        .expect("bootstrap");
        let bundle_path = bootstrap_result
            .session_dir
            .join("bundles")
            .join(format!("{}.bundle", commonware_utils::hex(local.as_ref())));
        fs::remove_file(&bundle_path).expect("remove local bundle");

        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_dkg_session_dir: Some(bootstrap_result.session_dir),
                genesis_dkg_dealer_pubkey: Some(bootstrap_result.dealer_public_key),
                ..Default::default()
            },
            ..NodeConfig::default()
        };
        let err = match load_bls_material_for_signer(&config, &local, &participants) {
            Ok(_) => panic!("missing bundle must fail"),
            Err(err) => err,
        };
        assert!(err
            .to_string()
            .contains("failed to load BLS bundle material"));
    }

    #[test]
    fn bls_material_load_requires_explicit_dealer_key() {
        let participants = vec![
            ed25519::PrivateKey::from_seed(401).public_key(),
            ed25519::PrivateKey::from_seed(402).public_key(),
        ];
        let local = participants[0].clone();
        let session_root = temp_dir("missing-dealer-key");
        let bootstrap_result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
            session_id: 120,
            output_dir: session_root,
            participants: participants.clone(),
        })
        .expect("bootstrap");

        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_dkg_session_dir: Some(bootstrap_result.session_dir),
                ..Default::default()
            },
            ..NodeConfig::default()
        };
        let err = match load_bls_material_for_signer(&config, &local, &participants) {
            Ok(_) => panic!("missing dealer key must fail"),
            Err(err) => err,
        };
        assert!(err
            .to_string()
            .contains("requires --genesis-dkg-dealer-pubkey"));
    }

    #[test]
    fn bls_material_load_rejects_dealer_key_without_session_dir() {
        let participants = vec![
            ed25519::PrivateKey::from_seed(410).public_key(),
            ed25519::PrivateKey::from_seed(411).public_key(),
        ];
        let local = participants[0].clone();
        let dealer = ed25519::PrivateKey::from_seed(499).public_key();
        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_dkg_dealer_pubkey: Some(dealer),
                ..Default::default()
            },
            ..NodeConfig::default()
        };

        let err = match load_bls_material_for_signer(&config, &local, &participants) {
            Ok(_) => panic!("dealer key without session directory must fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("requires --genesis-dkg-session-dir"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn bls_material_load_rejects_invalid_manifest_hash() {
        let participants = vec![
            ed25519::PrivateKey::from_seed(51).public_key(),
            ed25519::PrivateKey::from_seed(52).public_key(),
        ];
        let local = participants[0].clone();
        let session_root = temp_dir("invalid-manifest");
        let bootstrap_result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
            session_id: 101,
            output_dir: session_root,
            participants: participants.clone(),
        })
        .expect("bootstrap");
        fs::write(
            bootstrap_result.session_dir.join("manifest.sha256"),
            [0u8; 32],
        )
        .expect("tamper manifest hash");

        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_dkg_session_dir: Some(bootstrap_result.session_dir),
                genesis_dkg_dealer_pubkey: Some(bootstrap_result.dealer_public_key),
                ..Default::default()
            },
            ..NodeConfig::default()
        };
        let err = match load_bls_material_for_signer(&config, &local, &participants) {
            Ok(_) => panic!("invalid manifest hash must fail"),
            Err(err) => err,
        };
        assert!(err
            .to_string()
            .contains("failed to load BLS bundle material"));
    }

    #[test]
    fn bls_material_load_rejects_foreign_bundle() {
        let participants = vec![
            ed25519::PrivateKey::from_seed(61).public_key(),
            ed25519::PrivateKey::from_seed(62).public_key(),
        ];
        let local = participants[0].clone();
        let other = participants[1].clone();
        let session_root = temp_dir("foreign-bundle");
        let bootstrap_result = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
            session_id: 102,
            output_dir: session_root,
            participants: participants.clone(),
        })
        .expect("bootstrap");
        let bundles_dir = bootstrap_result.session_dir.join("bundles");
        let local_path =
            bundles_dir.join(format!("{}.bundle", commonware_utils::hex(local.as_ref())));
        let other_path =
            bundles_dir.join(format!("{}.bundle", commonware_utils::hex(other.as_ref())));
        fs::copy(&other_path, &local_path).expect("overwrite local bundle with foreign bundle");

        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_dkg_session_dir: Some(bootstrap_result.session_dir),
                genesis_dkg_dealer_pubkey: Some(bootstrap_result.dealer_public_key),
                ..Default::default()
            },
            ..NodeConfig::default()
        };
        let err = match load_bls_material_for_signer(&config, &local, &participants) {
            Ok(_) => panic!("foreign bundle must fail"),
            Err(err) => err,
        };
        assert!(err
            .to_string()
            .contains("failed to load BLS bundle material"));
    }

    #[test]
    fn bls_material_load_rejects_stale_bundle_session_id() {
        let participants = vec![
            ed25519::PrivateKey::from_seed(71).public_key(),
            ed25519::PrivateKey::from_seed(72).public_key(),
        ];
        let local = participants[0].clone();
        let session_root = temp_dir("stale-bundle");
        let session_a = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
            session_id: 103,
            output_dir: session_root.clone(),
            participants: participants.clone(),
        })
        .expect("bootstrap session A");
        let session_b = run_trusted_dealer_bootstrap(TrustedDealerBootstrapConfig {
            session_id: 104,
            output_dir: session_root,
            participants: participants.clone(),
        })
        .expect("bootstrap session B");

        let local_file = format!("{}.bundle", commonware_utils::hex(local.as_ref()));
        fs::copy(
            session_b.session_dir.join("bundles").join(&local_file),
            session_a.session_dir.join("bundles").join(&local_file),
        )
        .expect("inject stale bundle from another session");

        let config = NodeConfig {
            bootstrap: crate::config::BootstrapConfig {
                genesis_dkg_session_dir: Some(session_a.session_dir),
                genesis_dkg_dealer_pubkey: Some(session_a.dealer_public_key),
                ..Default::default()
            },
            ..NodeConfig::default()
        };
        let err = match load_bls_material_for_signer(&config, &local, &participants) {
            Ok(_) => panic!("stale session bundle must fail"),
            Err(err) => err,
        };
        assert!(err
            .to_string()
            .contains("failed to load BLS bundle material"));
    }
}
