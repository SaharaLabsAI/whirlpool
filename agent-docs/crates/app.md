# app

## Purpose
Application-facing interfaces and adapters that bridge execution logic to consensus.

## Interface/Implementation Split
- Interface module: `crates/app/traits/src/traits.rs`
  - `Application`
  - `TxSource`: interface for transaction storage and retrieval. Methods: `pending()`, `push(tx: Vec<u8>)`. Bounds: `Send + Sync`.
- Implementation module: `crates/app/traits/src/tx_source.rs`
  - `NoopTxSource`: no-op implementation of `TxSource`.
  - `InMemoryTxPool`: in-memory implementation of `TxSource`.
- Adapter module: `crates/app/traits/src/adapter.rs`
  - `ApplicationAdapter` maps `Application` to `consensus::traits::ConsensusApp`.

## Canonical Imports
- Consensus traits: `consensus::traits::{Block, ConsensusApp, EventSink, ConsensusEngine}`
- App traits: `app::traits::{Application, TxSource}`
- Tx source implementations: `app::{InMemoryTxPool, NoopTxSource}`

## Key Types
- `EvmBlock`: block type used by the app layer. Fields: `height: u64`, `parent_id: [u8; 32]`, `state_root: [u8; 32]`, `transactions_root: [u8; 32]`, `receipts_root: [u8; 32]`, `proposer_public_key: [u8; 32]`, `proposer_fee_recipient: [u8; 20]`, `extra_data: Vec<u8>`, `gas_used: u64`, `base_fee_per_gas: u64`, `timestamp: u64`, `transactions: Vec<Vec<u8>>`.
- `extra_data` helpers (`types.rs`): canonical sectioned codec (`RawEth`, `FullDkgV1`), strict/legacy/rpc decode modes, raw-eth projection helper for RPC compatibility.
- Canonical envelope decode enforces section order (`RawEth` before `FullDkgV1` when both are present) and rejects out-of-order sections.
- `Receipt`: alloy-consensus receipt type, re-exported for app-layer use.
- `ExecutionResult`: execution output returned by `Application::propose`/`Application::verify`.
- `ApplicationError`: app-layer error type.

## Status
Complete. Traits are isolated in `traits.rs`; concrete tx sources live in `tx_source.rs`.
