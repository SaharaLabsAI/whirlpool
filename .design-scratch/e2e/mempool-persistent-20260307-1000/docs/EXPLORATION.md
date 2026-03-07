# EXPLORATION

## Summary

Seven exploration agents investigated the Whirlpool codebase across four dimensions: architecture/consensus flow, transaction types/encoding, reth-db/MDBX persistence patterns, and rpc-eth generification/node wiring. Findings are synthesized below.

---

## 1. Architecture: Consensus Flow & Mempool Integration

### Full Transaction Lifecycle

```
RPC (eth_sendRawTransaction)
  -> InMemoryTxPool.push(raw_bytes)
  -> EvmApplication.propose() drains tx_source.pending()
  -> decode EIP-2718, execute in REVM, compute roots
  -> ConsensusApp wraps via ApplicationAdapter
  -> Simplex BFT reaches consensus
  -> AppAdapter::Reporter::report on ConsensusEvent::Finalized
  -> PersistingFinalizationSink persists block
  -> EvmApplication.store_finalized_block writes to BlockStorage
```

### Key Observations

1. **`pending()` DRAINS the pool** — proposed-but-not-finalized txs are removed from pool, no automatic re-queue
2. **EvmApplication already uses `Arc<dyn TxSource + Send + Sync>`** — trait object abstraction already in place on consensus side
3. **Finalization failure risk**: If `store_finalized_block` fails, receipts are already `take()`'d, but engine continues (height updates). Persistence must be independent.
4. **PersistingFinalizationSink** (`whirlpool-node/src/persisting_sink.rs:34-64`) wraps consensus finalization with BlockStorage writes — this is the finalization callback pattern.

---

## 2. Types: Transaction Encoding & Data Structures

### Core Types

- **EvmBlock** (`app/src/types.rs:21`): `Vec<Vec<u8>>` raw EIP-2718 txs + header (roots, gas, timestamp). Implements codec Read/Write + EncodeSize.
- **Raw transactions**: Always `Vec<u8>` (EIP-2718 encoded bytes) throughout the system
- **RecoveredTx**: `type RecoveredTx = Recovered<TransactionSigned>` (`app-evm/src/executor.rs:25`)

### Encoding Details

- `TransactionSigned::encode_2718/decode_2718` are canonical serializers
- Trie root computed over raw bytes: `ordered_trie_root_with_encoder`
- Hash available only after decode (`tx_signed.hash()`) — InMemoryTxPool does NO decoding or hashing at push time

### Deduplication

- InMemoryTxPool does NO dedup — stores whatever pushed, returns FIFO
- Only idempotency: state-reth refuses re-store of block at same height with different hash

### Persistence Key Decision

Raw EIP-2718 bytes are the natural persistence unit. If keyed by tx hash, must decode on insert (cost tradeoff). Alternative: auto-increment key for FIFO ordering.

---

## 3. Dependencies: reth-db Usage Patterns & MDBX Features

### Current Usage

- Only `state-reth` uses reth-db: depends on `reth-db` (vendor path, `mdbx` feature), `reth-db-api`, `reth-db-models`
- Init: `reth_db::init_db` -> `create_db` + `create_db_version_file` + register tables -> `DatabaseEnv`

### CRUD Patterns

```rust
// Read
self.db.tx()?.get::<TableName>(key).map_err(...)
// Write
let tx = self.db.tx_mut()?;
tx.put::<TableName>(key, value)?;
tx.commit()?;
// Cursor
tx.cursor_dup_write::<TableName>()?
```

### Table System

- `Table` trait requires `KEY: Encode+Decode`, `VALUE: Compress+Decompress`
- state-reth uses reth's built-in `Tables` enum — no custom tables declared
- Tables enum is tightly coupled to reth's set

### Multi-DB Coexistence

- MDBX databases live in directories — each `init_db`/`open_db` with distinct path yields separate `DatabaseEnv`
- Persistent mempool CAN use a separate MDBX directory

### Custom Table Challenge

