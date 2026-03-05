# Shared Wiring Skeleton

## Grounded wiring
- `main` creates:
  - `height: Arc<AtomicU64>`
  - `sink: Arc<FinalizationSink<_>>`
  - `state_db: Arc<RwLock<TestStateDb>>`
  - `tx_pool: Arc<InMemoryTxPool>`
  - `evm_app: EvmApplication<TestStateDb>`
  - `app: Arc<ApplicationAdapter<_>>`
  - `engine: CommonwareEngine<_>`
- Engine starts once with `engine.start()`.
- Runtime is kept alive via pending future.

Evidence: `crates/whirlpool-node/src/main.rs`.

## [PROPOSED] wiring additions

```text
main runtime
  ├─ consensus engine task (existing)
  └─ rpc server task (new)
       ├─ EthRpcContext
       │   ├─ chain_id: u64 (= SAHARA_CHAIN_ID)
       │   ├─ tx_pool: Arc<InMemoryTxPool>
       │   ├─ state_db: Arc<RwLock<TestStateDb>>
       │   ├─ finalized_height: Arc<AtomicU64>
       │   └─ receipt_index: Arc<RwLock<ReceiptIndex>>
       └─ jsonrpsee server handle
```

### Handler -> state dependencies
- `eth_chainId` -> `chain_id`
- `eth_getBalance` -> `state_db.read().basic(address)`/equivalent
- `eth_getTransactionCount` -> account nonce from state + optional pending overlay policy
- `eth_estimateGas` -> EVM dry-run path on cloned DB (or deterministic fallback policy for v1)
- `eth_gasPrice` -> static/dev policy value
- `eth_sendRawTransaction` -> `tx_pool.push(raw)` + tx hash index insert
- `eth_getTransactionReceipt` -> `receipt_index` lookup

## Integration seam constraints
- Do not add RPC dependencies into `consensus` or `consensus-simplex`.
- Do not alter `app::Application`/`TxSource` trait signatures for this minimal scope.
- Keep all new shared mutable RPC state under node-level synchronization primitives.
