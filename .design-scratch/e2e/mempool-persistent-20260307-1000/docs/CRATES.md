# CRATES

This document lists all workspace crates and their relationship to the persistent mempool implementation.

## Crate Inventory

| Crate | Role | Status | Changes | Dependencies |
|-------|------|--------|---------|--------------|
| **mempool** | Persistent transaction pool storage | **NEW** | Implements `PersistentTxPool` with embedded MDBX database, provides `MempoolStore` wrapper for raw libmdbx operations | `app` (TxSource trait), `libmdbx-rs`, `parking_lot` |
| **app** | Application-layer traits and shared types | **MODIFIED** | Extend `TxSource` trait with `fn push(&self, tx: Vec<u8>)` method; update existing implementors (`InMemoryTxPool`, `NoopTxSource`) to implement new method | None (foundational crate) |
| **rpc-eth** | Ethereum JSON-RPC handler | **MODIFIED** | Change `EthRpcContext.tx_pool` field from `Arc<InMemoryTxPool>` to `Arc<dyn TxSource + Send + Sync>` to accept trait objects | `app`, `state`, `serde`, `jsonrpsee` |
| **whirlpool-node** | Binary entrypoint, wiring layer | **MODIFIED** | Wire `PersistentTxPool::open(path)` instead of `InMemoryTxPool::new()`, compute mempool path from `persistent_storage_dir`, pass trait object to `EvmApplication` and `EthRpcContext` | `app`, `app-evm`, `rpc-eth`, `consensus-simplex`, `state-reth`, `mempool`, `commonware` |
| **app-evm** | EVM execution engine, block building | **UNCHANGED** | Already uses `Arc<dyn TxSource + Send + Sync>` trait object — no changes required | `app`, `state`, `reth-primitives`, `revm` |
| **consensus** | Consensus trait definitions | **UNCHANGED** | Foundational traits — not impacted by mempool changes | None |
| **consensus-simplex** | BFT consensus adapter (Simplex) | **UNCHANGED** | No mempool interaction — wraps `ConsensusApp` callbacks only | `consensus`, `commonware` |
| **p2p** | P2P networking trait definitions | **UNCHANGED** | No mempool interaction in current design | None |
| **p2p-commonware** | Commonware P2P implementation | **UNCHANGED** | No mempool interaction in current design | `p2p`, `commonware` |
| **state** | State storage trait definitions | **UNCHANGED** | Mempool persistence is orthogonal to state persistence | None |
| **state-memory** | In-memory state implementation | **UNCHANGED** | No mempool interaction | `state` |
| **state-reth** | Persistent state via reth-db/MDBX | **UNCHANGED** | Serves as reference pattern for MDBX usage, but no code changes needed (separate DB directory) | `state`, `reth-db`, `reth-primitives` |
| **integration-tests** | Integration test harness | **UNCHANGED** (potentially extended) | Existing tests continue to pass; new tests may be added to verify persistence behavior | All crates (test dependencies) |

## Key Changes Summary

### New Crate: `mempool`
- **Location**: `crates/mempool/`
- **Exports**: `PersistentTxPool`, `MempoolStore`
- **Storage**: Embedded MDBX database with auto-increment u64 keys, raw EIP-2718 bytes as values
- **Semantics**: Drain-on-`pending()` to match `InMemoryTxPool` behavior

### Modified Crates

#### `app` — Trait Foundation
- **File**: `src/traits.rs` — Add `fn push(&self, tx: Vec<u8>)` to `TxSource` trait
- **File**: `src/tx_source.rs` — Implement `push()` for `InMemoryTxPool` (already exists as method, formalize in trait) and `NoopTxSource` (empty impl)
- **Impact**: Breaking change for trait implementors (all in-tree, updated atomically)

#### `rpc-eth` — Generification
- **File**: `src/context.rs` — Change `tx_pool` field type to `Arc<dyn TxSource + Send + Sync>`
- **Impact**: Minimal — field type change only, no logic modifications

#### `whirlpool-node` — Integration
- **File**: `src/main.rs` — Replace `InMemoryTxPool::new()` with `PersistentTxPool::open(mempool_path)?`
- **File**: `Cargo.toml` — Add `mempool` dependency
- **Impact**: Single integration point — change tx pool constructor, path management

## Dependency Graph

```
mempool → app (TxSource trait)
         ↓
     libmdbx-rs

app → (no deps)

rpc-eth → app (TxSource trait)

whirlpool-node → app, app-evm, rpc-eth, mempool, consensus-simplex, state-reth

app-evm → app (TxSource trait object)
```

## Implementation Order

1. **app** — Extend trait, update implementors (Phase 1: Trait Foundation)
2. **rpc-eth** — Generify `EthRpcContext` (Phase 2: RPC Generification)
3. **mempool** — Implement persistent pool (Phase 3: Persistent Implementation, parallel to Phase 2)
4. **whirlpool-node** — Wire persistent pool (Phase 4: Integration)

## Test Impact

- **app**: Unit tests for `InMemoryTxPool` and `NoopTxSource` continue to pass
- **mempool**: New unit tests for persistence (push → restart → recover)
- **app-evm**: Integration tests continue to pass (trait object abstraction unchanged)
- **rpc-eth**: Test helpers updated to accept trait objects
- **integration-tests**: New end-to-end test for RPC → propose → restart → recover flow
