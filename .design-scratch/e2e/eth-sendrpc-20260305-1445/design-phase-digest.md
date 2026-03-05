# Design Phase Digest

## Summary
Design for adding minimal Ethereum JSON-RPC server (7 methods) to whirlpool-node. Server enables alloy client to perform basic ETH balance transfers and verify results via integration tests.

## Key Findings
- No existing RPC infrastructure in workspace — greenfield addition
- InMemoryTxPool already Arc-wrapped and thread-safe — ready for sharing
- StateDb accessible via Arc<RwLock<TestStateDb>> — read lock for balance/nonce queries
- Critical gap: receipts dropped during block execution — resolved with in-memory ReceiptStore
- jsonrpsee 0.26 + alloy-primitives 1.5.0 + alloy-rpc-types 1.4.3 (matching reth vendor)

## Architecture Decision
RPC implemented as modules inside whirlpool-node (not separate crate). Shared state via EthRpcContext struct holding Arc references to tx_pool, state_db, receipt_store, block_height.

## Files Produced
INTENT.md, SHARED_CONTEXT.md, STRATEGY.md, CRATES.md, WORKSPACE.md, DOMAINS.md, FLOWS.md, TESTS.md, BLOCKERS.md, SUMMARY.md, INDEX.md, EXPLORATION.md, EXPLORATION_DIGEST.md, app/README.md, whirlpool-node/README.md

## Verdict
PASS — no open blockers, design is coherent and complete.
