# STRATEGY.md

## Overview

Add persistent block storage and history queries to Whirlpool via three coordinated streams: (1) BlockStorage trait/impl using existing MDBX tables (Headers, BlockBodyIndices, Transactions, Receipts), (2) finalization hook at application layer to persist blocks with receipts, (3) eth_getBlock* RPC endpoints querying the new storage. Reuse existing EvmBlock→reth Header conversion functions (`build_header_from_evm_block`, `decode_transactions`) and follow established state-reth patterns (trait in `state`, MDBX impl in `state-reth`).

## Implementation Streams

### Stream 1: BlockStorage Layer
**Crates**: `state` (trait), `state-reth` (impl)

**Trait Definition** (`state/src/block_storage.rs`):
```rust
pub trait BlockStorage: Send + Sync {
    fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<()>;
    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>>;
    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>>;
    fn get_receipts_by_block(&self, number: u64) -> Result<Vec<Receipt>>;
}
```

**Implementation** (`state-reth/src/block_storage.rs`):
- Use existing MDBX tables created by `init_db()`:
  - `Headers(BlockNumber → Header)`: Store reth Header via `build_header_from_evm_block()`
  - `HeaderNumbers(BlockHash → BlockNumber)`: Reverse lookup
  - `BlockBodyIndices(BlockNumber → StoredBlockBodyIndices)`: Track first_tx_num + tx_count
  - `Transactions(TxNumber → TransactionSigned)`: Store decoded txs via `decode_transactions()`
  - `TransactionHashNumbers(TxHash → TxNumber)`: Tx hash lookup
  - `TransactionBlocks(TxNumber → BlockNumber)`: Tx→block reverse mapping
  - `Receipts(TxNumber → Receipt)`: Per-tx receipts
- **NOT needed**: BlockOmmers, HeaderTerminalDifficulties, BlockWithdrawals
- **Write strategy**: Single MDBX write transaction per block, batch all inserts
- **Read strategy**: Retrieve header + reconstruct EvmBlock by reading txs via BlockBodyIndices
- **Error handling**: Use existing 4-tier taxonomy (Database, State, Internal, Config) from state-reth

**Key conversion functions** (already exist in `app-evm/src/executor.rs`):
- `build_header_from_evm_block(&EvmBlock, seal: B256) -> Header` — converts EvmBlock to reth Header with Compact encoding
- `decode_transactions(raw_txs: &[Vec<u8>]) -> Vec<TransactionSigned>` — recovers typed transactions from raw bytes

**Transaction numbering**: Maintain global `TxNumber` counter (append-only). Store last used TxNumber in new MDBX cursor or compute from BlockBodyIndices on startup.

### Stream 2: Finalization Persistence Hook
**Crates**: `app`, `app-evm`, `consensus-simplex`, `consensus`

**Problem**: Currently, receipts are computed during `EvmApp::propose()` execution but only `ExecutionResult { state_root, receipts_root, gas_used, receipt_count }` flows out. Full receipt data is lost.

**Solution**: Extend the finalization event to carry receipts.

**Changes**:
1. **app-evm/src/lib.rs (EvmApp)**:
   - Store receipts in EvmApp state during `propose()` (currently discarded after computing receipts_root)
   - Add `receipts: Option<Vec<Receipt>>` field to EvmApp struct (populated during propose, cleared after finalization)
   - In `EvmApp::handle(ConsensusEvent::Finalized(block))`, retrieve receipts from state and call `BlockStorage::store_block(&block, &receipts)`

2. **state-reth/src/lib.rs**:
   - Add `BlockStorage` trait bound to RethStateDb struct (already has `StateDb` impl)
   - Implement `BlockStorage` for RethStateDb (reuse same `env: Arc<DatabaseEnv>`)

3. **app/src/types.rs**:
   - EvmBlock already has all required fields; no changes needed
   - Ensure receipts flow: propose → EvmApp state → finalization → BlockStorage

4. **consensus-simplex/src/finalization_sink.rs**:
   - No changes required — persistence happens at application layer (EvmApp), not consensus layer
   - Generic `B: Block` constraint prevents consensus-simplex from knowing about concrete storage types

