# Block Verification

## Trigger

Consensus invokes `ConsensusApp::verify(parent, block)` to validate a candidate block.

## Stages

1. Consensus runtime calls verify callback on app boundary.
2. `ApplicationAdapter::verify` delegates to `Application::verify` implemented by `EvmApplication`.
3. `EvmApplication::verify` reads current DB `state_root`.
4. Current logic compares only `block.state_root` against computed current root.
5. On match, verify returns `ExecutionResult` echoing block artifacts.
6. On mismatch, verify returns `EvmAppError::StateRootMismatch`.
7. Adapter maps app error into `ConsensusError::InvalidBlock(err.to_string())`.

## Outputs

- Consensus-visible output: `Result<(), ConsensusError>`.
- App-visible output: `Result<ExecutionResult, EvmAppError>` before adapter mapping.
- No grounded canonical state commit/mutation output in current verify path.

## Stage ownership

- Verification trigger: `consensus`, `consensus-simplex`.
- Error translation seam: `app` (`ApplicationAdapter`).
- Verification semantics: `app-evm`.
- Root source: `state` through `StateProvider`/DB abstraction.

## Handoff contracts

- `consensus` -> `app`: `ConsensusApp::verify(parent, block) -> Result<(), ConsensusError>`.
- `app` adapter -> `app-evm`: `Application::verify(parent, block) -> Result<ExecutionResult, Error>`.
- `app-evm` -> `state`: state-root read contract.
- Contract limitation: consensus receives stringified invalid-block reason, not structured execution mismatch taxonomy.

## Error propagation

- `EvmAppError::StateRootMismatch` is the grounded mismatch class in current path.
- Adapter maps all verify errors to `ConsensusError::InvalidBlock(String)`.
- `BLOCKER`: verification integrity invariant requires replay/recompute of tx/receipt/gas artifacts, not just state-root compare.

## Pseudo-code sketch

```rust
fn consensus_verify(parent, block) -> Result<(), ConsensusError> {
    app_adapter.verify(parent, block)
}

fn app_adapter_verify(parent, block) -> Result<(), ConsensusError> {
    evm_app
        .verify(parent, block)
        .map(|_exec| ())
        .map_err(|err| ConsensusError::InvalidBlock(err.to_string()))
}
```

## Open questions / TODOs

- `BLOCKER`: add replay/recompute checks for `transactions_root`, `receipts_root`, and `gas_used`.
- `UNKNOWN`: read-only guarantees once full replay is introduced (current root-read path is simple and grounded).
- [PROPOSED] introduce structured mismatch variants for diagnostics across seams.
- [PROPOSED] define deterministic replay context tied to parent state snapshot.
