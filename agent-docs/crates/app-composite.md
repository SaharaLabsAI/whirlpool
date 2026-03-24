# app-composite

## Purpose
Consensus-facing composite application that owns mixed transaction ingestion, classifies raw mempool bytes, and delegates domain-specific execution to `app-evm`.

## Dependency Boundaries
- `app`: `Application`, `TxSource`, `EvmBlock`, `ExecutionResult`, `Receipt`.
- `app-evm`: pure EVM execution and receipt capture.
- `tx-dispatch`: neutral mixed-tx classification into EVM or mem lanes.
- `state`: `BlockStorage` for finalized block persistence.

## Responsibilities
- Drain the shared raw-byte `TxSource`.
- Classify each pending transaction once.
- Preserve mem transactions in block order.
- Pass only EVM transactions to `app-evm`.
- Rebuild the final mixed `EvmBlock.transactions` list using EVM inclusion outcomes plus mem pass-through ordering.
- Cache the last proposal per height so duplicate simplex `propose()` calls do not re-drain the mempool.

## Key Type
- `CompositeApplication<DB>`
  - Wraps an internal `EvmApplication<DB>`.
  - Owns the shared `TxSource`.
  - Maintains composite-level pending receipts and duplicate-proposal cache.
  - `store_finalized_block(&self, block, storage)`: persists finalized block receipts captured during propose/verify.

## Verify Path
- Re-classifies `block.transactions`.
- Recomputes the full mixed transaction root from the block payload.
- Extracts only EVM transactions for deterministic re-execution through `app-evm`.
- Relies on classification failure to reject malformed mem transactions deterministically.

## Status
Active. This crate now sits between `whirlpool-node` and the domain apps for mixed EVM/mem transaction flow.

