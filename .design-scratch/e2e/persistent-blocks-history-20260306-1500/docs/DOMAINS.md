# Domains & Wiring

## Domain Map

The persistent block storage feature spans five architectural domains:

### 1. Storage Domain
**Purpose**: Persistent state and block data management via MDBX backend

**Owning Crates**:
- `state` — Trait definitions (StateDb, BlockStorage)
- `state-reth` — MDBX implementation (RethStateDb with both StateDb and BlockStorage impls)
- `state-memory` — In-memory implementation (test/dev only, not involved in this feature)

**Key Types**:
- `StateDb` trait (existing): `state::traits::StateDb`
- `BlockStorage` trait (new): [PROPOSED] `state::block_storage::BlockStorage`
- `RethStateDb` struct: `state_reth::db::RethStateDb`
- `RethStateError`: `state_reth::error::RethStateError`

### 2. Consensus/Finalization Domain
**Purpose**: Block proposal, verification, and finalization orchestration

**Owning Crates**:
- `consensus` — Core traits (Block, ConsensusApp, EventSink, ConsensusEngine)
- `consensus-simplex` — Commonware Simplex adapter (AppAdapter, MailboxActor, FinalizationSink)

**Key Types**:
- `Block` trait: `consensus::block::Block`
- `ConsensusApp<Block>` trait: `consensus::app::ConsensusApp`
- `EventSink<Block>` trait: `consensus::event::EventSink`
- `ConsensusEvent<Block>`: `consensus::event::ConsensusEvent`
- `AppAdapter<A, S, B, Sig>`: `consensus_simplex::adapter::AppAdapter`
- `BlockStore<B>`: `consensus_simplex::BlockStore` (ephemeral HashMap)

### 3. Application Domain
**Purpose**: EVM execution, block building, and finalization handling

**Owning Crates**:
- `app` — Core types (EvmBlock, ExecutionResult, BlockId)
- `app-evm` — EVM execution via reth-evm (EvmApp, executor functions)

**Key Types**:
- `EvmBlock`: `app::types::EvmBlock`
- `ExecutionResult`: `app::types::ExecutionResult`
- `Receipt`: [PROPOSED] re-export from `alloy_consensus::Receipt`
- `EvmApp`: [PROPOSED] `app_evm::EvmApp` (to be modified with receipts field)
- Conversion functions: `app_evm::executor::build_header_from_evm_block`, `app_evm::executor::decode_transactions`

### 4. RPC/Query Domain
**Purpose**: Ethereum JSON-RPC API surface for external clients

**Owning Crates**:
- `rpc-eth` — Ethereum JSON-RPC endpoints (EthApiServer, EthRpcContext, eth_* method implementations)

**Key Types**:
- `EthApiServer` trait: `rpc_eth::eth_api::EthApiServer`
- `EthRpcContext<S>`: `rpc_eth::context::EthRpcContext`
- `ReceiptStore`: `rpc_eth::receipt_store::ReceiptStore` (in-memory)
- Response types: `alloy_rpc_types::Block`, `alloy_rpc_types::Transaction`, `alloy_rpc_types::ReceiptEnvelope`

### 5. Node Wiring Domain
**Purpose**: Component assembly and lifecycle management

**Owning Crates**:
- `whirlpool-node` — Binary entry point, component initialization

### Unaffected Crates (Not Mapped to Domains)
The following workspace crates require **no changes** and are not assigned to any domain for this feature:
- `consensus` — Core trait definitions, unchanged
- `p2p` — No block storage interaction
- `p2p-commonware` — No block storage interaction
- `state-memory` — In-memory StateDb impl, not involved in this feature
- `integration-tests` — May gain new e2e tests, but crate structure unchanged

---

## Domain Boundaries

### Storage Domain Public API

**Inbound Dependencies**:
- `alloy_genesis::GenesisAccount` (for StateDb initialization)
- `revm::database::BundleState` (for state commits)
- `revm::primitives::{Address, B256, U256}` (primitive types)
- `app::types::EvmBlock` [PROPOSED] (for BlockStorage)
- `alloy_consensus::Receipt` [PROPOSED] (for BlockStorage)

