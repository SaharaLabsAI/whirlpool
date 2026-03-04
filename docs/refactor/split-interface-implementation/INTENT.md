# INTENT — Split Interface From Implementation

## Objective

Separate trait/interface definitions from concrete implementations across `app`, `consensus`, `p2p`, `state`, `consensus-simplex`, `p2p-commonware`, and `app-evm`, so each crate has a clear interface module boundary and implementation modules focused on behavior.

## Motivation

The workspace currently mixes abstraction and implementation boundaries:
- `app` and `p2p` already have trait modules, but `app::traits` includes concrete implementations (`NoopTxSource`, `InMemoryTxPool`).
- `consensus` traits are split across implementation-oriented modules (`app.rs`, `block.rs`, `event.rs`, `engine.rs`) without a dedicated interface boundary.
- `state` has concrete state DB implementation but no local interface module.
- `consensus-simplex` defines a trait (`CommonwareBlock`) inside `types.rs`.
- `p2p-commonware` is implementation-heavy and has no explicit local interface module boundary.
- `app-evm` defines `StateProvider` trait inside `executor.rs` with concrete implementation code.

A consistent split improves maintainability, testability, and API clarity between crates.

## Scope

### Crates

- `app`
- `consensus`
- `p2p`
- `state`
- `consensus-simplex`
- `p2p-commonware`
- `app-evm`

### Symbols

| Name | Current Path | Change Type | Target Path |
|---|---|---|---|
| Application | `app::traits::Application` | modify | `[PROPOSED] app::traits::Application` (interface-only module retained) |
| TxSource | `app::traits::TxSource` | modify | `[PROPOSED] app::traits::TxSource` (interface-only module retained) |
| NoopTxSource | `app::traits::NoopTxSource` | move | `[PROPOSED] app::tx_source::NoopTxSource` |
| InMemoryTxPool | `app::traits::InMemoryTxPool` | move | `[PROPOSED] app::tx_source::InMemoryTxPool` |
| ConsensusApp | `consensus::app::ConsensusApp` | move | `[PROPOSED] consensus::traits::ConsensusApp` |
| Block | `consensus::block::Block` | move | `[PROPOSED] consensus::traits::Block` |
| EventSink | `consensus::event::EventSink` | move | `[PROPOSED] consensus::traits::EventSink` |
| ConsensusEngine | `consensus::engine::ConsensusEngine` | move | `[PROPOSED] consensus::traits::ConsensusEngine` |
| PeerId | `p2p::traits::PeerId` | modify | `[PROPOSED] p2p::traits::PeerId` (interface-only module retained) |
| NetworkSender | `p2p::traits::NetworkSender` | modify | `[PROPOSED] p2p::traits::NetworkSender` (interface-only module retained) |
| NetworkReceiver | `p2p::traits::NetworkReceiver` | modify | `[PROPOSED] p2p::traits::NetworkReceiver` (interface-only module retained) |
| NetworkProvider | `p2p::traits::NetworkProvider` | modify | `[PROPOSED] p2p::traits::NetworkProvider` (interface-only module retained) |
| StateDb | `[MISSING]` | introduce | `[PROPOSED] state::traits::StateDb` |
| CommonwareBlock | `consensus-simplex::types::CommonwareBlock` | move | `[PROPOSED] consensus-simplex::traits::CommonwareBlock` |
| CommonwareTransport | `[MISSING]` | introduce | `[PROPOSED] p2p-commonware::traits::CommonwareTransport` |
| StateProvider | `app-evm::executor::StateProvider` | move | `[PROPOSED] app-evm::traits::StateProvider` |

### Depth

- `structural`

## Symbol List (Pre-computed)

1. **Application**
   - Name: `Application`
   - Current Path: `app::traits::Application`
   - Change Type: `modify`
   - Target Path: `[PROPOSED] app::traits::Application`
2. **TxSource**
   - Name: `TxSource`
   - Current Path: `app::traits::TxSource`
   - Change Type: `modify`
   - Target Path: `[PROPOSED] app::traits::TxSource`
3. **NoopTxSource**
   - Name: `NoopTxSource`
   - Current Path: `app::traits::NoopTxSource`
   - Change Type: `move`
   - Target Path: `[PROPOSED] app::tx_source::NoopTxSource`
