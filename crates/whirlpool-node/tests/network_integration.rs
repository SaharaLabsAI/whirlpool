//! Integration tests using real [`CommonwareNetworkProvider`] backed by
//! commonware's authenticated discovery networking layer.
//!
//! Each test spins up one or more nodes with real TCP networking on
//! localhost ephemeral ports (port 0) so there is no port-conflict risk.
//!
//! Because [`commonware_runtime::tokio::Runner::start`] is a blocking call
//! that creates its own tokio runtime, every test launches its runner on a
//! dedicated OS thread and communicates results back via a `std::sync::mpsc`
//! channel.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use commonware_cryptography::{ed25519, Signer};
use commonware_runtime::{tokio as cw_tokio, Metrics, Runner};

use consensus::ConsensusEngine;
use consensus_simplex::{CommonwareConfig, CommonwareEngine, FinalizationSink};
use p2p_commonware::CommonwareNetworkProviderBuilder;
use whirlpool_node::app::EmptyBlockApp;

/// Namespace shared by all test nodes so they can discover each other.
const TEST_NAMESPACE: &[u8] = b"whirlpool-integration-test";

/// Maximum p2p message size (1 MiB, same as production).
const MAX_MESSAGE_SIZE: u32 = 1_048_576;

/// Build a [`CommonwareConfig`] with short timeouts suitable for tests.
fn test_engine_config() -> CommonwareConfig {
    CommonwareConfig {
        namespace: String::from("integration-test"),
        leader_timeout: Duration::from_millis(500),
        notarization_timeout: Duration::from_millis(500),
        nullify_retry: Duration::from_millis(200),
        activity_timeout: 10,
        skip_timeout: 5,
        mailbox_size: 128,
        replay_buffer: NonZeroUsize::new(64).unwrap(),
        write_buffer: NonZeroUsize::new(64).unwrap(),
        epoch: 0,
        fetch_timeout: Duration::from_secs(2),
        fetch_concurrent: 4,
    }
}

/// Returns a localhost [`SocketAddr`] with an OS-assigned ephemeral port.
fn localhost_ephemeral() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}

// ---------------------------------------------------------------------------
// Test 1 – Single node lifecycle with real networking
// ---------------------------------------------------------------------------

#[test]
fn test_single_node_real_network_lifecycle() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        cw_tokio::Runner::default().start(|context| async move {
            // Create signer
            let signer = ed25519::PrivateKey::from_seed(100);
            let public_key = signer.public_key();

            let listen = localhost_ephemeral();
            let (network_provider, _oracle_handle) =
                CommonwareNetworkProviderBuilder::new(signer, TEST_NAMESPACE)
                    .listen_addr(listen)
                    .max_message_size(MAX_MESSAGE_SIZE)
                    .initial_validators(0, vec![public_key])
                    .build(context.with_label("network"));
            let app = Arc::new(EmptyBlockApp);
            let sink = Arc::new(FinalizationSink::new(Arc::new(AtomicU64::new(0))));
            let config = test_engine_config();
            let engine = CommonwareEngine::new(app, sink.clone(), config, network_provider);

            // Start the consensus engine
            let running = engine.start();
            match running {
                Ok(running) => {
                    // Poll until the engine produces at least one block (height >= 1)
                    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
                    loop {
                        let status = running.status();
                        if status.current_height >= 1 {
                            tx.send(Ok(status.current_height)).ok();
                            let _ = running.shutdown().await;
                            break;
                        }
                        if tokio::time::Instant::now() > deadline {
                            tx.send(Err("timed out waiting for block production".into()))
                                .ok();
                            let _ = running.shutdown().await;
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                }
                Err(e) => {
                    tx.send(Err(format!("engine start failed: {e}"))).ok();
                }
            }
        });
    });

    let result = rx.recv_timeout(Duration::from_secs(60));
    handle.join().expect("runner thread panicked");
    match result {
        Ok(Ok(height)) => {
            assert!(height >= 1, "expected height >= 1, got {height}");
        }
        Ok(Err(e)) => panic!("test failed: {e}"),
        Err(_) => panic!("test timed out waiting for result from runner"),
    }
}

// ---------------------------------------------------------------------------
// Test 2 – Two nodes discover each other with real p2p
// ---------------------------------------------------------------------------

