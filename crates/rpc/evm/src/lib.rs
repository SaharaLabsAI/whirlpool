//! rpc-eth: Ethereum JSON-RPC server backed by reth's RPC stack.

use std::net::SocketAddr;
use std::sync::Arc;

use reth_chainspec::ChainSpec;
use reth_rpc_builder::RpcServerHandle;
use state_reth::db::RethStateDb;

// Internal modules (adapter types, not part of public API).
#[path = "convert.rs"]
mod convert_impl;
#[path = "network.rs"]
mod network_impl;
#[path = "pool.rs"]
mod pool_impl;
#[path = "provider/mod.rs"]
mod provider_impl;
#[path = "server.rs"]
mod server_impl;

// Compatibility exports used by integration contracts.
#[doc(hidden)]
pub mod convert {
    pub use super::convert_impl::{decode_transaction, evmblock_to_block, evmblock_to_header};
}

#[doc(hidden)]
pub mod network {
    pub use super::network_impl::WhirlpoolNetwork;
}

#[doc(hidden)]
pub mod pool {
    pub use super::pool_impl::WhirlpoolTxPool;
}

#[doc(hidden)]
pub mod provider {
    pub use super::provider_impl::WhirlpoolProvider;
}

#[doc(hidden)]
pub mod server {
    pub use super::server_impl::start_rpc_server;
}

/// Configuration for starting the JSON-RPC server.
pub struct RpcConfig {
    /// State database handle.
    pub state_db: Arc<RethStateDb>,
    /// Chain specification.
    pub chain_spec: Arc<ChainSpec>,
    /// Transaction source for the mempool.
    pub tx_source: Arc<dyn app::traits::TxSource>,
    /// Address to bind the HTTP server.
    pub addr: SocketAddr,
}

/// Errors from RPC server operations.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Server startup or runtime error.
    #[error("RPC server error: {0}")]
    Server(Box<dyn std::error::Error + Send + Sync>),
}

/// Start the JSON-RPC server with the given configuration.
///
/// Returns a handle to the running server and the actual bound address.
pub async fn start_rpc_server(
    config: RpcConfig,
) -> Result<(RpcServerHandle, SocketAddr), RpcError> {
    server_impl::start_rpc_server(
        config.state_db,
        config.chain_spec,
        config.tx_source,
        config.addr,
    )
    .await
    .map_err(RpcError::Server)
}
