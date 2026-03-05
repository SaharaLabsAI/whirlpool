# EXPLORATION

## Architecture Layer: RPC Server Placement

### Current Architecture
```
consensus (traits) → consensus-simplex (adapter) → whirlpool-node (binary)
                  ↗
app (traits/txpool) → app-evm (executor)
                  ↗
state (traits) → state-memory (impl)
                  ↗
p2p (traits) → p2p-commonware (adapter)
```

### RPC Crate Placement
New `rpc` crate sits as a sibling to `app-evm`, depending on `app` (TxPool), `state` (StateDb), and `app-evm` (config). whirlpool-node depends on `rpc` and wires it.

```
whirlpool-node → rpc → app (InMemoryTxPool)
                    → state (StateDb trait)
                    → app-evm (SAHARA_CHAIN_ID)
                    → jsonrpsee + alloy types
```

## Type Exploration

### State Reading (for eth_getBalance, eth_getTransactionCount)
- `StateDb::get_account(addr) -> Option<AccountInfo>`
- `AccountInfo { balance: U256, nonce: u64, code_hash: B256 }`
- State is `Arc<RwLock<TestStateDb>>` — read lock for RPC queries
- No block-tagged state queries (no historical state) — always "latest"

### Transaction Submission (for eth_sendRawTransaction)
- `InMemoryTxPool::push(tx: Vec<u8>)` — raw EIP-2718 bytes
- Need to compute tx hash (keccak256) before pushing for return value
- Basic validation: decode RLP, verify signature (optional for v1)

### Receipt Retrieval (for eth_getTransactionReceipt)
- **GAP**: Receipts currently DROPPED by BlockExecutor::finish()
- Need new `ReceiptStore` (HashMap<B256, TransactionReceipt>) in memory
- Populated during block execution (propose path)
- Indexed by tx hash

### Gas Estimation (for eth_estimateGas)
- Clone state, build EVM env from TransactionRequest
- Use binary-search pattern: try gas limit, check success/failure, narrow
- Don't commit state changes

### Block Number (for eth_blockNumber — needed as bonus)
- FinalizationSink tracks height as AtomicU64
- Share height reference with RPC for block number queries

## Dependency Exploration

### New External Dependencies
| Crate | Version | Features | Purpose |
|-------|---------|----------|---------|
| jsonrpsee | 0.26.0 | server, macros | JSON-RPC server + proc macro |
| alloy-primitives | 1.5.0 | map-foldhash | Address, B256, U256, U64, Bytes |
| alloy-rpc-types | 1.4.3 | eth | TransactionRequest, TransactionReceipt, FeeHistory |
| serde | 1 | derive | Serialization for RPC types |
| serde_json | 1 | — | JSON handling |
| tokio | 1 | — | Async runtime (already in workspace) |

### Internal Dependencies
| Crate | Purpose |
|-------|---------|
| app | InMemoryTxPool type, TxSource trait |
| state | StateDb trait, AccountInfo type |
| app-evm | SAHARA_CHAIN_ID, WhirlpoolEvmConfig |

## Domain Exploration

### RPC Server Domain
- Single domain: `rpc` crate with EthApi trait + impl
- Server lifecycle managed by whirlpool-node main.rs
- Shared state via `RpcContext` struct holding Arc refs

### Shared State Pattern
```
struct RpcContext {
    tx_pool: Arc<InMemoryTxPool>,
    state_db: Arc<RwLock<TestStateDb>>,
    receipt_store: Arc<RwLock<HashMap<B256, TransactionReceipt>>>,
    chain_id: u64,
    block_height: Arc<AtomicU64>,
}
```

### Integration Test Domain
- Test binary using alloy ProviderBuilder
- Spins up whirlpool-node (or just RPC server + mock state)
- Tests: send ETH transfer, check balance, verify receipt
