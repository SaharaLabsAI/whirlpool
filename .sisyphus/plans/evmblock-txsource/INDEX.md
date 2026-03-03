# evmblock-txsource — Execution Plan

## Execution Order

### Wave 1
- [ ] Task 1: InMemoryTxPool implementation + unit tests [**S**] → [tasks/01-impl-and-unit-tests.md](tasks/01-impl-and-unit-tests.md)
- [ ] Task 2: Node wiring update [**S**] → [tasks/02-node-wiring.md](tasks/02-node-wiring.md)

### Wave 2
- [ ] Task 3: Integration test [**S**] → [tasks/03-integration-test.md](tasks/03-integration-test.md)

### Wave 3
- [ ] Task 4: Full compliance audit [**S**] → [tasks/04-compliance-audit.md](tasks/04-compliance-audit.md)

<!-- TASKS_START -->
1. [01-impl-and-unit-tests](tasks/01-impl-and-unit-tests.md)
2. [02-node-wiring](tasks/02-node-wiring.md)
3. [03-integration-test](tasks/03-integration-test.md)
4. [04-compliance-audit](tasks/04-compliance-audit.md)
<!-- TASKS_END -->

## Dependency Graph
```
Task 01 (Pool implementation)  ──┐
                                 ├──→ Task 03 (Integration test) ──→ Task 04 (Audit)
Task 02 (Node wiring)          ──┘
```
