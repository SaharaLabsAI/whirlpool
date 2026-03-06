# rpc-eth crate — Persistent Block Storage Contract

## Purpose

**Today**: Exposes Ethereum JSON-RPC endpoints via `jsonrpsee`. The `EthApi` trait defines 7 methods (`chain_id`, `gas_price`, `get_balance`, `get_transaction_count`, `send_raw_transaction`, `estimate_gas`, `get_transaction_receipt`). `EthApiHandler<S: StateDb>` implements the trait using `EthRpcContext<S>` which holds `state_db`, `tx_pool`, `receipt_store`, `chain_id`, and `block_height`. No block query endpoints exist.

**Changes**:
1. Add `eth_getBlockByHash` and `eth_getBlockByNumber` to the `EthApi` trait
2. Add `block_storage: Arc<RwLock<dyn BlockStorage<Error = RethStateError>>>` field to `EthRpcContext`
3. Implement the new endpoints in `EthApiHandler` with EvmBlock -> `alloy_rpc_types::Block` conversion
4. Update `EthRpcContext::new()` constructor to accept block storage parameter

## Public API Changes

### Modified file: `src/eth_api.rs`

```rust
use alloy_primitives::{Address, Bytes, B256, U256};
use alloy_rpc_types::{Block, BlockId, BlockNumberOrTag, TransactionReceipt, TransactionRequest};
use jsonrpsee::proc_macros::rpc;

#[rpc(server, namespace = "eth")]
pub trait EthApi {
    // --- Existing methods (unchanged) ---

    #[method(name = "chainId")]
    async fn chain_id(&self) -> jsonrpsee::core::RpcResult<U256>;

    #[method(name = "gasPrice")]
    async fn gas_price(&self) -> jsonrpsee::core::RpcResult<U256>;

    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        address: Address,
        block_id: Option<BlockId>,
    ) -> jsonrpsee::core::RpcResult<U256>;

    #[method(name = "getTransactionCount")]
    async fn get_transaction_count(
        &self,
        address: Address,
        block_id: Option<BlockId>,
    ) -> jsonrpsee::core::RpcResult<U256>;

    #[method(name = "sendRawTransaction")]
    async fn send_raw_transaction(&self, bytes: Bytes) -> jsonrpsee::core::RpcResult<B256>;

    #[method(name = "estimateGas")]
    async fn estimate_gas(
        &self,
        request: TransactionRequest,
        block_id: Option<BlockId>,
    ) -> jsonrpsee::core::RpcResult<U256>;

    #[method(name = "getTransactionReceipt")]
    async fn get_transaction_receipt(
        &self,
        hash: B256,
    ) -> jsonrpsee::core::RpcResult<Option<TransactionReceipt>>;

    // --- NEW methods ---

    /// Returns block information by block hash.
    ///
    /// When `full` is true, returns full transaction objects;
    /// when false, returns only transaction hashes.
    #[method(name = "getBlockByHash")]
    async fn get_block_by_hash(
        &self,
        hash: B256,
        full: bool,
    ) -> jsonrpsee::core::RpcResult<Option<Block>>;

    /// Returns block information by block number or tag.
    ///
    /// Supported tags: Latest, Finalized, Number(n).
    /// When `full` is true, returns full transaction objects;
    /// when false, returns only transaction hashes.
    #[method(name = "getBlockByNumber")]
    async fn get_block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> jsonrpsee::core::RpcResult<Option<Block>>;
}
```

### Modified file: `src/context.rs`

```rust
use app::tx_source::InMemoryTxPool;
use state::BlockStorage;
use state::StateDb;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

use crate::receipt_store::ReceiptStore;

#[derive(Clone)]
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    pub tx_pool: Arc<InMemoryTxPool>,
    pub state_db: Arc<RwLock<S>>,
    pub receipt_store: Arc<ReceiptStore>,
    pub block_storage: Arc<RwLock<B>>,  // NEW — persistent block queries
    pub chain_id: u64,
    pub block_height: Arc<AtomicU64>,
}

impl<S: StateDb, B: BlockStorage> EthRpcContext<S, B> {
    pub fn new(
        tx_pool: Arc<InMemoryTxPool>,
        state_db: Arc<RwLock<S>>,
        block_storage: Arc<RwLock<B>>,  // NEW parameter
        chain_id: u64,
    ) -> Self {
        Self {
            tx_pool,
            state_db,
            receipt_store: Arc::new(ReceiptStore::new()),
            block_storage,
            chain_id,
            block_height: Arc::new(AtomicU64::new(0)),
        }
    }
}
```