**Finalization flow** (updated):
1. Simplex → AppAdapter::report(Finalization)
2. AppAdapter retrieves block from BlockStore, forwards to EventSink
3. **NEW**: EvmApp::handle(Finalized(block)) retrieves receipts from EvmApp state, calls `state_db.store_block(&block, &receipts)`
4. FinalizationSink updates height counter (unchanged)

### Stream 3: RPC Endpoints
**Crates**: `rpc-eth`

**Add endpoints** (`rpc-eth/src/eth_api.rs`):
```rust
// In EthApiServer trait:
async fn eth_get_block_by_hash(&self, hash: BlockHash, full: bool) -> RpcResult<Option<RpcBlock>>;
async fn eth_get_block_by_number(&self, number: BlockNumberOrTag, full: bool) -> RpcResult<Option<RpcBlock>>;
```

**EthRpcContext changes** (`rpc-eth/src/context.rs`):
- Add `block_storage: Arc<dyn BlockStorage>` field
- Construct from `RethStateDb` (which implements BlockStorage) in whirlpool-node

**Response conversion** (`rpc-eth/src/eth_rpc.rs`):
- Convert `EvmBlock + Vec<Receipt>` → `alloy_rpc_types::Block`
- Map transactions: raw bytes → TransactionSigned → alloy Transaction
- Map receipts → alloy ReceiptEnvelope
- Handle `full: bool` (include full tx objects vs just hashes)

**Special handling**:
- `BlockNumberOrTag::Latest` / `Pending` / `Finalized`: query current finalized height from FinalizationSink, then fetch by number
- `BlockNumberOrTag::Number(n)`: direct query via `get_block_by_number`

## Key Design Decisions

### 1. Type Conversion Strategy
**Decision**: Use existing `build_header_from_evm_block()` and `decode_transactions()` to convert EvmBlock → reth types for storage.

**Rationale**: EvmBlock uses commonware_codec binary encoding, but reth-db tables require Compact trait encoding. Direct storage is impossible. Conversion functions already exist in `app-evm/src/executor.rs` for state root computation. Risk R1 (type encoding mismatch) resolved by this finding.

**Implementation**: In `BlockStorage::store_block()`:
1. Convert `&EvmBlock` → `Header` via `build_header_from_evm_block(block, seal)`
2. Decode `block.transactions` → `Vec<TransactionSigned>` via `decode_transactions(&block.transactions)`
3. Store Header (Compact), TransactionSigned (Compact), Receipt (Compact) in respective tables

### 2. Persistence Hook Location
**Decision**: Persistence at application layer (EvmApp), not consensus-simplex layer.

**Rationale**: consensus-simplex uses generic `B: Block` and cannot know about concrete EVM types (Header, TransactionSigned). Persistence requires EVM-specific conversions. Risk R5 (generic type constraint) mitigated by this architectural choice.

**Trade-off**: Non-EVM applications must implement their own persistence. Acceptable because Whirlpool is EVM-focused.

### 3. Receipt Flow
**Decision**: Store receipts in EvmApp state during `propose()`, retrieve and persist during finalization.

**Rationale**: Receipts are computed during execution but currently discarded after receipts_root computation. Finalization path has no access to receipt data. Extending finalization event to carry receipts would require consensus-simplex changes. Storing in EvmApp state is simpler and isolated to application layer. Risk R3 (receipt reconciliation) mitigated by this design.

**Alternative rejected**: Re-execute block on query to reconstruct receipts. Too expensive for RPC hot path.

### 4. MDBX Table Usage
**Decision**: Reuse all existing block tables created by `init_db()`. Do NOT create custom tables.

**Rationale**: `init_db()` already creates Headers, BlockBodyIndices, Transactions, Receipts, etc. These are standard reth-db tables with proper Compact encoding. Creating parallel custom tables would duplicate data and diverge from reth conventions. Tables are currently empty but structurally ready.

