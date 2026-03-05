# Task 07: Wire RPC server in `main.rs` and add alloy e2e integration test (S6)

**Status**: [ ] pending
**Dependencies**: Task 06
**Wave**: 1
**Complexity**: L

## AC Coverage
- AC-11 (RPC server starts with consensus engine)
- AC-12 (alloy end-to-end balance transfer flow)

## Pre-Task Gate
- Confirm all RPC methods from Tasks 03-06 pass.

## Context
Finalize runtime integration and black-box e2e coverage with alloy `ProviderBuilder`.

## What to do
1. Update `crates/whirlpool-node/src/config.rs` with `RPC_BIND_ADDR` constant if missing.
2. Wire `crates/whirlpool-node/src/main.rs` to:
   - construct `EthRpcContext` with cloned `tx_pool`, `state_db`, `receipt_store`, `chain_id`, `block_height` handles,
   - start RPC server only after `engine.start()`,
   - keep runtime alive with both consensus and RPC tasks.
3. Ensure runtime path fails fast or logs clearly on RPC bind failure per design policy.
4. Add integration tests in `crates/whirlpool-node/tests/` using alloy `ProviderBuilder`:
   - transfer send-and-confirm flow,
   - balance delta assertions after confirmation.
5. Confirm this task is the final implementation slice and includes end-to-end validation.

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/main/proven-ac.md` (AC-11, AC-12)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/FLOWS.md` (S6)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/TESTS.md` (TC-009, TC-010)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/whirlpool-node/README.md`

## Acceptance Criteria
- Node starts consensus and RPC server in same runtime lifecycle.
- Alloy e2e transfer test passes including receipt polling and balance-delta checks.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/07-main-wiring-alloy-e2e.log`
