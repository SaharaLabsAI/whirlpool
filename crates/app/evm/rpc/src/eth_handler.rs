use crate::context::EthRpcContext;
use crate::eth_api::EthApiServer;
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_rpc_types::{
    Block, BlockId, BlockNumberOrTag, BlockTransactions, Header as RpcHeader, TransactionReceipt,
    TransactionRequest,
};
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use state::{BlockStorage, StateDb};
use std::sync::atomic::Ordering;

/// Implements the `EthApi` JSON-RPC trait for the Sahara/Whirlpool node.
pub struct EthApiHandler<S: StateDb, B: BlockStorage> {
    ctx: EthRpcContext<S, B>,
}

impl<S: StateDb, B: BlockStorage> EthApiHandler<S, B> {
    pub fn new(ctx: EthRpcContext<S, B>) -> Self {
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
impl<S: StateDb + Send + Sync + 'static, B: BlockStorage + 'static> EthApiServer
    for EthApiHandler<S, B>
{
    async fn chain_id(&self) -> RpcResult<U256> {
        Ok(U256::from(self.ctx.chain_id))
    }

    async fn block_number(&self) -> RpcResult<U256> {
        let height = self
            .ctx
            .block_height
            .load(std::sync::atomic::Ordering::Relaxed);
        Ok(U256::from(height))
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
            .map_err(|e| ErrorObjectOwned::owned(-32000, format!("state error: {e}"), None::<()>))?
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
        let nonce = state
            .get_account(address)
            .map_err(|e| ErrorObjectOwned::owned(-32000, format!("state error: {e}"), None::<()>))?
            .map(|a| a.nonce)
            .unwrap_or(0);
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

    async fn get_block_by_number(
        &self,
        block_number: BlockNumberOrTag,
        full_transactions: bool,
    ) -> RpcResult<Option<Block>> {
        let number = match block_number {
            BlockNumberOrTag::Latest | BlockNumberOrTag::Finalized | BlockNumberOrTag::Safe => {
                self.ctx.block_height.load(Ordering::Relaxed)
            }
            BlockNumberOrTag::Earliest => 0,
            BlockNumberOrTag::Pending => return Ok(None),
            BlockNumberOrTag::Number(n) => n,
        };

        let block = self
            .ctx
            .block_storage
            .get_block_by_number(number)
            .map_err(|e| {
                ErrorObjectOwned::owned(-32603, format!("block storage error: {e}"), None::<()>)
            })?;

        match block {
            Some(evm_block) => Ok(Some(evm_block_to_rpc_block(&evm_block, full_transactions))),
            None => Ok(None),
        }
    }

    async fn get_block_by_hash(
        &self,
        block_hash: B256,
        full_transactions: bool,
    ) -> RpcResult<Option<Block>> {
        let block = self
            .ctx
            .block_storage
            .get_block_by_hash(block_hash)
            .map_err(|e| {
                ErrorObjectOwned::owned(-32603, format!("block storage error: {e}"), None::<()>)
            })?;

        match block {
            Some(evm_block) => Ok(Some(evm_block_to_rpc_block(&evm_block, full_transactions))),
            None => Ok(None),
        }
    }
}

/// Convert an `EvmBlock` to an alloy RPC `Block`.
fn evm_block_to_rpc_block(evm_block: &app::EvmBlock, _full_transactions: bool) -> Block {
    let inner_header = alloy_consensus::Header {
        number: evm_block.height,
        parent_hash: B256::from_slice(&evm_block.parent_id),
        state_root: B256::from_slice(&evm_block.state_root),
        transactions_root: B256::from_slice(&evm_block.transactions_root),
        receipts_root: B256::from_slice(&evm_block.receipts_root),
        gas_used: evm_block.gas_used,
        base_fee_per_gas: Some(evm_block.base_fee_per_gas),
        timestamp: evm_block.timestamp,
        gas_limit: 30_000_000,
        ..Default::default()
    };

    let header = RpcHeader {
        inner: inner_header,
        hash: B256::ZERO,
        total_difficulty: None,
        size: None,
    };

    // Return hash-only transaction list. Raw tx bytes are keccak256-hashed.
    let tx_hashes: Vec<B256> = evm_block
        .transactions
        .iter()
        .map(|tx_bytes| alloy_primitives::keccak256(tx_bytes))
        .collect();

    Block {
        header,
        transactions: BlockTransactions::Hashes(tx_hashes),
        uncles: vec![],
        withdrawals: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EthRpcContext;
    use app::traits::TxSource;
    use app::tx_source::InMemoryTxPool;
    use app::{EvmBlock, Receipt};
    use state::block_storage::BlockStorageError;
    use state_memory::InMemoryStateDb;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex, RwLock};

    /// Mock BlockStorage for testing
    struct MockBlockStorage {
        blocks: Mutex<Vec<EvmBlock>>,
        should_fail: AtomicBool,
    }

    impl MockBlockStorage {
        fn new() -> Self {
            Self {
                blocks: Mutex::new(Vec::new()),
                should_fail: AtomicBool::new(false),
            }
        }

        fn with_blocks(blocks: Vec<EvmBlock>) -> Self {
            Self {
                blocks: Mutex::new(blocks),
                should_fail: AtomicBool::new(false),
            }
        }

        fn with_failure() -> Self {
            Self {
                blocks: Mutex::new(Vec::new()),
                should_fail: AtomicBool::new(true),
            }
        }
    }

    impl BlockStorage for MockBlockStorage {
        fn store_block(
            &self,
            block: &EvmBlock,
            _receipts: &[Receipt],
        ) -> Result<(), BlockStorageError> {
            self.blocks.lock().unwrap().push(block.clone());
            Ok(())
        }

        fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, BlockStorageError> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(BlockStorageError::Database("mock failure".into()));
            }
            Ok(self
                .blocks
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.height == number)
                .cloned())
        }

        fn get_block_by_hash(&self, hash: B256) -> Result<Option<EvmBlock>, BlockStorageError> {
            if self.should_fail.load(Ordering::Relaxed) {
                return Err(BlockStorageError::Database("mock failure".into()));
            }
            use commonware_cryptography::Digestible;
            Ok(self
                .blocks
                .lock()
                .unwrap()
                .iter()
                .find(|b| B256::from_slice(&b.digest()) == hash)
                .cloned())
        }

        fn get_receipts_by_block(
            &self,
            _number: u64,
        ) -> Result<Option<Vec<Receipt>>, BlockStorageError> {
            Ok(None)
        }

        fn get_latest_block_number(&self) -> Result<Option<u64>, BlockStorageError> {
            Ok(self.blocks.lock().unwrap().iter().map(|b| b.height).max())
        }
    }