**Tables used**:
- Headers, HeaderNumbers: Header storage and lookup
- BlockBodyIndices: Track transaction range per block
- Transactions, TransactionHashNumbers, TransactionBlocks: Transaction storage and reverse lookup
- Receipts: Per-transaction receipt storage

### 5. Transaction Numbering
**Decision**: Use global append-only TxNumber counter, stored implicitly via BlockBodyIndices.

**Rationale**: reth-db Transactions table is keyed by TxNumber (global transaction index, not per-block). BlockBodyIndices maps `BlockNumber → (first_tx_num, tx_count)`. On startup, reconstruct next TxNumber by reading last block's BlockBodyIndices entry.

**Implementation**: In `store_block()`:
1. Query last block's BlockBodyIndices to get `last_first_tx_num + tx_count`
2. Assign TxNumbers starting from `last_tx_num + 1`
3. Insert transactions with sequential TxNumbers
4. Store BlockBodyIndices for new block

## Crate Changes

### state (trait definitions)
- **New file**: `src/block_storage.rs` with BlockStorage trait
- **Changes to lib.rs**: Export BlockStorage trait
- **Dependencies**: Add `alloy-consensus` for Receipt type (or re-export from app)

### state-reth (MDBX implementation)
- **New file**: `src/block_storage.rs` implementing BlockStorage for RethStateDb
- **Changes to lib.rs**: Implement BlockStorage trait for RethStateDb struct
- **Table usage**: Headers, HeaderNumbers, BlockBodyIndices, Transactions, TransactionHashNumbers, TransactionBlocks, Receipts (all already created by init_db)
- **Error handling**: Reuse existing `StateError` enum with Database/State/Internal variants
- **Dependencies**: No new dependencies (already has reth-db, alloy-primitives)

### app (types)
- **Changes to types.rs**: No structural changes to EvmBlock
- **New**: Re-export Receipt type from alloy-consensus for BlockStorage trait signature

### app-evm (EVM execution)
- **Changes to lib.rs**:
  - Add `receipts: Option<Vec<Receipt>>` field to EvmApp struct
  - Store receipts during `propose()` execution (currently discarded after `receipts_root()` call)
  - In `handle(Finalized)`, retrieve receipts and call `state_db.store_block(&block, &receipts)`
  - Clear stored receipts after persistence
- **Changes to executor.rs**: Export `build_header_from_evm_block` and `decode_transactions` as public (currently module-private)
- **Dependencies**: No new dependencies

### consensus-simplex (consensus adapter)
- **Changes**: None required — persistence happens at application layer
- **Alternative considered**: Extend AppAdapter with persistence hook. Rejected due to generic B: Block constraint.

### rpc-eth (RPC endpoints)
- **Changes to eth_api.rs**: Add `eth_get_block_by_hash` and `eth_get_block_by_number` to EthApiServer trait
- **Changes to context.rs**: Add `block_storage: Arc<dyn BlockStorage>` field to EthRpcContext
- **Changes to eth_rpc.rs**:
  - Implement new endpoints
  - Convert EvmBlock → alloy_rpc_types::Block
  - Handle BlockNumberOrTag variants (Latest, Pending, Finalized, Number)
- **Dependencies**: No new dependencies (already has alloy-rpc-types)

### whirlpool-node (binary)
- **Changes to node.rs**:
  - Pass `state_db` (RethStateDb) to rpc-eth as `block_storage` parameter
  - No new initialization required (BlockStorage impl is on existing RethStateDb instance)

## Dependencies

### No New Crate Dependencies
All required dependencies already exist via state-reth:
- `reth-db` (MDBX backend) — already present
- `reth-db-api` (table definitions, Compact trait) — already present
- `alloy-primitives` (B256, Address, etc.) — already present
- `alloy-consensus` (Receipt, Header, TransactionSigned) — already present via reth types

### Internal Dependencies (Updated)
- `rpc-eth` gains dependency on `state` crate (for BlockStorage trait)
- `app-evm` uses `state-reth` for persistence (already depends on `state`)

## Ordering

