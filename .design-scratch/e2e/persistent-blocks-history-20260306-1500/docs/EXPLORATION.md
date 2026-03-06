# Exploration

## 1. Block Storage Architecture (Current)

### In-Memory BlockStore (consensus-simplex)
- `BlockStore<B>`: `Arc<RwLock<HashMap<Digest, B>>>` — ephemeral, consensus-round scoped
- Shared between MailboxActor (inserts proposed/verified blocks) and AppAdapter (reads on finalization)
- Blocks DROPPED after finalization processing — no persistence

### StateDb Block Hash Mapping
- `insert_block_hash(number, hash)` / `get_block_hash(number)` in StateDb trait
- state-memory: HashMap<u64, B256>
- state-reth: MDBX CanonicalHeaders table (number → hash)
- This stores ONLY the hash-by-number mapping, NOT full blocks

### Finalization Flow
1. Simplex → AppAdapter::report(Activity::Finalization)
2. Extracts Digest from fin.proposal.payload, finds block in BlockStore
3. Computes height, forwards ConsensusEvent::Finalized(block) to EventSink
4. FinalizationSink::handle updates Arc<AtomicU64> height + logs
5. Block is consumed and dropped — no persistence side effect

## 2. reth-db MDBX Tables (Available)

Already initialized by `init_db()` in state-reth — ALL tables exist:

| Table | Key | Value | Purpose |
|---|---|---|---|
| CanonicalHeaders | BlockNumber | HeaderHash | Number→hash (USED by state-reth) |
| HeaderNumbers | BlockHash | BlockNumber | Hash→number (reverse lookup) |
| Headers | BlockNumber | Header | Block headers |
| BlockBodyIndices | BlockNumber | StoredBlockBodyIndices | Maps block→{first_tx_num, tx_count} |
| Transactions | TxNumber | TransactionSigned | Individual txs by global index |
| TransactionHashNumbers | TxHash | TxNumber | Tx hash→number lookup |
| TransactionBlocks | TxNumber | BlockNumber | Tx→block reverse lookup |
| Receipts | TxNumber | Receipt | Per-tx receipts |
| BlockOmmers | BlockNumber | StoredBlockOmmers | Uncle blocks |
| BlockWithdrawals | BlockNumber | StoredBlockWithdrawals | Withdrawals |
| HeaderTerminalDifficulties | BlockNumber | CompactU256 | PoW difficulty |

Tables NOT needed for our use case: BlockOmmers, HeaderTerminalDifficulties, BlockWithdrawals (PoW/beacon artifacts).

## 3. Block Types

### EvmBlock (crates/app/src/types.rs)
```
struct EvmBlock {
    height: u64,
    parent_id: [u8; 32],
    state_root: [u8; 32],
    transactions_root: [u8; 32],
    receipts_root: [u8; 32],
    gas_used: u64,
    timestamp: u64,
    transactions: Vec<Vec<u8>>,  // raw tx bytes
}
```
- Uses commonware_codec (binary), NOT serde/RLP/Compact
- id() computed via sha256(height + parent_id + state_root + tx_root)
- No direct mapping to alloy/reth Header type

### Encoding Challenge
- reth-db tables expect `Compact` trait encoding
- EvmBlock uses `commonware_codec` encoding
- **MISMATCH**: Cannot directly store EvmBlock in reth-db Headers table
- Options: (a) custom table with raw codec bytes, (b) convert to reth Header, (c) new Compact impl

## 4. RPC Layer (Current)

### Existing
- Framework: jsonrpsee 0.26
- 7 methods: chainId, gasPrice, getBalance, getTransactionCount, sendRawTransaction, estimateGas, getTransactionReceipt
- NO block endpoints
- EthRpcContext has: state_db, tx_pool, receipt_store, chain metadata — no block store

### Extension Points
- Add methods to EthApiServer trait (eth_api.rs)
- Add block data source to EthRpcContext (context.rs)
- alloy_rpc_types::Block available for response types

## 5. Dependency Graph

state-reth already depends on:
- reth-db (mdbx backend)
- reth-db-api (table definitions, Compact trait)
- alloy-primitives
- reth-trie, reth-trie-common (for state root computation)

rpc-eth depends on:
- app (EvmBlock type), state (StateDb trait)
- jsonrpsee, alloy-rpc-types, alloy-primitives
- Does NOT depend on state-reth or reth-db

## 6. Prior Art
- state-reth completion (persistent-state-rethdb-20260305-1347): established MDBX patterns, error taxonomy, per-method txn pattern, init_db reuse
