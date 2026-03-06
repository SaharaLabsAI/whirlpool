# CRATES.md

Per-crate change descriptions for persistent block storage feature.

---

## state

**Current Role**: Defines core traits for state management (`StateDb`) and error types.

**Proposed Changes**:
- Add new `BlockStorage` trait in `src/block_storage.rs`
- Export trait from `lib.rs`

**New Types/Traits**:
```rust
pub trait BlockStorage: Send + Sync {
    fn store_block(&mut self, block: &EvmBlock, receipts: &[Receipt]) -> Result<()>;
    fn get_block_by_number(&self, number: u64) -> Result<Option<EvmBlock>>;
    fn get_block_by_hash(&self, hash: &[u8; 32]) -> Result<Option<EvmBlock>>;
    fn get_receipts_by_block(&self, number: u64) -> Result<Vec<Receipt>>;
}
```

**New Dependencies**:
- `alloy-consensus = "1.4.3"` (for Receipt type)

**Impact Level**: **Minor** — New trait definition only, no changes to existing code

---

## state-reth

**Current Role**: Implements `StateDb` trait using MDBX backend (reth-db). Provides persistent state storage.

**Proposed Changes**:
- Implement `BlockStorage` trait for `RethStateDb` struct in new `src/block_storage.rs`
- Export implementation from `lib.rs`
- Use existing MDBX tables: Headers, HeaderNumbers, BlockBodyIndices, Transactions, TransactionHashNumbers, TransactionBlocks, Receipts
- Add conversion logic: EvmBlock → reth Header via `build_header_from_evm_block()`, raw tx bytes → TransactionSigned via `decode_transactions()`

**New Types/Traits**:
- `impl BlockStorage for RethStateDb` (new implementation)

**New Dependencies**:
- None (already has reth-db, reth-db-api, alloy-primitives, alloy-consensus via reth types)

**Impact Level**: **Moderate** — New trait implementation using existing tables, requires conversion logic and transaction numbering strategy

---

## app

**Current Role**: Defines `EvmBlock` type and consensus adapter types.

**Proposed Changes**:
- No structural changes to EvmBlock type
- Re-export `Receipt` type from `alloy-consensus` for BlockStorage trait signature consistency

**New Types/Traits**:
- None

**New Dependencies**:
- `alloy-consensus = "1.4.3"` (re-export Receipt)

**Impact Level**: **Minor** — Type re-export only

---

## app-evm

**Current Role**: Implements EVM execution via reth-evm. Handles block proposal, verification, and finalization events.

**Proposed Changes**:
- Add `receipts: Option<Vec<Receipt>>` field to `EvmApp` struct
- Store receipts during `propose()` execution (currently discarded after `receipts_root()` computation)
- In `handle(ConsensusEvent::Finalized(block))`, retrieve receipts and call `state_db.store_block(&block, &receipts)`
- Clear stored receipts after persistence
- Export `build_header_from_evm_block()` and `decode_transactions()` as public from `executor.rs` (currently module-private)

**New Types/Traits**:
- Modified `EvmApp` struct with receipts field

**New Dependencies**:
- None

**Impact Level**: **Moderate** — Receipt flow changes, finalization hook added, but no external API changes

---

## consensus-simplex

**Current Role**: Commonware Simplex adapter. Handles consensus proposal/verification/finalization via MailboxActor and AppAdapter.

**Proposed Changes**:
- None required — persistence happens at application layer (EvmApp), not consensus layer
- Generic `B: Block` constraint prevents consensus-simplex from knowing about concrete storage types

**New Types/Traits**:
- None

**New Dependencies**:
- None

**Impact Level**: **None** — No changes

---

## rpc-eth

**Current Role**: Exposes Ethereum JSON-RPC endpoints (chainId, gasPrice, sendRawTransaction, etc.). Currently 7 methods, no block query endpoints.

**Proposed Changes**:
- Add `eth_get_block_by_hash` and `eth_get_block_by_number` to `EthApiServer` trait in `eth_api.rs`
- Add `block_storage: Arc<dyn BlockStorage>` field to `EthRpcContext` in `context.rs`
- Implement new endpoints in `eth_rpc.rs`:
  - Query BlockStorage for block data
  - Convert EvmBlock → `alloy_rpc_types::Block`
  - Handle `BlockNumberOrTag` variants (Latest, Pending, Finalized, Number)
  - Handle `full: bool` parameter (full tx objects vs just hashes)
- Update `eth_get_transaction_receipt()` to fall back to BlockStorage for finalized receipts (DEFERRED to post-MVP per BLK-3 in BLOCKERS.md — currently only checks in-memory ReceiptStore)

**New Types/Traits**:
- New RPC methods in `EthApiServer` trait

**New Dependencies**:
- None (already has alloy-rpc-types, jsonrpsee)

**Impact Level**: **Moderate** — New RPC endpoints, context field addition, EvmBlock conversion logic

---

## whirlpool-node

**Current Role**: Binary entry point. Constructs RethStateDb, EvmApp, EthRpcContext, and starts consensus + RPC server.

**Proposed Changes**:
- Pass `state_db` (which now implements `BlockStorage`) to `EthRpcContext` as `block_storage` parameter
- No new initialization required (BlockStorage impl is on existing RethStateDb instance)

**New Types/Traits**:
- None

**New Dependencies**:
- None

**Impact Level**: **Minor** — Wiring change only, no new components

---

## Unaffected Crates

The following crates require **no changes**:

- **consensus**: Core trait definitions unchanged
- **p2p**: No block storage interaction
- **p2p-commonware**: No block storage interaction
- **state-memory**: In-memory StateDb impl, not used in production node
- **integration-tests**: May need new tests, but crate structure unchanged

---

## Summary Table

| Crate | Impact Level | New Files | Modified Files | New Dependencies | New Public API |
|-------|--------------|-----------|----------------|------------------|----------------|
| state | Minor | `src/block_storage.rs` | `src/lib.rs` | alloy-consensus | BlockStorage trait |
| state-reth | Moderate | `src/block_storage.rs` | `src/lib.rs` | None | BlockStorage impl |
| app | Minor | None | `src/lib.rs` | alloy-consensus | Receipt re-export |
| app-evm | Moderate | None | `src/lib.rs`, `src/executor.rs` | None | Public conversion fns |
| consensus-simplex | None | None | None | None | None |
| rpc-eth | Moderate | None | `src/eth_api.rs`, `src/context.rs`, `src/eth_rpc.rs` | None | eth_getBlock* methods |
| whirlpool-node | Minor | None | `src/main.rs` or `src/node.rs` | None | None |

**Total**: 7 crates modified, 4 new files, ~10 existing files modified, 2 new external dependencies (alloy-consensus added to state and app).