### Phase 1: BlockStorage Trait + Implementation (Foundation)
**Duration**: ~3-4 hours
1. Define BlockStorage trait in `state/src/block_storage.rs`
2. Export `build_header_from_evm_block` and `decode_transactions` from `app-evm/src/executor.rs`
3. Implement BlockStorage for RethStateDb in `state-reth/src/block_storage.rs`
4. Write unit tests for storage/retrieval round-trip

**Verification**: `cargo test -p state-reth` passes, block storage functions work independently

### Phase 2: Finalization Persistence Hook (Integration)
**Duration**: ~2-3 hours
1. Add `receipts: Option<Vec<Receipt>>` field to EvmApp struct
2. Store receipts during `EvmApp::propose()` execution
3. Call `state_db.store_block(&block, &receipts)` in `EvmApp::handle(Finalized)`
4. Test end-to-end: propose → finalize → verify block persisted in MDBX

**Verification**: `cargo test -p app-evm` passes, finalized blocks appear in database

### Phase 3: RPC Endpoints (API Surface)
**Duration**: ~2-3 hours
1. Add BlockStorage to EthRpcContext
2. Implement eth_get_block_by_hash and eth_get_block_by_number
3. Add EvmBlock → alloy_rpc_types::Block conversion
4. Wire block_storage in whirlpool-node

**Verification**: `cargo test -p rpc-eth` passes, manual RPC query returns block data

**Dependencies**:
- Phase 2 depends on Phase 1 (needs BlockStorage trait)
- Phase 3 depends on Phase 1 (needs BlockStorage trait), but can partially proceed in parallel with Phase 2
- Final integration requires all three phases complete

### Testing Strategy
1. **Unit tests**: state-reth block storage round-trip
2. **Integration tests**: app-evm propose → finalize → query
3. **E2E tests**: whirlpool-node startup → RPC query
4. **Performance test**: Finalization latency with MDBX writes (expect <5ms overhead)

## Open Questions

### Q1: Receipt Storage Timing
**Question**: Should receipts be persisted immediately after execution (in propose path) or only on finalization?

**Options**:
- A. Store on finalization only (current design): Simpler, fewer writes, but receipts unavailable for unfinalized blocks
- B. Store on propose, mark as "pending": Allows RPC queries for pending blocks, but increases write load and requires cleanup on reorg

**Recommendation**: Option A for MVP. Extend to Option B if pending block queries are needed.

### Q2: Block Reconstruction Strategy
**Question**: On `get_block_by_number()`, reconstruct full EvmBlock from Header + Transactions, or store EvmBlock bytes directly?

**Current design**: Reconstruct EvmBlock from reth types (Header → EvmBlock fields, Transactions → raw bytes via RLP encoding)

**Trade-off**: Reconstruction adds CPU cost but avoids duplicate storage. Storing EvmBlock directly would require custom MDBX table and duplicate transaction data.

**Recommendation**: Stick with reconstruction for MVP. Profile if RPC queries show performance issues.

### Q3: Historical Receipt Queries
**Question**: Should eth_getTransactionReceipt query both in-memory ReceiptStore (for pending txs) AND persistent Receipts table (for finalized txs)?

**Current state**: ReceiptStore in rpc-eth is in-memory, populated during execution.

**Recommendation**: Yes — eth_getTransactionReceipt should check in-memory first, fall back to BlockStorage. However, this is **DEFERRED to post-MVP** per BLK-3 in BLOCKERS.md. MVP scope covers only `eth_getBlockByHash` and `eth_getBlockByNumber`.

### Q4: State Root Verification
**Question**: Should block storage verify state_root matches the current StateDb state during persistence?

**Trade-off**: Adds safety (catch state corruption) but increases finalization latency.

**Recommendation**: No verification for MVP. State root is already verified during consensus. Add as optional debug feature later if needed.

### Q5: Block Pruning / Archival Policy
**Question**: Should there be a configurable retention policy (keep last N blocks, prune older)?

**Scope**: Out of scope for MVP. MDBX handles compaction automatically. Add pruning in future if disk usage becomes issue.

**Recommendation**: Defer to future work. Document as limitation in README.
