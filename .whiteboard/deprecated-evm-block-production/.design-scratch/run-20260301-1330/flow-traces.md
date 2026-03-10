# Flow Traces: EVM Block Production

## Flow 1: Block Proposal

### Trigger
- Consensus engine invokes `ConsensusApp::propose` through the `CommonwareEngine` startup path.

### Stages
1. Consensus layer requests a proposal from the app boundary (`ConsensusApp::propose`).
2. `ApplicationAdapter::propose` delegates to `Application::propose`.
3. `EvmApplication::propose` reads `state_root` from DB and calls `TxSource::pending()`.
4. Runtime wiring currently uses `NoopTxSource`, so pending txs are empty.
5. `EvmApplication::propose` constructs an empty `EvmBlock` (`transactions=[]`, `transactions_root=EMPTY_ROOT_HASH`, `receipts_root=EMPTY_ROOT_HASH`, `gas_used=0`, `timestamp=parent.timestamp+12`) and returns `(block, ExecutionResult)`.
6. `ApplicationAdapter::propose` returns `Some(block)` and discards `ExecutionResult`.
7. If app proposal returns `Err`, adapter maps it to `None` (abstain).

### Stage ownership
- Consensus trigger and callback ownership: `consensus` / `consensus-simplex`.
- App-to-consensus bridge ownership: `app` (`ApplicationAdapter`).
- Proposal semantics ownership: `app-evm` (`EvmApplication`).
- Pending transaction source ownership: `app` trait + node wiring (`NoopTxSource` in `whirlpool-node`).

### Handoff contracts
- Consensus -> app: `ConsensusApp::propose(parent, height) -> Option<Block>`.
- Adapter -> app implementation: `Application::propose(parent, height) -> Result<(Block, ExecutionResult), Error>`.
- Node -> app runtime dependency: `TxSource::pending() -> Vec<Vec<u8>>`.
- Adapter handoff truncates app output by forwarding only `Block` and dropping `ExecutionResult`.

### Error propagation
- `EvmApplication::propose` errors do not surface as typed consensus errors.
- `ApplicationAdapter::propose` maps all `Err(_)` to `None`, so failure details are swallowed at the boundary.

### UNKNOWN/BLOCKER
- BLOCKER: no tx decode/execute path is implemented in proposal.
- BLOCKER: runtime uses `NoopTxSource`, so non-empty tx proposal is not exercised.
- UNKNOWN: policy for tx ordering/selection is not evidenced in the current handoff contracts.

## Flow 2: Block Verification

### Trigger
- Consensus engine invokes `ConsensusApp::verify(parent, block)` for candidate block validation.

### Stages
1. Consensus layer calls app verification callback (`ConsensusApp::verify`).
2. `ApplicationAdapter::verify` delegates to `EvmApplication::verify`.
3. `EvmApplication::verify` reads current DB `state_root` and compares it with `block.state_root`.
4. On match, verify returns `ExecutionResult` echoing block artifacts (with `receipt_count=0`).
5. On mismatch, verify returns `EvmAppError::StateRootMismatch`.
6. Adapter maps any app verify error to `ConsensusError::InvalidBlock(err.to_string())`.

### Stage ownership
- Verification trigger ownership: `consensus` / `consensus-simplex`.
- Error mapping seam ownership: `app` adapter.
- Verification logic ownership: `app-evm`.
- State root source ownership: `state` DB implementation via `StateProvider`.

### Handoff contracts
- Consensus -> app: `ConsensusApp::verify(parent, block) -> Result<(), ConsensusError>`.
- Adapter -> app implementation: `Application::verify(parent, block) -> Result<ExecutionResult, EvmAppError>`.
- State read contract: `StateProvider::state_root()`.
- Adapter narrows app error surface to `ConsensusError::InvalidBlock(String)`.

### Error propagation
- Verification mismatch is represented in app domain as `EvmAppError::StateRootMismatch`.
- Consensus domain receives only `ConsensusError::InvalidBlock(String)` from adapter.

### UNKNOWN/BLOCKER
- BLOCKER: verification does not replay transactions.
- BLOCKER: verification does not recompute/compare tx root, receipts root, gas usage, or other execution artifacts.
- UNKNOWN: expected canonical replay context (snapshot/rollback/finality coupling) is not grounded in current seam.

## Flow 3: State Commitment

### Trigger
- Intended trigger is post-EVM execution artifact production (bundle state) followed by canonical commit.

