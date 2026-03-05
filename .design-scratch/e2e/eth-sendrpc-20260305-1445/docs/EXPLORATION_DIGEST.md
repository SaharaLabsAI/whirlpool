# EXPLORATION DIGEST

## Architecture
- New `rpc` crate sits alongside app-evm, depends on app + state + app-evm + jsonrpsee + alloy
- whirlpool-node wires RPC server in main.rs alongside consensus engine
- Shared state via RpcContext struct holding Arc references to tx_pool, state_db, receipt_store, block_height

## Key Types Found
- StateDb::get_account(addr) → Option<AccountInfo{balance: U256, nonce: u64, code_hash: B256}>
- InMemoryTxPool::push(tx: Vec<u8>) — raw EIP-2718 bytes, Arc<Mutex<Vec<Vec<u8>>>>
- EvmBlock: height, parent_id, state_root, receipts_root, gas_used, timestamp, transactions
- State DB: Arc<RwLock<TestStateDb>> — clonable, read lock for queries

## Critical Gaps
1. **Receipt storage**: Receipts DROPPED by BlockExecutor::finish() after computing receipts_root. Need new in-memory ReceiptStore (HashMap<B256, Receipt>).
2. **Gas estimation**: Need EVM dry-run capability — clone state, build env, binary-search gas.
3. **Block height exposure**: FinalizationSink has AtomicU64 but not shared with RPC. Need Arc<AtomicU64> passed to RpcContext.

## Dependencies
- External: jsonrpsee 0.26.0, alloy-primitives 1.5.0, alloy-rpc-types 1.4.3
- Internal: app (TxPool), state (StateDb), app-evm (chain ID + EVM config)
- Chain ID: SAHARA_CHAIN_ID = 313371 (app-evm/src/config.rs)

## No Historical State
- All queries return "latest" state only — no block-tagged queries
- Simplifies implementation significantly
