# STRATEGY.md

## 1. Goal

Add minimal Ethereum JSON-RPC server (7 methods) to whirlpool-node so alloy clients can do basic ETH balance transfers and verify via integration tests.

## 2. Approach

- Create new `rpc` crate in workspace (not a module in whirlpool-node — follows existing pattern of separate concern crates)
- Use jsonrpsee 0.26.0 with proc macro pattern (#[rpc(server, namespace = "eth")])
- Share state via RpcContext struct holding Arc references
- Spawn RPC server alongside consensus engine in main.rs

## 3. Implementation Strategy (ordered phases)

### Phase A: Foundation
- New `crates/rpc/` crate with Cargo.toml
- Define EthApi trait with 7 methods using jsonrpsee proc macro
- Define RpcContext struct for shared state
- Add rpc crate to workspace Cargo.toml members

### Phase B: Core Methods
- eth_chainId: return SAHARA_CHAIN_ID (313371) as U64
- eth_sendRawTransaction: decode tx bytes, compute keccak256 hash, push to InMemoryTxPool, return hash
- eth_getBalance: read lock state_db, get_account(addr), return balance (or U256::ZERO)
- eth_getTransactionCount: read lock state_db, get_account(addr), return nonce as U256

### Phase C: Receipt + Gas
- Add ReceiptStore (Arc<RwLock<HashMap<B256, TransactionReceipt>>>) to RpcContext
- eth_getTransactionReceipt: look up receipt by hash from ReceiptStore
- eth_gasPrice: return hardcoded gas price (e.g., 1 gwei) for dev
- eth_estimateGas: return hardcoded gas estimate (21000 for simple transfer) for v1, proper EVM dry-run later

### Phase D: Wiring
- Add rpc dependency to whirlpool-node
- In main.rs: construct RpcContext with cloned Arcs, build and spawn RPC server
- Add RPC_BIND_ADDR config constant

### Phase E: Integration Tests
- Add integration test in rpc crate or whirlpool-node
- Use alloy ProviderBuilder to connect to local RPC
- Test flow: check balance → send signed tx → poll receipt → verify balance changed

## 4. Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Separate crate vs module | Separate `rpc` crate | Follows workspace pattern of concern-separated crates. Allows independent testing. |
| jsonrpsee vs axum | jsonrpsee 0.26 | Matches reth vendor. Purpose-built for JSON-RPC. Proc macro reduces boilerplate. |
| Historical state queries | Not supported (v1) | Always returns "latest". No block tag support. Simplifies significantly. |
| Gas estimation (v1) | Hardcoded 21000 | Simple transfers only for v1. Full EVM dry-run is Phase 2 work. |
| Receipt storage | In-memory HashMap | No persistence needed for dev/test node. |
| Gas price | Hardcoded 1 gwei | Dev node, no fee market. |
| RPC port | Configurable constant | Default 8545, configurable via constant in config module. |

## 5. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Receipt gap — receipts dropped by BlockExecutor | eth_getTransactionReceipt returns None | Phase C adds ReceiptStore; block execution must be modified to populate it |
| State read contention | RPC reads block consensus writes | RwLock allows concurrent reads; writes are infrequent (per block) |
| alloy version mismatch | Type incompatibilities | Pin same alloy versions as reth vendor (alloy-primitives 1.5.0, alloy-rpc-types 1.4.3) |
| Test flakiness | Async timing between tx send and receipt | Poll loop with timeout in integration tests |

## 6. Out of Scope (v1)

- eth_call (requires full EVM execution without commit)
- eth_getLogs / event filtering
- WebSocket subscriptions (eth_subscribe)
- Historical state queries (block tags other than "latest")
- Proper EVM-based gas estimation (use hardcoded for v1)
- Transaction validation beyond basic RLP decode
- Persistent state / receipt storage
