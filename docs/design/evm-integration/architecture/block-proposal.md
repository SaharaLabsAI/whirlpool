# Flow: Block Proposal

## Trigger
Consensus engine (via `consensus-simplex`) calls `ConsensusApp::propose(parent, height)` when this validator is the block leader for the given height.

## Stages

### Stage 1: Consensus → App bridge
- **Owner**: `app::ApplicationAdapter`
- **Input**: `parent: &EvmBlock`, `height: u64`
- **Action**: Delegates to `Application::propose(parent, height)`
- **Output**: `Option<EvmBlock>` (strips `ExecutionResult`)

### Stage 2: Environment construction
- **Owner**: `app-evm::EvmApplication`
- **Input**: `parent` header, CL attributes (timestamp, gas limit, etc.)
- **Action**: Calls `WhirlpoolEvmConfig::next_evm_env(parent_header, attrs)` → `EvmEnv`
- **Handoff type**: `EvmEnv<SpecId, BlockEnv>` (vendor type)
- **Action**: Calls `WhirlpoolEvmConfig::context_for_next_block(parent_header, attrs)` → `EthBlockExecutionCtx`

### Stage 3: Block builder creation
- **Owner**: `app-evm::EvmApplication` via `WhirlpoolEvmConfig`
- **Input**: `EvmEnv`, `EthBlockExecutionCtx`, `State<DB>`
- **Action**: `config.builder_for_next_block(state, parent_sealed_header, attrs)`
- **Output**: `BasicBlockBuilder<EthBlockExecutorFactory, Executor, EthBlockAssembler, EthPrimitives>`
- **Internal**: Creates `Executor` via `BlockExecutorFactory::create_executor()`, binds with `BlockAssembler`

### Stage 4: Transaction execution
- **Owner**: `BasicBlockBuilder` (vendor)
- **Input**: Pending transactions from tx source
- **Action**: For each tx: `builder.execute_transaction(recovered_tx)`
  - Creates EVM via `EvmFactory::create_evm()`
  - Executes tx against current state
  - Builds receipt via `ReceiptBuilder::build_receipt()`
  - Accumulates state changes in `BundleState`
- **Error**: `BlockExecutionError` on any tx failure

### Stage 5: Block assembly
- **Owner**: `BasicBlockBuilder::finish()` → `EthBlockAssembler` (vendor)
- **Input**: `BlockAssemblerInput { evm_env, execution_ctx, parent, transactions, output, bundle_state, state_provider, state_root }`
- **Action**: Assembles `Header` (parent_hash, state_root, transactions_root, receipts_root, gas_used, etc.), creates `BlockBody`
- **Output**: `BlockBuilderOutcome { block, execution_result, hashed_state, trie_updates }`

<!-- continuation round 2 -->
**State root in assembly**: `state_root` is an **input** to the assembler, not computed by it. The caller (`EvmApplication`) must:
1. Extract `BundleState` via `State::take_bundle()` after execution
2. Call `state_db.commit(&bundle_state)` to apply the diff to `InMemoryStateDb`
3. Call `state_db.state_root()` to compute the new root
4. Pass this root into `BlockAssemblerInput.state_root`

### Stage 6: Conversion to EvmBlock
- **Owner**: `app-evm::EvmApplication`
- **Input**: `BlockBuilderOutcome`
- **Action**: Extracts consensus fields (height, parent_id) + EVM fields (state_root, receipts_root, gas_used, transactions) into `EvmBlock`
- **Output**: `(EvmBlock, ExecutionResult)`

### Stage 7: Return to consensus
- **Owner**: `app::ApplicationAdapter`
- **Input**: `Result<(EvmBlock, ExecutionResult), EvmAppError>`
- **Action**: Maps to `Option<EvmBlock>` — `Ok((block, _)) → Some(block)`, `Err(_) → None`
- **Output**: `Option<EvmBlock>` back to consensus engine

## Error propagation

```
EvmFactory error → BlockExecutionError → EvmAppError::Execution → ApplicationAdapter returns None
State/DB error → ProviderError → EvmAppError::State → ApplicationAdapter returns None
```

## Pseudo-code summary

```rust
// In EvmApplication::propose()
async fn propose(&self, parent: &EvmBlock, height: u64) -> Result<(EvmBlock, ExecutionResult), EvmAppError> {
    let parent_header = self.to_sealed_header(parent)?;
    let attrs = NextBlockEnvAttributes {
        timestamp: now_secs(),
        suggested_fee_recipient: self.fee_recipient,
        prev_randao: parent.prev_randao_or_default(),
        gas_limit: self.gas_limit,
        parent_beacon_block_root: None,
        withdrawals: None,
        extra_data: Bytes::default(),
    };

    // <!-- continuation round 2: clone-based state isolation with Arc<RwLock> -->
    // Application trait methods take &self, so state_db is wrapped in Arc<RwLock<InMemoryStateDb>>.
    // We clone the inner InMemoryStateDb (not the Arc) to get an independent snapshot.
    let state_snapshot = self.state_db.read().unwrap().clone();  // independent snapshot
    let state = State::new(state_snapshot.clone());  // revm State wrapper
    let evm_env = self.evm_config.next_evm_env(&parent_header, attrs.clone())?;
    let ctx = self.evm_config.context_for_next_block(&parent_header, attrs)?;
    let mut executor = self.evm_config.executor_factory().create_executor(state);

    // Execute transactions one-by-one against the snapshot state
    for tx in self.pending_transactions() {
        executor.execute_transaction(tx)?;
    }

    // Extract BundleState from executor, commit to snapshot, compute root
    let BlockExecutionOutput { state: bundle_state, result, .. } = executor.finish();
    let mut committed_snapshot = state_snapshot;
    committed_snapshot.commit(&bundle_state);
    let state_root = committed_snapshot.state_root();

    // Assemble block with computed state_root (assembler receives root as input)
    let block = self.assemble_block(&parent_header, &evm_env, &ctx, &result, state_root)?;

    // Return block + bundle_state. Canonical state is NOT committed here.
    // Commitment happens only when consensus finalizes this block,
    // via EvmFinalizationSink::finalized() which calls:
    //   self.state_db.write().unwrap().commit(&bundle_state);
    //   self.state_db.write().unwrap().insert_block_hash(height, block_hash);
    Ok((block, ExecutionResult { state_root, bundle_state, .. }))
}
```
