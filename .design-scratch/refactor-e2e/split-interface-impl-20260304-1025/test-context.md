# Test Context (Explore Step 2b)

## Test Impact Summary

- Existing Step 2 coverage artifact reports passing crate-level suites for all 7 focus crates.
- No red failures reported for current symbol surfaces; highest operational risk is regressions from import-path churn and trait relocation.
- Existing reliability note: `p2p-commonware` required `CARGO_BUILD_JOBS=1` retry due to transient resource/ICE issue, then passed.

## Breakage Risks by Test Type

| Test type | Likely break trigger during split | Expected signal |
| --- | --- | --- |
| Unit tests | Trait module moves without local import rewrites | compile errors in defining crates |
| Integration tests | Re-export path instability | unresolved import errors in `app-evm` and node tests |
| Mock-feature tests | Trait path changes under cfg (`consensus`, `p2p`) | feature-specific compile failures |
| Adapter behavior tests | Changed associated type bounds / moved traits | type mismatch errors in `consensus-simplex` and `p2p-commonware` |
| Doc tests/examples | Public API path churn | doctest compile failures or stale examples |

## Required Test Contracts for Migration

1. **Path compatibility contract**
   - Old and new trait paths must compile during transition window (via re-exports/deprecation shims).

2. **Trait contract parity**
   - Moved traits preserve signatures and associated types exactly.
   - Adapter impl blocks (`NetworkProvider`, `ConsensusEngine`, `Application`) remain type-equivalent.

3. **Behavioral parity for moved concrete types**
   - `NoopTxSource` and `InMemoryTxPool` maintain existing `TxSource` semantics.
   - `StateProvider` relocation does not alter EVM propose/verify state flow.

4. **New trait introduction contract**
   - `StateDb` introduction requires conformance tests against `InMemoryStateDb` behaviors already covered by `state` tests.
   - `CommonwareTransport` introduction requires send/recv/channel mapping tests mirroring existing `p2p-commonware` provider/receiver/sender behavior.

## Common Test Patterns to Reuse

- cfg-gated module tests already present in all relevant crates; keep these near moved traits while paths stabilize.
- Cross-crate integration tests in `app-evm/tests/*` are the primary regression detector for app/consensus/state interface changes.
- `consensus-simplex/src/tests.rs` and `p2p-commonware/src/tests.rs` provide adapter-level validation for trait bound integrity and message flow.
- Node binaries (`whirlpool-node`, `whirlpool-node-simple`) serve as end-to-end compile/use checks for re-export surface correctness.

## Suggested Verification Sequence During Implementation

1. `cargo test -p consensus --features mock`
2. `cargo test -p p2p`
3. `cargo test -p state`
4. `cargo test -p app`
5. `cargo test -p p2p-commonware` (fallback with `CARGO_BUILD_JOBS=1` if needed)
6. `cargo test -p consensus-simplex`
7. `cargo test -p app-evm`
8. `cargo test -p whirlpool-node -p whirlpool-node-simple` (or workspace subset compile/test)

## Raw Data Pointers

- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-test-coverage.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-module-structure.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-dependency-graph.md`
- `docs/refactor/split-interface-implementation/INTENT.md`
