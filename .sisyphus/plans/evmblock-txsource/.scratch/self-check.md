# Self-check — evmblock-txsource plan vs design docs

## Verdict: REVISE

## Issues (must fix)
1. **SC coverage gap (SC-5: node wiring uses `InMemoryTxPool`)**: Task 02 AC only proves the node builds; it does **not** uniquely verify the `NoopTxSource → InMemoryTxPool` swap (node would still compile if it continued using `NoopTxSource`). Add an explicit verification step to Task 02 AC (e.g., a manual check item) that confirms `crates/whirlpool-node/src/main.rs` instantiates `Arc<InMemoryTxPool>` and passes it into `EvmApplication::new()`.
2. **SC coverage gap (SC-6: existing tests continue to pass)**: Task 04 AC runs `cargo test` only for `app` and `app-evm`, not the whole workspace as required by INTENT SC-6 (verification listed as `cargo test`). Update Task 04 AC to include `nix develop --command cargo test` (or `... cargo test --workspace`) so SC-6 is actually exercised.
3. **Plan ordering inconsistency**: `INDEX.md` labels Wave 1 as “parallel”, but Task 02 declares a dependency on Task 01. Either (a) remove the dependency (only if truly independent), or (b) change Wave 1 to sequential / “can be started in parallel but completes after Task 01”.

## Coverage checks

### 1) Success criteria (SC-1..SC-7) each covered by ≥1 task AC
- **SC-1..SC-4**: Covered by **Task 01** AC (`nix develop --command cargo test -p app ...`).
- **SC-5**: **Not adequately covered** (see Issue #1).
- **SC-6**: **Not adequately covered** (see Issue #2).
- **SC-7**: Covered by **Task 03** AC (`nix develop --command cargo test -p app-evm --test integration test_propose_with_in_memory_pool`).

### 2) Test contracts (T-1..T-7) map to tasks
- **T-1..T-6** → Task 01 (`01-impl-and-unit-tests.md` Design Refs include T-1..T-6)
- **T-7** → Task 03 (`03-integration-test.md` Design Refs include T-7)

**Note**: TESTS.md T-7 details assert *gas_used > 0* and *recipient balance updated*; Task 03’s text currently only mentions asserting inclusion. Consider explicitly stating those assertions in Task 03 to match the design contract.

### 3) Implementation slices (S-1..S-4) map to tasks
- **S-1, S-2** → Task 01
- **S-3** → Task 02
- **S-4** → Task 03

### 4) Dependency ordering (no task depends on a higher-numbered task)
- **OK**: Task 02 → Task 01; Task 03 → Tasks 01 & 02; Task 04 → Tasks 01–03.

### 5) AC commands use `nix develop --command`
- **OK**: All tasks’ AC command blocks use the prefix.

### 6) No vendor modifications proposed
- **OK**: All tasks target `crates/app`, `crates/whirlpool-node`, `crates/app-evm` only.

### 7) No XL tasks that should be split
- **OK**: Tasks are small and scoped; Task 04 is a verification/audit step.