**Outbound API** (trait surface):
```rust
// Existing
pub trait StateDb {
    fn state_root(&self) -> Result<B256>;
    fn commit(&mut self, bundle: &BundleState) -> Result<()>;
    fn get_account(&self, address: Address) -> Result<Option<AccountInfo>>;
    fn get_block_hash(&self, number: u64) -> Result<B256>;
    fn insert_block_hash(&mut self, number: u64, hash: B256) -> Result<()>;
    // ... (other StateDb methods)
}

// [PROPOSED] New trait
pub trait BlockStorage: Send + Sync {
    fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<()>;
    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>>;
    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>>;
    fn get_receipts_by_block(&self, number: u64) -> Result<Vec<Receipt>>;
}
```

**Internal Implementation Details** (state-reth only):
- MDBX tables: Headers, HeaderNumbers, BlockBodyIndices, Transactions, TransactionHashNumbers, TransactionBlocks, Receipts
- Conversion: EvmBlock → reth Header via `build_header_from_evm_block()`
- Conversion: raw tx bytes → TransactionSigned via `decode_transactions()`
- Transaction numbering: global TxNumber counter derived from BlockBodyIndices

### Consensus/Finalization Domain Public API

**Inbound Dependencies**: None (pure generic traits)

**Outbound API**:
```rust
pub trait Block {
    type Id;
    fn id(&self) -> Self::Id;
    fn parent_id(&self) -> Self::Id;
    fn height(&self) -> u64;
}

pub trait ConsensusApp {
    type Block: Block;
    async fn genesis(&self) -> Self::Block;
    async fn propose(&self, parent: &Self::Block, height: u64) -> Option<Self::Block>;
    async fn verify(&self, parent: &Self::Block, block: &Self::Block) -> Result<()>;
}

pub trait EventSink {
    type Block: Block;
    async fn handle(&self, event: ConsensusEvent<Self::Block>);
}
```

**Key Constraint**: Generic `B: Block` prevents consensus-simplex from knowing about concrete EVM types or storage

**Finalization Flow** (consensus_simplex::adapter::AppAdapter):
1. Simplex → `AppAdapter::report(Activity::Finalization)`
2. Extract Digest from finalization payload
3. Retrieve block from ephemeral `BlockStore<B>` (HashMap)
4. Forward `ConsensusEvent::Finalized { block, height, proof }` to EventSink
5. Block removed from ephemeral store (no persistence at this layer)

Evidence: `consensus_simplex::adapter::AppAdapter::report` (lines 162-194)

### Application Domain Public API

**Inbound Dependencies**:
- `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEvent}` (implements these traits)
- `state::StateDb` (for state operations)
- [PROPOSED] `state::BlockStorage` (for block persistence)

**Outbound API**:
```rust
// app/src/types.rs
pub struct EvmBlock {
    pub height: u64,
    pub parent_id: [u8; 32],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub timestamp: u64,
    pub transactions: Vec<Vec<u8>>,  // raw tx bytes
}

pub struct ExecutionResult {
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub gas_used: u64,
    pub receipt_count: usize,
}

// [PROPOSED] app-evm receipts handling
impl EvmApp {
    // Store receipts during propose() for later retrieval
    fn propose(&self, parent: &EvmBlock, height: u64) -> Option<EvmBlock> { ... }
    
    // Persist block+receipts on finalization
    fn handle(&self, event: ConsensusEvent<EvmBlock>) {
        match event {
            ConsensusEvent::Finalized { block, .. } => {
                // Retrieve stored receipts
                // Call state_db.store_block(&block, &receipts)
            }
        }
    }
}
```

**Key Integration Point**: [PROPOSED] EvmApp will implement finalization persistence hook

Evidence: `app::types::EvmBlock` (lines 21-30), STRATEGY.md Stream 2

### RPC/Query Domain Public API

**Inbound Dependencies**:
- `state::StateDb` (for state queries)
- [PROPOSED] `state::BlockStorage` (for block queries)
- `app::tx_source::InMemoryTxPool` (for transaction pool)
- `alloy_primitives` types (B256, Address, U256)
- `alloy_rpc_types` (Block, Transaction, ReceiptEnvelope)

