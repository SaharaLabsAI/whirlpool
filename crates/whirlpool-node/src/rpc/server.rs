use jsonrpsee::server::{ServerBuilder, ServerHandle};
use state::StateDb;
use std::net::SocketAddr;
use tracing::info;

use super::context::EthRpcContext;
use super::eth_api::EthApiServer;
use super::eth_handler::EthApiHandler;

/// Start the JSON-RPC server and return a handle for graceful shutdown.
pub async fn start_rpc_server<S: StateDb + Send + Sync + 'static>(
    ctx: EthRpcContext<S>,
    addr: SocketAddr,
) -> Result<ServerHandle, Box<dyn std::error::Error + Send + Sync>> {
    let handler = EthApiHandler::new(ctx);
    let server = ServerBuilder::default().build(addr).await?;
    let addr = server.local_addr()?;
    info!("JSON-RPC server listening on {addr}");
    let handle = server.start(handler.into_rpc());
    Ok(handle)
}
