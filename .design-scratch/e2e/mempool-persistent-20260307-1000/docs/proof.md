# Design Proof: Persistent Mempool

## S0: Pre-conditions

- **0 Active Blockers**: Confirmed. `BLOCKERS.md` (line 10) explicitly states "No blockers identified."
- **Design Completeness**: Confirmed. All required architectural documents (INTENT, STRATEGY, SUMMARY, FLOWS, TESTS, BLOCKERS, CRATES, DOMAINS) and crate-specific specifications (crates/mempool.md, crates/app.md) are present and reviewed.
- **Evidence Traceability**: Confirmed. All design claims, architectural decisions, and implementation strategies in the design documents are supported by existing code citations (e.g., `app/src/traits.rs:23`) or explicitly marked as `[PROPOSED]` refinements.

## S1: Split Justification

This project focuses on a single core intent: "Add persistent storage to the mempool so transactions survive node restarts" (`INTENT.md`, line 4). The design follows a cohesive architectural path where every change—from trait extension to the new storage crate—directly serves this one goal. No split is required as the implementation is a single functional increment with no independent sub-features.

## S2: Invariants

| ID | Invariant | Design Citation | Test Validation |
|----|-----------|-----------------|-----------------|
| **INV-1** | FIFO ordering preserved | `STRATEGY.md` (Section 3), `DOMAINS.md` (Boundary 1) | `UT-MEMPOOL-06`, `PROP-01` |
| **INV-2** | Drain semantics | `STRATEGY.md` (Section 4), `SUMMARY.md` (Decision 4) | `UT-MEMPOOL-03`, `PROP-04` |
| **INV-3** | Crash durability | `FLOWS.md` (Crash Recovery Flow), `SUMMARY.md` (Decision 1) | `UT-MEMPOOL-04`, `INT-CR-01`, `INT-E2E-01` |
| **INV-4** | TxSource backward compatibility | `STRATEGY.md` (Migration Strategy), `DOMAINS.md` (Domain 1) | `UT-APP-05`, `UT-RPC-03`, `INT-FLOW-02` |
| **INV-5** | Thread safety | `INTENT.md` (Requirement 5), `DOMAINS.md` (Boundary 3) | `UT-MEMPOOL-07`, `UT-MEMPOOL-08`, `PROP-02` |
| **XINV-1** | Cross-crate trait compatibility | `STRATEGY.md` (Section 5), `CRATES.md` (rpc-eth) | `UT-RPC-01`, `UT-RPC-02`, `UT-NODE-01` |
| **XINV-2** | Build system integrity | `STRATEGY.md` (Implementation Ordering), `SUMMARY.md` (Success Criteria) | `UT-NODE-01`, `UT-APP-01` |

## S3: Acceptance Criteria

### Criteria Definitions

- **AC-1**: Transactions persist across node restart. Verified by crash recovery flow (`FLOWS.md`, Scenario 1) and recovery tests.
- **AC-2**: Existing tests continue to pass (regression). Verified by running existing suites in `app` and `rpc-eth`.
- **AC-3**: New mempool crate has unit tests covering public API. Verified by test suite in `crates/mempool/tests/`.
- **AC-4**: EthRpcContext works with trait object. Verified by constructor signature change in `crates/rpc-eth.md`.
- **AC-5**: FIFO ordering verified by tests. Verified by auto-increment key strategy and property tests.
- **QA-1**: No clippy warnings on new/modified code. Enforced by workspace-wide linting standards.
- **QA-2**: No unsafe code in mempool crate. Confirmed by use of `libmdbx-rs` safe wrappers and standard Rust concurrency primitives.
- **QA-3**: Error types properly propagated. Confirmed by `MempoolError` definition in `crates/mempool.md`.

### Coverage Matrix

| AC/QA ID | Test Case IDs (from TESTS.md) |
|----------|-------------------------------|
| **AC-1** | `UT-MEMPOOL-04`, `INT-CR-01`, `INT-WIRE-02`, `INT-E2E-01` |
| **AC-2** | `UT-APP-05`, `UT-RPC-03`, `INT-FLOW-02` |
| **AC-3** | `UT-MEMPOOL-01` through `UT-MEMPOOL-13` |
| **AC-4** | `UT-RPC-01`, `UT-RPC-02`, `UT-NODE-01` |
| **AC-5** | `UT-MEMPOOL-06`, `INT-FLOW-04`, `PROP-01` |
| **QA-1** | [Inferred from workspace standards] |
| **QA-2** | [Inferred from dependency policy] |
| **QA-3** | `UT-MEMPOOL-09`, `UT-MEMPOOL-10`, `UT-NODE-03` |

## S4: Dependency Contract

- **Inter-crate Dependencies**:
    - `mempool` depends on `app` for the `TxSource` trait definition (`CRATES.md`, line 9).
    - `rpc-eth` depends on `app` for the `TxSource` trait object interface (`CRATES.md`, line 11).
    - `whirlpool-node` depends on `mempool`, `app`, `rpc-eth`, and `app-evm` for assembly (`CRATES.md`, line 12).
- **External Dependencies**:
    - `libmdbx-rs`: New dependency for the `mempool` crate to provide MDBX storage (`STRATEGY.md`, line 15).
    - `parking_lot`: Dependency for `mempool` to provide lightweight synchronization (`STRATEGY.md`, line 159).
- **Breaking Changes**:
    - `TxSource` trait: Added `push(tx: Vec<u8>)` method. Requires updates to all implementors: `InMemoryTxPool`, `NoopTxSource`, and all test mocks (`STRATEGY.md`, line 40).
- **Vendor Policy**:
    - No changes to `vendor/**` crates.
    - No modification to `reth-db` tables enum, avoided by using raw `libmdbx-rs` (`STRATEGY.md`, line 17).

## S5: Risk Assessment

- **Stability Assumption**: The `libmdbx-rs` crate is assumed to be stable and compatible with the current Nix-managed toolchain. This is high confidence as MDBX is already used via `reth-db` in `state-reth` (`STRATEGY.md`, line 23).
- **Performance Uncertainty**: The exact latency overhead of MDBX writes on the RPC submission path is unmeasured. This is mitigated by the fact that mempool operations are not on the consensus hot path and MDBX is highly optimized for such workloads (`STRATEGY.md`, Risk 4).
- **Breaking Trait Change**: Extending `TxSource` is a breaking change. This is mitigated by the fact that all implementors are internal to the workspace and can be updated atomically (`BLOCKERS.md`, INFO 2).
- **Crash Window**: Transactions are still lost if a crash occurs between the `pending()` drain and consensus finalization. This risk is explicitly accepted as it matches current system behavior and is documented as a post-MVP lifecycle tracking candidate (`BLOCKERS.md`, WARNING 1).
- **Path Overlap**: Misconfiguration could lead to mempool data overlapping other stores. Mitigated by enforcing a dedicated `{persistent_storage_dir}/mempool` subdirectory (`BLOCKERS.md`, WARNING 2).
