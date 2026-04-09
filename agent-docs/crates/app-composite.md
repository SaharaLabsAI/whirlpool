# app-composite

## Purpose
Consensus-facing composite application that owns mixed transaction ingestion, classifies raw mempool bytes, and delegates domain-specific execution to `app-evm`.

## Location
`crates/app/execute/mem/composite/`

## Dependency Boundaries
- `app`: `Application`, `TxSource`, `EvmBlock`, `ExecutionResult`, `Receipt`.
- `app-evm`: pure EVM execution and receipt capture.
- `tx-dispatch`: mem-scoped mixed-tx classification into EVM or mem lanes.
- `state`: `BlockStorage` for finalized block persistence.

## Key Type
- `CompositeApplication<DB>`
  - Wraps an internal `EvmApplication<DB>`.
  - Owns the shared `TxSource`.
  - Maintains composite-level pending receipts and duplicate-proposal cache.
  - `store_finalized_block(&self, block, storage)`: persists finalized block receipts captured during propose/verify.
  - Mixed-tx propose path keeps only user mem+EVM tx bytes; epoch advancement happens in `app-evm` as an internal boundary system call.
  - Verify path delegates EVM subset verification to `app-evm`, which rejects reserved epoch-namespace synthetic tx artifacts.

## Status
Active. Moved under `crates/app/execute/mem/`; `whirlpool-node` no longer depends on it directly, and it consumes `tx-dispatch` from the same mem subtree.
