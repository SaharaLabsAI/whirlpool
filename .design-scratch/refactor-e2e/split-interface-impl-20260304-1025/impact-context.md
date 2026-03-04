# Impact Context (Explore Step 2b)

## Blast Radius Summary

| Area | Primary symbols | Directly affected crates | Structural dependents | Impact |
| --- | --- | --- | --- | --- |
| Application boundary | `Application`, `TxSource`, `NoopTxSource`, `InMemoryTxPool` | `app`, `app-evm` | `whirlpool-node` | High (trait+concrete split in same module today) |
| Consensus boundary | `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine` | `consensus` | `app`, `consensus-simplex`, `app-evm`, `whirlpool-node*` | High (widely imported trait paths) |
| P2P boundary | `PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider` | `p2p`, `p2p-commonware` | `consensus-simplex`, `whirlpool-node*` | High (adapter-heavy downstream impls) |
| State boundary | `StateDb` (new), existing `InMemoryStateDb` | `state` | `app-evm`, `whirlpool-node` | Medium (new trait introduction) |
| Commonware adapter boundary | `CommonwareBlock`, `CommonwareTransport` (new) | `consensus-simplex`, `p2p-commonware` | `whirlpool-node*` | Medium-High (generic bounds + adapter contracts) |
| EVM boundary | `StateProvider` | `app-evm` | `whirlpool-node` | Medium (trait currently colocated with executor impl) |

## Affected Symbols (Current -> Proposed)

1. `Application` (trait): `app::traits::Application` -> `app::traits::Application` (interface-only retained)
2. `TxSource` (trait): `app::traits::TxSource` -> `app::traits::TxSource` (interface-only retained)
3. `NoopTxSource` (struct): `app::traits::NoopTxSource` -> `app::tx_source::NoopTxSource`
4. `InMemoryTxPool` (struct): `app::traits::InMemoryTxPool` -> `app::tx_source::InMemoryTxPool`
5. `ConsensusApp` (trait): `consensus::app::ConsensusApp` -> `consensus::traits::ConsensusApp`
6. `Block` (trait): `consensus::block::Block` -> `consensus::traits::Block`
7. `EventSink` (trait): `consensus::event::EventSink` -> `consensus::traits::EventSink`
8. `ConsensusEngine` (trait): `consensus::engine::ConsensusEngine` -> `consensus::traits::ConsensusEngine`
9. `PeerId` (trait): `p2p::traits::PeerId` -> `p2p::traits::PeerId` (retained)
10. `NetworkSender` (trait): `p2p::traits::NetworkSender` -> `p2p::traits::NetworkSender` (retained)
11. `NetworkReceiver` (trait): `p2p::traits::NetworkReceiver` -> `p2p::traits::NetworkReceiver` (retained)
12. `NetworkProvider` (trait): `p2p::traits::NetworkProvider` -> `p2p::traits::NetworkProvider` (retained)
13. `StateDb` (trait): `[MISSING]` -> `state::traits::StateDb` (introduce)
14. `CommonwareBlock` (trait): `consensus-simplex::types::CommonwareBlock` -> `consensus-simplex::traits::CommonwareBlock`
15. `CommonwareTransport` (trait): `[MISSING]` -> `p2p-commonware::traits::CommonwareTransport` (introduce)
16. `StateProvider` (trait): `app-evm::executor::StateProvider` -> `app-evm::traits::StateProvider`

## Cross-Crate Dependency Notes

- Stable layering from dependency analysis: `consensus/p2p/state` (foundation) -> `app` -> `consensus-simplex`, `p2p-commonware`, `app-evm` -> `whirlpool-node`, `whirlpool-node-simple`.
- `consensus-simplex` couples strongly to consensus traits in generic bounds (`ConsensusApp`, `EventSink`, `ConsensusEngine`, `CommonwareBlock`) and is the highest sensitivity zone for path changes.
- `p2p-commonware` is the principal implementation site for `p2p` traits (`NetworkSender`, `NetworkReceiver`, `NetworkProvider`) and must keep trait imports stable during migration.
- `app-evm` bridges `app::Application` and local `StateProvider` with `state::InMemoryStateDb`; introducing `state::traits::StateDb` must avoid cycles by keeping `state` interface-only.

## Gaps / Known Unknowns

- `StateDb` and `CommonwareTransport` are planned interfaces and currently absent in source scans.
- Impact raw data for all symbols is partly consolidated from existing shard plus direct symbol scans.

## Raw Data Pointers

- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-module-structure.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-dependency-graph.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-test-coverage.md`
- `.design-scratch/refactor-e2e/split-interface-impl-20260304-1025/shared-impact-analysis-shard-06.md`
- `docs/refactor/split-interface-implementation/INTENT.md`