**Outbound API** (JSON-RPC methods):
```rust
// Existing methods (rpc_eth::eth_api::EthApiServer)
async fn eth_chain_id(&self) -> RpcResult<String>;
async fn eth_gas_price(&self) -> RpcResult<U256>;
async fn eth_get_balance(&self, address: Address, block_num: Option<BlockNumber>) -> RpcResult<U256>;
async fn eth_send_raw_transaction(&self, bytes: Bytes) -> RpcResult<B256>;
async fn eth_get_transaction_receipt(&self, hash: TxHash) -> RpcResult<Option<ReceiptEnvelope>>;

// [PROPOSED] New methods
async fn eth_get_block_by_hash(&self, hash: BlockHash, full: bool) -> RpcResult<Option<Block>>;
async fn eth_get_block_by_number(&self, number: BlockNumberOrTag, full: bool) -> RpcResult<Option<Block>>;
```

**Context Extension** [PROPOSED]:
```rust
pub struct EthRpcContext<S: StateDb> {
    pub tx_pool: Arc<InMemoryTxPool>,
    pub state_db: Arc<RwLock<S>>,
    pub receipt_store: Arc<ReceiptStore>,  // in-memory (pending receipts)
    pub block_storage: Arc<dyn BlockStorage>,  // [PROPOSED] persistent blocks
    pub chain_id: u64,
    pub block_height: Arc<AtomicU64>,
}
```

Evidence: `rpc_eth::context::EthRpcContext` (lines 13-19), STRATEGY.md Stream 3

---

## Cross-Domain Wiring

### 1. Storage ← Application (Finalization Persistence)

**Direction**: Application domain writes to Storage domain on finalization

**Integration Point**: [PROPOSED] `app_evm::EvmApp::handle(ConsensusEvent::Finalized)`

**Data Flow**:
1. EvmApp receives `ConsensusEvent::Finalized { block: EvmBlock, height, proof }`
2. EvmApp retrieves receipts from internal state (stored during `propose()`)
3. EvmApp calls `state_db.store_block(&block, &receipts)`
4. RethStateDb (impl BlockStorage) converts EvmBlock → reth types, writes to MDBX

**Type Boundary Crossings**:
- `EvmBlock` (app domain) → `Header` + `Vec<TransactionSigned>` (storage domain)
- `Vec<Receipt>` (app domain) → stored per-tx in Receipts table (storage domain)
- Conversion functions: `build_header_from_evm_block(&EvmBlock) -> Header`, `decode_transactions(&[Vec<u8>]) -> Vec<TransactionSigned>`

**Evidence**: STRATEGY.md Phase 2, app_evm::EvmApp proposed changes

### 2. Storage ← RPC (Block Queries)

**Direction**: RPC domain reads from Storage domain for eth_getBlock* endpoints

**Integration Point**: [PROPOSED] `rpc_eth::eth_rpc::EthRpcImpl::eth_get_block_by_*`

**Data Flow**:
1. JSON-RPC request → `eth_get_block_by_number(number, full)`
2. RPC handler calls `block_storage.get_block_by_number(number)`
3. RethStateDb reads MDBX tables, reconstructs EvmBlock from Header + Transactions
4. RPC handler converts EvmBlock → `alloy_rpc_types::Block` for response

**Type Boundary Crossings**:
- Query: `u64` (block number) or `[u8; 32]` (block hash) → MDBX lookup
- Response: `EvmBlock` (storage domain) → `alloy_rpc_types::Block` (RPC domain)
- Transaction conversion: raw bytes → `TransactionSigned` → `alloy_rpc_types::Transaction`

**Evidence**: STRATEGY.md Stream 3, CRATES.md rpc-eth section

### 3. Consensus → Application (Finalization Event)

**Direction**: Consensus domain forwards finalization to Application domain

**Integration Point**: `consensus_simplex::adapter::AppAdapter::report` → `EventSink::handle`

**Data Flow** (existing, no changes):
1. Simplex consensus finalizes block → `Activity::Finalization`
2. AppAdapter retrieves block from ephemeral BlockStore (HashMap)
3. AppAdapter calls `sink.handle(ConsensusEvent::Finalized { block, height, proof })`
4. EvmApp (implements EventSink) receives event

