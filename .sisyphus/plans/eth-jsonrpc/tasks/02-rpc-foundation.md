# Task 02: RPC foundation modules and server bootstrap (S1)

**Status**: [ ] pending
**Dependencies**: Task 01
**Wave**: 1
**Complexity**: M

## AC Coverage
- AC-11 (RPC server lifecycle scaffolding alongside node runtime)

## Pre-Task Gate
- Confirm Task 01 dependency pins are present in manifests.

## Context
Create node-local RPC module skeleton and core context/server wiring contract inside `whirlpool-node` crate.

## What to do
1. Create module tree under `crates/whirlpool-node/src/rpc/`:
   - `mod.rs`
   - `eth_api.rs`
   - `eth_handler.rs`
   - `context.rs`
   - `receipt_store.rs`
   - `server.rs`
2. Define `EthRpcContext` in `context.rs` with Arc fields:
   - `tx_pool`, `state_db`, `receipt_store`, `chain_id`, `block_height`.
3. Define in-memory receipt store contract in `receipt_store.rs`:
   - `HashMap<B256, TransactionReceipt>` read/write API for handler usage.
4. Define `eth` namespace trait in `eth_api.rs` with all 7 methods (signatures only / placeholder returns).
5. Define `EthApiHandler` type in `eth_handler.rs` implementing trait with placeholder behavior sufficient to compile.
6. Implement `start_rpc_server(ctx, addr) -> ServerHandle` in `server.rs` and export modules in `mod.rs`.
7. Do not wire `main.rs` yet (reserved for S6).

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/FLOWS.md` (S1)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/DOMAINS.md`
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/whirlpool-node/README.md`

## Acceptance Criteria
- `crates/whirlpool-node/src/rpc/` contains all six required module files.
- `EthRpcContext` includes all required Arc references.
- RPC trait includes all seven required method definitions.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/02-rpc-foundation.log`
