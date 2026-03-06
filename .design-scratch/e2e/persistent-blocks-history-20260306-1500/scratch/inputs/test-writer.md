# Test Writer Input Pack

## INTENT Success Criteria

1. SC-1: Finalized blocks (header+body+txs+receipts) persisted to MDBX atomically
2. SC-2: Automatic persistence on finalization events (no manual trigger)
3. SC-3: `eth_getBlockByNumber(number|tag, full)` returns persisted block data
4. SC-4: `eth_getBlockByHash(hash, full)` returns persisted block data
5. SC-5: Node wiring integrates persistence + query without breaking existing flows

## Public Interfaces Per Crate

### state — `BlockStorage` trait (`src/block_storage.rs`) [NEW]
```
pub trait BlockStorage: Send + Sync {
  type Error: std::error::Error + Send + Sync + 'static;
  fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<(), Self::Error>;
  fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>, Self::Error>;
  fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>, Self::Error>;
  fn get_receipts_by_block(&self, number: u64) -> Result<Vec<Receipt>, Self::Error>;
}
```

### state-reth — `impl BlockStorage for RethStateDb` (`src/block_storage.rs`) [NEW]
- `store_block`: EvmBlock→Header via `build_header_from_evm_block`, decode txs, assign TxNumbers, single MDBX write tx to 8 tables
- `get_block_by_number`: Read Headers→BlockBodyIndices→Transactions, reconstruct EvmBlock
- `get_block_by_hash`: HeaderNumbers[hash]→number, delegate to get_block_by_number
- `get_receipts_by_block`: BlockBodyIndices→Receipts range read
- Internal: `next_tx_number()`, `reconstruct_evm_block()`, `encode_transaction()`

### app-evm — Modified `EvmApplication<DB>` (`src/executor.rs`, `src/lib.rs`) [MODIFIED]
- `pub fn build_header_from_evm_block(block: &EvmBlock) -> Header` — visibility change from private
- New field: `pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>` — set in propose(), consumed in finalization
- `pub fn store_finalized_block(&self, block: &EvmBlock) -> Result<(), EvmAppError>` — takes receipts, calls db.store_block()

### rpc-eth — Modified `EthApi` trait + handler (`src/eth_api.rs`, `src/eth_handler.rs`, `src/context.rs`) [MODIFIED]
- `EthRpcContext<S: StateDb, B: BlockStorage>` — new generic param B, new field `block_storage: Arc<RwLock<B>>`
- `async fn get_block_by_hash(&self, hash: B256, full: bool) -> RpcResult<Option<Block>>`
- `async fn get_block_by_number(&self, number: BlockNumberOrTag, full: bool) -> RpcResult<Option<Block>>`
- Internal: `evm_block_to_rpc_block(block, full)` conversion, `BlockNumberOrTag` resolution

### whirlpool-node — Binary wiring (`src/main.rs`) [MODIFIED]
- `PersistingFinalizationSink` wraps `EvmApplication` + `FinalizationSink`
- `EthRpcContext::new(tx_pool, state_db, state_db, chain_id)` — same RethStateDb for both StateDb and BlockStorage

### consensus-simplex — NO CHANGES

## Flow Steps (condensed)

**Flow 1 (Finalization→Storage)**: consensus emits Finalized→AppAdapter resolves block→sink.handle()→PersistingFinalizationSink calls evm_app.store_finalized_block(block)→pending_receipts.take()→db.store_block(block, receipts)→single MDBX write tx to 8 tables
**Flow 2 (getBlockByNumber)**: RPC request→resolve BlockNumberOrTag→storage.get_block_by_number(n)→MDBX read Headers+BlockBodyIndices+Transactions→reconstruct EvmBlock→evm_block_to_rpc_block→JSON response
**Flow 3 (getBlockByHash)**: RPC request→storage.get_block_by_hash(hash)→HeaderNumbers lookup→delegate to Flow 2 read path→response
**Flow 4 (Node Startup)**: open_state_db→EvmApplication::new→PersistingFinalizationSink→EthRpcContext::new(with block_storage)→engine start

## Domain Boundaries (test isolation)

1. Storage domain (state + state-reth): owns BlockStorage trait + MDBX impl — test with temp MDBX
2. Application domain (app-evm): owns receipt lifecycle + store_finalized_block — test with mock BlockStorage
3. RPC domain (rpc-eth): owns endpoint handlers + conversion — test with mock BlockStorage
4. Wiring domain (whirlpool-node): owns PersistingFinalizationSink — integration test only
5. Consensus domain (consensus-simplex): unchanged — no new tests

## Unknowns
- Receipt timing edge case: propose() without subsequent finalization leaves stale receipts
- EvmBlock reconstruction fidelity: round-trip EvmBlock→Header→EvmBlock may lose unmapped fields
