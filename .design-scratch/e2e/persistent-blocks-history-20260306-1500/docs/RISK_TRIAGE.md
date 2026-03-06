# Risk Triage

## Risk Classification

### R1: Type Encoding Mismatch — RESOLVED ✅
- **Initial concern**: EvmBlock uses commonware_codec, reth-db tables expect Compact encoding
- **Finding**: `build_header_from_evm_block()` in app-evm/src/executor.rs already converts EvmBlock→reth Header. `decode_transactions()` recovers typed TransactionSigned from raw bytes.
- **Mitigation**: Use existing conversion functions. Store reth Header (has Compact impl) in Headers table. Store TransactionSigned (has Compact impl) in Transactions table.
- **Residual risk**: None

### R2: Transaction Decoding — LOW RISK ⚠️
- **Concern**: EvmBlock stores transactions as Vec<Vec<u8>>, need typed TransactionSigned for storage
- **Finding**: `decode_transactions()` already handles this conversion
- **Mitigation**: Decode raw txs during block persistence, store as TransactionSigned in Transactions table
- **Residual risk**: Performance of decoding on every finalization (mitigated by batch insert)

### R3: Receipt Storage Reconciliation — MEDIUM RISK ⚠️
- **Concern**: Receipts currently in in-memory ReceiptStore (rpc-eth). Need persistent receipt storage.
- **Finding**: Receipts table exists in reth-db. However, receipts are computed during execution but NOT currently passed to the finalization path — only `ExecutionResult { state_root, receipts_root, gas_used, receipt_count }` flows out.
- **Mitigation options**:
  A. Store receipts in MDBX Receipts table during block persistence (requires receipt data to flow to persistence layer)
  B. Keep receipts separate, reconstruct by re-executing block when queried (expensive but simple)
  C. Extend finalization event to carry receipt data alongside the block
- **Recommendation**: Option A — extend the finalization/persistence path to include receipts

### R4: Finalization Performance — LOW RISK ⚠️
- **Concern**: MDBX write on every finalization may impact consensus latency
- **Finding**: state-reth already does MDBX writes (commit()) in the propose path without issues. Single-block writes are fast in MDBX.
- **Mitigation**: Batch header + body indices + transactions + receipts in single MDBX write transaction
- **Residual risk**: Negligible for single-block writes

### R5: Generic Block Type in Consensus-Simplex — MEDIUM RISK ⚠️
- **Concern**: consensus-simplex uses generic `B: Block`, but persistent storage needs concrete types (Header, TransactionSigned)
- **Finding**: The persistence layer needs EVM-specific knowledge (EvmBlock→Header conversion). This means persistence cannot be fully generic at the consensus-simplex layer.
- **Mitigation**: Persistence hook at the application layer (app/app-evm), not at consensus-simplex. The EventSink or a new BlockPersistence trait handles storage with concrete types.
- **Recommendation**: New trait `BlockStorage` in `state` crate with EVM-specific impl in `state-reth`

## Summary
- 0 BLOCKERS
- 2 MEDIUM risks (receipt flow, generic types) — both have clear mitigation paths
- 2 LOW risks (tx decoding perf, finalization perf)
- 1 RESOLVED (type encoding mismatch)
