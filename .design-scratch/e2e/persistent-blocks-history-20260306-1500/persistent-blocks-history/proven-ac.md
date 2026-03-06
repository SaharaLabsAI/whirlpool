# Proven Acceptance Criteria — Persistent Block Storage & History Queries

Extracted from proof.md after Phase 2 PROVE approval.

## Invariants (10)

| ID | Title | Scope | Verification |
|----|-------|-------|-------------|
| INV-1 | MDBX Atomicity | state-reth | TC-SR-01, TC-SR-07 |
| INV-2 | TxNumber Monotonicity | state-reth | TC-SR-03 |
| INV-3 | Block Reconstruction Fidelity | state-reth | TC-SR-05, TC-UNK-02 |
| INV-4 | Hash→Number Consistency | state-reth | TC-SR-06 |
| INV-5 | Receipt Count Invariant | state-reth, app-evm | TC-SR-04, TC-AE-02 |
| INV-6 | Finalization Idempotency | app-evm, state-reth | TC-AE-04 |
| INV-7 | Consensus Independence | whirlpool-node | TC-INT-02 |
| INV-8 | RPC Type Fidelity | rpc-eth | TC-RPC-02, TC-RPC-04 |
| INV-9 | Existing Functionality Preservation | all | TC-INT-02, existing test suites |
| INV-10 | Thread Safety | state-reth, rpc-eth | TC-SR-08 |

## Acceptance Criteria (12)

| AC | SC | Title | QA |
|----|-----|-------|----|
| AC-1 | SC-1 | Header persistence | QA-1 |
| AC-2 | SC-1 | Transaction persistence | QA-2 |
| AC-3 | SC-1 | Receipt persistence | QA-3 |
| AC-4 | SC-2 | Finalization triggers storage | QA-4 |
| AC-5 | SC-2 | Receipts captured from propose | QA-5 |
| AC-6 | SC-3 | getBlockByNumber with numeric | QA-6 |
| AC-7 | SC-3 | getBlockByNumber with tags | QA-7 |
| AC-8 | SC-3 | Full vs hash-only response | QA-8 |
| AC-9 | SC-4 | getBlockByHash with full | QA-9 |
| AC-10 | SC-4 | getBlockByHash returns None for unknown | QA-10 |
| AC-11 | SC-5 | Node starts with block storage wired | QA-11 |
| AC-12 | SC-5 | Existing RPC methods still work | QA-12 |

## Coverage Matrix

| AC | QA | INV | SC |
|----|----|-----|-----|
| AC-1 | QA-1 | INV-1, INV-3 | SC-1 |
| AC-2 | QA-2 | INV-1, INV-2 | SC-1 |
| AC-3 | QA-3 | INV-1, INV-5 | SC-1 |
| AC-4 | QA-4 | INV-6, INV-7 | SC-2 |
| AC-5 | QA-5 | INV-5 | SC-2 |
| AC-6 | QA-6 | INV-3, INV-8 | SC-3 |
| AC-7 | QA-7 | INV-8, INV-9 | SC-3 |
| AC-8 | QA-8 | INV-8 | SC-3 |
| AC-9 | QA-9 | INV-4, INV-8 | SC-4 |
| AC-10 | QA-10 | INV-8 | SC-4 |
| AC-11 | QA-11 | INV-9, INV-10 | SC-5 |
| AC-12 | QA-12 | INV-9 | SC-5 |

## Build Order
1. `app` — Receipt re-export
2. `state` — BlockStorage trait
3. `app-evm` — Visibility changes + pending_receipts + store_finalized_block
4. `state-reth` — BlockStorage MDBX impl
5. `rpc-eth` — New endpoints + EthRpcContext generic
6. `whirlpool-node` — PersistingFinalizationSink wiring
