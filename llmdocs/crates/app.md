# app

## Purpose
Application-facing interfaces and adapters that bridge execution logic to consensus.

## Interface/Implementation Split
- Interface module: `crates/app/src/traits.rs`
  - `Application`
  - `TxSource`
- Implementation module: `crates/app/src/tx_source.rs`
  - `NoopTxSource`
  - `InMemoryTxPool`
- Adapter module: `crates/app/src/adapter.rs`
  - `ApplicationAdapter` maps `Application` to `consensus::traits::ConsensusApp`.

## Canonical Imports
- Consensus traits: `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine}`
- App traits: `app::traits::{Application, TxSource}`
- Tx source implementations: `app::{InMemoryTxPool, NoopTxSource}`

## Key Types
- `EvmBlock`: block type used by the app layer.
- `ExecutionResult`: execution output returned by `Application::propose`/`Application::verify`.
- `ApplicationError`: app-layer error type.

## Status
Complete. Traits are isolated in `traits.rs`; concrete tx sources live in `tx_source.rs`.
