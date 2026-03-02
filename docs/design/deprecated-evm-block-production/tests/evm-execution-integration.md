# EVM Execution Integration Test Contracts

| Test name | Test case ID | Label | Invariant ref | Preconditions | Actions | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|
| `genesis_then_propose_empty_execution_contract` | `EVM-INT-001` | `[GROUNDED]` | INV-06, INV-07 | `EvmApplication` built with in-memory DB + `NoopTxSource`. | Run `genesis()` then `propose(parent, height)`. | Proposed block has empty tx list, empty tx/receipt roots, `gas_used=0`, and timestamp offset `+12`. | In-memory DB fixture and noop tx source. | High | Active |
| `verify_accepts_matching_state_root_block` | `EVM-INT-002` | `[GROUNDED]` | INV-02, INV-03 | Valid block from current proposal path. | Call `verify(parent, block)`. | Returns `Ok(ExecutionResult)` with state root aligned to block and no canonical mutation signal. | Same fixtures as EVM-INT-001. | High | Active |
| `verify_rejects_state_root_mismatch` | `EVM-INT-003` | `[GROUNDED]` | INV-02 | Block clone with mutated `state_root`. | Call `verify(parent, tampered)`. | Returns `Err(EvmAppError::StateRootMismatch { .. })`. | Same fixtures as EVM-INT-001. | High | Active |
| `non_empty_pending_transactions_are_executed_in_propose` | `EVM-INT-004` | `[PROPOSED]` | INV-01, INV-06 | Concrete tx source supplies >=1 valid tx; execution pipeline enabled. | Call `propose` with non-empty pending tx source. | Block includes txs; execution artifacts/roots are non-empty and derived from executed txs. | Requires non-noop tx source and execution integration path. | High | Blocked |
| `verify_replays_and_rejects_transactions_root_mismatch` | `EVM-INT-005` | `[PROPOSED]` | INV-02, INV-06 | Replay verification compares recomputed tx root. | Mutate `transactions_root` on valid block and call `verify`. | Verification rejects mismatch with explicit invalidity signal. | Requires replay verification seam. | High | Blocked |
| `verify_replays_and_rejects_receipts_root_mismatch` | `EVM-INT-006` | `[PROPOSED]` | INV-02, INV-06 | Replay verification compares recomputed receipts root. | Mutate `receipts_root` on valid block and call `verify`. | Verification rejects mismatch with explicit invalidity signal. | Requires replay verification seam. | High | Blocked |
| `verify_replays_and_rejects_gas_used_mismatch` | `EVM-INT-007` | `[PROPOSED]` | INV-02 | Replay verification compares recomputed gas usage. | Mutate `gas_used` on valid block and call `verify`. | Verification rejects mismatch with explicit invalidity signal. | Requires replay verification seam. | High | Blocked |
| `snapshot_safety_across_failed_propose_or_verify` | `EVM-INT-008` | `[PROPOSED]` | INV-04 | Snapshot/rollback orchestration contract defined across execution boundaries. | Inject failure in propose/verify and compare pre/post canonical snapshot. | Canonical state is byte-identical to pre-call snapshot after failure. | Requires snapshot boundary definition and orchestration seam. | High | Revise |

## Pseudo-code outlines

```rust
// EVM-INT-001 [GROUNDED]
let genesis = app.genesis().await;
let (block, exec) = app.propose(&genesis, 1).await?;
assert!(block.transactions.is_empty());
assert_eq!(block.gas_used, 0);
assert_eq!(exec.state_root, block.state_root);
```

```rust
// EVM-INT-004 [PROPOSED] BLOCKER
let app = build_app_with_tx_source(FixtureTxSource::with_pending(vec![valid_tx_bytes()]));
let (block, exec) = app.propose(&parent, 1).await?;
assert!(!block.transactions.is_empty());
assert_eq!(block.state_root, exec.state_root);
```

```rust
// EVM-INT-007 [PROPOSED] BLOCKER
let (block, _) = app.propose(&parent, 1).await?;
let mut tampered = block.clone();
tampered.gas_used += 1;
assert!(app.verify(&parent, &tampered).await.is_err());
```
