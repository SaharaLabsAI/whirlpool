use super::*;
use crate::context::EthRpcContext;
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

fn test_ctx_with_blocks(blocks: Vec<EvmBlock>) -> EthRpcContext<InMemoryStateDb, MockBlockStorage> {
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
