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
        .map(alloy_primitives::keccak256)
        .collect();

    Block {
        header,
        transactions: BlockTransactions::Hashes(tx_hashes),
        uncles: vec![],
        withdrawals: None,
    }
}

#[cfg(test)]
#[path = "tests/eth_handler.rs"]
mod tests;
