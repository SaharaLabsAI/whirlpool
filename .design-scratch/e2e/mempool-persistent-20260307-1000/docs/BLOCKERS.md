# BLOCKERS

Reviewed inputs:
- `STRATEGY.md`
- `EXPLORATION.md`
- `EXPLORATION_DIGEST.md`

## BLOCKER

No blockers identified.

## WARNING

1. **Crash between propose and finalize can lose transactions**
   - Issue: `pending()` drain semantics mean a crash in this window can drop txs.
   - Impact: Potential transaction loss across this failure window.
   - Proposed resolution: Keep as accepted MVP behavior, document it clearly, and plan lifecycle-state tracking (`submitted -> proposed -> finalized`) as a post-MVP enhancement.

2. **Storage path misconfiguration can cause overlap risk**
   - Issue: Incorrect path wiring could overlap mempool and other persistent stores.
   - Impact: Possible corruption/data-loss risk.
   - Proposed resolution: Enforce dedicated `{persistent_storage_dir}/mempool` subdirectory and validate non-overlap during integration.

3. **No deduplication remains an efficiency risk**
   - Issue: Duplicate txs are preserved by design.
   - Impact: Extra disk usage and repeated processing opportunities.
   - Proposed resolution: Track as post-MVP optimization (e.g., optional hash index) without changing MVP semantics.

## INFO

1. **reth-db table extension limitation is already mitigated**
   - Evidence: Strategy selects raw `libmdbx-rs` to avoid vendor table coupling.

2. **`TxSource` trait break is controlled and in-tree**
   - Evidence: Known implementors are in-tree and slated for same-phase update.

3. **Persistence overhead is expected low and measurable**
   - Evidence: Marked low likelihood in strategy with optional benchmark phase.
