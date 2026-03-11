# Task 06: Implement `WhirlpoolTxPool` and blob rejection

## Status
- pending

## Dependencies
- 05

## Wave
- Wave 3

## Complexity
- M

## Target crates
- `rpc-eth` - primary implementation crate

## Pre-Task Gate
- [ ] Task 05 is complete and committed.
- [ ] Provider work is complete enough for later server wiring.
- [ ] Artifact Registry shows TST-2 pending for this task and TST-10 pending for later integration coverage.
- [ ] Scope is limited to the transaction-pool adapter and Type-3 rejection.
- [ ] This task remains commit-ready.

## Context
- Design references:
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/strategy.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/crates.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/flows.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/agent/handoff.md`
  - `.whiteboard/rpc-eth-reth-jsonrpc/e2e/rpc-eth-reth-jsonrpc-20260311-0522/scratch/agent/tests.md`
- Codebase references:
  - `crates/rpc-eth/src/pool.rs`
  - `crates/rpc-eth/tests/pool_contract.rs`
  - `crates/rpc-eth/src/lib.rs`

## Vendor Usage Patterns
- `WhirlpoolTxPool` implements `reth_transaction_pool::TransactionPool` and bridges into `app::TxSource`.
- Blob tx handling is excluded at ingress by rejecting Type-3 transactions before they reach `TxSource`.

## What to do
1. Create TST-2-first contract coverage in `crates/rpc-eth/tests/pool_contract.rs` that checks push/pending bridging and blob rejection seams.
2. Create `crates/rpc-eth/src/pool.rs` with `WhirlpoolTxPool { tx_source: Arc<dyn TxSource> }`.
3. Implement the core `TransactionPool` methods required by reth builder bounds: external add, pending reads, pool sizing, and any no-op removals allowed by the design.
4. Decode raw bytes at the pool boundary into `TransactionSigned`, reject Type-3/blob transactions explicitly, and pass accepted raw bytes into `TxSource`.
5. Export only the minimum module surface needed for later server wiring.

## Mock Boundary
- Use a deterministic test double or in-memory `TxSource` implementation for pool contract tests.
- Do not add full mempool eviction or peer propagation behavior; those are outside the adapter scope.

## AC trace
- REQ-3
- REQ-5
- TST-2
- TST-9
- TST-10

## Must NOT do
- Do not wire the pool into `server.rs` yet.
- Do not implement unsupported `eth_blobBaseFee` handling here; only ingress rejection belongs in this task.
- Do not touch `whirlpool-node`.

## Acceptance Criteria
- [ ] `WhirlpoolTxPool` exists and satisfies the required `TransactionPool` bounds.
- [ ] Blob/Type-3 transactions are rejected at ingress.
- [ ] TST-2 contract coverage passes.
- [ ] `nix develop --command cargo build -p rpc-eth` passes.
- [ ] `.sisyphus/evidence/task-06-txpool-adapter-and-blob-rejection.md` captures commands and outcomes.

## Post-Task Gate
- [ ] `nix develop --command cargo build -p rpc-eth`
- [ ] `nix develop --command cargo test -p rpc-eth pool_contract`
- [ ] Evidence file records a non-blob acceptance case and a blob rejection case.
- [ ] Artifact Registry updates TST-2 with actual test names/locations.
- [ ] Create one dedicated git commit for this task before starting Task 07.

## Post-Task Reconciliation
- Update the TST-2 row with actual pool contract test names and keep TST-9/TST-10 marked integration-pending.

## QA Scenarios
- Happy path: valid raw tx bytes are accepted and appear in pending reads.
- Failure path: malformed bytes fail decode and do not hit `TxSource`.
- Boundary case: blob/Type-3 tx bytes are rejected with the expected error path.

## Evidence
- `.sisyphus/evidence/task-06-txpool-adapter-and-blob-rejection.md`
