# EXPLORATION DIGEST

## Intent Summary
Add persistent storage to the Whirlpool mempool so pending transactions survive node restarts.

## Affected Crates & Scope

| Crate | Change Type | Scope |
|---|---|---|
| `app` | Trait extension | Add `push()` to `TxSource` trait |
| `app` (or new `mempool`) | New impl | `PersistentTxPool` implementing `TxSource` |
| `rpc-eth` | Generification | `EthRpcContext.tx_pool` → `Arc<dyn TxSource + Send + Sync>` |
| `whirlpool-node` | Wiring | Instantiate persistent pool, pass to consumers |
| `app-evm` | None | Already uses `Arc<dyn TxSource>` — no changes needed |
| `state-reth` | None | Reference only — not modified |

## Key Design Decisions Required

### 1. Persistence Backend
**Options**: (a) Raw MDBX via libmdbx crate, (b) redb, (c) sled, (d) reth-db with custom approach
**Constraint**: reth-db's `Tables` enum is tightly coupled — custom tables require vendor changes (forbidden).
**Recommendation**: Raw libmdbx or redb — lightweight, embedded, zero vendor coupling.

### 2. TxSource Trait Extension
**Current**: `fn pending(&self) -> Vec<Vec<u8>>` only.
**Needed**: `fn push(&self, tx: Vec<u8>)` for RPC layer to use trait object.
**Impact**: All implementors (InMemoryTxPool, NoopTxSource, MockTxSource in tests) need update.

### 3. Storage Key Strategy
**Options**: (a) Auto-increment u64 (preserves FIFO), (b) tx hash (natural dedup, requires decode)
**Trade-off**: Hash keying deduplicates but costs a decode + keccak256 per insert.
**Recommendation**: Auto-increment with optional hash index — keep insert cheap, add dedup later.

### 4. Drain Semantics & Crash Recovery
**Current**: `pending()` drains Vec. Post-drain, txs exist nowhere.
**Persistent**: Must track tx lifecycle — `submitted` → `proposed` → `finalized` → `deleted`.
**Minimum viable**: Delete from DB on `pending()` call (matches current drain behavior). Improvement: mark as "proposed", delete on finalize.

### 5. EthRpcContext Generification Approach
**Options**: (a) Add type parameter `T: TxSource`, (b) Use `Arc<dyn TxSource + Send + Sync>`
**Recommendation**: Option (b) — matches EvmApplication's existing pattern, minimal code churn.

## Cross-Cutting Constraints

1. No vendor modifications allowed
2. All cargo commands via `nix develop --command`
3. Drain semantics on `pending()` must be preserved for consensus compatibility
4. Raw EIP-2718 bytes are the natural persistence unit
5. Multi-DB coexistence is fine — each MDBX directory is independent

## Risk Summary

| Risk | Severity | Mitigation |
|---|---|---|
| reth-db custom tables | Medium | Use independent persistence (raw MDBX/redb) |
| Crash between propose & finalize | Medium | Track tx lifecycle states or accept current drain-and-lose behavior initially |
| TxSource trait change breaks downstream | Low | Only 3 implementors, all in-tree |
| Performance regression on persistence | Low | MDBX is fast; mempool ops are not hot path |
| Dedup not addressed | Low | Not in scope for initial implementation; can add later |
