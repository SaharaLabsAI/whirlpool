# IMPACT

## Blast Radius Summary

| Metric | Value | Notes |
| --- | --- | --- |
| In-scope crates | 4 direct, 2 structural dependents | Direct: `state`, `[PROPOSED] state-memory`, `app-evm`, `whirlpool-node`; structural dependents include workspace node binaries. |
| In-scope symbols | 7 | `StateDb`, `StateError`, `DBErrorMarker impl`, `DbAccount`, `InMemoryStateDb`, `DatabaseRef impl`, `Database impl`. |
| Public API path changes | 4 concrete surfaces | `DbAccount`/`InMemoryStateDb` and `DatabaseRef`/`Database` impl home move from `state::db` to `state_memory::db`. |
| Interface contract continuity | 3 retained surfaces | `state::traits::StateDb`, `state::error::StateError`, and `DBErrorMarker` remain in `state`. |
| Highest-risk seams | 3 | `crates/state/src/db.rs` extraction, `crates/app-evm/src/executor.rs`, `crates/whirlpool-node/src/main.rs`. |
| Expected breakage class | Compile-time | Import-path churn, missing dependency entries, trait-impl visibility mistakes. |
| Runtime semantic change target | None | Split is boundary-only; data/state semantics are expected unchanged. |

## Key Decisions

- **Grounded decision**: Keep interface contract symbols in `state` (`StateDb`, `StateError`, `DBErrorMarker`) because they are already consumed as stable trait/error surfaces (`crates/state/src/traits.rs::StateDb`, `crates/state/src/error.rs::StateError`, `crates/state/src/error.rs::DBErrorMarker impl`).
- **[PROPOSED] decision**: Move all concrete in-memory DB symbols into `state-memory` (`DbAccount`, `InMemoryStateDb`, `DatabaseRef`, `Database`) to make implementation opt-in and keep `state` interface-first.
- **Assumption**: Only `app-evm` and `whirlpool-node` are concrete-code consumers in workspace Cargo graph (`.design-scratch/refactor-e2e/split-state-interface-impl-20260305-0847/shared-dependency-graph.md::Reverse dependencies`).
- **Rejected alternative**: Keep concrete types re-exported from `state` after split. Rejected because it preserves interface/implementation coupling and risks a forbidden reverse edge (`state -> state-memory`).
- **Rationale**: The split must preserve current behavior while changing dependency shape (`state-memory -> state` only), which minimizes runtime risk and concentrates changes in imports/dependencies.

## Call Site Analysis

| Symbol | Old -> New | Change type | Primary call sites/files | Migration impact | Risk |
| --- | --- | --- | --- | --- | --- |
| `StateDb` | `state::traits::StateDb` -> unchanged | retain | `crates/app-evm/src/traits.rs`, `crates/app-evm/src/executor.rs` generic bounds | Trait imports should remain stable; no rename expected | Medium |
| `StateError` | `state::StateError` -> unchanged | retain | `crates/state/src/db.rs` impls, `crates/whirlpool-node/src/main.rs` `type Error` | Keep path stable for revm error contract continuity | Low-Medium |
| `DBErrorMarker impl` | `revm::database::DBErrorMarker for StateError` in `state` -> unchanged | retain | Indirectly required by `Database`/`DatabaseRef` impls | Must remain visible/publicly reachable via `StateError` | Low |
| `DbAccount` | `state::db::DbAccount` -> `state_memory::db::DbAccount` (and root re-export) | move | `crates/state/src/db.rs` internals/tests | Update concrete imports and crate dependency edges | Low-Medium |
| `InMemoryStateDb` | `state::db::InMemoryStateDb`/`state::InMemoryStateDb` -> `state_memory::db::InMemoryStateDb`/`state_memory::InMemoryStateDb` | move | `crates/app-evm/src/executor.rs`, `crates/app-evm/tests/*`, `crates/whirlpool-node/src/main.rs` | Broadest path churn; requires Cargo dependency updates | High |
| `DatabaseRef impl` | `impl DatabaseRef for state::db::InMemoryStateDb` -> `impl DatabaseRef for state_memory::db::InMemoryStateDb` | move | revm state access in `app-evm` execution flow | Compile breaks if impl visibility/error type linkage is wrong | Medium-High |
| `Database impl` | `impl Database for state::db::InMemoryStateDb` -> `impl Database for state_memory::db::InMemoryStateDb` | move | Runtime wrapper delegation in `whirlpool-node` | Compile/runtime integration seam; must preserve `StateError` linkage | Medium-High |

## Trait Impact

| Trait / Contract | Implementors / users | Files | Downstream consequence |
| --- | --- | --- | --- |
| `StateDb` | Implementor: `InMemoryStateDb`; users: `StateProvider` and EVM executor bounds | `crates/state/src/traits.rs`, `crates/state/src/db.rs`, `crates/app-evm/src/traits.rs` | Interface-only consumers remain on `state`; implementor moves crates without trait churn. |
| `revm::DatabaseRef` for in-memory DB | Implementor relocates with `InMemoryStateDb` | currently `crates/state/src/db.rs` | `app-evm` execution database reads continue only if impl migrates intact. |
| `revm::Database` for in-memory DB | Implementor relocates with `InMemoryStateDb` and `StateError` contract | currently `crates/state/src/db.rs`, wrapper usage in `crates/whirlpool-node/src/main.rs` | Node runtime wiring fails if impl/error marker continuity breaks. |

## Dependency Graph Impact

### Before

- `state` is mixed: interface + concrete implementation in one crate.
- `app-evm` and `whirlpool-node` import concrete DB directly from `state`.
- Concrete and interface concerns are inseparable in dependency terms.

### After ([PROPOSED])

- `state` remains interface/shared contract crate (`StateDb`, `StateError`, `DBErrorMarker`).
- New `state-memory` owns concrete DB and revm impls, and depends on `state`.
- Concrete consumers (`app-evm`, `whirlpool-node`) depend on `state-memory` for implementation and `state` for interface contracts.
- Dependency direction stays acyclic: `state-memory -> state` only.

## Cross-crate Boundary Changes

- **Public API shifts**: Concrete paths move from `state::*` to `state_memory::*`; trait/error interface paths remain stable in `state`.
- **Visibility updates**: `state` should stop being a facade for concrete DB types; `state-memory` becomes the concrete facade.
- **Compile-time seam**: Consumer rewiring in `app-evm` and `whirlpool-node` is the primary blast radius.
- **Compatibility posture**: Temporary re-exports are optional but must not introduce reverse dependency or long-term interface leakage.

## Known Unknowns

- `UNKNOWN`: Full non-code reference inventory (docs/scripts/examples) mentioning `state::InMemoryStateDb`.
- `UNKNOWN`: Whether transitional re-export period is required, or direct atomic path switch is feasible in one migration wave.
- `UNKNOWN`: Exact external (non-workspace) consumers of `state` concrete exports, if any.
