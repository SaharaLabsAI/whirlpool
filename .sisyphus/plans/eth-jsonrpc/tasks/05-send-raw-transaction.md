# Task 05: Implement `eth_sendRawTransaction` tx-pool ingress (S4)

**Status**: [ ] pending
**Dependencies**: Task 04
**Wave**: 1
**Complexity**: M

## AC Coverage
- AC-7 (`eth_sendRawTransaction` returns tx hash)
- AC-8 (`eth_sendRawTransaction` pushes bytes into tx pool)

## Pre-Task Gate
- Confirm state-read RPC methods from Task 04 pass.

## Context
Enable transaction ingress through RPC by validating input bytes minimally, hashing deterministically, and pushing to shared `InMemoryTxPool`.

## What to do
1. Implement `eth_sendRawTransaction(bytes)` in `eth_handler.rs`:
   - Validate input is non-empty / decodable for v1 policy.
   - Compute deterministic transaction hash (`B256`) from raw bytes.
   - Push bytes into `tx_pool` via context handle.
   - Return hash to caller.
2. Ensure errors map to JSON-RPC invalid params for malformed tx input.
3. Add targeted test(s) proving both returned hash and tx-pool insertion behavior.

## Design Refs
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/main/proven-ac.md` (AC-7, AC-8)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/FLOWS.md` (tx ingress flow)
- `.design-scratch/e2e/eth-sendrpc-20260305-1445/docs/TESTS.md` (TC-006)

## Acceptance Criteria
- Valid raw transaction returns a stable `B256` hash.
- Raw bytes are pushed into shared `InMemoryTxPool`.
- Invalid transaction bytes produce JSON-RPC error response.

## Post-Task Gate
- `nix develop --command cargo build`
- `nix develop --command cargo test`
- Expected: both commands exit 0.

## Evidence
- `.sisyphus/evidence/eth-jsonrpc/05-send-raw-transaction.log`
