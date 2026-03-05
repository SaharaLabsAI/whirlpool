# STRATEGY

## Approach

- Use an **interface-first crate split**: keep `state` as the contract crate, introduce `state-memory` as the concrete in-memory implementation crate.
- Perform migration in bounded waves: stabilize interfaces, move implementation, rewire consumers, then clean transitional surfaces.
- Preserve behavioral semantics by moving code with minimal edits first, then applying only path/dependency rewiring.
- Keep dependency layering strict: `state-memory -> state`; never `state -> state-memory`.

## Key Decisions

- **Grounded**: `StateDb` and `StateError` are already consumed as interface contracts and must stay in `state` (`crates/state/src/traits.rs::StateDb`, `crates/state/src/error.rs::StateError`).
- **Grounded**: `InMemoryStateDb` import churn is concentrated in `app-evm` and `whirlpool-node` (`.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/impact-context.md::Primary Impact Findings`).
- **[PROPOSED]**: Add `state-memory` crate with root exports for `InMemoryStateDb` and `DbAccount` to keep consumer ergonomics stable after path migration.
- **Assumption**: Revm trait impls (`DatabaseRef`/`Database`) can move intact with `InMemoryStateDb` without semantic changes, because they already use `StateError` and local storage fields only.
- **Rejected alternative**: Keep mixed crate and split only by modules/files (`traits.rs` + concrete modules in same crate). Rejected due to explicit interface-first rule requiring physical crate split.
- **Rejected alternative**: Move `StateError` into `state-memory`. Rejected because all implementations (including wrappers like node DB adapters) require a shared contract error type anchored in the interface crate.
- **Rationale**: This ordering minimizes breakage by keeping trait/error surfaces stable while localizing high-risk edits to concrete import and Cargo dependency rewiring.

## Risk Assessment

| Risk | Impact | Likelihood | Assessment | Mitigation |
| --- | --- | --- | --- | --- |
| Missed concrete import rewrites in `app-evm` tests/runtime | High | High | Critical | Grep all `state::InMemoryStateDb` call sites, migrate in single wave with compile gate. |
| Incorrect trait impl relocation (`DatabaseRef`/`Database`) | High | Medium | High | Move impl blocks with type unchanged; immediately run crate-level compile checks. |
| Reverse dependency introduced (`state -> state-memory`) | High | Low-Medium | High | Enforce one-way dependency during Cargo edits and review metadata graph gate. |
| Transitional export ambiguity | Medium | Medium | Medium | Time-box compatibility re-exports; document canonical paths in migration steps. |
| Non-code path drift (docs/examples/scripts) | Low-Medium | Medium | Medium | Track as cleanup task and mark unknown references explicitly until scanned. |

## Ordering Constraints

1. Preserve and verify `state` interface exports (`StateDb`, `StateError`, `DBErrorMarker`) before moving concrete code.
2. Create and stabilize `state-memory` public exports before touching downstream consumers.
3. Migrate `app-evm` and `whirlpool-node` concrete imports only after `state-memory` compiles independently.
4. Keep Cargo graph acyclic at all times (`state-memory -> state` only).
5. Remove temporary compatibility surfaces only after consumer compile gates pass.

## Rollback Strategy

### Per-step rollback

- If interface stabilization fails: revert interface-surface edits only and restore previous `state` exports.
- If implementation move fails: revert `state-memory` move chunk and restore concrete code in `state::db`.
- If consumer rewiring fails: revert consumer imports/dependencies while keeping crate split branch intact for targeted retry.

### Full rollback

- Revert split sequence in reverse order: consumer rewiring -> implementation move -> interface stabilization.
- Restore original single-crate concrete paths (`state::InMemoryStateDb`, `state::DbAccount`).
- Re-run dependency-cycle and compile gates to validate baseline restoration.

### Roll-forward preference

- Prefer roll-forward fixes for isolated path/import issues (missing `use`, missing dependency entry, missing re-export).
- Use rollback when dependency direction or core trait/error contract integrity is compromised.
