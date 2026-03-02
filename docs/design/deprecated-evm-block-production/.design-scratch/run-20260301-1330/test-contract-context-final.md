# Final test-contract context - docs/design/evm-block-production

## Intent criteria mapping (INTENT.md -> test obligations)

| Intent success criterion | Flows | Primary invariants | Grounded status |
|---|---|---|---|
| #1 Transaction execution in `EvmApplication::propose()` | block-proposal, state-commitment | INV-01, INV-06, INV-07 | BLOCKER: `propose()` currently emits empty blocks only (`transactions=[]`, `gas_used=0`, empty tx/receipt roots). |
| #2 Transaction verification in `EvmApplication::verify()` | block-verification | INV-02, INV-03 | BLOCKER/PARTIAL: `verify()` checks only `state_root`; no tx/receipt/gas replay. |
| #3 Non-`NoopTxSource` implementation | node-startup, block-proposal | INV-01, INV-07 | BLOCKER: runtime wiring uses `Arc::new(NoopTxSource)`. |
| #4 Snapshot/commit/rollback lifecycle | block-proposal, block-verification, state-commitment | INV-03, INV-04, INV-05 | UNKNOWN/BLOCKER: `InMemoryStateDb: Clone` and `commit()` exist, but cross-crate orchestration is not grounded. |
| #5 Block assembly correctness | block-proposal, block-verification | INV-02, INV-06 | BLOCKER for non-empty blocks; grounded for MVP-empty behavior. |
| #6 Node wiring connects tx source + EVM + state | node-startup | INV-01, INV-05 | BLOCKER: node wires `NoopTxSource`; no finalize->commit integration seam. |
| #7 End-to-end propose -> finalize -> commit | all flows | INV-01..INV-06 | BLOCKER: consensus callback surface is only `genesis/propose/verify`; finalization sink updates height only. |

## CONFIRMED invariants (INV-01..INV-07)

These statements are CONFIRMED as required intent-level invariants. "CONFIRMED" does not imply current implementation satisfies them.

| Invariant | Statement | Current status |
|---|---|---|
| INV-01 | Execution Visibility - if `TxSource` provides >=1 valid tx, `propose()` output must show execution effects. | BLOCKER |
| INV-02 | Verification Integrity - `verify()` must recompute execution artifacts and reject mismatch. | BLOCKER/PARTIAL |
| INV-03 | Verification Read-Only - `verify()` must not mutate canonical state. | PARTIAL/GROUNDED for current root-read path |
| INV-04 | Snapshot Safety - failed `propose()`/`verify()` leaves state byte-identical to pre-call state. | UNKNOWN/BLOCKER |
| INV-05 | Commit Atomicity - finalization applies all block effects exactly once. | BLOCKER |
| INV-06 | Root Consistency - roots derive from actual execution of included txs. | MIXED: grounded for empty state-root derivation, blocked for non-empty tx execution |
| INV-07 | Proposal Determinism - identical state + identical `TxSource` response => identical proposal. | GROUNDED only for current MVP-empty path; UNKNOWN for non-empty ordering policy |

## Grounded interfaces and seams by crate

### consensus
- `ConsensusApp::{genesis, propose, verify}` is the consensus callback boundary.
- No finalize callback exists on `ConsensusApp`.

### consensus-simplex
- `FinalizationSink::handle(ConsensusEvent::Finalized { .. })` updates finalized height (`AtomicU64`).
- No canonical state commit is triggered here.

### app
- `Application` trait returns `(Block, Result)` from `propose` and `Result` from `verify`.
- `TxSource::pending() -> Vec<Vec<u8>>` and `NoopTxSource` (returns empty vec).
- `ApplicationAdapter` maps:
  - `propose`: `Ok((block, _)) -> Some(block)`, `Err(_) -> None`
  - `verify`: `Err(err) -> ConsensusError::InvalidBlock(err.to_string())`
- `EvmBlock` and `ExecutionResult` are the app-level contracts.

### app-evm
- `EvmApplication<DB>` with `state_db: Arc<RwLock<DB>>` and `tx_source: Arc<dyn TxSource + Send + Sync>`.
- `genesis`: reads `state_root`, empty tx/receipt roots.
- `propose`: MVP empty-block stub.
- `verify`: root equality check only; emits `StateRootMismatch` on mismatch.

### state
- `InMemoryStateDb::commit(&BundleState)` applies state updates.
- `InMemoryStateDb::state_root()` is deterministic (sorted account/storage keccak, empty -> `KECCAK_EMPTY`).
- Implements `DatabaseRef` (`&self`) and `Database`.

### whirlpool-node
- Runtime wiring in `main.rs` injects:
  - `TestStateDb(InMemoryStateDb)`
  - `Arc::new(NoopTxSource)`
  - `EvmApplication` -> `ApplicationAdapter` -> `CommonwareEngine`

## Flow-to-test obligations (architecture)

### block-proposal
Grounded obligations:
- Adapter returns `Some(block)` on app success and `None` on app error.
- MVP proposal fields are empty tx list, empty roots, zero gas, `timestamp = parent.timestamp + 12`.

Blocker obligations:
- Non-empty tx execution visibility and non-empty artifact derivation.
- Preserve commit-ready artifact across propose -> finalize seam.

### block-verification
Grounded obligations:
- Reject state-root mismatch.
- Adapter maps app verify errors to `ConsensusError::InvalidBlock`.
- Current verify path is read-only for rooted implementation.

Blocker obligations:
- Replay and compare `transactions_root`, `receipts_root`, and `gas_used`.

### state-commitment
Grounded obligations:
- `commit()` mutates state from `BundleState`.
- `state_root()` reflects deterministic commitment.

Blocker obligations:
- No grounded production/ownership seam for commit-ready artifacts.
- No grounded finalize->commit callback path.

### node-startup
Grounded obligations:
- Node composes network/provider/config/state/tx source/app/adapter/engine.
- Startup failure is treated as fatal (`expect`).

Revise obligations:
- Keep assertions aligned with current simplex runtime behavior; avoid asserting ungrounded production guarantees.

### block-finalization
Grounded obligations:
- Finalized events update shared finalized height.

Blocker obligations:
- Finalization-to-canonical-commit seam is not grounded.

## Required test files and ownership

| File | Owner | Primary scope |
|---|---|---|
| `tests/overview.md` | test contracts | strategy, invariant and intent mapping |
| `tests/cross-crate-flows.md` | test contracts | architecture flow tests across crates |
| `tests/app-evm-unit.md` | app-evm | EVM app unit contracts |
| `tests/app-unit.md` | app | adapter and tx source unit contracts |
| `tests/state-unit.md` | state | DB commit/root/read contracts |
| `tests/whirlpool-node-unit.md` | whirlpool-node | startup and wiring contracts |
| `tests/block-production-integration.md` | block-production domain | node/consensus runtime integration |
| `tests/evm-execution-integration.md` | evm-execution domain | propose/verify integration |

## Blockers and unknowns to encode in tests

### BLOCKER
- Non-empty tx execution in `EvmApplication::propose`.
- Replay verification for tx/receipt/gas artifacts.
- Finalize->commit callback seam and exactly-once commit path.
- Commit-ready artifact storage/ownership across propose->finalize.

### REVISE/UNKNOWN
- `TxSource::pending()` byte semantics and deterministic ordering policy.
- Exact byte-identical snapshot/rollback boundary for INV-04.
- Structured mismatch taxonomy beyond stringified `InvalidBlock` mapping.
