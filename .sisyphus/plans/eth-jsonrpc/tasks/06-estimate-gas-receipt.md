# Task 06: Implement `eth_estimateGas` and `eth_getTransactionReceipt` (S5)

**Status**: [ ] pending
**Dependencies**: Task 05
**Wave**: 1
**Complexity**: M

## AC Coverage
- AC-5 (`eth_estimateGas` returns `21000`)
- AC-9 (`eth_getTransactionReceipt` returns None for unknown hash)
- AC-10 (`eth_getTransactionReceipt` returns receipt for confirmed tx)

## Pre-Task Gate
- Confirm sendRawTransaction behavior from Task 05 passes.

## Context
Complete remaining method behavior using v1 constants and in-memory receipt index.

## What to do
1. Implement `eth_estimateGas(tx_request, block_id?)` in `eth_handler.rs`:
   - For v1 transfer path, return hardcoded `21000`.
   - Reject unsupported transaction shapes or selectors explicitly.
2. Implement `eth_getTransactionReceipt(hash)`:
   - Lookup in `receipt_store` keyed by tx hash.
   - Return `None` for unknown/unconfirmed transactions.
   - Return stored `TransactionReceipt` for confirmed transactions.
3. Implement receipt store helpers in `receipt_store.rs` for lookup/insert paths needed by handlers and runtime wiring.
4. Add targeted tests for gas estimate constant and receipt unknown/confirmed paths.

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/main/proven-ac.md` (AC-5, AC-9, AC-10)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/FLOWS.md` (gas + receipt flows)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/TESTS.md` (TC-004, TC-007, TC-008)

## Acceptance Criteria
- `eth_estimateGas` returns `21000` for supported transfer requests.
- Unknown receipt hash returns `None`.
- Confirmed receipt path returns receipt object from in-memory store.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/06-estimate-gas-receipt.log`
