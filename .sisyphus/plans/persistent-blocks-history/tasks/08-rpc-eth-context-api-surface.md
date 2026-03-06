# Task 08: rpc-eth-context-api-surface

**Status**: pending
**Dependencies**: 02, 07
**Wave**: 4
**Complexity**: M
**Target Crate(s)**: rpc-eth (role: impl)

## Pre-Task Gate
- `nix develop --command cargo build -p state` succeeds.
- Task 07 tests exist and fail (as expected).

## Context
Before implementing the block query handlers, the `rpc-eth` crate must be updated to support the `BlockStorage` dependency. This involves adding a generic `B: BlockStorage` to the `EthRpcContext` and adding the new `eth_getBlockByNumber` and `eth_getBlockByHash` endpoints to the `EthApi` trait.

## What to do

### TDD Flow
1. Add `B: BlockStorage` generic to the `EthRpcContext` struct.
2. Update the `EthRpcContext::new()` constructor to accept the new `block_storage` field.
3. Add `eth_getBlockByNumber` and `eth_getBlockByHash` to the `EthApi` RPC trait.
4. Verify the tests from Task 07 now compile (but still fail implementation).

### Specific steps
1. Edit `crates/rpc-eth/src/context.rs`:
   - Change `EthRpcContext<S: StateDb>` to `EthRpcContext<S: StateDb, B: BlockStorage>`.
   - Add `pub block_storage: Arc<RwLock<B>>` to the struct fields.
   - Update `pub fn new(...) -> Self` to accept `block_storage: Arc<RwLock<B>>`.
2. Edit `crates/rpc-eth/src/eth_api.rs`:
   - Add `#[method(name = "getBlockByNumber")]` to the `EthApi` trait with parameters `block_number: BlockNumberOrTag` and `full: bool`.
   - Add `#[method(name = "getBlockByHash")]` to the `EthApi` trait with parameters `hash: B256` and `full: bool`.
3. Edit `crates/rpc-eth/src/eth_handler.rs`:
   - Add stub implementations for the new RPC methods that return `Result::Err(Error::MethodNotFound)`.
   - Update `EthApiHandler<S>` to `EthApiHandler<S, B>`.
4. Edit `crates/rpc-eth/src/server.rs`:
   - Update `start_rpc_server` to accept the new generic `B`.

## Mock Boundary
N/A (actual implementation)

## Must NOT do
- Do NOT implement the actual handler logic in this task.
- Do NOT change existing RPC method signatures.

## References
- `.design-scratch/e2e/persistent-blocks-history-20260306-1500/scratch/plan/grounding-map.md`
- `docs/crates/rpc-eth.md`

## Acceptance Criteria
- `nix develop --command cargo test -p rpc-eth` succeeds in compilation (even if tests fail).
- `nix develop --command cargo build -p rpc-eth` succeeds.

## Post-Task Gate
- Command: `nix develop --command cargo test -p rpc-eth && nix develop --command cargo build -p rpc-eth`
- Expected: exit 0
- Max retries: 3

## Post-Task Reconciliation
- Update Artifact Registry in INDEX.md (TC-RPC-01, 04, 05, 07 status: pending_impl).

## QA Scenarios
- QA-6, QA-7, QA-9: RPC queries.

## Evidence
`.sisyphus/evidence/task-08-rpc-eth-context-api-surface.txt`
