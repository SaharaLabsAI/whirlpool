# Intent

## Objective
Split the current `state` crate into two physical crates so interface-only consumers can depend on `state`, while concrete in-memory DB users depend on `state-memory`.

## Motivation
The current crate mixes interface and implementation concerns (`StateDb` + `StateError` + in-memory/revm implementation), which forces consumers to pull concrete DB details even when they only need the trait surface.

## Scope
### Crates
- `state` (kept as interface crate)
- `state-memory` ([PROPOSED] new implementation crate)

### Symbols
| Name | Current Path | Change Type | Target Path |
|---|---|---|---|
| `StateDb` | `state::traits::StateDb` (`crates/state/src/traits.rs`) | modify | `state::traits::StateDb` (interface-only crate) |
| `StateError` | `state::error::StateError` (`crates/state/src/error.rs`) | modify | `state::error::StateError` (interface-only crate) |
| `DBErrorMarker impl` | `revm::database::DBErrorMarker for StateError` (`crates/state/src/error.rs`) | modify | stays with `StateError` in `state` |
| `DbAccount` | `state::db::DbAccount` (`crates/state/src/db.rs`) | move | `state_memory::db::DbAccount` |
| `InMemoryStateDb` | `state::db::InMemoryStateDb` (`crates/state/src/db.rs`) | move | `state_memory::db::InMemoryStateDb` |
| `DatabaseRef impl` | `revm::DatabaseRef for InMemoryStateDb` (`crates/state/src/db.rs`) | move | `state_memory::db::impl DatabaseRef for InMemoryStateDb` |
| `Database impl` | `revm::Database for InMemoryStateDb` (`crates/state/src/db.rs`) | move | `state_memory::db::impl Database for InMemoryStateDb` |

### Depth
- `architectural` (explicit): changes crate boundaries and dependency shape.

## Success Criteria
- `state` exports only interface/shared API (`StateDb`, `StateError`, shared types) and retains the `DBErrorMarker` impl for `StateError`.
- `state-memory` contains `InMemoryStateDb`, `DbAccount`, and revm database integration impls.
- Interface-only consumers can depend on `state` without implementation payload.
- Concrete DB consumers can opt into `state-memory`.

## Constraints
- Keep behavior and semantics of existing `StateDb` and in-memory DB logic unchanged.
- Preserve compatibility with revm integration points during the split.
- Follow interface-first crate split rule (no `traits.rs`-only reshuffle inside a mixed crate).

## Out-of-Scope
- Rewriting in-memory DB behavior or state-root algorithm.
- Introducing alternative DB backends.
- Performing broad dependency cleanup outside split requirements.

## Threshold Gate
- Crates in scope: 2 (`state`, `state-memory`) — below `>6` threshold.
- Symbols in scope: 7 — below `>8` threshold.
- Depth levels affected: 1 (`architectural`) — below `>3` threshold.
- Cross-crate boundaries: estimated 2-3 — below `>4` threshold.
- Estimated migration steps: 8-12 — below `>15` threshold.

Verdict: **No additional sub-refactoring split required** for this intake.
