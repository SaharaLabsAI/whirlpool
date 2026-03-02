# Test Contract Context (Seed Phase) — EVM Block Production

## 1. Intent Success Criteria (from INTENT.md)
| ID | Success Criterion | Source |
|---|---|---|
| #1 | Transaction execution in `propose()`: `EvmApplication` executes txs via reth pipeline. | INTENT.md:38 |
| #2 | Transaction verification in `verify()`: Re-execution against parent state with result validation. | INTENT.md:39 |
| #3 | Concrete `TxSource` implementation: Real mempool or queue-based source (not `NoopTxSource`). | INTENT.md:40 |
| #4 | Correct state lifecycle: Snapshot before execution, commit after success, rollback on failure. | INTENT.md:41 |
| #5 | Block assembly correctness: Valid roots (state, txs, receipts) and accurate gas_used. | INTENT.md:42 |
| #6 | Wiring in `whirlpool-node`: Binary correctly connects TxSource, EVM pipeline, and state. | INTENT.md:43 |
| #7 | End-to-end flow: Consensus triggers proposal, execution, assembly, and finalization. | INTENT.md:44 |

---

## 2. Major Risks & Failure Modes
| Risk / Failure Mode | Observable Symptom | Evidence |
|---|---|---|
| `propose()` not executing txs | Blocks produced with zero transactions; no state changes. | shared-context:47 |
| `verify()` not re-executing | Invalid blocks accepted; state root mismatches undetected. | shared-context:48 |
| No concrete `TxSource` | Block production limited to empty blocks; system cannot ingest txs. | shared-context:49 |
| State snapshot/rollback failure | State corruption across block proposal/verification attempts. | INTENT #4 |
| Non-deterministic inputs | `propose()` and `verify()` diverge on the same block/tx set. | INTENT #2/#7 |
| Root/gas miscalculation | Header roots or `gasUsed` values do not match execution results. | INTENT #5 |

---

## 3. Candidate End-to-End Scenarios
| Scenario | Action | Expected Pass/Fail Signal | Criteria |
|---|---|---|---|
| **Happy Path: Propose → Finalize** | `propose()` with txs → `finalize()` | Block has txs; state changes persisted; roots match. | #1, #3, #4, #5, #7 |
| **Verify Happy Path** | `verify()` proposed block | Re-execution matches block header; block accepted. | #2, #5, #7 |
| **Verify: Tampered Tx List** | `verify()` with mutated tx list | Reject block; roots/gas mismatch detected; no commit. | #2, #4, #5 |
| **Rollback on Propose Failure** | Trigger error during `propose()` | State equals pre-call snapshot; no effects persisted. | #1, #4 |
| **Rollback on Verify Failure** | `verify()` returns `Reject` | No side effects on canonical state from verification. | #2, #4 |
| **Empty Mempool** | `propose()` with empty `TxSource` | Valid empty block produced; roots/gas=0 consistent. | #1, #3, #5 |

---

## 4. Proposed Invariants
- **[PROPOSED] Execution Visibility**: If `TxSource` provides ≥1 valid tx, the block from `propose()` must show state deltas or logs.
- **[PROPOSED] Verification Integrity**: `verify()` must recompute execution artifacts and reject if they differ from the block header.
- **[PROPOSED] Verification Read-Only**: Calling `verify()` must never alter the canonical state, regardless of the outcome.
- **[PROPOSED] Snapshot Safety**: State after a failed `propose()` or `verify()` call must be byte-for-byte identical to the pre-call state.
- **[PROPOSED] Commit Atomicity**: `finalize()` must apply all block effects exactly once; partial commits are prohibited.
- **[PROPOSED] Root Consistency**: Block roots (state, tx, receipts) must always be derived from the actual execution of included txs.
- **[PROPOSED] Proposal Determinism**: Given identical state and `TxSource` responses, `propose()` must produce identical blocks.

---

## 5. Open Unknowns
- Authoritative set of artifacts validated in `verify()` (which specific roots/fields).
- Tx validity rules enforced before vs. during execution (nonce, balance, gas).
- Failure policy for individual invalid txs within a proposed block (skip vs. record failure).
- State model boundaries (what exactly is covered by snapshot/rollback).
- Exact trigger point for finalization (which component calls `commit`).
- `TxSource` semantics (max block gas limits, transaction ordering rules).

---

## 6. Domain & Wiring Implications
- **Domain: Execution Determinism**: Proposal and verification MUST achieve parity on state/artifact results for identical inputs.
- **Domain: State Lifecycle**: Explicit snapshot/rollback assertions are required around all execution paths.
- **Wiring: Node Orchestration**: `whirlpool-node` must wire `TxSource` → `propose()` → `verify()` → `finalize/commit`.
- **Wiring: Resource Access**: `propose()` and `verify()` need access to the same EVM config and a state handle supporting rollback.
- **Contract: app-evm**: `propose()` and `verify()` interfaces must be updated from stubs to execution-aware logic.
- **Contract: state**: `InMemoryStateDb` must guarantee atomic commits and precise snapshots.
