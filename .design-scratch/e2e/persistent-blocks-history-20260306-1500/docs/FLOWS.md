# FLOWS.md

## Scope

This document captures the key architecture flows for persistent block storage and historical block queries, grounded in:
- `STRATEGY.md`
- `DOMAINS.md`
- `CRATES.md`
- `EXPLORATION.md`
- existing flow code in `consensus-simplex`, `app-evm`, `rpc-eth`, `whirlpool-node`

Legend:
- `[EXISTING]` = behavior/code already present
- `[NEW]` = behavior/code introduced by this feature design

---

## Flow 1: Block Finalization -> Persistent Storage

Goal: persist a finalized `EvmBlock` plus receipts into MDBX (`Headers`, `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, `Receipts`, `CanonicalHeaders`, `HeaderNumbers`, plus `TransactionBlocks`) in one write transaction.

1. `[EXISTING]` **Consensus finalization activity emitted**
   - **crate/file**: `consensus-simplex/src/adapter.rs`
   - **function**: `impl Reporter for AppAdapter::report(activity)`
   - **input types**: `Activity<Sig, Digest>` (`Activity::Finalization(fin)` branch)
   - **output types**: internal branch continuation (no return value)
   - **notes**: extracts proposal payload digest (`commitment`) from finalization event.

2. `[EXISTING]` **Digest -> block resolution from ephemeral store**
   - **crate/file**: `consensus-simplex/src/adapter.rs`
   - **function**: `AppAdapter::report` (`self.finalized_blocks.write().await.remove(&commitment)`)
   - **input types**: `Digest` key, `BlockStore<B> = Arc<RwLock<HashMap<Digest, B>>>`
   - **output types**: `Option<B>`
   - **error/edge path**:
     - if `None`: logs `warn!("finalization received for unknown block")`; flow stops (no persistence possible)

3. `[EXISTING]` **Finalization event forwarded to sink**
   - **crate/file**: `consensus-simplex/src/adapter.rs`
   - **function**: `self.sink.handle(ConsensusEvent::Finalized { block, height, proof })`
   - **input types**: `ConsensusEvent<B>`
   - **output types**: async completion of sink handling
   - **notes**: this is the only consensus->application/event handoff point in current flow.

4. `[NEW]` **PersistingFinalizationSink receives finalized EVM block and triggers persistence**
   - **crate/file**: `whirlpool-node/src/main.rs` (proposed `PersistingFinalizationSink` wrapper)
   - **function**: `PersistingFinalizationSink::handle(ConsensusEvent::Finalized { block, .. })`
   - **input types**: `ConsensusEvent<EvmBlock>`
   - **output types**: side effect (`EvmApp::store_finalized_block`) + delegates to inner `FinalizationSink`
   - **error/edge path**:
     - missing cached receipts for finalized block (`receipts: Option<Vec<Receipt>>` is `None`): log and skip persistence for that block
     - persistence failure: log error; do not crash consensus loop

5. `[NEW]` **Receipts recovered from propose-time app state**
   - **crate/file**: `app-evm/src/lib.rs` (proposed field lifecycle)
   - **function**: propose-time capture + finalization-time `take()`
   - **input types**: `Vec<Receipt>` produced from execution path (`execution_result.receipts` origin)
   - **output types**: `&[Receipt]` passed to storage layer
   - **error/edge path**:
     - duplicate/forked finalization events can observe stale/missing cached receipts if lifecycle is not synchronized

6. `[NEW]` **Storage write entrypoint**
   - **crate/file**: `state/src/block_storage.rs` (trait), `state-reth/src/block_storage.rs` (impl)
   - **function**: `BlockStorage::store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<()>`
   - **input types**: `&EvmBlock`, `&[Receipt]`
   - **output types**: `Result<(), StateError/RethStateError>`
   - **error/edge path**:
     - database transaction open/commit failure
     - invariant mismatch (`receipts.len()` vs decoded tx count)

7. `[EXISTING->NEW USE]` **EvmBlock -> Header conversion**
   - **crate/file**: `app-evm/src/executor.rs`
   - **function**: `build_header_from_evm_block(block: &EvmBlock) -> Header` (to be exported for cross-crate use)
   - **input types**: `&EvmBlock`
   - **output types**: `reth_primitives_traits::Header`
   - **error/edge path**:
     - deterministic conversion (no `Result`); correctness risks are semantic (field mapping) rather than runtime errors

8. `[EXISTING->NEW USE]` **Raw tx bytes -> typed signed tx decode**
   - **crate/file**: `app-evm/src/executor.rs`
   - **function**: `decode_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError>`
   - **input types**: `&[Vec<u8>]` from `EvmBlock.transactions`
   - **output types**: `Result<Vec<Recovered<TransactionSigned>>, EvmAppError>`
   - **error/edge path**:
     - malformed EIP-2718 payload or signer recovery failure -> `EvmAppError::InvalidBlock`

9. `[NEW]` **Global transaction numbering and block body index computation**
   - **crate/file**: `state-reth/src/block_storage.rs`
   - **function**: `store_block` internal tx numbering logic
   - **input types**: prior `BlockBodyIndices` state + decoded tx list length
   - **output types**: `StoredBlockBodyIndices { first_tx_num, tx_count }`
   - **error/edge path**:
     - no prior block indices: bootstrap at genesis/first tx number
     - inconsistent prior index chain: return storage/state error

10. `[NEW]` **Single MDBX write transaction persists all block artifacts**
   - **crate/file**: `state-reth/src/block_storage.rs`
   - **function**: `store_block` internal batched writes
   - **input types**: converted `Header`, tx list with assigned `TxNumber`, receipts
   - **output types**: committed MDBX state
   - **tables written**:
     - `Headers` (`BlockNumber -> Header`)
     - `CanonicalHeaders` (`BlockNumber -> HeaderHash`)
     - `HeaderNumbers` (`BlockHash -> BlockNumber`)
     - `BlockBodyIndices` (`BlockNumber -> StoredBlockBodyIndices`)
     - `Transactions` (`TxNumber -> TransactionSigned`)
     - `TransactionHashNumbers` (`TxHash -> TxNumber`)
     - `TransactionBlocks` (`TxNumber -> BlockNumber`)
     - `Receipts` (`TxNumber -> Receipt`)
   - **error/edge path**:
     - any single table write failure aborts whole transaction (atomicity)
     - partial persistence is not allowed

11. `[EXISTING]` **Finalized height counter still updated**
   - **crate/file**: `consensus-simplex/src/sink.rs`
   - **function**: `FinalizationSink::handle(ConsensusEvent::Finalized { .. })`
   - **input types**: `ConsensusEvent<B>`
   - **output types**: `Arc<AtomicU64>` updated, structured log emitted
   - **error/edge path**:
     - no explicit error return; only logging side effects

---

## Flow 2: `eth_getBlockByNumber` Query

Goal: serve historical block by number from persistent MDBX data.

1. `[NEW]` **JSON-RPC request routed to block endpoint**
   - **crate/file**: `rpc-eth/src/eth_api.rs`, `rpc-eth/src/eth_handler.rs` (or planned `eth_rpc.rs` naming from design docs)
   - **function**: `get_block_by_number(number, full)` (via EthApiServer trait method `eth_getBlockByNumber`)
   - **input types**: `BlockNumberOrTag`, `bool`
   - **output types**: `RpcResult<Option<alloy_rpc_types::Block>>`

2. `[NEW]` **Tag resolution to concrete block number**
   - **crate/file**: `rpc-eth/src/eth_handler.rs` (planned implementation)
   - **function**: block tag resolution helper inside endpoint
   - **input types**: `BlockNumberOrTag`, `Arc<AtomicU64>` finalized height
   - **output types**: resolved `u64` block number
   - **edge cases**:
     - `Number(n)` -> direct
     - `Latest`/`Finalized` -> current finalized height
     - `Pending` -> MVP maps to latest/finalized policy per strategy
     - unsupported tags -> JSON-RPC error (`-32000` style)

3. `[NEW]` **RPC -> BlockStorage read call**
   - **crate/file**: `rpc-eth/src/context.rs`, `rpc-eth/src/eth_handler.rs`
   - **function**: `ctx.block_storage.get_block_by_number(number)`
   - **input types**: `u64`
   - **output types**: `Result<Option<EvmBlock>>`
   - **error path**:
     - storage/database error mapped to JSON-RPC error object

4. `[NEW]` **MDBX header/body read and block reconstruction**
   - **crate/file**: `state-reth/src/block_storage.rs`
   - **function**: `get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>>`
   - **input types**: `u64`
   - **output types**: `Result<Option<EvmBlock>>`
   - **read sequence**:
     - `Headers[number]` -> `Header`
     - `BlockBodyIndices[number]` -> `{ first_tx_num, tx_count }`
     - `Transactions[first_tx_num .. first_tx_num + tx_count)` -> tx list
     - rebuild `EvmBlock.transactions` from typed tx encoding
   - **error/edge path**:
     - missing header -> `Ok(None)`
     - header exists but body indices missing/corrupt -> state/internal error
     - tx gap within expected range -> state/internal error

5. `[NEW]` **EvmBlock -> RPC block conversion**
   - **crate/file**: `rpc-eth/src/eth_handler.rs` (planned conversion helper)
   - **function**: response assembly for `alloy_rpc_types::Block`
   - **input types**: `EvmBlock`, `full: bool`
   - **output types**: `alloy_rpc_types::Block`
   - **edge cases**:
     - `full = true` -> decode and include full transaction objects
     - `full = false` -> include transaction hashes only
     - decode failure while rendering response -> RPC internal error

6. `[NEW]` **JSON-RPC response returned**
   - **crate/file**: `rpc-eth/src/server.rs`
   - **function**: jsonrpsee method return path
   - **input types**: `RpcResult<Option<Block>>`
   - **output types**: serialized JSON-RPC result (`block` or `null`)

---

## Flow 3: `eth_getBlockByHash` Query

Goal: serve historical block by hash, using `HeaderNumbers` reverse lookup before normal number-based read.

1. `[NEW]` **JSON-RPC request routed to hash endpoint**
   - **crate/file**: `rpc-eth/src/eth_api.rs`, `rpc-eth/src/eth_handler.rs` (or planned `eth_rpc.rs`)
   - **function**: `get_block_by_hash(hash, full)` (via EthApiServer trait method `eth_getBlockByHash`)
   - **input types**: `BlockHash/B256`, `bool`
   - **output types**: `RpcResult<Option<alloy_rpc_types::Block>>`

2. `[NEW]` **RPC -> BlockStorage hash lookup call**
   - **crate/file**: `rpc-eth/src/eth_handler.rs`
   - **function**: `ctx.block_storage.get_block_by_hash(&hash_bytes)`
   - **input types**: `&[u8; 32]`
   - **output types**: `Result<Option<EvmBlock>>`

3. `[NEW]` **MDBX reverse map read (`HeaderNumbers`)**
   - **crate/file**: `state-reth/src/block_storage.rs`
   - **function**: `get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>>`
   - **input types**: `&[u8; 32]`
   - **output types**: `Result<Option<EvmBlock>>`
   - **read sequence**:
     - `HeaderNumbers[hash]` -> `BlockNumber`
     - delegate to `get_block_by_number(number)` reconstruction path
   - **error/edge path**:
     - hash missing in `HeaderNumbers` -> `Ok(None)`
     - reverse map exists but numbered block missing/corrupt -> state/internal error

4. `[NEW]` **Block conversion and response formatting**
   - **crate/file**: `rpc-eth/src/eth_handler.rs`
   - **function**: same conversion path as Flow 2
   - **input types**: `EvmBlock`, `full`
   - **output types**: `alloy_rpc_types::Block`
   - **edge cases**:
     - `full=false` default/typical for hash lookups -> hashes only
     - conversion decode issues -> RPC error

5. `[NEW]` **JSON-RPC response returned**
   - **crate/file**: `rpc-eth/src/server.rs`
   - **function**: jsonrpsee method return path
   - **input types**: `RpcResult<Option<Block>>`
   - **output types**: JSON-RPC result or error

---

## Flow 4: Node Startup Wiring

Goal: initialize persistent DB once, wire it into consensus/app execution and RPC block query path.

1. `[EXISTING]` **Node boots and opens persistent MDBX state DB**
   - **crate/file**: `whirlpool-node/src/main.rs`
   - **function**: `main()` async runtime startup block, `state_reth::open_state_db(&db_path)`
   - **input types**: `PathBuf`
   - **output types**: `RethStateDb` wrapped as `Arc<RwLock<_>>`
   - **error path**:
     - open/init failure currently `expect("failed to open state database")` -> process abort

2. `[EXISTING]` **EVM application initialized with shared state DB and tx pool**
   - **crate/file**: `whirlpool-node/src/main.rs`, `app-evm/src/executor.rs`
   - **function**: `EvmApplication::new(evm_config, state_db.clone(), tx_pool.clone())`
   - **input types**: `WhirlpoolEvmConfig`, `Arc<RwLock<RethStateDb>>`, `Arc<InMemoryTxPool>`
   - **output types**: `EvmApplication<RethStateDb>`

3. `[NEW]` **App layer gains finalization persistence capability**
   - **crate/file**: `whirlpool-node/src/main.rs` (PersistingFinalizationSink), `app-evm/src/lib.rs` (store_finalized_block)
   - **function**: `PersistingFinalizationSink::handle` calls `EvmApp::store_finalized_block(&block)`
   - **input types**: `ConsensusEvent<EvmBlock>`, cached `Vec<Receipt>`
   - **output types**: persisted block side effect
   - **edge cases**:
     - receipts unavailable at finalization -> persistence skipped/logged
     - storage failure -> logged; consensus continues

4. `[EXISTING]` **Consensus engine wiring remains generic**
   - **crate/file**: `whirlpool-node/src/main.rs`, `consensus-simplex/src/adapter.rs`
   - **function**: `CommonwareEngine::new(app, sink, ...)` + `AppAdapter::report`
   - **input types**: app adapter, sink, network/runtime config
   - **output types**: running engine handle
   - **notes**: consensus layer remains `B: Block` generic; no direct MDBX coupling.

5. `[EXISTING->NEW WIRING]` **RPC context created; extended with BlockStorage data source**
   - **crate/file**: `rpc-eth/src/context.rs`, `whirlpool-node/src/main.rs`
   - **function**: `EthRpcContext::new(...)` (to be extended with `block_storage`)
   - **input types**:
     - existing: `Arc<InMemoryTxPool>`, `Arc<RwLock<S: StateDb>>`, `chain_id: u64`
     - new: `Arc<dyn BlockStorage>` (implemented by `RethStateDb`)
   - **output types**: `EthRpcContext<S>` with persistent block query capability
   - **error/edge path**:
     - trait object/wiring mismatch at compile time if bounds are incomplete (`StateDb` vs `BlockStorage`)

6. `[NEW]` **RPC server exposes block history methods using shared storage**
   - **crate/file**: `rpc-eth/src/eth_api.rs`, `rpc-eth/src/eth_handler.rs`, `rpc-eth/src/server.rs`
   - **function**: server registration via `handler.into_rpc()` including `eth_get_block_by_number`/`eth_get_block_by_hash`
   - **input types**: `EthRpcContext` with block storage handle
   - **output types**: JSON-RPC API surface with persistent historical block reads

---

## Error Paths and Edge Cases Summary

1. **Finalization block missing in ephemeral `BlockStore`** (`AppAdapter::report`): finalization warning; no event payload, no persistence.
2. **Receipts unavailable at finalization** (`EvmApp` proposed receipts cache): finalized block cannot be persisted with receipts; should log and continue.
3. **Transaction decode failure** (`decode_transactions`): persistence/read conversion fails for malformed tx bytes.
4. **MDBX atomic write failure** (`store_block`): no partial writes; block remains unpersisted until retried by higher-level policy.
5. **Header/body inconsistency on read** (`get_block_by_number`): treat as storage corruption/internal error, return RPC error.
6. **Unknown block number/hash** (`get_block_by_number` / `get_block_by_hash`): return `Ok(None)` -> JSON-RPC `null`.
7. **Unsupported `BlockNumberOrTag` mapping policy**: reject with JSON-RPC error until explicitly handled.
8. **Startup DB open failure** (`open_state_db(...).expect(...)`): process exits immediately.
