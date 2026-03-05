# Test Context (Explore Step 2b)

## Coverage Baseline

- Strong current coverage exists in `state` unit tests (`crates/state/src/db.rs`) for commit/state-root/storage behavior and `revm` read paths.
- `app-evm` tests exercise concrete DB behavior through propose/verify execution flows, providing cross-crate behavioral signal.
- `whirlpool-node` concrete bridge (`TestStateDb`) has limited direct test coverage, creating a targeted validation gap.

## Breakage Classes and Signals

| Test class | Likely break trigger | Expected signal |
| --- | --- | --- |
| Compile-time trait wiring | Missing `StateDb`/`StateError` visibility or marker impl | trait bound / impl errors |
| Concrete import migration | Old `state::InMemoryStateDb` paths left behind | unresolved import errors |
| Runtime DB behavior | Drift after moving `DbAccount`/`InMemoryStateDb` | state_root/commit assertion failures |
| `revm` integration | Missing/misplaced `DatabaseRef`/`Database` impls | execution/setup failures in `app-evm` |

## Required Test Contracts for Split

1. **Interface contract**
   - `state::traits::StateDb` remains importable and signature-compatible for interface-only consumers.

2. **Error/marker contract**
   - `StateError` stays usable as `revm` DB error with `DBErrorMarker` intact.

3. **Concrete behavior parity contract**
   - `state-memory::InMemoryStateDb` preserves existing commit/state_root/getter semantics.

4. **Consumer integration contract**
   - `app-evm` and `whirlpool-node` continue functioning with new concrete import paths.

## Recommended Verification Sequence

1. `nix develop --command cargo test -p state`
2. `nix develop --command cargo test -p app-evm`
3. `nix develop --command cargo test -p whirlpool-node`
4. `nix develop --command cargo test` (workspace confidence sweep)

## Gaps to Address During Implementation

- Add explicit error-path test coverage for `StateError::Internal` propagation.
- Add targeted node bridge test or smoke coverage for `TestStateDb` delegation.
- Add compile-only check ensuring interface-only use of `state::traits::StateDb` without concrete crate coupling.

## Source Artifacts

- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-test-coverage.md`
- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-impact-analysis.md`
- `.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-module-structure.md`
