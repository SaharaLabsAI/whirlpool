# Test Contracts

## Strategy
- Prioritize contract-boundary behavior over internals: `ConsensusApp <-> ApplicationAdapter <-> EvmApplication <-> InMemoryStateDb`.
- Keep current MVP behavior covered as `[GROUNDED]` while encoding missing seams as explicit `[PROPOSED]` blocker/revise tests.
- Treat INV-01..INV-07 as CONFIRMED invariant targets and map each to observable pass/fail oracles.
- Use pseudo-code Rust outlines only (no full implementations) so these contracts are TDD-ready without inventing APIs.

## Confirmed invariants

| ID | Invariant (CONFIRMED) | Current status |
|---|---|---|
| INV-01 | Execution Visibility | BLOCKER |
| INV-02 | Verification Integrity | BLOCKER/PARTIAL |
| INV-03 | Verification Read-Only | PARTIAL/GROUNDED (current root-read path) |
| INV-04 | Snapshot Safety | UNKNOWN/BLOCKER |
| INV-05 | Commit Atomicity | BLOCKER |
| INV-06 | Root Consistency | MIXED (empty-path grounded, non-empty blocked) |
| INV-07 | Proposal Determinism | GROUNDED for current MVP-empty path; UNKNOWN for non-empty ordering policy |

## Intent success-criteria mapping

| INTENT success criterion | Test contract file(s) | Test case IDs |
|---|---|---|
| #1 Transaction execution in propose() | `tests/app-evm-unit.md`, `tests/evm-execution-integration.md`, `tests/cross-crate-flows.md` | satisfies-now: none; gap-sentinels: `XFLOW-001`; target-pass: `AEVM-U-006`, `EVM-INT-004`, `XFLOW-006` |
| #2 Transaction verification in verify() | `tests/app-evm-unit.md`, `tests/evm-execution-integration.md`, `tests/cross-crate-flows.md` | satisfies-now: none; gap-sentinels: `AEVM-U-003`, `AEVM-U-004`, `EVM-INT-002`, `XFLOW-002`; target-pass: `AEVM-U-007`, `EVM-INT-005`, `EVM-INT-006`, `EVM-INT-007` |
| #3 TxSource implementation | `tests/app-unit.md`, `tests/whirlpool-node-unit.md`, `tests/cross-crate-flows.md` | satisfies-now: none; gap-sentinels: `APP-U-001`, `NODE-U-001`, `XFLOW-004`; target-pass: `APP-U-006`, `NODE-U-004` |
| #4 State lifecycle (snapshot/commit/rollback) | `tests/state-unit.md`, `tests/app-evm-unit.md`, `tests/evm-execution-integration.md`, `tests/cross-crate-flows.md` | `STATE-U-006`, `AEVM-U-008`, `EVM-INT-008`, `XFLOW-003` |
| #5 Block assembly correctness | `tests/app-evm-unit.md`, `tests/evm-execution-integration.md`, `tests/cross-crate-flows.md` | `AEVM-U-001`, `AEVM-U-006`, `AEVM-U-007`, `EVM-INT-001`, `EVM-INT-004`, `EVM-INT-005`, `EVM-INT-006`, `EVM-INT-007`, `XFLOW-001`, `XFLOW-002` |
| #6 Wiring in whirlpool-node | `tests/whirlpool-node-unit.md`, `tests/block-production-integration.md`, `tests/cross-crate-flows.md` | `NODE-U-001`, `NODE-U-002`, `NODE-U-004`, `BP-INT-006`, `XFLOW-004` |
| #7 End-to-end propose -> finalize -> commit | `tests/block-production-integration.md`, `tests/cross-crate-flows.md` | satisfies-now: none; gap-sentinels: `BP-INT-004`, `XFLOW-003`, `XFLOW-005`; target-pass: `BP-INT-006` |

## Index

| Category | File | Scope |
|---|---|---|
| Unit: `app-evm` | `tests/app-evm-unit.md` | Proposal/verification contract behavior and gaps |
| Unit: `app` | `tests/app-unit.md` | Adapter mapping and tx-source contract boundary |
| Unit: `state` | `tests/state-unit.md` | Commit/root/read semantics and snapshot primitive |
| Unit: `whirlpool-node` | `tests/whirlpool-node-unit.md` | Startup wiring and finalization seam expectations |
| Integration: block production | `tests/block-production-integration.md` | Runtime + consensus progression and finalize coupling |
| Integration: evm execution | `tests/evm-execution-integration.md` | End-to-end propose/verify behavior in EVM app stack |
| Cross-crate flows | `tests/cross-crate-flows.md` | Architecture flow contracts across crate boundaries |

## Grounded status notes
- Current proposal path is MVP-empty (`transactions=[]`, empty tx/receipt roots, `gas_used=0`) and must stay covered as `[GROUNDED]`.
- Current verify path compares `state_root` only; tx/receipt/gas replay checks remain `[PROPOSED]` blockers.
- `ConsensusApp` has no finalize callback, and `FinalizationSink` only updates finalized height; finalize->commit remains a blocker seam.

## Open questions
- `TxSource::pending()` ordering and deterministic selection contract for non-empty proposals.
- Ownership/storage of commit-ready artifact across propose -> finalize boundary.
- Byte-identical snapshot boundary definition for INV-04 in integration scope.
- Structured invalid-block mismatch taxonomy beyond stringified adapter mapping.
