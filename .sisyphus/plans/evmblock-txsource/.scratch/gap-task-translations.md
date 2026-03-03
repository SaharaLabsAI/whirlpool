# Gap→Task Translations — evmblock-txsource

## Implementation Status

All 4 implementation slices from FLOWS.md are **complete** per the grounding map.
This plan is a **verification plan** confirming the implementation matches the design.

## Task List

### Task 1: InMemoryTxPool implementation + unit tests
- **Slice**: S-1, S-2
- **Complexity**: S
- **Wave**: 1
- **Dependencies**: none
- **Files**: `crates/app/src/traits.rs`, `crates/app/src/lib.rs`
- **Status**: COMPLETE — InMemoryTxPool exists with new(), push(), TxSource::pending(), Default impl, all 6 unit tests pass
- **Verification**: `nix develop --command cargo test -p app -- traits`

### Task 2: Node wiring update
- **Slice**: S-3
- **Complexity**: S
- **Wave**: 1 (parallel with Task 1)
- **Dependencies**: Task 1
- **Files**: `crates/whirlpool-node/src/main.rs`
- **Status**: COMPLETE — NoopTxSource replaced with InMemoryTxPool, Arc handle retained as tx_pool
- **Verification**: `nix develop --command cargo build -p whirlpool-node --bin whirlpool-node`

### Task 3: Integration test
- **Slice**: S-4
- **Complexity**: S
- **Wave**: 2
- **Dependencies**: Task 1, Task 2
- **Files**: `crates/app-evm/tests/integration.rs`
- **Status**: COMPLETE — test_propose_with_in_memory_pool exists and passes
- **Verification**: `nix develop --command cargo test -p app-evm --test integration test_propose_with_in_memory_pool`

### Task 4: Full compliance audit
- **Complexity**: S
- **Wave**: 3
- **Dependencies**: Task 1, Task 2, Task 3
- **Verification**: 
  - `nix develop --command cargo build`
  - `nix develop --command cargo test -p app`
  - `nix develop --command cargo test -p app-evm`
  - Check SC-1 through SC-7 from INTENT.md

## Dependency Ordering

```
Task 1 (impl + unit tests)  ──┐
                                ├── Task 3 (integration test) ── Task 4 (audit)
Task 2 (node wiring)        ──┘
```

## Wave Assignment

- **Wave 1**: Task 1, Task 2 (parallel)
- **Wave 2**: Task 3
- **Wave 3**: Task 4
