# Proof: Ethereum JSON-RPC Server for Balance Transfers

## Section 0: Intent Decomposition

### Intent Statement
Add a JSON-RPC server to whirlpool-node implementing 7 Ethereum RPC methods so an alloy client can perform and verify basic ETH balance transfers in integration tests.

### Sub-intent Analysis
This is a **single intent** — no decomposition needed. All 7 methods serve one cohesive goal: enable the alloy client's `send_transaction → poll_receipt → check_balance` flow for ETH transfers.

**Evidence**: INTENT.md defines a single objective with 7 SC criteria that form a dependency chain (SC-01 through SC-07). No method is independently useful without the others for the stated goal.

### Completeness Check
The 7 methods map to alloy's `ProviderBuilder` default fillers:
- `ChainIdFiller` → eth_chainId (SC-01)
- `NonceFiller` → eth_getTransactionCount (SC-04)
- `GasFiller` → eth_estimateGas + eth_gasPrice (SC-05)
- `WalletFiller` → (client-side signing, no RPC needed)
- Core send → eth_sendRawTransaction (SC-03)
- Confirmation → eth_getTransactionReceipt (SC-06)
- Verification → eth_getBalance (SC-04)

All alloy fillers are covered. **No missing methods for the stated goal.**

## Section 1: Strategy Validation

### Approach Assessment
The design proposes RPC as node-local modules inside whirlpool-node using jsonrpsee 0.26 proc macros. This is validated by:

- **Grounded**: Existing architecture keeps node composition in whirlpool-node binary (`crates/whirlpool-node/src/main.rs`) — *STRATEGY.md §2, DOMAINS.md D1*
- **Grounded**: jsonrpsee 0.26 is already in the vendor dependency tree via reth — *SHARED_CONTEXT.md, vendor/reth/Cargo.toml*
- **Grounded**: Shared state handles (Arc<InMemoryTxPool>, Arc<RwLock<TestStateDb>>) already exist in main.rs — *INTENT.md grounded facts*

### Phased Strategy Coherence
5 implementation phases (A→E) form a valid dependency chain:
- Phase A (Foundation) has no dependencies — creates the crate/modules
- Phase B (Core Methods) depends on A — implements methods using existing types
- Phase C (Receipt + Gas) depends on A — adds new receipt store + gas stubs
- Phase D (Wiring) depends on A+B+C — connects everything in main.rs
- Phase E (Integration Tests) depends on all — validates end-to-end

**No circular dependencies. No missing phases.**

### Note on STRATEGY.md inconsistency
STRATEGY.md §2 says "Create new `rpc` crate" while CRATES.md/WORKSPACE.md/SUMMARY.md say "RPC as node-local modules in whirlpool-node." The majority of docs (CRATES.md, WORKSPACE.md, SUMMARY.md, DOMAINS.md) consistently describe node-local modules. **The node-local modules approach is the canonical decision.** STRATEGY.md Phase A should be read as "create modules" not "create crate."

## Section 2: Wiring Correctness

### Data Flow Verification

**eth_sendRawTransaction flow**:
1. Client sends hex-encoded signed tx → jsonrpsee deserializes to `Bytes`
2. Handler computes `keccak256(bytes)` → `B256` tx hash
3. Handler calls `tx_pool.push(bytes.to_vec())`
4. Returns tx hash to client

**Evidence**: InMemoryTxPool::push(tx: Vec<u8>) accepts raw bytes — *Grounded (crates/app/src/tx_source.rs)*. keccak256 available from alloy-primitives.

**eth_getBalance / eth_getTransactionCount flow**:
1. Client sends address → handler acquires `state_db.read()`
2. Calls `state_db.get_account(addr)` → `Option<AccountInfo>`
3. Returns `account_info.balance` or `U256::ZERO` (balance), `account_info.nonce` as `U256` (count)

**Evidence**: StateDb::get_account returns Option<AccountInfo{balance: U256, nonce: u64}> — *Grounded (crates/state/src/traits.rs)*

**eth_getTransactionReceipt flow**:
1. Client sends tx hash → handler acquires `receipt_store.read()`
2. Looks up `receipt_store.get(&hash)`
3. Returns `Option<TransactionReceipt>`

**Evidence**: Receipt store is [PROPOSED] — in-memory HashMap<B256, TransactionReceipt>. Receipts must be populated during block execution. **This requires modifying EvmApplication::propose() or adding a post-execution hook.** — *DOMAINS.md D4, BLOCKERS.md BLK-01 (resolved)*

### Wiring Contracts Table Verification
All 6 contracts in DOMAINS.md wiring table have identified producer and consumer. Types match grounded signatures.

## Section 3: Risk and Boundary Analysis

### Risk Assessment

**R1: Receipt population gap** (MEDIUM)
- Receipts are computed but dropped by BlockExecutor::finish() — *Grounded (crates/app-evm/src/executor.rs)*
- Mitigation: Add ReceiptStore populated by a modified execution path or post-propose hook
- Residual risk: Modifying executor introduces regression risk in consensus path
- **Boundary**: Changes to executor must be minimal — add receipt extraction, don't restructure

**R2: State read contention** (LOW)
- RwLock allows concurrent reads; writes happen only per-block (every 5s)
- RPC reads are fast (single account lookup)
- **No mitigation needed for v1 dev node**

**R3: alloy version compatibility** (LOW)
- Pinning to same versions as reth vendor (alloy-primitives 1.5.0, alloy-rpc-types 1.4.3)
- Reth is a mature project; versions are stable
- **Boundary**: Don't upgrade independently; stay in lockstep with vendor

**R4: Test timing** (LOW)
- Receipt polling needs retry loop with timeout
- Consensus block interval is 5s — receipt should appear within 10s
- **Boundary**: Tests use exponential backoff with 30s max timeout

