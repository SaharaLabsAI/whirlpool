# Architecture

## Subsystem map

| Subsystem | Primary crate(s) | Cross-crate handoff | Current status |
|---|---|---|---|
| Node bootstrap and runtime wiring | `whirlpool-node` | Wires `EvmApplication` into `ApplicationAdapter`, then into `CommonwareEngine::start()` | Grounded |
| Consensus contract layer | `consensus` | Exposes `ConsensusApp::{genesis,propose,verify}` and `ConsensusEvent::Finalized` | Grounded |
| Consensus runtime adapter/sink | `consensus-simplex` | Implements `ConsensusEngine`; `FinalizationSink` updates finalized height | Grounded (engine path includes STUB simulation) |
| Application boundary | `app` | `ApplicationAdapter` maps app callbacks to consensus callbacks | Grounded |
| EVM execution domain | `app-evm` | Implements `Application` for propose/verify using state + tx source | Grounded (proposal/verify are MVP-limited) |
| State substrate | `state` | Exposes `InMemoryStateDb::{commit,state_root}` for execution and commit paths | Grounded |

## Flow index

| Flow | Summary | File |
|---|---|---|
| Block Proposal | Consensus asks app for a block; `propose()` is speculative (no canonical commit), and app-evm currently returns an empty EVM block while adapter drops `ExecutionResult`. | `docs/design/evm-block-production/architecture/block-proposal.md` |
| Block Verification | Consensus verifies candidate block; app-evm currently validates only state root equality. | `docs/design/evm-block-production/architecture/block-verification.md` |
| State Commitment | Canonical commit is [PROPOSED] to happen only after finalization via a finalize->commit seam; this seam is currently `UNKNOWN`/`BLOCKER`. | `docs/design/evm-block-production/architecture/state-commitment.md` |
| Node Startup | Node composes network, consensus config, state DB, tx source, app, adapter, and engine startup. | `docs/design/evm-block-production/architecture/node-startup.md` |
| Block Finalization | Finalization event reaches sink and finalized-height atomics; canonical commit is [PROPOSED] to occur only from this phase via finalize->commit seam (`BLOCKER`). | `docs/design/evm-block-production/architecture/block-finalization.md` |

## Key invariants

- `INV-01` Execution Visibility: [PROPOSED] If `TxSource` provides >=1 valid tx, `propose()` output must show execution effects; currently `BLOCKER` because runtime uses `NoopTxSource` and MVP-empty proposal.
- `INV-02` Verification Integrity: [PROPOSED] `verify()` recomputes execution artifacts and rejects mismatch; currently `BLOCKER` because only state-root compare is grounded.
- `INV-03` Verification Read-Only: [PROPOSED] `verify()` must not mutate canonical state; wording grounded from test context, full replay-path guarantee is `UNKNOWN`.
- `INV-04` Snapshot Safety: [PROPOSED] failed `propose()`/`verify()` leaves state byte-identical to pre-call state; orchestration seam is `UNKNOWN` and currently a `BLOCKER` for non-empty execution.
- `INV-05` Commit Atomicity: [PROPOSED] finalization applies all block effects exactly once with no partial commit; finalize-to-commit callback is `BLOCKER`.
- `INV-06` Root Consistency: [PROPOSED] `state_root`/`transactions_root`/`receipts_root` derive from actual executed txs; currently `BLOCKER` because proposal hardcodes empty roots for empty tx list.
- `INV-07` Proposal Determinism: [PROPOSED] same state + same tx source response => identical proposal output; real policy is `UNKNOWN` because tx ordering/selection contract is not defined.

## Glossary

- `ConsensusApp`: Consensus-facing trait (`genesis`, `propose`, `verify`) used as the app handoff seam.
- `Application`: App-facing trait implemented by `EvmApplication`; returns `(Block, ExecutionResult)` on propose and `ExecutionResult` on verify.
- `ApplicationAdapter`: Bridge from `Application` to `ConsensusApp`; proposal errors become `None` (abstain), verify errors become `ConsensusError::InvalidBlock`.
- `TxSource`: Transaction ingress trait (`pending() -> Vec<Vec<u8>>`); current node wiring uses `NoopTxSource`. [PROPOSED] `pending()` bytes are Ethereum transaction envelope bytes (typed transaction encoding), decode failures are rejected per-transaction with explicit error accounting policy, and deterministic ordering/selection is part of the TxSource policy contract.
- `ExecutionResult`: App-layer execution summary (state root, gas used, receipts root, receipt count) returned by app callbacks; it is not a commit-ready canonical state artifact.
- `FinalizationSink`: Consensus-simplex sink that records finalized height; canonical commit coupling is `UNKNOWN`.
- `BundleState`: Commit-ready state artifact accepted by `InMemoryStateDb::commit`; [PROPOSED] canonical commit requires preserving a `BundleState`-equivalent artifact across propose->finalize seam, and artifact storage/ownership is currently `UNKNOWN`/`BLOCKER`.

