# WORKSPACE.md

Workspace-level dependency graph and build order for persistent block storage feature.

---

## Workspace Members

Current workspace members (from `/Cargo.toml`):
```
crates/consensus
crates/consensus-simplex
crates/p2p
crates/p2p-commonware
crates/rpc-eth
crates/whirlpool-node
crates/state
crates/state-memory
crates/state-reth
crates/app
crates/app-evm
testing/integration-tests
```

**Modified crates**: state, state-reth, app, app-evm, rpc-eth, whirlpool-node (6 of 12)

---

## Current Dependency Graph (Relevant Subgraph)

```
whirlpool-node
├── app-evm
│   ├── app
│   ├── state
│   ├── state-memory
│   └── state-reth
│       └── state
├── rpc-eth
│   ├── app
│   └── state
└── state-reth
    └── state

consensus-simplex
└── consensus

app
└── consensus
```

---

## New Dependency Edges

```diff
state-reth
+ └── app-evm (NEW internal dependency for build_header_from_evm_block, decode_transactions)

rpc-eth
├── app
- └── state
+ ├── state (now uses BlockStorage trait)
+ └── app-evm (NEW internal dependency for EvmBlock→RPC block conversion helpers)

state
+ └── alloy-consensus (NEW external dependency for Receipt type)

app
+ └── alloy-consensus (NEW external dependency for Receipt re-export)
```

**New internal crate dependencies**:
- `state-reth` → `app-evm` (for conversion functions)
- `rpc-eth` → `app-evm` (for conversion helpers)

**New external dependencies**:
- `alloy-consensus` added to `state` and `app`
- `alloy-eips` and `reth-ethereum-primitives` may be needed transitively by `state-reth` (already present via reth-db)

---

## Dependency Graph After Changes

```
whirlpool-node
├── app-evm (modified - stores blocks on finalization)
│   ├── app (modified - Receipt re-export)
│   │   ├── consensus
│   │   └── alloy-consensus (NEW)
│   ├── state (modified - BlockStorage trait)
│   │   └── alloy-consensus (NEW)
│   ├── state-memory
│   └── state-reth (modified - BlockStorage impl)
│       └── state (modified)
├── rpc-eth (modified - eth_getBlock* endpoints)
│   ├── app (modified)
│   └── state (modified - BlockStorage trait)
└── state-reth (modified)
    └── state (modified)

consensus-simplex (unchanged)
└── consensus
```

**Key changes**:
- state gains dependency on alloy-consensus
- app gains dependency on alloy-consensus
- rpc-eth uses BlockStorage trait from state
- app-evm uses BlockStorage impl from state-reth
- whirlpool-node wires BlockStorage into rpc-eth context

---

## Build Order (Topological Sort)

### Phase 1: Foundation Crates (No changes)
1. `consensus`
2. `p2p`

### Phase 2: Trait Definitions (Modified)
3. **state** ← Add BlockStorage trait, add alloy-consensus dependency
4. **app** ← Add alloy-consensus dependency, re-export Receipt

### Phase 3: Implementations (Modified)
5. **state-reth** ← Implement BlockStorage for RethStateDb (depends on state)
6. `state-memory` (unchanged)

### Phase 4: Application Layer (Modified)
7. **app-evm** ← Store receipts, call state-reth BlockStorage (depends on app, state, state-reth)
8. `consensus-simplex` (unchanged)
9. `p2p-commonware` (unchanged)

### Phase 5: RPC Layer (Modified)
10. **rpc-eth** ← Add eth_getBlock* endpoints, use BlockStorage (depends on state, app)

### Phase 6: Binary (Modified)
11. **whirlpool-node** ← Wire block_storage into rpc-eth (depends on all above)

### Phase 7: Testing (Optional)
12. `integration-tests` ← Add new e2e tests for block queries

**Critical path**: state → state-reth → app-evm → whirlpool-node

**Parallel work possible**:
- state + app can be done in parallel (both add alloy-consensus)
- rpc-eth can be developed in parallel with app-evm (both depend on state)
- Integration in whirlpool-node requires all prior crates complete

---

## Integration Points

### 1. State → State-Reth (BlockStorage Trait/Impl)
- **Interface**: `BlockStorage` trait defined in state, implemented in state-reth
- **Data flow**: EvmBlock + receipts → MDBX tables (Headers, Transactions, Receipts, etc.)
- **Error handling**: Reuse existing StateError 4-tier taxonomy (Database/State/Internal/Config)

### 2. App-Evm → State-Reth (Finalization Persistence)
- **Interface**: `state_db.store_block(&block, &receipts)` called from `EvmApp::handle(Finalized)`
- **Data flow**: EvmApp stores receipts during propose(), retrieves on finalization, persists via BlockStorage
- **Conversion**: `build_header_from_evm_block()` and `decode_transactions()` exported from app-evm/executor.rs

