use consensus::ConsensusEngine;
use consensus_simplex::CommonwareEngine;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    tracing::info!("whirlpool-node starting");

    // 2. Create CommonwareEngine with starter from wire.rs
    let engine = CommonwareEngine::new(whirlpool_node::wire::create_starter());

    // 3. Start the engine
    let running = engine.start().expect("failed to start consensus engine");

    tracing::info!("consensus engine started, press Ctrl-C to stop");

    // 4. Wait for Ctrl-C
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");

    tracing::info!("shutting down...");

    // 5. Shutdown gracefully
    running.shutdown().await.expect("failed to shutdown engine");

    tracing::info!("whirlpool-node stopped");

    Ok(())
}
