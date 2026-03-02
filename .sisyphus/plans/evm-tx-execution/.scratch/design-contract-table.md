# Design Contract Table — EVM Transaction Execution

> Extracted from `docs/design/evm-tx-execution/` design doc set.
> Source: INTENT.md, STRATEGY.md, FLOWS.md, TESTS.md, DOMAINS.md, BLOCKERS.md, app-evm/README.md, state/README.md

---

## 1. Success Criteria

| ID | Criterion | Verification |
|----|-----------|-------------|
| SC-1 | propose() executes EVM transactions and returns a correct EvmBlock (gas > 0, valid roots) | T-1, T-2 |
| SC-2 | verify() re-executes and validates all 4 fields (state_root, tx_root, receipts_root, gas_used) | T-4, T-5, T-8 |
| SC-3 | commit() handles BundleState correctly (accounts, storage, bytecodes) | T-9, T-10, T-11 |
| SC-4 | Invalid transactions are skipped in propose, rejected in verify | T-3, T-6 |
| SC-5 | State is not corrupted when verify rejects a block (clone isolation) | T-12 |
| SC-6 | All existing tests pass + new test coverage ≥ 13 tests | All T-1..T-13 |

## 2. Design Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D-1 | BlockBuilder for propose, BasicBlockExecutor for verify | Propose needs tx-by-tx (skip failures); verify needs batch (all-or-nothing) |
| D-2 | Clone-based state snapshots for rollback | Simple, correct; O(n) acceptable for MVP |
| D-3 | Verify does NOT commit to canonical state | Isolation via clone; only propose commits |
| D-4 | Skip invalid transactions during propose | Matches Ethereum behavior |
| D-5 | Default NextBlockEnvAttributes (zero fee_recipient, zero randao) | Sufficient for MVP; no MEV/randomness needed |
| D-6 | alloy-trie for tx_root / receipts_root computation | Standard Ethereum trie root computation |

## 3. Blockers & Workarounds

| ID | Blocker | Workaround |
|----|---------|-----------|
| B-1 | builder.finish() requires StateRootProvider | Bypass: use state.take_bundle() + InMemoryStateDb::state_root() directly |
| B-2 | No finalization callback in ConsensusApp trait | Clone snapshots; commit during propose (MVP limitation) |
| B-3 | Skip vs fail on invalid txs | D-4: Skip in propose, fail in verify |
| B-4 | Clone vs COW snapshots | D-2: Clone for MVP |

## 4. Implementation Slices (Dependency Order)

| Slice | Name | Crate | Depends On | Description |
|-------|------|-------|-----------|-------------|
| S-1 | Transaction decode/recover | app-evm | — | `decode_transactions(&[Vec<u8>]) -> Result<Vec<RecoveredTx>>` using decode_2718 + try_recover |
| S-2 | Propose execution flow | app-evm | S-1 | Replace propose() stub with 11-step EVM execution flow |
| S-3 | State snapshot (Clone) | state | — | Add `#[derive(Clone)]` to InMemoryStateDb + DbAccount |
| S-4 | Verify re-execution flow | app-evm | S-1, S-3 | Replace verify() stub with batch re-execution + 4-field comparison |
| S-5 | Block field computation | app-evm | S-2 | tx_root + receipts_root via alloy-trie ordered trie root |
| S-6 | Integration tests | app-evm, state | S-1..S-5 | T-1 through T-13 |

## 5. Crate Contracts

### 5a. app-evm (Primary Target)

**Existing API (to be modified):**
- `EvmApplication<DB>` struct: `{ evm_config, state_db: Arc<RwLock<DB>>, tx_source: Arc<dyn TxSource> }`
- `propose(&self, parent: &EvmBlock, timestamp: u64) -> Result<EvmBlock>` — STUB, returns empty block
- `verify(&self, parent: &EvmBlock, proposed: &EvmBlock) -> Result<()>` — STUB, checks only state_root

**Proposed new function:**
- `decode_transactions(raw_txs: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError>` — decode_2718 + try_recover

**Error variants (EvmAppError):**
- `Execution(String)` — EVM execution failure
- `StateRootMismatch { expected: B256, got: B256 }` — verify root mismatch
- `State(StateError)` — state operation failure
- `InvalidBlock(String)` — decode failure, field mismatch

**New dependencies needed:**
- reth-evm (BlockBuilder, BasicBlockExecutor, ConfigureEvm)
- reth-revm (State wrapper)
- reth-execution-types (BlockExecutionResult)
- alloy-trie (ordered_trie_root_with_encoder)
- alloy-consensus (TxReceipt encoding)

### 5b. state (Secondary Target)

**Existing API (to be modified):**
- `InMemoryStateDb` struct — needs `#[derive(Clone)]`
- `DbAccount` struct — needs `#[derive(Clone)]`
- `commit(&mut self, bundle: &BundleState)` — verify correctness (may already be correct)
- `state_root(&self) -> B256` — flat keccak256 (NOT MPT, accepted for MVP)

**No new public API** — only derive additions.

## 6. Flow Contracts

