use std::net::SocketAddr;
use std::sync::Arc;

use app_evm::WhirlpoolEvmConfig;
use reth_chainspec::ChainSpec;
use reth_consensus::noop::NoopConsensus;
use reth_rpc_builder::{
    RpcModuleBuilder, RpcServerConfig, RpcServerHandle, TransportRpcModuleConfig,
};
use reth_rpc_server_types::RpcModuleSelection;
use reth_tasks::TokioTaskExecutor;
use reth_tokio_util::EventSender;
use state_reth::db::RethStateDb;
use tracing::info;

use crate::network::WhirlpoolNetwork;
use crate::pool::WhirlpoolTxPool;
use crate::provider::WhirlpoolProvider;

/// Start the JSON-RPC server wired through reth's RpcModuleBuilder.
pub async fn start_rpc_server(
    state_db: Arc<RethStateDb>,
    chain_spec: Arc<ChainSpec>,
    tx_source: Arc<dyn app::traits::TxSource>,
    addr: SocketAddr,
) -> Result<(RpcServerHandle, SocketAddr), Box<dyn std::error::Error + Send + Sync>> {
    let provider = WhirlpoolProvider::new(state_db, chain_spec.clone());
    let pool = WhirlpoolTxPool::new(tx_source);
    let network = WhirlpoolNetwork::new(chain_spec.chain().id());

    let builder = RpcModuleBuilder::default()
        .with_provider(provider)
        .with_pool(pool)
        .with_network(network)
        .with_executor(Box::new(TokioTaskExecutor::default()))
        .with_evm_config(WhirlpoolEvmConfig::new(chain_spec))
        .with_consensus(NoopConsensus::default());

    let eth_api = builder.bootstrap_eth_api();
    let modules = builder.build(
        TransportRpcModuleConfig::set_http(RpcModuleSelection::standard_modules()),
        eth_api,
        EventSender::new(1),
    );

    let server = RpcServerConfig::http(Default::default())
        .with_http_address(addr)
        .start(&modules)
        .await?;

    let local_addr = server.http_local_addr().unwrap_or(addr);
    info!("JSON-RPC server listening on {local_addr}");

    Ok((server, local_addr))
}