4. **InMemoryTxPool**
   - Name: `InMemoryTxPool`
   - Current Path: `app::traits::InMemoryTxPool`
   - Change Type: `move`
   - Target Path: `[PROPOSED] app::tx_source::InMemoryTxPool`
5. **ConsensusApp**
   - Name: `ConsensusApp`
   - Current Path: `consensus::app::ConsensusApp`
   - Change Type: `move`
   - Target Path: `[PROPOSED] consensus::traits::ConsensusApp`
6. **Block**
   - Name: `Block`
   - Current Path: `consensus::block::Block`
   - Change Type: `move`
   - Target Path: `[PROPOSED] consensus::traits::Block`
7. **EventSink**
   - Name: `EventSink`
   - Current Path: `consensus::event::EventSink`
   - Change Type: `move`
   - Target Path: `[PROPOSED] consensus::traits::EventSink`
8. **ConsensusEngine**
   - Name: `ConsensusEngine`
   - Current Path: `consensus::engine::ConsensusEngine`
   - Change Type: `move`
   - Target Path: `[PROPOSED] consensus::traits::ConsensusEngine`
9. **PeerId**
   - Name: `PeerId`
   - Current Path: `p2p::traits::PeerId`
   - Change Type: `modify`
   - Target Path: `[PROPOSED] p2p::traits::PeerId`
10. **NetworkSender**
   - Name: `NetworkSender`
   - Current Path: `p2p::traits::NetworkSender`
   - Change Type: `modify`
   - Target Path: `[PROPOSED] p2p::traits::NetworkSender`
11. **NetworkReceiver**
   - Name: `NetworkReceiver`
   - Current Path: `p2p::traits::NetworkReceiver`
   - Change Type: `modify`
   - Target Path: `[PROPOSED] p2p::traits::NetworkReceiver`
12. **NetworkProvider**
   - Name: `NetworkProvider`
   - Current Path: `p2p::traits::NetworkProvider`
   - Change Type: `modify`
   - Target Path: `[PROPOSED] p2p::traits::NetworkProvider`
13. **StateDb**
   - Name: `StateDb`
   - Current Path: `[MISSING]`
   - Change Type: `introduce`
   - Target Path: `[PROPOSED] state::traits::StateDb`
14. **CommonwareBlock**
   - Name: `CommonwareBlock`
   - Current Path: `consensus-simplex::types::CommonwareBlock`
   - Change Type: `move`
   - Target Path: `[PROPOSED] consensus-simplex::traits::CommonwareBlock`
15. **CommonwareTransport**
   - Name: `CommonwareTransport`
   - Current Path: `[MISSING]`
   - Change Type: `introduce`
   - Target Path: `[PROPOSED] p2p-commonware::traits::CommonwareTransport`
16. **StateProvider**
   - Name: `StateProvider`
   - Current Path: `app-evm::executor::StateProvider`
   - Change Type: `move`
   - Target Path: `[PROPOSED] app-evm::traits::StateProvider`

## Success Criteria

1. Each focus crate has an explicit interface module boundary and separate implementation module boundary.
2. Public exports remain coherent and backwards-compatible where feasible through re-exports.
3. Trait definitions are not co-located with concrete runtime/storage/network implementations.
4. Module moves are staged so the workspace remains compilable at each migration step.

## Constraints

- Preserve existing crate-level API behavior unless explicit breaking changes are approved.
- Avoid vendor changes under `vendor/`.
- Keep refactor incremental and compatible with existing tests.
- Respect existing async trait patterns and associated type contracts.

## Out-of-Scope

- Redesign of consensus protocol logic.
- P2P protocol semantics or wire format changes.
- EVM execution semantics changes unrelated to interface extraction.
- New features unrelated to interface/implementation separation.

## Threshold Gate

- Crates in scope: 7 (`> 6` threshold)
- Symbols to change (core list): 16 (`> 8` threshold)
- Depth levels: 1 (`<= 3` threshold)
- Estimated cross-crate boundaries: 7 (`> 4` threshold)
- Estimated migration steps: > 20

Verdict: **SPLIT PROPOSED** (scope exceeds intake threshold; proceed only after splitting execution into manageable waves while preserving one shared intent baseline).
