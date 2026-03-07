# MANIFEST

## Inputs Consumed
- crates/app/src/tx_source.rs — InMemoryTxPool implementation
- crates/app/src/traits.rs — TxSource trait definition
- crates/rpc-eth/src/context.rs — EthRpcContext with tx_pool field
- crates/rpc-eth/src/eth_handler.rs — send_raw_transaction usage
- crates/whirlpool-node/src/main.rs — Node wiring of tx_pool
- llmdocs/crates/state-reth.md — Existing persistence patterns

## Outputs Produced
- .design-scratch/e2e/mempool-persistent-20260307-1000/e2e-state.md
- .design-scratch/e2e/mempool-persistent-20260307-1000/SKILL_DIGEST.md
- .design-scratch/e2e/mempool-persistent-20260307-1000/STATE_DELTA.md
- .design-scratch/e2e/mempool-persistent-20260307-1000/MANIFEST.md
