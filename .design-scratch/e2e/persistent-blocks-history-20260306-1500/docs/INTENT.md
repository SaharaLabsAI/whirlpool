# Intent

## What
Implement persistent block storage and real history block queries for the Whirlpool consensus framework.

## Why
Currently, finalized blocks are ephemeral — stored in an in-memory `HashMap` during consensus rounds and dropped after finalization. The `FinalizationSink` only updates a height counter (`Arc<AtomicU64>`). There is no way to retrieve historical block data after finalization, which blocks:
- eth_getBlockByHash / eth_getBlockByNumber RPC endpoints (required for Ethereum compatibility)
- Block explorer / indexer integration
- Chain state reconstruction after restart

## Success Criteria

- **SC-1**: Finalized blocks (header + body + transactions + receipts) are persisted atomically to MDBX in a single write transaction
- **SC-2**: Block persistence triggers automatically on finalization events (no manual trigger required)
- **SC-3**: `eth_getBlockByNumber(number|tag, full)` returns persisted block data from MDBX
- **SC-4**: `eth_getBlockByHash(hash, full)` returns persisted block data from MDBX
- **SC-5**: Node wiring integrates persistence and query without breaking existing consensus or RPC flows

## Scope
1. **Persistent block storage** (SC-1): Store full finalized blocks (headers, bodies, transactions, receipts) durably via MDBX/reth-db (reusing the vendor stack already used by state-reth)
2. **Finalization hook** (SC-2): Wire block persistence into the finalization path so blocks are stored automatically on finalization
3. **History query API** (SC-3, SC-4): Expose block history queries through eth_getBlock* RPC endpoints in the rpc-eth crate
4. **Integration** (SC-5): Wire the new block store into whirlpool-node startup

## Constraints
- Must use existing reth-db/MDBX vendor stack (already a dependency via state-reth)
- Must not break existing consensus flow (block proposal, verification, finalization)
- Must be compatible with the existing `Block` trait and generic block types
- Must follow established crate patterns (trait-first boundaries, adapter isolation)

## Crates Involved
- **consensus-simplex**: Finalization hook (AppAdapter/FinalizationSink need to persist blocks)
- **state-reth**: Extend with block storage tables (or new sibling crate)
- **state**: May need BlockStore trait addition
- **rpc-eth**: Add eth_getBlock* endpoint implementations
- **whirlpool-node**: Wire block store into node startup
- **app / app-evm**: May need block type changes for serialization

## Prior Art
- `persistent-state-rethdb-20260305-1347`: Completed e2e for persistent StateDb via MDBX. Established patterns for reth-db table definitions, error handling, and node wiring. Same architectural approach applies here.