### Stages
1. Intended execution output is `BundleState` from EVM execution.
2. Intended persistence path is `InMemoryStateDb::commit(&BundleState)`.
3. State hash can then be derived through `InMemoryStateDb::state_root()`.
4. Current proposal path does not produce `BundleState` for handoff.
5. No grounded finalize hook is shown committing canonical state.

### Stage ownership
- Execution artifact ownership (intended): `app-evm`.
- Commit and state root ownership: `state` crate (`InMemoryStateDb`).
- Canonical lifecycle trigger ownership (intended): consensus/app integration seam.

### Handoff contracts
- Grounded DB API exists: `InMemoryStateDb::commit(&BundleState)` and `InMemoryStateDb::state_root()`.
- Missing grounded upstream handoff: proposal/verify paths do not provide a `BundleState` artifact to commit.
- Missing grounded downstream handoff: no consensus finalization callback into commit path.

### Error propagation
- No end-to-end commit error path is evidenced for finalized block state persistence.
- Current evidence only shows commit API availability, not integration-time error plumbing.

### UNKNOWN/BLOCKER
- BLOCKER: proposal path lacks `BundleState` production.
- BLOCKER: no grounded finalization-to-commit callback for canonical state persistence.
- UNKNOWN: canonical persistence semantics under reorg/fault are not evidenced.

## Flow 4: Node Startup

### Trigger
- `whirlpool-node` process start (`main`).

### Stages
1. Node initializes runtime/tracing and networking provider.
2. Node builds consensus config (`CommonwareConfig`).
3. Node wires `state_db` (`TestStateDb` wrapping `InMemoryStateDb`), chain spec, and `WhirlpoolEvmConfig`.
4. Node wires `NoopTxSource`, constructs `EvmApplication`, then wraps it in `ApplicationAdapter`.
5. Node constructs `CommonwareEngine` with app, sink, config, and network provider.
6. Node calls `engine.start()`.
7. Current `CommonwareEngine::start` behavior is explicitly stub/simulated.

### Stage ownership
- Composition/wiring ownership: `whirlpool-node`.
- App runtime ownership: `app` + `app-evm`.
- State backend ownership: `state`.
- Consensus runtime ownership: `consensus-simplex` implementation of `consensus` traits.

### Handoff contracts
- Node -> app: concrete `EvmApplication` injected into `ApplicationAdapter`.
- Node -> engine: `ConsensusEngine::start()` via `CommonwareEngine`.
- Node -> sink: `FinalizationSink` shared with atomic finalized height.

### Error propagation
- `engine.start()` failure is surfaced at node boundary and currently treated as fatal (`expect`).
- Simplex startup maps network/startup failures into `ConsensusError::Other`.

### UNKNOWN/BLOCKER
- BLOCKER: consensus start path is currently stubbed/simulated, limiting grounding of full production flow behavior.
- UNKNOWN: full simplex wiring behavior across mailbox/app adapter/sink in non-stub mode is not evidenced here.

## Flow 5: Block Finalization

### Trigger
- Consensus emits `ConsensusEvent::Finalized`.

### Stages
1. Consensus event stream emits finalized event carrying block/height/proof.
2. `consensus-simplex` `FinalizationSink` handles the event.
3. Sink updates shared `AtomicU64` finalized height.
4. Node observes finalized height as progress signal.
5. No grounded callback from this path into canonical state commit is shown.

### Stage ownership
- Finalization event shape ownership: `consensus` (`ConsensusEvent`).
- Finalization event handling ownership: `consensus-simplex` (`FinalizationSink`).
- Runtime observation ownership: `whirlpool-node` (shared atomic height wiring).
- Canonical state persistence ownership: UNKNOWN at current seam.

### Handoff contracts
- Event contract: `ConsensusEvent::Finalized { block, height, proof }` into `EventSink::handle`.
- Sink side effect contract: update `AtomicU64` finalized height.
- Missing contract: finalized block handoff into app/state commit path.

### Error propagation
- Finalization sink path shown here performs atomic update/logging; no canonical commit step means no commit error propagation path is grounded.

### UNKNOWN/BLOCKER
- BLOCKER: no finalize -> state commit callback is grounded.
- UNKNOWN: canonical persistence semantics for finalized blocks are not evidenced.
- UNKNOWN: interaction between finalization, replay, and durability guarantees is not grounded.
