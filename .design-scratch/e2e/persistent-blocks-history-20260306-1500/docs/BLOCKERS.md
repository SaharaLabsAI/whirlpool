# Blockers

## Active Blockers
No active blockers identified. Implementation can proceed with the MVP defaults documented in `STRATEGY.md`.

## Deferred to Post-MVP

### BLK-1
- **Source**: STRATEGY Q1 (Receipt Storage Timing)
- **Description**: Decide whether to persist receipts only on finalization or also for pending/proposed blocks.
- **Severity**: Low
- **Proposed resolution**: Use finalization-only persistence for MVP (Option A). Revisit pending receipt persistence if pending-block RPC support becomes a requirement.
- **Status**: DEFERRED

### BLK-2
- **Source**: STRATEGY Q2 (Block Reconstruction Strategy)
- **Description**: Decide between reconstructing `EvmBlock` from reth tables on read vs storing raw `EvmBlock` bytes in a custom table.
- **Severity**: Low
- **Proposed resolution**: Keep reconstruction for MVP to avoid schema duplication; profile and reconsider only if RPC latency becomes an issue.
- **Status**: DEFERRED

### BLK-3
- **Source**: STRATEGY Q3 (Historical Receipt Queries)
- **Description**: Decide whether `eth_getTransactionReceipt` should check both in-memory pending receipts and persistent finalized receipts.
- **Severity**: Medium
- **Proposed resolution**: Keep current MVP scope focused on persistent block/history paths; add dual-source receipt lookup as a follow-up API enhancement.
- **Status**: DEFERRED

### BLK-4
- **Source**: STRATEGY Q4 (State Root Verification)
- **Description**: Decide whether to verify state-root consistency during block persistence.
- **Severity**: Low
- **Proposed resolution**: Skip verification in MVP to avoid finalization latency impact; add optional debug verification later.
- **Status**: DEFERRED

### BLK-5
- **Source**: STRATEGY Q5 (Block Pruning / Archival Policy)
- **Description**: Define retention and pruning behavior for persisted historical blocks.
- **Severity**: Low
- **Proposed resolution**: Defer pruning policy to post-MVP; rely on MDBX baseline behavior initially and add configurable pruning if disk growth warrants it.
- **Status**: DEFERRED

### BLK-6
- **Source**: RISK_TRIAGE R2 (Transaction Decoding Performance)
- **Description**: Decoding raw transactions into `TransactionSigned` on each finalized block may add overhead.
- **Severity**: Low
- **Proposed resolution**: Keep batch decoding/inserts as designed; validate with perf tests and optimize only if finalization latency regresses.
- **Status**: DEFERRED

### BLK-7
- **Source**: RISK_TRIAGE R4 (Finalization Performance)
- **Description**: Per-block MDBX writes could impact consensus/finalization latency under load.
- **Severity**: Low
- **Proposed resolution**: Use one MDBX write transaction per block (batched header/body/tx/receipts) and monitor against MVP latency targets.
- **Status**: DEFERRED

### BLK-11
- **Source**: TESTS.md TC-UNK-03 (MDBX Write Failure Handling)
- **Description**: Behavior when MDBX write transaction fails during `store_block()` is not fully specified. Design says "log error, don't crash consensus" but error propagation path and retry policy need definition.
- **Severity**: Medium
- **Proposed resolution**: For MVP, `store_block()` returns `Result<(), Error>`. `PersistingFinalizationSink` logs the error and continues (block is lost from persistence but consensus proceeds). No retry. Add retry/recovery policy post-MVP if needed.
- **Status**: DEFERRED

## Resolved

### BLK-8
- **Source**: RISK_TRIAGE R1 (Type Encoding Mismatch)
- **Description**: `EvmBlock` codec format differs from reth-db Compact-encoded table requirements.
- **Severity**: High (initially)
- **Proposed resolution**: Reuse existing conversion functions: `build_header_from_evm_block()` and `decode_transactions()` before persistence.
- **Status**: RESOLVED

### BLK-9
- **Source**: RISK_TRIAGE R3 + STRATEGY Key Decision #3 (Receipt Flow)
- **Description**: Receipt data was not available in the finalization path for persistence.
- **Severity**: Medium
- **Proposed resolution**: Store receipts in `EvmApp` during propose and persist on finalized block handling.
- **Status**: RESOLVED

### BLK-10
- **Source**: RISK_TRIAGE R5 + STRATEGY Key Decision #2 (Generic Consensus Type)
- **Description**: Generic `B: Block` in consensus-simplex prevents EVM-specific persistence logic at consensus layer.
- **Severity**: Medium
- **Proposed resolution**: Place persistence at application layer via `BlockStorage` trait (`state`) and `RethStateDb` implementation (`state-reth`).
- **Status**: RESOLVED
