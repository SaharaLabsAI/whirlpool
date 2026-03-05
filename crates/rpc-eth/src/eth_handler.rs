use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionReceipt, TransactionRequest};
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use state::StateDb;
use crate::context::EthRpcContext;
use crate::eth_api::EthApiServer;

/// Implements the `EthApi` JSON-RPC trait for the Sahara/Whirlpool node.
pub struct EthApiHandler<S: StateDb> {
    ctx: EthRpcContext<S>,
}

impl<S: StateDb> EthApiHandler<S> {
    pub fn new(ctx: EthRpcContext<S>) -> Self {
        Self { ctx }
    }
}

/// Hardcoded gas for simple ETH transfers (v1).
const TRANSFER_GAS: u64 = 21_000;

/// Hardcoded gas price: 1 gwei (v1).
const GAS_PRICE_WEI: u64 = 1_000_000_000;

/// Validate that a block ID refers to "latest" or is absent.
/// Rejects specific block numbers / tags we don't yet support.
fn validate_block_id(block_id: &Option<BlockId>) -> RpcResult<()> {
    match block_id {
        None => Ok(()),
        Some(BlockId::Number(BlockNumberOrTag::Latest)) => Ok(()),
        Some(BlockId::Number(BlockNumberOrTag::Pending)) => Ok(()),
        Some(other) => Err(ErrorObjectOwned::owned(
            -32000,
            format!("unsupported block id: {other:?}"),
            None::<()>,
        )),
    }
}

#[async_trait::async_trait]
impl<S: StateDb + Send + Sync + 'static> EthApiServer for EthApiHandler<S> {
    async fn chain_id(&self) -> RpcResult<U256> {
        Ok(U256::from(self.ctx.chain_id))
    }

    async fn gas_price(&self) -> RpcResult<U256> {
        Ok(U256::from(GAS_PRICE_WEI))
    }

    async fn get_balance(&self, address: Address, block_id: Option<BlockId>) -> RpcResult<U256> {
        validate_block_id(&block_id)?;
        let state = self.ctx.state_db.read().map_err(|e| {
            ErrorObjectOwned::owned(-32000, format!("state lock poisoned: {e}"), None::<()>)
        })?;
        let balance = state
            .get_account(address)
            .map(|a| a.balance)
            .unwrap_or(U256::ZERO);
        Ok(balance)
    }

    async fn get_transaction_count(
        &self,
        address: Address,
        block_id: Option<BlockId>,
    ) -> RpcResult<U256> {
        validate_block_id(&block_id)?;
        let state = self.ctx.state_db.read().map_err(|e| {
            ErrorObjectOwned::owned(-32000, format!("state lock poisoned: {e}"), None::<()>)
        })?;
        let nonce = state.get_account(address).map(|a| a.nonce).unwrap_or(0);
        Ok(U256::from(nonce))
    }

    async fn send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256> {
        if bytes.is_empty() {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "empty transaction bytes",
                None::<()>,
            ));
        }
        // Compute keccak256 hash of the raw transaction bytes.
        let hash = alloy_primitives::keccak256(&bytes);
        // Push raw tx into the pool for consensus to pick up.
        self.ctx.tx_pool.push(bytes.to_vec());
        Ok(hash)
    }

    async fn estimate_gas(
        &self,
        _request: TransactionRequest,
        _block_id: Option<BlockId>,
    ) -> RpcResult<U256> {
        // v1: hardcoded gas for simple transfers.
        Ok(U256::from(TRANSFER_GAS))
    }

    async fn get_transaction_receipt(&self, hash: B256) -> RpcResult<Option<TransactionReceipt>> {
        Ok(self.ctx.receipt_store.get(&hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EthRpcContext;
    use app::tx_source::InMemoryTxPool;
    use state_memory::InMemoryStateDb;
    use std::sync::Arc;
    use std::sync::RwLock;

    fn test_ctx() -> EthRpcContext<InMemoryStateDb> {
        let pool = Arc::new(InMemoryTxPool::new());
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        EthRpcContext::new(pool, db, 313_371)
    }

    #[tokio::test]
    async fn test_chain_id() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler.chain_id().await.unwrap();
        assert_eq!(result, U256::from(313_371u64));
    }

    #[tokio::test]
    async fn test_gas_price() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler.gas_price().await.unwrap();
        assert_eq!(result, U256::from(1_000_000_000u64));
    }

    #[tokio::test]
    async fn test_get_balance_unknown_account() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler.get_balance(Address::ZERO, None).await.unwrap();
        assert_eq!(result, U256::ZERO);
    }

    #[tokio::test]
    async fn test_get_balance_known_account() {
        let ctx = test_ctx();
        {
            let mut db = ctx.state_db.write().unwrap();
            db.insert_account(
                Address::ZERO,
                revm::state::AccountInfo {
                    balance: U256::from(1000u64),
                    nonce: 5,
                    code_hash: B256::ZERO,
                    code: None,
                    ..Default::default()
                },
            );
        }
        let handler = EthApiHandler::new(ctx);
        let result = handler.get_balance(Address::ZERO, None).await.unwrap();
        assert_eq!(result, U256::from(1000u64));
    }

    #[tokio::test]
    async fn test_get_transaction_count() {
        let ctx = test_ctx();
        {
            let mut db = ctx.state_db.write().unwrap();
            db.insert_account(
                Address::ZERO,
                revm::state::AccountInfo {
                    balance: U256::ZERO,
                    nonce: 42,
                    code_hash: B256::ZERO,
                    code: None,
                    ..Default::default()
                },
            );
        }
        let handler = EthApiHandler::new(ctx);
        let result = handler
            .get_transaction_count(Address::ZERO, None)
            .await
            .unwrap();
        assert_eq!(result, U256::from(42u64));
    }

    #[tokio::test]
    async fn test_send_raw_transaction() {
        let ctx = test_ctx();
        let handler = EthApiHandler::new(ctx.clone());
        let tx_bytes = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
        let hash = handler.send_raw_transaction(tx_bytes.clone()).await.unwrap();
        let expected = alloy_primitives::keccak256(&tx_bytes);
        assert_eq!(hash, expected);
    }

    #[tokio::test]
    async fn test_send_raw_transaction_empty_fails() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler.send_raw_transaction(Bytes::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_estimate_gas() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler
            .estimate_gas(TransactionRequest::default(), None)
            .await
            .unwrap();
        assert_eq!(result, U256::from(21_000u64));
    }

    #[tokio::test]
    async fn test_get_transaction_receipt_not_found() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler
            .get_transaction_receipt(B256::ZERO)
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
