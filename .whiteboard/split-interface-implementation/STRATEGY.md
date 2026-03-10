# STRATEGY

## Approach

- Use an **interface-first structural refactor**: introduce/normalize trait modules first, move trait definitions next, then relocate concrete implementations.
- Keep workspace compilable after each batch by preserving compatibility re-exports until downstream imports are migrated.
- Enforce dependency layering (`foundation -> app -> adapters -> nodes`) as a hard ordering constraint.
- Treat this effort as boundary extraction only; do not alter trait semantics, associated types, or async contracts.

## Key Decisions

- **Grounded**: Migration direction is `foundation -> app -> adapters -> consumers` (`.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/migration-context.md::Dependency Chain`).
- **Grounded**: Existing high-risk zones are `consensus-simplex` generic bounds, `p2p-commonware` provider associated types, and `app-evm` executor-local trait coupling (`.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/migration-context.md::High-Risk Areas`).
- **[PROPOSED]**: Canonicalize traits into `traits` modules for all focus crates, with compatibility exports kept until final cleanup wave.
- **[PROPOSED]**: Introduce missing traits (`StateDb`, `CommonwareTransport`) in additive mode first, then wire implementors and consumers.
- **[PROPOSED]**: Defer compatibility-export removal to the final consumer cleanup wave only after full build/test green.

## Wave / Batch Ordering

| Wave | Scope | Rationale | Exit gate |
| --- | --- | --- | --- |
| Wave A (low risk) | `consensus` trait consolidation + `state::traits::StateDb` introduction | Foundation crates first; minimizes downstream churn early | All foundational crates compile with old and new paths available |
| Wave B (medium risk) | `app` concrete tx-source split + `app-evm::StateProvider` relocation | Align app abstraction and EVM boundary once foundation contracts are stable | `app`, `app-evm`, node imports compile through compatibility shims |
| Wave C (high risk) | `consensus-simplex::CommonwareBlock` relocation + `p2p-commonware::CommonwareTransport` introduction + consumer import cleanup | Most coupled generic/adapter surfaces; done last to reduce blast radius | Nodes compile on canonical paths; temporary compatibility exports removable |

## Ordering Constraints

- Introduce trait/interface modules before moving symbol definitions.
- Add compatibility re-exports before updating dependent imports.
- Update internal crate imports before cross-crate consumers.
- Migrate generic-bound-heavy crates (`consensus-simplex`, `p2p-commonware`) only after upstream interfaces are stable.
- Keep `state`, `p2p`, and `consensus` dependency direction unchanged to prevent cycle creation.

## Risk Assessment

| Risk | Impact | Likelihood | Assessment | Mitigation |
| --- | --- | --- | --- | --- |
| Trait path breakage in generic bounds (`consensus-simplex`) | High | High | Critical | Dual-path transition (new canonical + compatibility re-export), migrate internal uses first |
| Executor/state trait relocation regressions (`app-evm`) | High | Medium | High | Move trait to `traits` with temporary executor re-export and unchanged bounds |
| Provider associated-type mismatch (`p2p-commonware`) | High | Medium | High | Introduce transport trait additively; avoid changing existing associated type signatures in same step |
| Downstream node import drift | Medium | High | High | Consumer cleanup as final wave; keep crate-root exports stable until completion |
| Hidden test regressions during staged moves | Medium | Medium | Medium | Require per-wave `cargo check` + targeted crate tests before next wave |

## Rollback Strategy

### Per-wave rollback

- If a wave fails compilation, revert only that wave’s changeset and restore prior compatibility exports/imports.
- Re-run `cargo check` for all crates touched by the failed wave before attempting a reordered replay.
- Do not proceed to subsequent waves until the current wave is green.

### Full rollback

- Revert all synthesize/migration implementation commits related to interface relocation in reverse wave order (C -> B -> A).
- Restore original symbol homes and crate-root exports.
- Validate baseline with workspace-wide `cargo check` and existing integration tests.

### Roll-forward preference

- Prefer roll-forward fixes (restore missing re-export, patch import path) when failure is isolated and semantics are unchanged.
- Use rollback only when trait/module ordering assumptions are violated or dependency layering is threatened.

## Verification Spine (for migration execution)

- Wave A gate: foundational crates compile with compatibility exports.
- Wave B gate: `app` and `app-evm` compile with unchanged trait bounds.
- Wave C gate: adapter crates and node binaries compile with canonical paths.
- Final gate: remove transitional exports only after all consumers are updated and tests are green.
