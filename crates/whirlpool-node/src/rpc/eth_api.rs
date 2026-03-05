use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_rpc_types::{BlockId, TransactionReceipt, TransactionRequest};
use jsonrpsee::proc_macros::rpc;

/// Ethereum JSON-RPC namespace trait.
///
/// Defines the minimal set of `eth_*` methods required to support
/// basic ETH balance transfers via an alloy client.
#[rpc(server, namespace = "eth")]
pub trait EthApi {
    /// Returns the chain ID of the network.
    #[method(name = "chainId")]
    async fn chain_id(&self) -> jsonrpsee::core::RpcResult<U256>;

    /// Returns the current gas price in wei.
    #[method(name = "gasPrice")]
    async fn gas_price(&self) -> jsonrpsee::core::RpcResult<U256>;

    /// Returns the balance of the given address at the specified block.
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        address: Address,
        block_id: Option<BlockId>,
    ) -> jsonrpsee::core::RpcResult<U256>;

    /// Returns the transaction count (nonce) for the given address.
    #[method(name = "getTransactionCount")]
    async fn get_transaction_count(
        &self,
        address: Address,
        block_id: Option<BlockId>,
    ) -> jsonrpsee::core::RpcResult<U256>;

    /// Submits a signed transaction to the pool.
    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, bytes: Bytes) -> jsonrpsee::core::RpcResult<B256>;

    /// Returns a gas estimate for the given transaction.
    #[method(name = "estimateGas")]
    async fn estimate_gas(
        &self,
        request: TransactionRequest,
        block_id: Option<BlockId>,
    ) -> jsonrpsee::core::RpcResult<U256>;

    /// Returns the receipt for a given transaction hash.
    #[method(name = "getTransactionReceipt")]
    async fn get_transaction_receipt(
        &self,
        hash: B256,
    ) -> jsonrpsee::core::RpcResult<Option<TransactionReceipt>>;
}
