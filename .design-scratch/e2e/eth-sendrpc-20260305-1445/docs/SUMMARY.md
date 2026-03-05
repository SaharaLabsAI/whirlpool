# SUMMARY

## What this design covers
Adding a minimal Ethereum JSON-RPC server to whirlpool-node that implements 7 methods sufficient for an alloy client to perform and verify basic ETH balance transfers in integration tests.

## Key decisions
1. **RPC as node-local modules** (not a separate crate) — keeps it simple for v1, transport stays in node layer
2. **jsonrpsee 0.26** with proc macro — matches reth vendor patterns
3. **No historical state** — all queries return "latest" only
4. **Hardcoded gas price** (1 gwei) and **simple gas estimation** (21000 for transfers) for v1
5. **In-memory receipt store** — HashMap<B256, Receipt> populated during block execution
6. **No app crate changes** — RPC consumes existing InMemoryTxPool and StateDb interfaces

## Architecture
```
Client (alloy) → HTTP JSON-RPC → jsonrpsee server → EthApiHandler
    ↓ reads                                           ↓ shares
state_db (Arc<RwLock<TestStateDb>>)          tx_pool (Arc<InMemoryTxPool>)
receipt_store (Arc<RwLock<HashMap>>)          block_height (Arc<AtomicU64>)
```

## RPC methods
| Method | Returns | State access |
|--------|---------|-------------|
| eth_chainId | U64 (313371) | Config constant |
| eth_getBalance | U256 | state_db read lock |
| eth_getTransactionCount | U256 | state_db read lock |
| eth_estimateGas | U256 (21000) | Hardcoded v1 |
| eth_gasPrice | U256 (1 gwei) | Hardcoded v1 |
| eth_sendRawTransaction | B256 (tx hash) | tx_pool push |
| eth_getTransactionReceipt | Option<Receipt> | receipt_store read lock |

## Implementation slices
- S1: Server bootstrap + chainId + gasPrice
- S2: State reads (getBalance, getTransactionCount)
- S3: Tx ingress (sendRawTransaction)
- S4: Gas estimation (estimateGas)
- S5: Receipt tracking (getTransactionReceipt)
- S6: Alloy integration tests

## Test contracts
10 tests (TC-001..TC-010) covering all 7 methods individually + 2 e2e alloy tests.

## Blockers
None open. Receipt storage gap resolved (in-memory store).

## Verdict
PASS — design is complete, coherent, and ready for planning.
