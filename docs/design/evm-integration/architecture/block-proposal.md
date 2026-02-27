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
- **Action**: Computes state root from trie, assembles `Header` (parent_hash, state_root, transactions_root, receipts_root, gas_used, etc.), creates `BlockBody`
- **Output**: `BlockBuilderOutcome { block, execution_result, hashed_state, trie_updates }`

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

    let state = State::new(self.state_provider.clone());
    let mut builder = self.evm_config.builder_for_next_block(state, &parent_header, attrs)?;

    for tx in self.pending_transactions() {
        builder.execute_transaction(tx)?;
    }

    let outcome = builder.finish(&self.state_provider)?;
    Ok(self.outcome_to_evm_block(outcome, height, parent))
}
```