**Type Boundary Crossings**:
- `Digest` (consensus domain) → `EvmBlock` (application domain) via BlockStore lookup
- Generic `B: Block` constraint maintained (no concrete type dependency)

**Evidence**: `consensus_simplex::adapter::AppAdapter::report` (lines 162-178)

### 4. Application → Storage (State Updates)

**Direction**: Application domain writes state changes to Storage domain during execution

**Integration Point**: `app_evm::executor::execute_block` (existing, unchanged)

**Data Flow** (existing):
1. EvmApp proposes block → executes transactions via reth-evm
2. Execution produces `BundleState` (account/storage deltas)
3. EvmApp calls `state_db.commit(&bundle)`
4. RethStateDb writes to MDBX AccountsTrie, StoragesTrie tables

**Type Boundary Crossings**:
- `BundleState` (revm domain) → MDBX trie updates (storage domain)

**Evidence**: StateDb trait existing usage

### 5. RPC ← Application (Transaction Pool)

**Direction**: RPC domain forwards transactions to Application domain

**Integration Point**: `rpc_eth::eth_rpc::EthRpcImpl::eth_send_raw_transaction` (existing, unchanged)

**Data Flow** (existing):
1. JSON-RPC request → `eth_send_raw_transaction(bytes)`
2. RPC handler validates transaction, adds to InMemoryTxPool
3. Transaction available for next block proposal

**Evidence**: Existing rpc-eth implementation

---

## Data Flow Sequences

### Sequence 1: Store Finalized Block

**Trigger**: Consensus finalizes block

**Flow**:
1. **Simplex** (consensus-simplex): Finalization decision → `Activity::Finalization`
2. **AppAdapter** (consensus-simplex): Extract Digest, lookup block in ephemeral BlockStore
3. **AppAdapter** → **EventSink** (app-evm): Forward `ConsensusEvent::Finalized { block: EvmBlock, height, proof }`
4. **EvmApp** (app-evm): Retrieve receipts from internal state (populated during propose)
5. **EvmApp** → **BlockStorage** (state-reth): Call `store_block(&block, &receipts)`
6. **RethStateDb** (state-reth):
   - Convert `EvmBlock` → `Header` via `build_header_from_evm_block()`
   - Decode `block.transactions` → `Vec<TransactionSigned>` via `decode_transactions()`
   - Open MDBX write transaction
   - Write Header to Headers table (BlockNumber → Header)
   - Write HeaderNumbers entry (BlockHash → BlockNumber)
   - Assign global TxNumbers, write Transactions, TransactionHashNumbers, TransactionBlocks
   - Write BlockBodyIndices (BlockNumber → {first_tx_num, tx_count})
   - Write Receipts (TxNumber → Receipt)
   - Commit transaction
7. **FinalizationSink** (consensus-simplex): Update `Arc<AtomicU64>` height counter

**Cross-Domain Hops**: Consensus → Application → Storage

### Sequence 2: Query Block by Number

**Trigger**: External client calls `eth_getBlockByNumber(10, true)`

**Flow**:
1. **JSON-RPC server** (rpc-eth): Deserialize request, route to `EthRpcImpl::eth_get_block_by_number`
2. **EthRpcImpl** (rpc-eth): Parse `BlockNumberOrTag::Number(10)`
3. **EthRpcImpl** → **BlockStorage** (state-reth): Call `get_block_by_number(10)`
4. **RethStateDb** (state-reth):
   - Open MDBX read transaction
   - Read Headers[10] → Header
   - Read BlockBodyIndices[10] → {first_tx_num: 50, tx_count: 3}
   - Read Transactions[50], Transactions[51], Transactions[52] → Vec<TransactionSigned>
   - Convert Header → EvmBlock fields
   - RLP-encode TransactionSigned → raw tx bytes for EvmBlock.transactions
   - Return `Some(EvmBlock)`
5. **EthRpcImpl** (rpc-eth): Convert EvmBlock → `alloy_rpc_types::Block`
   - Decode transactions: raw bytes → TransactionSigned → alloy Transaction
   - Include full transaction objects (full=true) or just hashes (full=false)
6. **JSON-RPC server** (rpc-eth): Serialize response, return to client

