# IMPACT

## Blast Radius Summary

| Metric | Value | Notes |
| --- | --- | --- |
| In-scope crates | 7 | `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, `app-evm` |
| In-scope symbols | 16 | 12 existing symbols adjusted/moved, 2 new traits introduced, 2 retained as interface-only cleanups |
| Structural dependents | 2 | `whirlpool-node`, `whirlpool-node-simple` consume adapter surfaces indirectly |
| Highest-risk seams | 3 | `consensus-simplex` generic bounds, `app-evm` executor/database boundary, `p2p-commonware` provider multiplexing |
| Expected breakage type | Compile-time | Import path churn, trait location changes, temporary re-export mismatches |
| Runtime semantic change target | None | Refactor is structural only; behavior should remain unchanged |
| Test impact | UNKNOWN | Detailed breakage mapping is deferred to test synthesis |

## Key Decisions

- **Grounded**: Keep compatibility re-exports during migration so downstream crates can compile while trait/concrete symbols move (`.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/migration-context.md::Compilation Order Constraints`).
- **Grounded**: Preserve dependency layering (`foundation -> app -> adapters -> nodes`) to avoid introducing reverse edges (`.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/impact-context.md::Cross-Crate Dependency Notes`).
- **[PROPOSED]**: Use interface-first moves (introduce traits modules first, then relocate implementations, then clean imports/re-exports).
- **[PROPOSED]**: Treat new interfaces (`state::traits::StateDb`, `p2p-commonware::traits::CommonwareTransport`) as additive contracts before any removal of compatibility exports.

## Affected Symbols by Crate

| Crate | Affected symbols | Change type | Crate-local impact |
| --- | --- | --- | --- |
| `app` | `Application`, `TxSource`, `NoopTxSource`, `InMemoryTxPool` | Modify + move | `traits` must become interface-only; concrete tx-source types move to implementation module with re-exports |
| `consensus` | `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine` | Move | Consolidate trait definitions behind `consensus::traits` while preserving old paths temporarily |
| `p2p` | `PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider` | Modify (retained paths) | Maintain stable trait module contract; no semantic trait redesign |
| `state` | `StateDb` (new) | Introduce | Add trait boundary over existing `InMemoryStateDb` without changing foundational dependency position |
| `consensus-simplex` | `CommonwareBlock` | Move | Extract trait from `types` into interface module; update internal generic bounds/imports |
| `p2p-commonware` | `CommonwareTransport` (new) | Introduce | Define explicit transport contract distinct from sender/receiver/provider implementations |
| `app-evm` | `StateProvider` | Move | Separate trait from executor implementation while keeping `EvmApplication<DB: StateProvider>` bounds stable |

## Call Site and Trait Impact (Cross-Crate)

| Symbol | Old -> Proposed | Primary implementors/consumers | Risk |
| --- | --- | --- | --- |
| `NoopTxSource`, `InMemoryTxPool` | `app::traits::*` -> `app::tx_source::*` | Consumed by `app-evm` tests and node wiring | Medium (constructor/import path churn) |
| `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine` | `consensus::{app,block,event,engine}` -> `consensus::traits::*` | Bound heavily by `consensus-simplex` engine/adapter generics | High |
| `CommonwareBlock` | `consensus-simplex::types::CommonwareBlock` -> `consensus-simplex::traits::CommonwareBlock` | Internal simplex adapter/engine bounds | High |
| `StateProvider` | `app-evm::executor::StateProvider` -> `app-evm::traits::StateProvider` | `EvmApplication`, state adapters, node shim types | Medium-High |
| `StateDb` | `[MISSING]` -> `state::traits::StateDb` | `state` + `app-evm` abstraction boundary | Medium |
| `CommonwareTransport` | `[MISSING]` -> `p2p-commonware::traits::CommonwareTransport` | `p2p-commonware` transport/provider abstraction | Medium-High |

## Dependency Graph Impact

### Before
- `consensus`, `p2p`, `state` are foundational crates.
- `app` depends on `consensus`.
- `consensus-simplex`, `p2p-commonware`, `app-evm` are adapter crates bridging foundational interfaces to concrete runtime behavior.
- Nodes consume adapters and wiring.

### After ([PROPOSED])
- Crate dependency graph remains unchanged.
- Module-internal boundaries become explicit: interface modules contain traits; implementation modules contain concrete behavior.
- Transitional re-exports absorb path migration cost until all consumers switch to canonical interface paths.

## Cross-Crate Boundary Change Analysis

- **Public API shifts**: Primary changes are canonical module paths for moved traits/structs; backward compatibility should be preserved via temporary re-exports.
- **Visibility updates**: New `traits` modules in `state`, `consensus-simplex`, `p2p-commonware`, and `app-evm` become interface entry points.
- **Most sensitive boundary**: `consensus-simplex` because it composes consensus and networking contracts through dense generic constraints.
- **Cycle risk**: Introducing `state::traits::StateDb` must remain interface-only to avoid adding reverse dependencies from `state` to adapter crates.

## Known Unknowns

- `StateDb` and `CommonwareTransport` are intentionally absent today and require first-time contract definition.
- Full broken-test inventory is `UNKNOWN` until test synthesis maps migration steps to concrete failures.