    fn test_ctx() -> EthRpcContext<InMemoryStateDb, MockBlockStorage> {
        let pool = Arc::new(InMemoryTxPool::new());
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let storage = Arc::new(MockBlockStorage::new());
        EthRpcContext::new(pool, db, storage, 313_371)
    }

    fn test_ctx_with_blocks(
        blocks: Vec<EvmBlock>,
    ) -> EthRpcContext<InMemoryStateDb, MockBlockStorage> {
        let pool = Arc::new(InMemoryTxPool::new());
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let storage = Arc::new(MockBlockStorage::with_blocks(blocks));
        EthRpcContext::new(pool, db, storage, 313_371)
    }

    fn test_ctx_with_failure() -> EthRpcContext<InMemoryStateDb, MockBlockStorage> {
        let pool = Arc::new(InMemoryTxPool::new());
        let db = Arc::new(RwLock::new(InMemoryStateDb::new()));
        let storage = Arc::new(MockBlockStorage::with_failure());
        EthRpcContext::new(pool, db, storage, 313_371)
    }

    fn sample_block(height: u64) -> EvmBlock {
        EvmBlock {
            height,
            parent_id: [0u8; 32],
            state_root: [1u8; 32],
            transactions_root: [2u8; 32],
            receipts_root: [3u8; 32],
            proposer_public_key: [0x33; 32],
            proposer_fee_recipient: [0x44; 20],
            gas_used: 21_000,
            base_fee_per_gas: 1_000_000_000,
            timestamp: 1_000_000 + height,
            transactions: vec![vec![0xde, 0xad]],
        }
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
        let hash = handler
            .send_raw_transaction(tx_bytes.clone())
            .await
            .unwrap();
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
        let result = handler.get_transaction_receipt(B256::ZERO).await.unwrap();
        assert!(result.is_none());
    }

