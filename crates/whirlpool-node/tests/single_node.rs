use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_single_node_finalizes_blocks() {
    // 1. Initialize tracing for test output
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    // 2. Create shared height and running atomics for observation
    let height = Arc::new(AtomicU64::new(0));
    let running = Arc::new(AtomicBool::new(true));

    // 3. Create and start engine directly via create_starter
    // We bypass CommonwareEngine to have direct access to the shared atomics
    let starter = whirlpool_node::wire::create_starter();
    let (shutdown_fn, handle) = starter(height.clone(), running.clone())
        .expect("starter should succeed");

    tracing::info!("Engine started, waiting for blocks to finalize...");

    // 4. Wait for at least 2 blocks (with timeout and polling)
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30);

    loop {
        if start.elapsed() > timeout {
            let current_height = height.load(Ordering::SeqCst);
            panic!(
                "Timeout waiting for block finalization. Current height: {}",
                current_height
            );
        }

        let current_height = height.load(Ordering::SeqCst);
        if current_height >= 2 {
            tracing::info!("✓ Reached height {}, test success", current_height);
            break;
        }

        std::thread::sleep(Duration::from_millis(500));
    }

    // 5. Shutdown
    tracing::info!("Shutting down engine...");
    shutdown_fn();

    // 6. Wait for handle to complete
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            handle
                .await
                .expect("handle join failed")
                .expect("engine should shutdown cleanly")
        });

    // 7. Assert final state
    let final_height = height.load(Ordering::SeqCst);
    assert!(
        final_height >= 2,
        "Expected height >= 2, got {}",
        final_height
    );
    assert!(!running.load(Ordering::SeqCst), "running should be false");

    tracing::info!("✓ Integration test passed. Final height: {}", final_height);
}
