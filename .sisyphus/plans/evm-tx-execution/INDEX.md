# Task Index — EVM Transaction Execution

## Execution Order

### Wave 1: Foundation (parallel)
- [x] [01-state-commit-tests](tasks/01-state-commit-tests.md) — S
- [x] [02-tx-decode-helper](tasks/02-tx-decode-helper.md) — S

### Wave 2: Core Execution (sequential)
- [x] [03-propose-execution](tasks/03-propose-execution.md) — L
- [x] [04-verify-execution](tasks/04-verify-execution.md) — L

### Wave 3: Integration (sequential)
- [x] [05-integration-test](tasks/05-integration-test.md) — M
- [x] [06-workspace-verification](tasks/06-workspace-verification.md) — XS
- [ ] [07-update-llmdocs](tasks/07-update-llmdocs.md) — S

## Dependency Graph
```
Task 01 (State tests) ────────┐
                              ├──→ Task 03 (Propose) ──→ Task 04 (Verify) ──→ Task 05 (Integration) ──→ Task 06 (Full verify) ──→ Task 07 (llmdocs)
Task 02 (Decode helper) ──────┘                    ↗
                                                   /
(Note: Task 04 depends on 03 for test block generation)
```