**Cross-Domain Hops**: RPC → Storage

### Sequence 3: Query Block by Hash

**Trigger**: External client calls `eth_getBlockByHash(0xabcd..., false)`

**Flow**:
1. **JSON-RPC server** (rpc-eth): Route to `EthRpcImpl::eth_get_block_by_hash`
2. **EthRpcImpl** → **BlockStorage** (state-reth): Call `get_block_by_hash(&hash)`
3. **RethStateDb** (state-reth):
   - Read HeaderNumbers[hash] → BlockNumber
   - Proceed as in Sequence 2 (query by number)
4. **EthRpcImpl** (rpc-eth): Convert EvmBlock → `alloy_rpc_types::Block` (full=false → tx hashes only)
5. Return response

**Cross-Domain Hops**: RPC → Storage

### Sequence 4: Query Transaction Receipt

**Trigger**: External client calls `eth_getTransactionReceipt(tx_hash)`

**Flow** (MVP — in-memory only, per BLK-3 deferral):
1. **JSON-RPC server** (rpc-eth): Route to `EthRpcImpl::eth_get_transaction_receipt`
2. **EthRpcImpl** (rpc-eth): Look up in-memory `ReceiptStore` (contains finalized receipts pushed during finalization)
3. Return result (or null if not found)

> **Post-MVP**: Extend with persistent receipt fallback via `BlockStorage::get_receipts_by_block()` + TransactionHashNumbers lookup. See BLK-3 in BLOCKERS.md.

**Cross-Domain Hops**: RPC only (no Storage hop in MVP)

**Note**: This is a [PROPOSED] enhancement to existing `eth_get_transaction_receipt` to support historical queries

---

## Type Boundaries

### Types Crossing Domain Boundaries

| Type | Source Crate | Target Domain | Conversion Required | Evidence |
|------|--------------|---------------|---------------------|----------|
| `EvmBlock` | app | Storage | Yes (→ Header + Vec<TransactionSigned>) | app::types (lines 21-30) |
| `Receipt` | alloy-consensus | Storage, RPC | No (stored as-is with Compact encoding) | [PROPOSED] |
| `BundleState` | revm | Storage | Yes (→ trie updates) | state::StateDb trait |
| `ConsensusEvent<EvmBlock>` | consensus | Application | No (generic trait) | consensus::event |
| `alloy_rpc_types::Block` | alloy-rpc-types | RPC response | Yes (← EvmBlock conversion) | [PROPOSED] |
| `TransactionSigned` | alloy-consensus | Storage, RPC | Yes (raw bytes ↔ TransactionSigned) | EXPLORATION.md |
| `Header` | alloy-consensus | Storage | Yes (← EvmBlock conversion) | STRATEGY.md |
| `B256`, `Address`, `U256` | alloy-primitives | All domains | No (shared primitive) | state::StateDb trait |

### Conversion Function Locations

| Conversion | Direction | Location | Visibility |
|------------|-----------|----------|------------|
| EvmBlock → Header | app → storage | app_evm::executor::build_header_from_evm_block | [PROPOSED] pub (currently private) |
| raw bytes → TransactionSigned | app → storage | app_evm::executor::decode_transactions | [PROPOSED] pub (currently private) |
| TransactionSigned → raw bytes | storage → app | RLP encoding (reth built-in) | N/A |
| EvmBlock → alloy_rpc_types::Block | storage → RPC | [PROPOSED] rpc_eth::conversions (new) | pub |
| Receipt → ReceiptEnvelope | storage → RPC | [PROPOSED] rpc_eth::conversions (new) | pub |

**Evidence**: STRATEGY.md Key Design Decision 1, CRATES.md app-evm section

### Type Encoding Boundaries

| Type | Encoding at Rest | Encoding in Transit | Evidence |
|------|------------------|---------------------|----------|
| `EvmBlock` | Compact (as Header) in MDBX | commonware_codec in consensus | app::types (lines 84-99) |
| `TransactionSigned` | Compact in MDBX | RLP in raw bytes field | EXPLORATION.md section 2 |
| `Receipt` | Compact in MDBX | JSON in RPC response | [PROPOSED] |
| `Header` | Compact in MDBX | N/A (internal) | state-reth tables |