> **Design note**: The STRATEGY.md proposes `Arc<dyn BlockStorage>` but using a generic `B: BlockStorage` parameter is more consistent with the existing `S: StateDb` pattern in the crate. Since `RethStateDb` implements both traits, `S` and `B` can be the same concrete type and share the same `Arc<RwLock<...>>`. The alternative is `Arc<dyn BlockStorage<Error = ...>>` which requires specifying the associated error type in the trait object — less ergonomic.

### Modified file: `src/eth_handler.rs`

```rust
use alloy_rpc_types::{Block as RpcBlock, BlockNumberOrTag};
use state::BlockStorage;

pub struct EthApiHandler<S: StateDb, B: BlockStorage> {
    ctx: EthRpcContext<S, B>,
}

impl<S: StateDb, B: BlockStorage> EthApiHandler<S, B> {
    pub fn new(ctx: EthRpcContext<S, B>) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl<S: StateDb + Send + Sync + 'static, B: BlockStorage + Send + Sync + 'static>
    EthApiServer for EthApiHandler<S, B>
{
    // ... existing 7 methods unchanged ...

    async fn get_block_by_hash(
        &self,
        hash: B256,
        full: bool,
    ) -> RpcResult<Option<RpcBlock>> {
        let storage = self.ctx.block_storage.read().map_err(|e| {
            ErrorObjectOwned::owned(-32000, format!("block storage lock poisoned: {e}"), None::<()>)
        })?;
        let evm_block = storage
            .get_block_by_hash(&hash.0)
            .map_err(|e| ErrorObjectOwned::owned(-32000, format!("storage error: {e}"), None::<()>))?;

        match evm_block {
            Some(block) => Ok(Some(evm_block_to_rpc_block(&block, full)?)),
            None => Ok(None),
        }
    }

    async fn get_block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<RpcBlock>> {
        let block_num = match number {
            BlockNumberOrTag::Number(n) => n,
            BlockNumberOrTag::Latest
            | BlockNumberOrTag::Finalized
            | BlockNumberOrTag::Safe => {
                self.ctx.block_height.load(std::sync::atomic::Ordering::SeqCst)
            }
            BlockNumberOrTag::Pending => {
                // Pending = latest finalized + 1 (not yet available)
                return Ok(None);
            }
            BlockNumberOrTag::Earliest => 0,
        };

        let storage = self.ctx.block_storage.read().map_err(|e| {
            ErrorObjectOwned::owned(-32000, format!("block storage lock poisoned: {e}"), None::<()>)
        })?;
        let evm_block = storage
            .get_block_by_number(block_num)
            .map_err(|e| ErrorObjectOwned::owned(-32000, format!("storage error: {e}"), None::<()>))?;

        match evm_block {
            Some(block) => Ok(Some(evm_block_to_rpc_block(&block, full)?)),
            None => Ok(None),
        }
    }
}
```

### New conversion function (in `src/eth_handler.rs` or new `src/conversions.rs`)

```rust
use alloy_rpc_types::Block as RpcBlock;
use app::EvmBlock;
use app_evm::executor::{build_header_from_evm_block, decode_transactions};

/// Convert an EvmBlock into an alloy RPC Block response.
///
/// When `full` is true, transaction objects are included; when false,
/// only transaction hashes are returned.
fn evm_block_to_rpc_block(block: &EvmBlock, full: bool) -> RpcResult<RpcBlock> {
    // 1. Build Header from EvmBlock
    let header = build_header_from_evm_block(block);
    let block_hash = header.hash_slow();

    // 2. Decode transactions from raw bytes
    let decoded_txs = decode_transactions(&block.transactions)
        .map_err(|e| ErrorObjectOwned::owned(-32000, format!("tx decode error: {e}"), None::<()>))?;

    // 3. Build alloy_rpc_types::Block with header fields
    // 4. If full: include Vec<alloy_rpc_types::Transaction>
    //    If !full: include Vec<B256> (tx hashes only)
    // 5. Return RpcBlock
    todo!()
}
```

## Internal Changes

### BlockNumberOrTag resolution