reth-db's `Table` trait and `Tables` enum are tightly coupled. Options for custom mempool tables:
1. Extend `Tables` enum (invasive, vendor modification)
2. Use raw MDBX API directly (bypasses reth abstractions)
3. Use a simpler KV store (sled, redb, or raw file-based)

**Recommendation**: Given the simplicity of mempool storage (key=id, value=raw_bytes), a simpler approach than full reth-db may be more appropriate. Raw MDBX or a lightweight embedded DB avoids vendor coupling.

---

## 4. Domains: rpc-eth Generification & Node Wiring

### InMemoryTxPool Concrete References (ALL)

| Location | Usage |
|---|---|
| `app/src/tx_source.rs:20-52` | Definition + TxSource impl |
| `app/src/lib.rs:10` | Re-export |
| `rpc-eth/src/context.rs:1-49` | `tx_pool: Arc<InMemoryTxPool>` field |
| `rpc-eth/src/eth_handler.rs:209-327` | Test helpers |
| `whirlpool-node/src/main.rs:3,108-134` | Instantiation, wiring |
| `app-evm/tests/integration.rs:8-182` | Integration tests |
| `app/src/tx_source.rs:54-132` | Unit tests |

### Generification Strategy

**Option A (Type parameter)**: `EthRpcContext<S, B>` -> `EthRpcContext<S, B, T: TxSource + Send + Sync>` — propagates through `EthApiHandler`, `EthApiServer` impl, `start_rpc_server`.

**Option B (Trait object, RECOMMENDED)**: `Arc<dyn TxSource + Send + Sync>` — matches EvmApplication's existing pattern, minimal propagation, no new type params.

### EthRpcContext Current Structure

```rust
pub struct EthRpcContext<S: StateDb, B: BlockStorage> {
    tx_pool: Arc<InMemoryTxPool>,  // <- change to Arc<dyn TxSource + Send + Sync>
    state_db: Arc<RwLock<S>>,
    block_storage: Arc<B>,
    receipt_store: ReceiptStore,
    chain_id: u64,
    block_height: AtomicU64,
}
```

Manual `Clone` impl clones Arcs, copies chain_id.

### Node Wiring Pattern

`whirlpool-node::main` creates `InMemoryTxPool`, wraps in Arc, passes to both `EvmApplication::new` and `EthRpcContext::new`. For persistent mempool: replace `InMemoryTxPool::new()` with `PersistentTxPool::open(path)`, both consumers get the same Arc.

---

## 5. Cross-Cutting Findings

### Design Constraints

1. **TxSource trait needs `push` method** — currently only has `pending()`. Persistent pool needs `push(&self, tx: Vec<u8>)` on the trait for RPC to use without concrete type knowledge.
2. **Drain semantics must be preserved** — `pending()` must still drain (consensus expects this). Persistent pool deletes from DB on `pending()` call, or marks as "proposed".
3. **Startup recovery** — on restart, load all un-finalized txs from DB into memory or serve from DB directly.
4. **Concurrency** — current `Mutex<Vec>` pattern works for single-writer. MDBX supports concurrent reads + single writer natively.
5. **No vendor modification** — cannot extend reth-db Tables enum. Must use independent persistence.

### Risk Areas

1. **Custom table problem**: reth-db doesn't easily support custom tables without vendor changes. Need alternative persistence approach.
2. **Drain-on-propose semantics**: If node crashes between propose and finalize, those txs are lost from both memory and DB (if deleted on drain). Need "proposed" state tracking.
3. **No dedup today**: Adding persistence doesn't change this — but persistent duplicate txs waste disk. Consider tx-hash keying for natural dedup.

### Recommended Architecture (Pre-Design)

1. New crate: `mempool` or extend `app` with `PersistentTxPool`
2. Use raw MDBX (via `libmdbx` crate directly) or redb for simplicity — avoid reth-db table coupling
3. Implement `TxSource` trait (with added `push` method)
4. Store raw EIP-2718 bytes, keyed by insertion order or tx hash
5. Generify `EthRpcContext.tx_pool` to `Arc<dyn TxSource + Send + Sync>` (Option B)
6. Wire in `whirlpool-node::main` with persistent path
