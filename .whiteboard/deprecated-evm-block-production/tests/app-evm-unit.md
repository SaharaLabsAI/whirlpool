# app-evm Unit Test Contracts

| Interface | Test case ID | Label | Invariant ref | Path | Preconditions | Stimulus | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| `EvmApplication::genesis` | `AEVM-U-001` | `[GROUNDED]` | INV-06 | happy | State DB available via `StateProvider`. | Call `genesis()`. | `height==0`; `state_root==db.state_root()`; `transactions_root==EMPTY_ROOT_HASH`; `receipts_root==EMPTY_ROOT_HASH`; `gas_used==0`. | In-memory state DB fixture. | High | Active |
| `EvmApplication::propose` | `AEVM-U-002` | `[GROUNDED]` | INV-06, INV-07 | happy | Parent block present. | Call `propose(parent, height)`. | Returns block with `transactions=[]`, empty roots, zero gas, and `timestamp=parent.timestamp+12`; returns `ExecutionResult` aligned to block roots/gas. | `NoopTxSource`; in-memory state DB fixture. | High | Active |
| `EvmApplication::verify` | `AEVM-U-003` | `[GROUNDED]` | INV-02 | failure | Valid parent; block whose `state_root` is tampered. | Call `verify(parent, tampered_block)`. | Returns `Err(EvmAppError::StateRootMismatch { expected, computed })`. | In-memory state DB fixture. | High | Active |
| `EvmApplication::verify` | `AEVM-U-004` | `[GROUNDED]` | INV-03 | happy | Block from current propose flow. | Call `verify(parent, block)`. | Returns `Ok(ExecutionResult)`; `state_root` equals block state root; no canonical DB mutation in current path. | In-memory state DB fixture. | High | Active |
| `EvmApplication::verify` | `AEVM-U-005` | `[GROUNDED]` | INV-03 | failure | Failed verify has occurred on tampered state root block. | Attempt new propose/verify cycle after failure. | Subsequent propose/verify remains healthy; failed verify does not corrupt canonical state root for current implementation. | In-memory state DB fixture. | Medium | Active |
| `EvmApplication::propose` (non-empty execution) | `AEVM-U-006` | `[PROPOSED]` | INV-01, INV-06 | happy | Concrete tx source returns >=1 valid tx; execution path exists. | Call `propose` with non-empty pending tx set. | Block includes tx bytes and non-empty execution-derived artifacts; `ExecutionResult` reflects executed tx effects. | Needs non-noop tx source + execution seam. | High | Blocked |
| `EvmApplication::verify` (artifact replay) | `AEVM-U-007` | `[PROPOSED]` | INV-02, INV-06 | failure | Replay verification implemented for tx/receipt/gas artifacts. | Mutate one of `transactions_root`, `receipts_root`, `gas_used`, then call `verify`. | Verification rejects each tampered artifact mismatch with explicit invalidity signal. | Needs replay verification seam. | High | Blocked |
| `EvmApplication` snapshot safety | `AEVM-U-008` | `[PROPOSED]` | INV-04 | failure | Snapshot/rollback orchestration contract exists. | Inject mid-execution failure in propose/verify path. | Post-failure canonical state is byte-identical to pre-call snapshot. | Needs explicit snapshot orchestration seam. | High | Revise |

## Pseudo-code outlines

```rust
// AEVM-U-002 [GROUNDED]
let parent = app.genesis().await;
let (block, exec) = app.propose(&parent, 1).await?;
assert!(block.transactions.is_empty());
assert_eq!(block.transactions_root, EMPTY_ROOT_HASH.0);
assert_eq!(block.receipts_root, EMPTY_ROOT_HASH.0);
assert_eq!(block.gas_used, 0);
assert_eq!(block.timestamp, parent.timestamp + 12);
assert_eq!(exec.state_root, block.state_root);
```

```rust
// AEVM-U-003 [GROUNDED]
let (block, _) = app.propose(&parent, 1).await?;
let mut tampered = block.clone();
tampered.state_root[0] ^= 0x01;
assert!(matches!(app.verify(&parent, &tampered).await, Err(EvmAppError::StateRootMismatch { .. })));
```

```rust
// AEVM-U-007 [PROPOSED] BLOCKER
let (block, _) = app.propose(&parent, 1).await?;
let mut tampered = block.clone();
tampered.gas_used += 1;
let res = app.verify(&parent, &tampered).await;
assert!(res.is_err());
```