The `validate_block_id()` helper in `eth_handler.rs` currently rejects non-Latest/Pending blocks. It needs to be updated to accept `Number(n)`, `Finalized`, `Safe`, and `Earliest` tags now that historical blocks are available.

### Receipt fallback (post-MVP)

Per STRATEGY.md Q3, `get_transaction_receipt` may later be updated to fall back to `BlockStorage::get_receipts_by_block()` for finalized receipts not in the in-memory `ReceiptStore`. This is deferred to post-MVP.

## Dependencies

### New in `Cargo.toml`

```toml
[dependencies]
# ... existing ...
app-evm = { path = "../app-evm" }   # For build_header_from_evm_block, decode_transactions
```

### Existing (unchanged, already sufficient)

- `app = { path = "../app" }` — for `EvmBlock` type
- `state = { path = "../state" }` — for `StateDb` + now `BlockStorage` trait
- `alloy-rpc-types = "1.4.3"` — for `Block`, `BlockNumberOrTag`
- `alloy-primitives = "1.5.0"` — for `B256`, `Address`
- `jsonrpsee = "0.26.0"` — for RPC server macros
- `async-trait = "0.1"` — for async trait impl

## Error Types

No new error types. RPC errors use `ErrorObjectOwned::owned(-32000, message, None::<()>)` following the existing pattern in `eth_handler.rs`.

| Error Case | RPC Error Code | Message Pattern |
|------------|----------------|-----------------|
| Block storage lock poisoned | -32000 | `"block storage lock poisoned: {e}"` |
| BlockStorage query error | -32000 | `"storage error: {e}"` |
| Transaction decode failure | -32000 | `"tx decode error: {e}"` |
| Unsupported block tag | -32000 | `"unsupported block id: {other:?}"` (existing) |

## Test Surface

### Unit tests (in `src/eth_handler.rs`)

1. **get_block_by_number returns None for missing block** — Query block 999 on empty storage, expect `Ok(None)`
2. **get_block_by_number returns block for stored block** — Store a block in mock BlockStorage, query by number, verify response fields
3. **get_block_by_hash returns None for unknown hash** — Query unknown hash, expect `Ok(None)`
4. **get_block_by_hash returns block for stored block** — Store a block, query by header hash, verify response
5. **get_block_by_number with Latest tag** — Verify it reads `block_height` and queries that number
6. **get_block_by_number with Pending tag** — Verify returns `Ok(None)`
7. **get_block_by_number with Earliest tag** — Verify queries block 0
8. **full=true includes transaction objects** — Verify response contains full tx details
9. **full=false includes only tx hashes** — Verify response contains B256 hashes, not full objects
10. **evm_block_to_rpc_block conversion correctness** — Verify all header fields map correctly (number, parent_hash, state_root, etc.)

### Mock BlockStorage for tests

```rust
#[cfg(test)]
struct MockBlockStorage {
    blocks: HashMap<u64, EvmBlock>,
    receipts: HashMap<u64, Vec<Receipt>>,
}

#[cfg(test)]
impl BlockStorage for MockBlockStorage {
    type Error = state::StateError;
    // ... mock implementations ...
}
```

## Integration Points

| Connected Crate | Direction | Interface | Data Flow |
|-----------------|-----------|-----------|-----------|
| `state` | Depends on | `BlockStorage` trait | Trait bound on `EthRpcContext` generic parameter |
| `state-reth` | Runtime impl | `RethStateDb` provides `BlockStorage` | Queries read from MDBX tables |
| `app-evm` | Imports from | `build_header_from_evm_block()`, `decode_transactions()` | Used in EvmBlock -> RPC Block conversion |
| `app` | Depends on | `EvmBlock` type | Input to conversion function |
| `whirlpool-node` | Wired by | `EthRpcContext::new()` constructor | Passes `block_storage` parameter |

**RPC Method → Storage Call Mapping**:

| RPC Method | BlockStorage Method | Response Type |
|------------|---------------------|---------------|
| `eth_getBlockByNumber` | `get_block_by_number(n)` | `Option<alloy_rpc_types::Block>` |
| `eth_getBlockByHash` | `get_block_by_hash(&hash)` | `Option<alloy_rpc_types::Block>` |

**Source**: STRATEGY.md Stream 3, CRATES.md rpc-eth section, DOMAINS.md Integration Points 3 & 4
