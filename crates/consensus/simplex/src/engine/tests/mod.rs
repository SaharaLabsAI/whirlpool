
use super::*;
use crate::sink::FinalizationSink;
use crate::tests::{MockApp, TestBlock};
use commonware_cryptography::Signer as _;
use commonware_cryptography::{
    bls12381::{dkg, primitives::variant::MinSig},
    ed25519::PrivateKey,
};
use commonware_runtime::{tokio as commonware_tokio, Clock, Metrics, Runner};
use commonware_utils::{ordered::Set, N3f1};
use network_commonware::CommonwareNetworkProviderBuilder;
use rand::rngs::OsRng;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

fn test_config() -> CommonwareConfig {
    let signer = PrivateKey::from_seed(19);
    let validators = vec![signer.public_key()];

    CommonwareConfig {
        namespace: "test".to_string(),
        leader_timeout: Duration::from_secs(1),
        notarization_timeout: Duration::from_secs(1),
        nullify_retry: Duration::from_millis(100),
        activity_timeout: 10,
        skip_timeout: 5,
        mailbox_size: 10,
        replay_buffer: NonZeroUsize::new(10).unwrap(),
        write_buffer: NonZeroUsize::new(10).unwrap(),
        epoch: 0,
        height: Arc::new(AtomicU64::new(0)),
        fetch_timeout: Duration::from_secs(1),
        fetch_concurrent: 4,
        signing_scheme: SigningSchemeConfig::Ed25519 { signer, validators },
    }
}

fn test_config_bls() -> CommonwareConfig {
    let signer = PrivateKey::from_seed(31);
    let participants = vec![signer.public_key()];
    let participant_set = Set::from_iter_dedup(participants.clone());
    let (output, shares) = dkg::deal::<MinSig, _, N3f1>(OsRng, Default::default(), participant_set)
        .expect("trusted dealer setup should succeed");
    let share = shares
        .get_value(&signer.public_key())
        .cloned()
        .expect("local share should exist");

    CommonwareConfig {
        namespace: "test-bls".to_string(),
        leader_timeout: Duration::from_secs(1),
        notarization_timeout: Duration::from_secs(1),
        nullify_retry: Duration::from_millis(100),
        activity_timeout: 10,
        skip_timeout: 5,
        mailbox_size: 10,
        replay_buffer: NonZeroUsize::new(10).unwrap(),
        write_buffer: NonZeroUsize::new(10).unwrap(),
        epoch: 0,
        height: Arc::new(AtomicU64::new(0)),
        fetch_timeout: Duration::from_secs(1),
        fetch_concurrent: 4,
        signing_scheme: SigningSchemeConfig::BlsThresholdVrf {
            participants,
            polynomial: output.public().clone(),
            share,
        },
    }
}

fn ed25519_signer_and_validators(
    config: &CommonwareConfig,
) -> (PrivateKey, Vec<ed25519::PublicKey>) {
    match &config.signing_scheme {
        SigningSchemeConfig::Ed25519 { signer, validators } => (signer.clone(), validators.clone()),
        SigningSchemeConfig::BlsThresholdVrf { .. } => {
            panic!("test helper only supports ed25519 signing configuration")
        }
    }
}

#[test]
fn test_engine_can_be_constructed() {
    let executor = commonware_runtime::deterministic::Runner::default();
    executor.start(|context| async move {
        let app = Arc::new(MockApp);
        let config = test_config();
        let (signer, validators) = ed25519_signer_and_validators(&config);
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(Arc::clone(
            &config.height,
        )));

        let (network, _oracle_handle) =
            CommonwareNetworkProviderBuilder::new(signer, config.namespace.as_bytes())
                .listen_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
                .dialable_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
                .initial_validators(config.epoch, validators)
                .build(context.with_label("network"))
                .await;
        let _engine = CommonwareEngine::new(app, sink, config, network, context);
    });
}

#[test]
fn test_engine_can_start_and_shutdown() {
    let runner = commonware_tokio::Runner::default();
    runner.start(|context| async move {
        let app = Arc::new(MockApp);
        let config = test_config();
        let (signer, validators) = ed25519_signer_and_validators(&config);
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(Arc::clone(
            &config.height,
        )));

        let (network, mut oracle_handle) =
            CommonwareNetworkProviderBuilder::new(signer, config.namespace.as_bytes())
                .listen_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
                .dialable_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
                .initial_validators(config.epoch, validators.clone())
                .build(context.with_label("network"))
                .await;
        oracle_handle
            .update_validators(config.epoch, validators)
            .await;
        let engine = CommonwareEngine::new(app, sink, config, network, context);
        let running = engine.start().expect("Engine should start");

        // Check status
        let status = running.status();
        assert!(status.is_running);
        assert_eq!(status.current_height, 0);

        // Shutdown
        drop(running);
    });
}

#[test]
fn test_engine_can_start_with_bls_threshold_scheme() {
    let runner = commonware_tokio::Runner::default();
    runner.start(|context| async move {
        let app = Arc::new(MockApp);
        let config = test_config_bls();
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(Arc::clone(
            &config.height,
        )));
        let network_signer = PrivateKey::from_seed(31);

        let (network, mut oracle_handle) =
            CommonwareNetworkProviderBuilder::new(network_signer, config.namespace.as_bytes())
                .listen_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
                .dialable_addr(SocketAddr::from(([127, 0, 0, 1], 0)))
                .initial_validators(config.epoch, config.signing_scheme.participants().to_vec())
                .build(context.with_label("network"))
                .await;
        oracle_handle
            .update_validators(config.epoch, config.signing_scheme.participants().to_vec())
            .await;

        let engine = CommonwareEngine::new(app, sink, config, network, context);
        let running = engine.start().expect("BLS threshold engine should start");
        assert!(running.status().is_running);
        drop(running);
    });
}

#[test]
#[ignore = "requires multi-node P2P connectivity for consensus progress"]
fn test_engine_simulates_block_finalization() {
    let runner = commonware_tokio::Runner::default();
    runner.start(|context| async move {
        let app = Arc::new(MockApp);
        let config = test_config();
        let (signer, validators) = ed25519_signer_and_validators(&config);
        let sink = Arc::new(FinalizationSink::<TestBlock>::new(Arc::clone(
            &config.height,
        )));

        let (network, mut oracle) =
            CommonwareNetworkProviderBuilder::new(signer, config.namespace.as_bytes())
                .listen_addr(SocketAddr::from(([127, 0, 0, 1], 31401)))
                .dialable_addr(SocketAddr::from(([127, 0, 0, 1], 31401)))
                .build(context.with_label("network"))
                .await;

        oracle.update_validators(config.epoch, validators).await;

        let engine = CommonwareEngine::new(app, sink, config, network, context.clone());
        let running = engine.start().expect("engine should start");

        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        let mut observed_height = 0u64;
        let mut reached_height = false;
        while std::time::Instant::now() < deadline {
            observed_height = running.status().current_height;
            if observed_height >= 1 {
                reached_height = true;
                break;
            }
            context.sleep(Duration::from_millis(200)).await;
        }

        drop(running);
        assert!(
            reached_height,
            "Should have finalized at least 1 block, observed height {}",
            observed_height
        );
    });
}
