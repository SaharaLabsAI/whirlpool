# Flow: Consensus ↔ App Bridge

## Trigger
`consensus-simplex::AppAdapter` calls `ConsensusApp` methods during consensus rounds.

## Overview

The `ApplicationAdapter<A: Application>` in the `app` crate bridges the richer `Application` interface to the simpler `ConsensusApp` interface that the consensus engine expects. This is a stateless, transparent adapter.

## Bridge mapping

| ConsensusApp method | Application method | Mapping |
|---|---|---|
| `genesis()` | `genesis()` | Direct passthrough, same return type |
| `propose(parent, height)` | `propose(parent, height)` | `Ok((block, result))` → `Some(block)`, `Err(_)` → `None` |
| `verify(parent, block)` | `verify(parent, block)` | `Ok(result)` → `Ok(())`, `Err(e)` → `Err(ConsensusError::Verification(e.to_string()))` |

## Data flow diagram

```
consensus-simplex (AppAdapter)
       │
       │ calls ConsensusApp::propose(parent, height)
       ▼
app::ApplicationAdapter<EvmApplication<DB>>
       │
       │ delegates to Application::propose(parent, height)
       ▼
app-evm::EvmApplication<DB>
       │
       │ returns Result<(EvmBlock, ExecutionResult), EvmAppError>
       ▼
app::ApplicationAdapter
       │
       │ maps: Ok((block, _result)) → Some(block)
       │       Err(_) → None
       ▼
consensus-simplex (AppAdapter)
       │
       │ receives Option<EvmBlock>
       ▼
consensus engine broadcasts block
```

## Key design decision

**Execution results are discarded at the consensus boundary.** The `ConsensusApp` trait has no concept of execution results — it only sees blocks. This means:

1. The `ApplicationAdapter` drops `ExecutionResult` from `propose()` return
2. The verifying node re-computes `ExecutionResult` independently during `verify()`
3. Post-finalization state commitment happens outside the consensus path (in the `EventSink` handler)

This is intentional — consensus only needs to agree on block ordering and identity. State transitions are deterministic from the block content, so every honest node arrives at the same state.

### State persistence path [PROPOSED]

Since `ExecutionResult` returned by `Application::verify()` and `Application::propose()` is a summary (hashes + gas), the full state diff (`BundleState`) must be persisted separately. Two approaches:

1. **Internal caching** (recommended): `EvmApplication` caches the `BundleState` from the last execution. When `EventSink::handle(Finalized{block})` fires, the node calls `evm_app.commit_state(block.id())` which flushes the cached `BundleState` to the state DB.
2. **Re-execution**: The `EventSink` handler re-executes the finalized block against the state DB to produce and commit the `BundleState`. Simpler but doubles execution cost.

**Recommendation**: Approach 1 — cache `BundleState` keyed by block ID in `EvmApplication`. The `EventSink` handler retrieves and commits it upon finalization. This avoids double execution while keeping `ExecutionResult` lightweight at the `Application` trait boundary.

## Ownership

| Component | Crate |
|---|---|
| `ApplicationAdapter` | `app` |
| `ConsensusApp` trait | `consensus` |
| `Application` trait | `app` |
| `AppAdapter` (consensus-simplex side) | `consensus-simplex` |
