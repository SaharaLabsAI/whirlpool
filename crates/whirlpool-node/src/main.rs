//! Whirlpool consensus node binary.

use clap::Parser;
use tracing::info;
use whirlpool_node::config::{load_config, NodeArgs};
use whirlpool_node::node::start_node;

fn main() {
    let args = NodeArgs::parse();
    let config = load_config(args).expect("config error");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!(?config, "Starting Whirlpool node");

    let _node = start_node(config).expect("failed to start node");

    loop {
        std::thread::park();
    }
}
