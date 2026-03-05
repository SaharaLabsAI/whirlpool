# Step 2 Refactor Contract Pack

## Intent scope summary
- Split `state` into interface-only `state` and concrete `state-memory` crates.
- Keep `StateDb`, `StateError`, and `DBErrorMarker` in `state`; move `DbAccount`, `InMemoryStateDb`, and revm impls to `state-memory`.
- Rewire concrete consumers (`app-evm`, `whirlpool-node`) to `state-memory` while preserving trait/error contracts from `state`.

## Migration steps
- Total steps: 6
1. Lock Interface Surface in `state`
2. Scaffold `state-memory` Crate
3. Move Concrete DB + `revm` Impl Blocks
4. Rewire `app-evm` to Concrete Crate
5. Rewire `whirlpool-node` Runtime Wrapper
6. Remove Transitional Concrete Paths from `state`

## Broken tests and TestIDs
- Broken-test count: 6
- TestIDs: TB-001, TB-002, TB-003, TB-004, TB-005, TB-006
- New-test IDs: TN-001, TN-002, TN-003, TN-004, TN-005, TN-006

## Blast-radius summary (IMPACT)
| Metric | Value |
|---|---|
| In-scope crates | 4 direct (`state`, `state-memory`, `app-evm`, `whirlpool-node`) |
| In-scope symbols | 7 |
| Public API path changes | 4 concrete surfaces move from `state::*` to `state_memory::*` |
| Interface continuity | `StateDb`, `StateError`, `DBErrorMarker` stay in `state` |
| Highest-risk seams | `crates/state/src/db.rs`, `crates/app-evm/src/executor.rs`, `crates/whirlpool-node/src/main.rs` |

## Strategy risk areas
- Missed concrete import rewrites in `app-evm` tests/runtime.
- Incorrect relocation of `DatabaseRef` / `Database` impl blocks.
- Reverse dependency introduction (`state -> state-memory`).
- Transitional export ambiguity and stale non-code references.

## Crate roles
- `state`: source + interface target
- `state-memory`: implementation target (new crate)
- `app-evm`: concrete consumer rewire
- `whirlpool-node`: concrete consumer rewire

## Blockers
- none (0 open, 0 active)