## Implementation slices

### 1) Transaction execution in propose()

- Goal: replace MVP-empty proposal with tx-driven EVM execution in `EvmApplication::propose`.
- Crates touched: `app-evm`, `app`, `state`.
- Public types: `app::traits::Application`, `app::types::{EvmBlock,ExecutionResult}`, `state::InMemoryStateDb`.
- Config changes: [PROPOSED] tx execution limits/order knobs may be needed; concrete keys are `UNKNOWN`.
- Interfaces: consume `TxSource::pending()`, produce non-empty `EvmBlock` fields and aligned `ExecutionResult`; `propose()` remains speculative with no canonical commit.
- Pseudo-code sketch:
```rust
fn propose(parent, height) -> Result<(EvmBlock, ExecutionResult), EvmAppError> {
    let pending = tx_source.pending();
    // [PROPOSED] build a speculative state view
    let mut speculative = snapshot_of_state()?;
    let exec = execute_transactions(&mut speculative, pending)?;
    let block = assemble_block_from_exec(parent, height, &exec)?;
    // [PROPOSED] no canonical commit in propose()
    Ok((block, exec.result))
}
```
- Acceptance checks: non-empty pending txs can yield non-empty `transactions`, non-empty roots, and non-zero gas when executable txs exist.
- Test-first hook: red test proving current MVP path emits empty artifacts, then green test asserting deterministic non-empty artifacts under fixture tx input.

### 2) Transaction verification in verify()

- Goal: move from root-only check to replay/recompute verification.
- Crates touched: `app-evm`, `app`, `state`.
- Public types: `app::traits::Application::verify`, `app::types::EvmBlock`, `app_evm::error::EvmAppError`.
- Config changes: [PROPOSED] replay limits/strictness flags may be required; exact config shape is `UNKNOWN`.
- Interfaces: consume candidate `EvmBlock`, recompute execution artifacts on read-only state view, compare against claimed block artifacts.
- Pseudo-code sketch:
```rust
fn verify(parent, block) -> Result<ExecutionResult, EvmAppError> {
    let snapshot = snapshot_of_state_read_only()?;
    let recomputed = replay_block_transactions(&snapshot, block.transactions)?;
    if recomputed.state_root != block.state_root { return Err(state_root_mismatch()); }
    if recomputed.transactions_root != block.transactions_root { return Err(tx_root_mismatch()); }
    if recomputed.receipts_root != block.receipts_root { return Err(receipts_root_mismatch()); }
    if recomputed.gas_used != block.gas_used { return Err(gas_mismatch()); }
    Ok(recomputed)
}
```
- Acceptance checks: tampering `transactions_root`, `receipts_root`, or `gas_used` is rejected.
- Test-first hook: mutation tests that alter one artifact at a time and expect `InvalidBlock` via adapter mapping.

### 3) Real TxSource implementation

- Goal: replace runtime `NoopTxSource` with a concrete source that returns pending tx bytes.
- Crates touched: `app`, `whirlpool-node` (plus [PROPOSED] concrete implementation module in in-scope crate).
- Public types: `app::traits::TxSource`, `app::traits::NoopTxSource` (to be de-emphasized in runtime wiring).
- Config changes: [PROPOSED] source sizing/backpressure knobs; exact keys are `UNKNOWN`.
- Interfaces: node injects `Arc<dyn TxSource>` into `EvmApplication` instead of `Arc::new(NoopTxSource)`; [PROPOSED] source contract includes typed-envelope bytes, per-tx decode-rejection accounting, and deterministic ordering/selection policy.
- Pseudo-code sketch:
```rust
struct QueueTxSource { /* [PROPOSED] queue handle */ }
impl TxSource for QueueTxSource {
    fn pending(&self) -> Vec<Vec<u8>> {
        // [PROPOSED] deterministic extraction policy
        self.read_batch()
    }
}

fn wire_node_app() {
    let tx_source: Arc<dyn TxSource> = Arc::new(QueueTxSource::new());
    let app = EvmApplication::new(config, db, tx_source);
}
```
- Acceptance checks: startup wiring no longer hardcodes `NoopTxSource`; proposal observes txs from source.
- Test-first hook: deterministic ordering fixture for repeated `pending()` calls with same source state.