### 6a. F1: Block Proposal (propose)

```
1. tx_source.pending() → Vec<Vec<u8>>
2. decode_transactions(raw_txs) → Vec<RecoveredTx> (skip decode failures)
3. Clone state_db → snapshot
4. reth_revm::State::builder().with_database(snapshot).with_bundle_update().build()
5. build_sealed_header(parent) → SealedHeader
6. NextBlockEnvAttributes { timestamp: parent+12, fee_recipient: ZERO, randao: ZERO, gas_limit: 30M }
7. evm_config.builder_for_next_block(state, header, attrs) → builder
8. builder.apply_pre_execution_changes()
9. FOR EACH tx: builder.execute_transaction(tx) — skip failures
10. state.take_bundle() → BundleState
11. canonical_db.commit(&bundle) → mutates canonical state
12. canonical_db.state_root() → state_root
13. Compute tx_root, receipts_root via alloy-trie
14. Assemble EvmBlock { header fields, transactions, state_root, tx_root, receipts_root, gas_used }
```

### 6b. F2: Block Verification (verify)

```
1. decode_transactions(proposed.transactions) → ALL must decode (fail = InvalidBlock)
2. Clone state_db → snapshot (isolation, never committed to canonical)
3. reth_revm::State::builder().with_database(snapshot).with_bundle_update().build()
4. Reconstruct RecoveredBlock from proposed block
5. BasicBlockExecutor::execute_one(&recovered_block) → BlockExecutionResult
6. Apply bundle to snapshot clone → compute state_root, tx_root, receipts_root, gas_used
7. Compare all 4 fields: state_root, tx_root, receipts_root, gas_used
8. All match → Ok(()); any mismatch → Err(InvalidBlock or StateRootMismatch)
```

### 6c. F3: State Commitment (commit)

```
1. FOR EACH (addr, account) IN bundle.state:
   - If destroyed → remove from accounts map
   - If created/changed → upsert AccountInfo + merge storage changes
2. FOR EACH (hash, bytecode) IN bundle.contracts:
   - Insert into bytecodes map
3. insert_block_hash(block_number, block_hash)
4. state_root() recomputed on demand (flat keccak256)
```

## 7. Test Contracts

| ID | Test Name | Crate | Slice | Validates |
|----|-----------|-------|-------|-----------|
| T-1 | propose_executes_transfer_transaction | app-evm | S-2, S-5 | SC-1 |
| T-2 | propose_executes_contract_deployment | app-evm | S-2, S-5 | SC-1 |
| T-3 | propose_skips_invalid_transactions | app-evm | S-2 | SC-4 |
| T-4 | verify_accepts_valid_block | app-evm | S-4 | SC-2 |
| T-5 | verify_rejects_wrong_state_root | app-evm | S-4 | SC-2 |
| T-6 | verify_rejects_undecodable_transactions | app-evm | S-4 | SC-4 |
| T-7 | propose_empty_txsource_produces_empty_block | app-evm | S-2 | SC-1 |
| T-8 | verify_rejects_wrong_gas_used | app-evm | S-4 | SC-2 |
| T-9 | commit_applies_account_changes | state | S-3 | SC-3 |
| T-10 | commit_applies_storage_changes | state | S-3 | SC-3 |
| T-11 | commit_handles_account_destruction | state | S-3 | SC-3 |
| T-12 | clone_provides_independent_snapshot | state | S-3 | SC-5 |
| T-13 | propose_verify_round_trip | app-evm | S-6 | SC-1, SC-2 |

## 8. Cross-Crate Boundary Contracts

| Boundary | Interface | Direction |
|----------|-----------|-----------|
| app-evm ↔ state | `Database` / `DatabaseRef` traits | app-evm reads state |
| app-evm ↔ state | `commit(&mut self, &BundleState)` | app-evm writes state |
| app-evm ↔ state | `state_root() -> B256` | app-evm reads root |
| app-evm ↔ state | `Clone` (snapshot) | app-evm clones for isolation |
| app-evm ↔ app | `Application` trait (propose/verify) | consensus calls app-evm |
| app-evm ↔ app | `TxSource::pending()` | app-evm reads pending txs |
| app-evm ↔ reth | `ConfigureEvm` (builder_for_next_block) | app-evm uses reth |
| app-evm ↔ reth | `BlockBuilder` (execute_transaction) | app-evm uses reth |
| app-evm ↔ reth | `BasicBlockExecutor` (execute_one) | app-evm uses reth |
| app-evm ↔ alloy | `ordered_trie_root_with_encoder` | app-evm computes roots |

## 9. Invariants

| ID | Invariant | Enforcement |
|----|-----------|-------------|
| INV-1 | State consistency: state_root after commit matches re-derivation | T-9, T-10, T-11 |
| INV-2 | Deterministic execution: same txs + state → same result | T-4, T-13 |
| INV-3 | Verify isolation: canonical state unchanged after verify | T-12 |
| INV-4 | Block field correctness: tx_root/receipts_root match Ethereum standard | T-1, T-5, T-8 |