#[test]
fn test_two_nodes_discover_and_run() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        cw_tokio::Runner::default().start(|context| async move {
            // Create two signers with distinct seeds
            let signer_0 = ed25519::PrivateKey::from_seed(200);
            let signer_1 = ed25519::PrivateKey::from_seed(201);
            let pk_0 = signer_0.public_key();
            let pk_1 = signer_1.public_key();

            let all_validators = vec![pk_0.clone(), pk_1.clone()];

            // --- Node 0 (bootstrapper) ---
            let listen_0 = localhost_ephemeral();
            let (provider_0, _oracle_handle_0) =
                CommonwareNetworkProviderBuilder::new(signer_0, TEST_NAMESPACE)
                    .listen_addr(listen_0)
                    .max_message_size(MAX_MESSAGE_SIZE)
                    .initial_validators(0, all_validators.clone())
                    .build(context.with_label("node_0"));
            let app_0 = Arc::new(EmptyBlockApp);
            let sink_0 = Arc::new(FinalizationSink::new(Arc::new(AtomicU64::new(0))));
            let engine_0 =
                CommonwareEngine::new(app_0, sink_0.clone(), test_engine_config(), provider_0);

            // --- Node 1 (bootstraps to node 0) ---
            // We need node 0's actual listen address. Since we used port 0,
            // discovery::Network doesn't expose the bound port before start().
            // So node 1 also uses no bootstrappers — both nodes rely on the
            // oracle peer-set for discovery on localhost.
            let listen_1 = localhost_ephemeral();
            let (provider_1, _oracle_handle_1) =
                CommonwareNetworkProviderBuilder::new(signer_1, TEST_NAMESPACE)
                    .listen_addr(listen_1)
                    .max_message_size(MAX_MESSAGE_SIZE)
                    .initial_validators(0, all_validators)
                    .build(context.with_label("node_1"));
            let app_1 = Arc::new(EmptyBlockApp);
            let sink_1 = Arc::new(FinalizationSink::new(Arc::new(AtomicU64::new(0))));
            let engine_1 =
                CommonwareEngine::new(app_1, sink_1.clone(), test_engine_config(), provider_1);

            // Start both engines
            let running_0 = match engine_0.start() {
                Ok(r) => r,
                Err(e) => {
                    tx.send(Err(format!("engine 0 start failed: {e}"))).ok();
                    return;
                }
            };
            let running_1 = match engine_1.start() {
                Ok(r) => r,
                Err(e) => {
                    tx.send(Err(format!("engine 1 start failed: {e}"))).ok();
                    let _ = running_0.shutdown().await;
                    return;
                }
            };

            // Wait for both nodes to produce blocks
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            loop {
                let s0 = running_0.status();
                let s1 = running_1.status();
                if s0.current_height >= 1 && s1.current_height >= 1 {
                    tx.send(Ok((s0.current_height, s1.current_height))).ok();
                    let _ = running_0.shutdown().await;
                    let _ = running_1.shutdown().await;
                    break;
                }
                if tokio::time::Instant::now() > deadline {
                    tx.send(Err(format!(
                        "timed out: node0 height={}, node1 height={}",
                        s0.current_height, s1.current_height
                    )))
                    .ok();
                    let _ = running_0.shutdown().await;
                    let _ = running_1.shutdown().await;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        });
    });

    let result = rx.recv_timeout(Duration::from_secs(60));
    handle.join().expect("runner thread panicked");
    match result {
        Ok(Ok((h0, h1))) => {
            assert!(h0 >= 1, "node 0: expected height >= 1, got {h0}");
            assert!(h1 >= 1, "node 1: expected height >= 1, got {h1}");
        }
        Ok(Err(e)) => panic!("test failed: {e}"),
        Err(_) => panic!("test timed out waiting for result from runner"),
    }
}

// ---------------------------------------------------------------------------
// Test 3 – Graceful shutdown with real networking
// ---------------------------------------------------------------------------

#[test]
fn test_real_network_graceful_shutdown() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        cw_tokio::Runner::default().start(|context| async move {
            let signer = ed25519::PrivateKey::from_seed(300);
            let public_key = signer.public_key();

            let listen = localhost_ephemeral();
            let (provider, _oracle_handle) =
                CommonwareNetworkProviderBuilder::new(signer, TEST_NAMESPACE)
                    .listen_addr(listen)
                    .max_message_size(MAX_MESSAGE_SIZE)
                    .initial_validators(0, vec![public_key])
                    .build(context.with_label("network"));
            let app = Arc::new(EmptyBlockApp);
            let sink = Arc::new(FinalizationSink::new(Arc::new(AtomicU64::new(0))));
            let engine = CommonwareEngine::new(app, sink, test_engine_config(), provider);

            match engine.start() {
                Ok(running) => {
                    // Verify the engine is running
                    let status = running.status();
                    assert!(status.is_running, "engine should be running after start");

                    // Immediate shutdown – must not hang or panic
                    let _ = running.shutdown().await;
                    tx.send(Ok(())).ok();
                }
                Err(e) => {
                    tx.send(Err(format!("engine start failed: {e}"))).ok();
                }
            }
        });
    });

    let result = rx.recv_timeout(Duration::from_secs(30));
    handle.join().expect("runner thread panicked");
    match result {
        Ok(Ok(())) => {} // success
        Ok(Err(e)) => panic!("test failed: {e}"),
        Err(_) => panic!("test timed out – shutdown may be hanging"),
    }
}