**Key Constraint**: EvmBlock uses commonware_codec but MDBX requires Compact trait → conversion via Header is mandatory

**Evidence**: EXPLORATION.md section 3 "Encoding Challenge"

---

## Integration Points

### Integration Point 1: EvmApp Finalization Handler

**Location**: [PROPOSED] `app_evm::EvmApp::handle(ConsensusEvent::Finalized)`

**Current State**: EvmApp implements `EventSink<Block = EvmBlock>`, but `handle()` method currently does not persist blocks

**Proposed Change**:
```rust
// app-evm/src/lib.rs
impl EvmApp {
    // New field
    receipts: Option<Vec<Receipt>>,

    // Modified method
    async fn handle(&self, event: ConsensusEvent<EvmBlock>) {
        match event {
            ConsensusEvent::Finalized { block, .. } => {
                if let Some(receipts) = self.receipts.take() {
                    if let Err(e) = self.state_db.store_block(&block, &receipts) {
                        tracing::error!(?e, "failed to persist finalized block");
                    }
                }
            }
        }
    }
}
```

**Dependencies**:
- Requires `BlockStorage` trait bound on `state_db` field
- Requires receipts storage during `propose()`

**Evidence**: STRATEGY.md Stream 2, CRATES.md app-evm section

### Integration Point 2: RethStateDb BlockStorage Implementation

**Location**: [PROPOSED] `state_reth::block_storage::impl BlockStorage for RethStateDb`

**Current State**: RethStateDb implements `StateDb`, has access to MDBX DatabaseEnv

**Proposed Structure**:
```rust
// state-reth/src/block_storage.rs
impl BlockStorage for RethStateDb {
    fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<()> {
        // 1. Convert EvmBlock → Header via build_header_from_evm_block()
        // 2. Decode transactions via decode_transactions()
        // 3. Compute next TxNumber from last BlockBodyIndices
        // 4. Open MDBX write transaction
        // 5. Write all tables (Headers, HeaderNumbers, Transactions, etc.)
        // 6. Commit transaction
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>> {
        // 1. Open MDBX read transaction
        // 2. Read Header from Headers[number]
        // 3. Read BlockBodyIndices to get tx range
        // 4. Read transactions, RLP-encode to raw bytes
        // 5. Reconstruct EvmBlock
    }

    // ... (other methods)
}
```

**Dependencies**:
- Requires `build_header_from_evm_block` and `decode_transactions` to be public in app-evm
- Requires `EvmBlock` and `Receipt` types in scope

**Evidence**: STRATEGY.md Stream 1, EXPLORATION.md section 2

### Integration Point 3: EthRpcContext BlockStorage Field

**Location**: [PROPOSED] `rpc_eth::context::EthRpcContext`

**Current State**: EthRpcContext has `state_db: Arc<RwLock<S>>` where `S: StateDb`

**Proposed Change**:
```rust
// rpc-eth/src/context.rs
pub struct EthRpcContext<S: StateDb> {
    pub tx_pool: Arc<InMemoryTxPool>,
    pub state_db: Arc<RwLock<S>>,
    pub receipt_store: Arc<ReceiptStore>,
    pub block_storage: Arc<dyn BlockStorage>,  // NEW
    pub chain_id: u64,
    pub block_height: Arc<AtomicU64>,
}
```

**Wiring in whirlpool-node**:
```rust
// whirlpool-node/src/main.rs or src/node.rs
let state_db = Arc::new(RwLock::new(open_state_db(db_path)?));

let rpc_context = EthRpcContext {
    tx_pool,
    state_db: Arc::clone(&state_db),
    block_storage: state_db.clone(),  // RethStateDb impls both StateDb and BlockStorage
    receipt_store,
    chain_id,
    block_height,
};
```

**Dependencies**:
- Requires `BlockStorage` trait in scope
- RethStateDb must implement `BlockStorage`

**Evidence**: STRATEGY.md Stream 3, CRATES.md rpc-eth and whirlpool-node sections

### Integration Point 4: RPC Block Query Endpoints

**Location**: [PROPOSED] `rpc_eth::eth_rpc::EthRpcImpl`

