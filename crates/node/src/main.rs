//! Whirlpool consensus node binary.

use clap::Parser;
use tracing::info;
use whirlpool_node::config::{load_config, NodeArgs};
use whirlpool_node::node::{run_genesis_bootstrap, start_node};

fn main() {
    let args = NodeArgs::parse();
    let config = load_config(args).expect("config error");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!(
        rpc_addr = %config.rpc.bind_addr,
        mem_rpc_addr = %config.rpc.mem_bind_addr,
        p2p_listen_addr = %config.network.listen_addr,
        p2p_dialable_addr = %config.network.dialable_addr,
        bootstrap_mode = config.bootstrap.genesis_bootstrap_dkg,
        "Starting Whirlpool node"
    );

    if config.bootstrap.genesis_bootstrap_dkg {
        run_genesis_bootstrap(&config).expect("failed to run genesis bootstrap");
        return;
    }

    let _node = start_node(config).expect("failed to start node");

    loop {
        std::thread::park();
    }
}
