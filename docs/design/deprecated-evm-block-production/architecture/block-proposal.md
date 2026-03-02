# Block Proposal

## Trigger

Consensus invokes `ConsensusApp::propose(parent, height)` via the consensus engine callback boundary.

## Stages

1. Consensus runtime requests a proposal from the app boundary (`consensus` -> `app` seam).
2. `ApplicationAdapter::propose` delegates into `Application::propose` (`app` -> `app-evm` seam).
3. `EvmApplication::propose` reads current `state_root` and fetches pending tx bytes from `TxSource`.
4. Current node wiring uses `NoopTxSource`, so pending tx list is empty.
5. `EvmApplication::propose` returns an empty `EvmBlock` (`transactions=[]`, empty tx/receipt roots, `gas_used=0`) plus `ExecutionResult`.
6. `ApplicationAdapter::propose` forwards `Some(block)` to consensus and drops `ExecutionResult`.
7. If app propose returns error, adapter maps it to `None` (abstain).
8. Canonical timing rule: `propose()` is speculative and does not perform canonical state commit.

## Outputs

- Consensus-visible output: `Option<EvmBlock>` from `ConsensusApp::propose`.
- App-local output (not forwarded): `ExecutionResult` from `Application::propose`.
- Canonical commit output: none in this flow; [PROPOSED] canonical commit happens only after finalization via finalize->commit seam (`BLOCKER`).

## Stage ownership

- Trigger + callback orchestration: `consensus`, `consensus-simplex`.
- Bridge mapping and loss of execution artifact: `app` (`ApplicationAdapter`).
- Proposal semantics and block assembly (current MVP behavior): `app-evm`.
- Tx ingress implementation choice: `app` trait + `whirlpool-node` wiring.

## Handoff contracts

- `consensus` -> `app`: `ConsensusApp::propose(parent, height) -> Option<Block>`.
- `app` adapter -> `app-evm`: `Application::propose(parent, height) -> Result<(Block, ExecutionResult), Error>`.
- `whirlpool-node` -> `app-evm`: injected `Arc<dyn TxSource>` dependency.
- [PROPOSED] TxSource byte contract: `TxSource::pending()` returns Ethereum transaction envelope bytes (typed transaction encoding).
- [PROPOSED] Decode failures are rejected per-transaction with explicit error accounting policy.
- [PROPOSED] Deterministic ordering/selection is part of TxSource policy contract.
- Contract gap: adapter currently truncates `(Block, ExecutionResult)` to `Block` for consensus.

## Error propagation

- `EvmApplication::propose` error does not propagate as typed consensus error.
- Adapter maps all proposal errors to `None`, so cause is not preserved across seam.
- [PROPOSED] richer proposal-failure diagnostics at consensus boundary remain unimplemented.

## Pseudo-code sketch

```rust
fn consensus_propose(parent, height) -> Option<EvmBlock> {
    match app_adapter.propose(parent, height) {
        Some(block) => Some(block),
        None => None, // abstain on app error
    }
}

fn app_adapter_propose(parent, height) -> Option<EvmBlock> {
    match evm_app.propose(parent, height) {
        Ok((block, _exec)) => Some(block), // execution result dropped at boundary
        Err(_err) => None,
    }
}
```

## Open questions / TODOs

- `BLOCKER`: implement non-empty tx decode/execute path in propose.
- `BLOCKER`: replace `NoopTxSource` runtime wiring with real `TxSource` implementation.
- `UNKNOWN`: deterministic tx ordering/selection policy details for `TxSource::pending()`.
- [PROPOSED] define explicit per-transaction decode failure accounting outputs (for example counters/rejection reports).
- [PROPOSED] decide whether proposal failure reasons should cross into consensus metrics/events.
