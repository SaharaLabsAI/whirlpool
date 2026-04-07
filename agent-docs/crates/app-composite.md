# app-composite

## Purpose
Consensus-facing composite application that owns mixed transaction ingestion, classifies raw mempool bytes, and delegates domain-specific execution to `app-evm`.

## Location
`crates/mem/composite/`

## Dependency Boundaries
- `app`: `Application`, `TxSource`, `EvmBlock`, `ExecutionResult`, `Receipt`.
- `app-evm`: pure EVM execution and receipt capture.
- `tx-dispatch`: neutral mixed-tx classification into EVM or mem lanes.
- `state`: `BlockStorage` for finalized block persistence.

## Key Type
- `CompositeApplication<DB>`
  - Wraps an internal `EvmApplication<DB>`.
  - Owns the shared `TxSource`.
  - Maintains composite-level pending receipts and duplicate-proposal cache.
  - `store_finalized_block(&self, block, storage)`: persists finalized block receipts captured during propose/verify.

## Status
Active. Moved under `crates/mem/`; `whirlpool-node` no longer depends on it directly.
