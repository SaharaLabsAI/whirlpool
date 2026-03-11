# Crate Contract: rpc-eth

## Identity
- **Crate**: `rpc-eth`
- **Path**: `crates/rpc-eth`
- **Role**: Ethereum JSON-RPC server adapter wiring reth's RPC stack to whirlpool backends

## Public API

### Functions
```rust
/// Start the JSON-RPC HTTP server using reth's RpcModuleBuilder.
/// Returns a handle that can be used to stop the server.
pub async fn start_rpc_server(config: RpcConfig) -> Result<RpcServerHandle, RpcError>
```

### Types
```rust
/// Configuration for the RPC server.
pub struct RpcConfig {
    pub state_db: Arc<RethStateDb>,
    pub tx_source: Arc<dyn TxSource>,
    pub chain_id: u64,
    pub bind_addr: SocketAddr,
}

/// Handle to a running RPC server (re-export of jsonrpsee::server::ServerHandle or reth equivalent)
pub type RpcServerHandle = reth_rpc_builder::ServerHandle;

/// RPC server error type
pub enum RpcError {
    Server(jsonrpsee::core::Error),
    Config(String),
}
```

### Internal Types (not exported)
- `WhirlpoolProvider` — implements ~20 reth storage/provider traits
- `WhirlpoolTxPool` — implements `TransactionPool`
- `WhirlpoolNetwork` — implements `NetworkInfo + Peers`

## Dependencies

### Required (inbound)
- `state-reth::RethStateDb` (for WhirlpoolProvider — provides StateDb + BlockStorage + revm::Database)
- `app::TxSource` (for WhirlpoolTxPool)
- `reth-rpc-builder` (RpcModuleBuilder, server config)
- `reth-rpc-eth-api` (EthApiServer trait surface)
- `reth-rpc` (EthApi concrete implementation)
- `reth-provider` + `reth-storage-api` (provider traits to implement)
- `reth-transaction-pool` (TransactionPool trait)
- `reth-network-api` (NetworkInfo + Peers traits)
- `reth-evm-ethereum` (EthEvmConfig)
- `reth-consensus` (NoopConsensus)

### Dependents (outbound)
- `whirlpool-node` — calls `start_rpc_server(RpcConfig)`

## Invariants
1. Server exposes standard `eth_*` methods required by requirements, with blob method explicitly unsupported
2. `eth_blobBaseFee` returns unsupported-feature style error
3. Type-3 (blob) transactions are rejected at tx-pool adapter boundary
4. Provider reads remain thread-safe and deterministic under adapter contracts
5. Server runs on tokio runtime (same as whirlpool-node)
6. No vendor code modifications — all adaptation via wrapper types