### 4) State snapshot/rollback

- Goal: enforce speculative execution isolation and rollback safety.
- Crates touched: `state`, `app-evm`.
- Public types: `state::InMemoryStateDb`, [PROPOSED] explicit snapshot execution artifact at app boundary.
- Config changes: none grounded; [PROPOSED] failure-injection test toggles only.
- Interfaces: proposal/verification operate on snapshot/read-only view; canonical DB mutation is deferred to [PROPOSED] finalize->commit seam only.
- Pseudo-code sketch:
```rust
fn run_speculative<F, T>(db: &InMemoryStateDb, f: F) -> Result<T, EvmAppError>
where F: FnOnce(&mut InMemoryStateDb) -> Result<T, EvmAppError> {
    let mut cloned = db.clone();
    let out = f(&mut cloned)?;
    // canonical db unchanged here
    Ok(out)
}

fn on_failure_keep_canonical(db_before, db_after) {
    assert_eq!(db_before.state_root(), db_after.state_root());
}
```
- Acceptance checks: failed propose/verify leaves canonical root unchanged.
- Test-first hook: inject execution failure mid-run and assert byte-level equivalence policy ([PROPOSED] exact byte-level harness is `UNKNOWN`).

### 5) Block assembly correctness

- Goal: ensure block fields match execution outputs and intent criteria.
- Crates touched: `app-evm`, `app`.
- Public types: `app::types::EvmBlock`, `app::types::ExecutionResult`.
- Config changes: none grounded.
- Interfaces: assembly function must bind tx list + roots + gas + state root into one coherent artifact; [PROPOSED] commit-ready artifact preservation is separate from `ExecutionResult`.
- Pseudo-code sketch:
```rust
fn assemble_block_from_exec(parent, height, exec) -> Result<EvmBlock, EvmAppError> {
    let tx_root = derive_transactions_root(&exec.transactions);
    let receipts_root = derive_receipts_root(&exec.receipts);
    Ok(EvmBlock {
        parent_hash: parent.hash(),
        number: height,
        transactions: exec.transactions,
        transactions_root: tx_root,
        receipts_root,
        gas_used: exec.gas_used,
        state_root: exec.state_root,
        ..header_defaults(parent)
    })
}
```
- Acceptance checks: block fields are internally consistent with execution artifacts.
- Test-first hook: fixture-based roundtrip test that reconstructs expected roots/gas from known tx execution and compares to produced block.

### 6) Node wiring updates

- Goal: update composition so startup and finalization seams support full block lifecycle.
- Crates touched: `whirlpool-node`, `app`, `app-evm`, `state`, [PROPOSED] `consensus`/`consensus-simplex` only if callback extensions are required.
- Public types: `CommonwareEngine`, `ApplicationAdapter`, `FinalizationSink`, [PROPOSED] finalize callback contract (`UNKNOWN` in current traits).
- Config changes: [PROPOSED] add explicit wiring/config for concrete tx source; finalize-commit configuration is `UNKNOWN`.
- Interfaces: startup wiring must inject real `TxSource`; [PROPOSED] canonical commit occurs only after finalization through finalize->commit seam.
- Pseudo-code sketch:
```rust
fn main_wiring() -> Result<(), Error> {
    let db = build_state_db();
    let tx_source = build_real_tx_source(); // [PROPOSED]
    let app = EvmApplication::new(cfg, db.clone(), tx_source);
    let adapter = ApplicationAdapter::new(app);
    let sink = FinalizationSink::new(shared_finalized_height);
    let engine = CommonwareEngine::new(consensus_cfg, adapter, sink, network);
    engine.start()?;
    Ok(())
}
```
- Acceptance checks: node startup still succeeds; proposal path uses real tx source; finalization-to-commit seam is explicitly wired or remains clearly marked `UNKNOWN` with blocker tracking.
- Test-first hook: startup integration test asserting dependency graph includes non-noop tx source and finalized-height progress signal.