**Current State**: EthApiServer trait has 7 methods, no block query endpoints

**Proposed Addition**:
```rust
// rpc-eth/src/eth_api.rs
#[async_trait]
pub trait EthApiServer {
    // ... existing methods ...

    async fn eth_get_block_by_hash(
        &self,
        hash: B256,
        full: bool,
    ) -> RpcResult<Option<alloy_rpc_types::Block>>;

    async fn eth_get_block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<alloy_rpc_types::Block>>;
}

// rpc-eth/src/eth_rpc.rs
impl<S: StateDb> EthRpcImpl<S> {
    async fn eth_get_block_by_number(
        &self,
        number: BlockNumberOrTag,
        full: bool,
    ) -> RpcResult<Option<alloy_rpc_types::Block>> {
        // 1. Resolve BlockNumberOrTag to u64
        // 2. Query block_storage.get_block_by_number()
        // 3. Convert EvmBlock → alloy_rpc_types::Block
        // 4. Return response
    }
}
```

**Dependencies**:
- Requires `block_storage` field in `EthRpcContext`
- Requires EvmBlock → alloy_rpc_types::Block conversion logic

**Evidence**: STRATEGY.md Stream 3, CRATES.md rpc-eth section

### Integration Point 5: Conversion Function Exports

**Location**: [PROPOSED] `app_evm::executor` module

**Current State**: `build_header_from_evm_block` and `decode_transactions` are module-private

**Proposed Change**:
```rust
// app-evm/src/executor.rs
pub fn build_header_from_evm_block(block: &EvmBlock, seal: B256) -> Header {
    // ... existing implementation ...
}

pub fn decode_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<TransactionSigned>, EvmAppError> {
    // ... existing implementation ...
}
```

**Rationale**: state-reth needs these functions for EvmBlock → reth type conversion

**Evidence**: STRATEGY.md Stream 1, CRATES.md app-evm section

---

## Cross-Domain Dependencies Summary

| From Domain | To Domain | Dependency Type | Interface |
|-------------|-----------|-----------------|-----------|
| Application | Storage | Write (persist) | BlockStorage::store_block |
| RPC | Storage | Read (query) | BlockStorage::get_block_by_* |
| Application | Storage | Write (state) | StateDb::commit |
| RPC | Storage | Read (state) | StateDb::get_account, get_storage |
| Consensus | Application | Event (finalization) | EventSink::handle |
| Consensus | Application | Callback (propose/verify) | ConsensusApp::propose, verify |
| Application | Consensus | Data (block) | Block trait implementation |
| Node Wiring | All | Initialization | Component assembly in main() |

**Architectural Constraint**: Consensus domain remains generic over `B: Block`, preventing it from depending on concrete EVM types or storage implementations. Persistence happens at Application layer boundary.

**Evidence**: STRATEGY.md Key Design Decision 2, consensus_simplex::adapter generic constraints

---

## Open Questions / Unknowns

### Q1: Receipt Storage Timing
**Status**: UNKNOWN — decision needed

**Question**: Should receipts be persisted on propose (immediately) or only on finalization?

**Impact**: Affects availability of pending block receipts, write load, and reorg handling

**Recommendation from STRATEGY.md**: Store on finalization only (Option A) for MVP

### Q2: Historical Receipt Query Integration
**Status**: [PROPOSED] — needs verification

**Question**: Should `eth_get_transaction_receipt` check both in-memory ReceiptStore AND persistent BlockStorage?

**Impact**: Affects completeness of receipt queries for finalized transactions

**Recommendation from STRATEGY.md**: Yes — fall back to BlockStorage for finalized receipts

### Q3: Block Reconstruction Performance
**Status**: UNKNOWN — profile needed

**Question**: Is reconstructing EvmBlock from Header + Transactions performant enough for RPC queries?

**Alternative**: Store EvmBlock bytes directly in custom MDBX table (duplicates data)

**Recommendation from STRATEGY.md**: Stick with reconstruction for MVP, profile if issues arise

---

## Blockers

**BLOCKERS_FOUND: none**

All required information for domain boundary design is available from exploration and strategy docs. Implementation blockers (if any) will be discovered during coding phase.
