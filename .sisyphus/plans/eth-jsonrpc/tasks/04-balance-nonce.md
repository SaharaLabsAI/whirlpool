# Task 04: Implement `eth_getBalance` and `eth_getTransactionCount` (S3)

**Status**: [ ] pending
**Dependencies**: Task 03
**Wave**: 1
**Complexity**: M

## AC Coverage
- AC-2 (`eth_getBalance` returns value for known account)
- AC-3 (`eth_getBalance` returns 0 for unknown account)
- AC-4 (`eth_getTransactionCount` returns nonce)

## Pre-Task Gate
- Confirm constant RPC methods from Task 03 pass.

## Context
Add read-only account state RPC methods against node-owned `Arc<RwLock<TestStateDb>>`.

## What to do
1. Implement `eth_getBalance(address, block_id?)` in `eth_handler.rs`:
   - Resolve supported block tags (at least `latest`; reject unsupported selectors explicitly).
   - Read account from `state_db`; return balance or `U256::ZERO` when absent.
2. Implement `eth_getTransactionCount(address, block_id?)`:
   - Same block selector policy as balance.
   - Return account nonce mapped to RPC U256 (or equivalent expected numeric RPC type).
3. Add/extend tests for known-account and unknown-account balance, and nonce retrieval.
4. Ensure lock/error paths map to internal RPC errors consistently.

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/main/proven-ac.md` (AC-2, AC-3, AC-4)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/FLOWS.md` (state-read flow)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/TESTS.md` (TC-002, TC-003)

## Acceptance Criteria
- `eth_getBalance` returns expected balance for seeded account.
- `eth_getBalance` returns zero for unknown account.
- `eth_getTransactionCount` returns nonce from account state.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/04-balance-nonce.log`
