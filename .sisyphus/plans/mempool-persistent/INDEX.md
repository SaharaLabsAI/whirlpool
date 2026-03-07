# mempool-persistent — Execution Plan

## TL;DR
- **Goal**: Implement persistent mempool storage so pending transactions survive node restart while preserving existing drain/FIFO semantics.
- **Primary crates**: `app`, `rpc-eth`, `mempool` (new), `whirlpool-node`, `integration-tests`.
- **Implementation order**: Strictly follows STRATEGY phases 1→7.

## Context
- Design root: `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/`
- Proven AC source: `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/proven-ac.md`
- Core constraints:
  - Do not modify `vendor/`
  - Preserve current drain-on-`pending()` semantics
  - Keep `TxSource` infallible at call boundary

## Phase / Wave Plan
- **Wave 1 (Phase 1)**: TxSource trait extension in `app`
- **Wave 2 (Phase 2)**: Update in-memory + no-op implementations in `app`
- **Wave 3 (Phase 3)**: `EthRpcContext` generification in `rpc-eth`
- **Wave 4 (Phase 4)**: New `mempool` crate (`MempoolStore` + `PersistentTxPool`)
- **Wave 5 (Phase 5)**: Node wiring in `whirlpool-node`
- **Wave 6 (Phase 6)**: Integration tests
- **Wave 7 (Phase 7)**: End-to-end validation and workspace verification

- **Wave 8 (Verification)**: Final verification of all ACs and invariants

## AC Coverage Matrix (from proven-ac.md)

| AC ID | Criterion | Covered By Tasks |
|---|---|---|
| AC-1 | Transactions persist across node restart | 05, 06, 07, 08, 09 |
| AC-2 | Existing tests continue to pass (regression) | 02, 03, 08, 09 |
| AC-3 | New mempool crate unit tests cover public API | 04, 05, 06 |
| AC-4 | `EthRpcContext` works with trait object | 01, 03, 07, 08 |
| AC-5 | FIFO ordering verified by tests | 05, 06, 08 |

## Task List
<!-- TASKS_START -->
- [ ] Task 01: app-txsource-trait-extension [**S**] → [tasks/01-app-txsource-trait-extension.md](tasks/01-app-txsource-trait-extension.md)
- [ ] Task 02: app-inmemory-noop-alignment [**S**] → [tasks/02-app-inmemory-noop-alignment.md](tasks/02-app-inmemory-noop-alignment.md)
- [ ] Task 03: rpc-eth-context-generification [**S**] → [tasks/03-rpc-eth-context-generification.md](tasks/03-rpc-eth-context-generification.md)
- [ ] Task 04: mempool-crate-scaffold [**S**] → [tasks/04-mempool-crate-scaffold.md](tasks/04-mempool-crate-scaffold.md)
- [ ] Task 05: mempool-store-implementation-tests [**M**] → [tasks/05-mempool-store-implementation-tests.md](tasks/05-mempool-store-implementation-tests.md)
- [ ] Task 06: persistent-txpool-trait-impl-tests [**M**] → [tasks/06-persistent-txpool-trait-impl-tests.md](tasks/06-persistent-txpool-trait-impl-tests.md)
- [ ] Task 07: whirlpool-node-persistent-wiring [**M**] → [tasks/07-whirlpool-node-persistent-wiring.md](tasks/07-whirlpool-node-persistent-wiring.md)
- [ ] Task 08: cross-crate-integration-tests [**M**] → [tasks/08-cross-crate-integration-tests.md](tasks/08-cross-crate-integration-tests.md)
- [ ] Task 09: e2e-and-workspace-validation [**S**] → [tasks/09-e2e-and-workspace-validation.md](tasks/09-e2e-and-workspace-validation.md)
- [ ] Task 10: final-verification [**M**] → [tasks/10-final-verification.md](tasks/10-final-verification.md)
<!-- TASKS_END -->

## Dependency Graph
```text
01 -> 02 -> 03 -> 07 -> 08 -> 09
         \-> 04 -> 05 -> 06 -/
```

## Required Build / Test Gates
- `nix develop --command cargo build`
- `nix develop --command cargo test`

## Must-Not Constraints
- Do not modify `vendor/`
- Do not change consensus semantics outside existing tx source behavior
- Do not alter design docs under `.design-scratch/e2e/mempool-persistent-20260307-1000/docs/`