### 3. Rpc-Eth → State (Block Queries)
- **Interface**: `block_storage: Arc<dyn BlockStorage>` field in EthRpcContext
- **Data flow**: RPC endpoint → BlockStorage::get_block_by_* → EvmBlock + receipts → alloy_rpc_types::Block
- **Conversion**: EvmBlock → alloy Header, TransactionSigned → alloy Transaction, Receipt → alloy Receipt

### 4. Whirlpool-Node → Rpc-Eth (Wiring)
- **Interface**: Pass `state_db` (RethStateDb) to EthRpcContext as `block_storage` parameter
- **Data flow**: Node startup → RethStateDb construction → shared between EvmApp and RpcEth
- **Concurrency**: Arc<RethStateDb> shared across consensus finalization and RPC queries

---

## Workspace-Level Constraints

### 1. No New Crate Creation
All changes fit within existing crates. No new crate members added.

### 2. Vendor Isolation
No changes to vendor dependencies (reth, commonware). Reuse existing reth-db tables and types.

### 3. Backward Compatibility
- Existing StateDb trait unchanged
- Existing consensus flow unchanged (proposal, verification, finalization)
- New BlockStorage trait is additive, not breaking

### 4. Cargo Features
No new cargo features required. Existing feature flags unchanged:
- state-reth: mdbx backend (already enabled)
- rpc-eth: jsonrpsee server + macros (already enabled)

### 5. External Dependencies Added
Only two new external dependencies at workspace level:
- `alloy-consensus = "1.4.3"` (to state and app)
- Already present transitively via reth types, now direct dependency

---

## Build Verification Strategy

### Per-Crate Checks
1. **state**: `cargo build -p state` (trait compiles)
2. **app**: `cargo build -p app` (Receipt re-export works)
3. **state-reth**: `cargo build -p state-reth && cargo test -p state-reth` (impl compiles and round-trip tests pass)
4. **app-evm**: `cargo build -p app-evm && cargo test -p app-evm` (finalization hook works)
5. **rpc-eth**: `cargo build -p rpc-eth && cargo test -p rpc-eth` (endpoints compile and mock tests pass)
6. **whirlpool-node**: `cargo build -p whirlpool-node` (binary links)

### Workspace-Level Checks
- `cargo build --workspace` (all crates build together)
- `cargo test --workspace` (all tests pass)
- `cargo clippy --workspace` (no new warnings)

### Integration Tests
- `cargo test -p integration-tests` (e2e test: propose → finalize → RPC query returns block)

---

## Rollout Strategy

### Phase 1: Trait Layer (Low Risk)
- Merge state + app changes (trait definition, type re-exports)
- No behavioral changes, safe to deploy

### Phase 2: Storage Layer (Medium Risk)
- Merge state-reth changes (BlockStorage impl)
- Test with unit tests, no production impact yet (trait not called)

### Phase 3: Finalization Hook (High Risk)
- Merge app-evm changes (persistence on finalization)
- Test thoroughly — this writes to MDBX on every finalized block
- Monitor finalization latency (expect <5ms overhead)

### Phase 4: RPC Endpoints (Medium Risk)
- Merge rpc-eth changes (eth_getBlock* endpoints)
- Test with manual RPC queries
- Monitor query latency and error handling

### Phase 5: Integration (High Risk)
- Merge whirlpool-node changes (wire block_storage)
- Deploy to testnet, verify end-to-end flow
- Monitor MDBX disk usage and performance

**Rollback strategy**: Each phase is independently revertible. Persistence data in MDBX is append-only (no destructive changes).

---

## Performance Considerations

### Write Path (Finalization)
- **Current**: 0ms (no-op after finalization)
- **New**: ~3-5ms MDBX write per block (Headers + Transactions + Receipts)
- **Mitigation**: Single write transaction, batch all inserts

### Read Path (RPC Queries)
- **Current**: N/A (no block queries)
- **New**: ~1-2ms per block query (MDBX read + EvmBlock reconstruction)
- **Mitigation**: MDBX uses mmap (OS page cache), hot blocks cached automatically

### Disk Usage
- **Current**: ~100MB for state trie (CanonicalHeaders only has hashes)
- **New**: +~500KB per 1000 blocks (headers + tx indices + receipts)
- **Mitigation**: MDBX auto-compaction, configurable pruning (future work)

---

## Risk Mitigation

### R1: Type Encoding Mismatch (EvmBlock vs reth Header)
**Mitigation**: Use existing `build_header_from_evm_block()` conversion function. Tested in executor.rs.

### R3: Receipt Reconciliation
**Mitigation**: Store receipts in EvmApp state during propose(), retrieve on finalization. No re-execution needed.

### R5: Generic Type Constraints
**Mitigation**: Persistence at application layer (EvmApp), not consensus layer (consensus-simplex). Avoid generic B: Block constraint.

### R7: Transaction Numbering
**Mitigation**: Use BlockBodyIndices to track global TxNumber. Reconstruct on startup from last block.

### R8: Finalization Latency
**Mitigation**: Single MDBX write transaction, benchmark on testnet. Fall back to async persistence if needed.
