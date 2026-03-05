# Refactor Split Tracking

- parent_intent: split-state-interface-impl
- threshold_result: NO_SPLIT_REQUIRED
- split_required: no
- rationale: Intake thresholds were not exceeded (crates <= 6, symbols <= 8, boundaries <= 4, migration steps <= 15).

## Sub-Refactorings
| ID | Description | Status |
|---|---|---|
| SR-001 | Split `state` interface from in-memory implementation into `state` + `state-memory` | deferred |