### Boundary Rules
1. No changes to consensus traits (crates/consensus/)
2. No changes to app trait surface (crates/app/src/traits.rs)
3. Executor changes limited to receipt extraction (crates/app-evm/src/executor.rs)
4. RPC types stay node-private — no new public types in interface crates
5. All new code in whirlpool-node/src/rpc/ modules

## Section 4: Dependency Verification

### New External Dependencies
| Dependency | Version | Justification | Conflict check |
|-----------|---------|---------------|----------------|
| jsonrpsee | 0.26.0 | JSON-RPC server framework. Matched to reth vendor. | No existing jsonrpsee in workspace. Clean add. |
| alloy-primitives | 1.5.0 | Address, B256, U256, Bytes types. | Already in vendor dep tree via reth. Compatible. |
| alloy-rpc-types | 1.4.3 | TransactionRequest, Receipt types. | Already in vendor dep tree via reth. Compatible. |
| serde/serde_json | 1.x | Serialization for RPC types. | Already in workspace (used by other crates). |

### Internal Dependencies (from whirlpool-node)
- `app` — InMemoryTxPool (already a dependency)
- `state` — StateDb, AccountInfo (already a dependency via app-evm)
- `app-evm` — SAHARA_CHAIN_ID, executor changes (already a dependency)

**No new internal crate dependencies needed.** whirlpool-node already depends on all required crates.

### Cargo.toml workspace-level additions
- Add jsonrpsee, alloy-primitives, alloy-rpc-types to `[workspace.dependencies]` for version pinning
- Add these as dependencies in `crates/whirlpool-node/Cargo.toml`

## Section 5: Summary and Acceptance Criteria

### Acceptance Criteria

| ID | Criterion | Verification method | Grounded evidence |
|----|-----------|-------------------|-------------------|
| AC-1 | eth_chainId returns U64(313371) | TC-001: call eth_chainId, assert == 313371 | SAHARA_CHAIN_ID = 313_371 (crates/app-evm/src/config.rs) |
| AC-2 | eth_getBalance returns U256 for known account | TC-002: fund account in genesis state, call eth_getBalance, assert matches | StateDb::get_account → AccountInfo.balance (crates/state/src/traits.rs) |
| AC-3 | eth_getBalance returns U256::ZERO for unknown account | TC-002: call for unknown addr, assert == 0 | StateDb::get_account returns None for missing (crates/state/src/traits.rs) |
| AC-4 | eth_getTransactionCount returns nonce as U256 | TC-003: call for account with known nonce | StateDb::get_account → AccountInfo.nonce (crates/state/src/traits.rs) |
| AC-5 | eth_estimateGas returns U256(21000) for simple transfer | TC-004: send transfer request, assert gas == 21000 | Design decision: hardcoded for v1 (STRATEGY.md §4) |
| AC-6 | eth_gasPrice returns U256(1_000_000_000) (1 gwei) | TC-005: call eth_gasPrice, assert == 1 gwei | Design decision: hardcoded for v1 (STRATEGY.md §4) |
| AC-7 | eth_sendRawTransaction accepts valid tx and returns B256 hash | TC-006: send signed tx, receive non-zero hash | InMemoryTxPool::push accepts Vec<u8> (crates/app/src/tx_source.rs) |
| AC-8 | eth_sendRawTransaction pushes tx bytes to pool | TC-006: verify pool contains tx after send | InMemoryTxPool stores in Mutex<Vec<Vec<u8>>> (crates/app/src/tx_source.rs) |
| AC-9 | eth_getTransactionReceipt returns None for unknown hash | TC-007: query unknown hash, assert None | [PROPOSED] ReceiptStore (HashMap<B256, Receipt>) |
| AC-10 | eth_getTransactionReceipt returns receipt for confirmed tx | TC-008: send tx, wait for block, query receipt | [PROPOSED] ReceiptStore populated during execution |
| AC-11 | RPC server starts alongside consensus engine without errors | TC-009: start node, verify RPC port responds | Node wiring in main.rs (WORKSPACE.md) |
| AC-12 | alloy client can send ETH transfer and verify balance change | TC-010: full e2e flow with alloy ProviderBuilder | All methods combined (FLOWS.md) |

### QA Scenarios

| ID | Scenario | Expected behavior |
|----|----------|-------------------|
| QA-1 | RPC server port already in use | Server start fails with clear error, node continues consensus |
| QA-2 | eth_sendRawTransaction with invalid RLP bytes | Returns JSON-RPC error (-32602 invalid params) |
| QA-3 | eth_getBalance with invalid address format | Returns JSON-RPC error (-32602 invalid params) |
| QA-4 | Concurrent RPC requests during block production | All succeed — RwLock allows concurrent reads |
| QA-5 | eth_getTransactionReceipt before block finalization | Returns None (pending) |

### Invariants

| ID | Invariant | Evidence |
|----|-----------|----------|
| INV-1 | RPC server never modifies consensus state directly | Boundary rule: RPC writes only to InMemoryTxPool (DOMAINS.md D2) |
| INV-2 | State reads are always consistent (no partial updates) | RwLock guarantees read consistency (Grounded: Arc<RwLock<TestStateDb>>) |
| INV-3 | tx_pool.push is the ONLY ingress path for RPC-submitted txs | InMemoryTxPool::push is the single write method (crates/app/src/tx_source.rs) |
| INV-4 | Receipt store is append-only during node lifetime | [PROPOSED]: HashMap::insert only during block execution, never delete |
| INV-5 | Chain ID is immutable during node lifetime | SAHARA_CHAIN_ID is a const (crates/app-evm/src/config.rs) |

### Cross-sub-intent Invariants
None — single intent, no cross-sub-intent concerns.
