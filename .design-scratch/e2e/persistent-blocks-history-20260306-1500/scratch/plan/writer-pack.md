# Plan Writer Input Pack — Persistent Block Storage & History Queries

## Plan Metadata
- **Topic**: persistent-blocks-history
- **Plan directory**: `.sisyphus/plans/persistent-blocks-history/`
- **Entry shim**: `.sisyphus/plans/persistent-blocks-history.md`
- **Workspace root**: `/home/dev/sahara/web3/agent/playground/whirlpool`
- **Design docs root**: `.design-scratch/e2e/persistent-blocks-history-20260306-1500/docs/`

## Task Summary (11 tasks, 5 waves)

| Task | Slug | Crate(s) | Type | Complexity | Wave | Deps |
|------|------|----------|------|-----------|------|------|
| 01 | app-receipt-reexport | app | interface | S | 1 | none |
| 02 | state-blockstorage-trait | state | interface | M | 1 | none |
| 03 | app-evm-receipt-lifecycle-tests | app-evm | test | M | 2 | 01,02 |
| 04 | app-evm-finalization-persistence | app-evm | impl | M | 2 | 01,02,03 |
| 05 | state-reth-blockstorage-tests | state-reth | test | M | 3 | 02,04 |
| 06 | state-reth-mdbx-blockstorage | state-reth | impl | L | 3 | 02,04,05 |
| 07 | rpc-eth-block-endpoint-tests | rpc-eth | test | M | 3 | 02 |
| 08 | rpc-eth-context-api-surface | rpc-eth | impl | M | 4 | 02,07 |
| 09 | rpc-eth-handler-impl | rpc-eth | impl | M | 4 | 04,06,07,08 |
| 10 | whirlpool-node-wiring | whirlpool-node | wiring | M | 5 | 04,06,08,09 |
| 11 | integration-e2e-tests | (cross-crate) | test | M | 5 | 06,09,10 |

## AC Coverage per Task
- T01: (enables AC-1..AC-3 receipt type)
- T02: AC-1,AC-2,AC-3 (BlockStorage trait)
- T03: AC-4,AC-5 (receipt lifecycle tests)
- T04: AC-4,AC-5 (finalization impl)
- T05: AC-1,AC-2,AC-3 (storage tests)
- T06: AC-1,AC-2,AC-3 (MDBX impl)
- T07: AC-6..AC-10 (RPC tests)
- T08: AC-6..AC-10 (RPC context)
- T09: AC-6..AC-10 (RPC handler)
- T10: AC-11,AC-12 (node wiring)
- T11: AC-4,AC-6,AC-9,AC-11 (end-to-end)

## Key Build Commands
- All cargo commands via: `nix develop --command cargo ...`
- Per-crate test: `nix develop --command cargo test -p {crate_name}`
- Full build: `nix develop --command cargo build`
- Full test: `nix develop --command cargo test`

## Design Contract Highlights
- BlockStorage trait: 4 methods (store_block, get_block_by_number, get_block_by_hash, get_receipts_by_block)
- 8 MDBX tables for storage (Headers, HeaderNumbers, BlockBodyIndices, Transactions, TransactionHashNumbers, TransactionBlocks, Receipts, HeaderTerminalDifficulties)
- EthRpcContext gains B: BlockStorage generic
- PersistingFinalizationSink wraps FinalizationSink at node level
- No vendor/ modifications