    // ---- Block query tests (TC-RPC-01..08) ----

    /// TC-RPC-01: get_block_by_number returns None for missing block
    #[tokio::test]
    async fn test_get_block_by_number_miss() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler
            .get_block_by_number(BlockNumberOrTag::Number(999), false)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    /// TC-RPC-02: get_block_by_number returns full block JSON
    #[tokio::test]
    async fn test_get_block_by_number_found() {
        let block = sample_block(1);
        let ctx = test_ctx_with_blocks(vec![block.clone()]);
        let handler = EthApiHandler::new(ctx);

        let result = handler
            .get_block_by_number(BlockNumberOrTag::Number(1), true)
            .await
            .unwrap();

        assert!(result.is_some());
        let rpc_block = result.unwrap();
        assert_eq!(rpc_block.header.number, 1);
        assert_eq!(rpc_block.header.gas_used, 21_000);
        assert_eq!(rpc_block.header.timestamp, 1_000_001);
    }

    /// TC-RPC-03: get_block_by_number with full_transactions=false returns hashes
    #[tokio::test]
    async fn test_get_block_by_number_hash_only() {
        let block = sample_block(1);
        let ctx = test_ctx_with_blocks(vec![block]);
        let handler = EthApiHandler::new(ctx);

        let result = handler
            .get_block_by_number(BlockNumberOrTag::Number(1), false)
            .await
            .unwrap()
            .unwrap();

        match &result.transactions {
            BlockTransactions::Hashes(hashes) => {
                assert_eq!(hashes.len(), 1);
            }
            _ => panic!("expected hash-only transactions"),
        }
    }

    /// TC-RPC-04: get_block_by_hash returns block
    #[tokio::test]
    async fn test_get_block_by_hash_found() {
        let block = sample_block(1);
        use commonware_cryptography::Digestible;
        let hash = B256::from_slice(&block.digest());
        let ctx = test_ctx_with_blocks(vec![block]);
        let handler = EthApiHandler::new(ctx);

        let result = handler.get_block_by_hash(hash, false).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().header.number, 1);
    }

    /// TC-RPC-05: get_block_by_hash returns None for unknown hash
    #[tokio::test]
    async fn test_get_block_by_hash_miss() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler.get_block_by_hash(B256::ZERO, false).await.unwrap();
        assert!(result.is_none());
    }

    /// TC-RPC-06: Latest tag resolves to current block height
    #[tokio::test]
    async fn test_get_block_by_number_latest_tag() {
        let block = sample_block(5);
        let ctx = test_ctx_with_blocks(vec![block]);
        ctx.block_height.store(5, Ordering::Relaxed);
        let handler = EthApiHandler::new(ctx);

        let result = handler
            .get_block_by_number(BlockNumberOrTag::Latest, false)
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().header.number, 5);
    }

    /// TC-RPC-07: Pending tag returns None
    #[tokio::test]
    async fn test_get_block_by_number_pending_returns_none() {
        let handler = EthApiHandler::new(test_ctx());
        let result = handler
            .get_block_by_number(BlockNumberOrTag::Pending, false)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    /// TC-RPC-08: Storage error surfaces as JSON-RPC internal error
    #[tokio::test]
    async fn test_get_block_by_number_storage_error() {
        let ctx = test_ctx_with_failure();
        let handler = EthApiHandler::new(ctx);

        let result = handler
            .get_block_by_number(BlockNumberOrTag::Number(1), false)
            .await;
        assert!(result.is_err());
    }
}
