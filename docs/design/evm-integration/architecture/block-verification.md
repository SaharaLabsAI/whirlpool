# Flow: Block Verification

## Trigger
Consensus engine (via `consensus-simplex`) calls `ConsensusApp::verify(parent, block)` when a proposed block is received from another validator.

## Stages

### Stage 1: Consensus → App bridge
- **Owner**: `app::ApplicationAdapter`
- **Input**: `parent: &EvmBlock`, `block: &EvmBlock`
- **Action**: Delegates to `Application::verify(parent, block)`
- **Output**: `Result<(), ConsensusError>` (maps `EvmAppError` → `ConsensusError`)

### Stage 2: Block header reconstruction
- **Owner**: `app-evm::EvmApplication`
- **Input**: `EvmBlock` (proposed block)
- **Action**: Reconstruct reth `Header` from `EvmBlock` fields. Reconstruct parent `SealedHeader`.
- **Output**: Reth-compatible block representation

### Stage 3: Environment construction
- **Owner**: `app-evm::EvmApplication` via `WhirlpoolEvmConfig`
- **Input**: Block header
- **Action**: `config.evm_env(block_header)` → `EvmEnv`, `config.context_for_block(block)` → `EthBlockExecutionCtx`
- **Output**: EVM environment + execution context

### Stage 4: Executor creation + block re-execution
- **Owner**: `app-evm::EvmApplication` via `WhirlpoolEvmConfig`
- **Input**: `EvmEnv`, `EthBlockExecutionCtx`, `State<DB>`
- **Action**: `config.create_executor(State::new(db))` → `BasicBlockExecutor`. Then `executor.execute_one(&evm_env, &ctx, block)`.
  - Internally: For each tx in block → create EVM → execute → accumulate receipts + state
- **Output**: `BlockExecutionResult<Receipt>` + modified state in executor

### Stage 5: State commitment and root computation
- **Owner**: `app-evm::EvmApplication`
- **Input**: `executor.into_state()` → `State<DB>` → `take_bundle()` → `BundleState`
- **Action**:
  <!-- continuation round 2: concrete state commitment -->
  1. Clone `self.state_db` as snapshot before execution (done in Stage 4)
  2. Call `state_snapshot.commit(&bundle_state)` — applies diff to `InMemoryStateDb` [PROPOSED]
  3. Call `state_snapshot.state_root()` — computes deterministic hash [PROPOSED]
- **Output**: Computed `state_root: B256`

### Stage 6: State root comparison
- **Owner**: `app-evm::EvmApplication`
- **Input**: Computed `state_root: [u8; 32]` vs `block.state_root: [u8; 32]`
- **Action**: If equal → verification passes. If different → `EvmAppError::StateRootMismatch`.
- **Output**: `Result<ExecutionResult, EvmAppError>`

### Stage 7: Return to consensus
- **Owner**: `app::ApplicationAdapter`
- **Input**: `Result<ExecutionResult, EvmAppError>`
- **Action**: Maps `Ok(_) → Ok(())`, `Err(e) → Err(ConsensusError::Verification(e.to_string()))`
- **Output**: `Result<(), ConsensusError>` back to consensus engine

## Error propagation

```
EVM execution error → BlockExecutionError → EvmAppError::Execution → ConsensusError::Verification
State root mismatch → EvmAppError::StateRootMismatch → ConsensusError::Verification
DB error → ProviderError → EvmAppError::State → ConsensusError::Verification
```

## Pseudo-code summary

```rust
// In EvmApplication::verify()
async fn verify(&self, parent: &EvmBlock, block: &EvmBlock) -> Result<ExecutionResult, EvmAppError> {
    let block_header = self.to_header(block)?;
    let reth_block = self.to_reth_block(block)?;

    let evm_env = self.evm_config.evm_env(&block_header)?;
    let ctx = self.evm_config.context_for_block(&reth_block)?;

    // <!-- continuation round 2: clone-based state isolation with Arc<RwLock> -->
    // Application trait methods take &self, so state_db is wrapped in Arc<RwLock<InMemoryStateDb>>.
    // We clone the inner InMemoryStateDb (not the Arc) to get an independent snapshot.
    let state_snapshot = self.state_db.read().unwrap().clone();  // independent snapshot
    let state = State::new(state_snapshot.clone());  // revm State wrapper
    let mut executor = self.evm_config.executor_factory().create_executor(state);
    let result = executor.execute_one(&evm_env, &ctx, &reth_block)?;

    // Extract BundleState, commit to snapshot, compute root
    let BlockExecutionOutput { state: bundle_state, .. } = executor.finish();
    let mut committed_snapshot = state_snapshot;
    committed_snapshot.commit(&bundle_state);
    let computed_root = committed_snapshot.state_root();

    if computed_root != block.state_root {
        // Discard snapshot — canonical state unchanged
        return Err(EvmAppError::StateRootMismatch {
            expected: block.state_root,
            computed: computed_root,
        });
    }

    // Root matches — return result + bundle_state for later finalization.
    // Canonical state is NOT committed here.
    // Commitment happens only when consensus finalizes this block,
    // via EvmFinalizationSink::finalized() which calls:
    //   self.state_db.write().unwrap().commit(&bundle_state);
    //   self.state_db.write().unwrap().insert_block_hash(height, block_hash);

    Ok(ExecutionResult {
        state_root: block.state_root,
        receipts_root: block.receipts_root,
        gas_used: result.gas_used,
        receipt_count: result.receipts.len(),
        bundle_state,
    })
}
```
