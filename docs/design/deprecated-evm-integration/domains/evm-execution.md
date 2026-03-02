# Domain: EVM Execution

## Definition

The EVM execution domain handles Ethereum Virtual Machine transaction processing: configuring the EVM environment, executing transactions within blocks, building execution results, and assembling valid blocks from those results. This domain is entirely backed by reth's EVM abstraction stack.

## Derived crates

| Crate | Role | Status |
|---|---|---|
| `app-evm` | [PROPOSED] `WhirlpoolEvmConfig` implementing `ConfigureEvm`, block executor + assembler wiring | Proposed |
| `reth-evm` (vendor) | Core traits: `ConfigureEvm`, `Executor`, `BlockBuilder`, `BlockAssembler` | Grounded |
| `reth-evm-ethereum` (vendor) | Reference impl: `EthEvmConfig`, `EthBlockAssembler`, `RethReceiptBuilder` | Grounded |
| `reth-revm` (vendor) | revm wrapper, `State<DB>` | Grounded |

## Key public contracts

### ConfigureEvm trait (grounded — vendor)
**Grounded**: `vendor/reth/crates/evm/evm/src/lib.rs::ConfigureEvm`

Central trait binding all EVM configuration. Associated types:
- `Primitives: NodePrimitives` — block/tx/receipt types
- `Error: Error + Send + Sync`
- `NextBlockEnvCtx: Debug + Clone` — CL attributes for next block
- `BlockExecutorFactory` — creates block-level executors
- `BlockAssembler` — assembles valid blocks from execution results

Required methods:
- `block_executor_factory(&self) -> &Self::BlockExecutorFactory`
- `block_assembler(&self) -> &Self::BlockAssembler`
- `evm_env(&self, header) -> Result<EvmEnv, Error>` — env for existing block
- `next_evm_env(&self, parent, attributes) -> Result<EvmEnv, Error>` — env for new block
- `context_for_block(&self, block) -> Result<ExecutionCtx, Error>` — execution context for existing block
- `context_for_next_block(&self, parent, attributes) -> Result<ExecutionCtx, Error>` — execution context for new block

Default methods (derived from above): `evm_factory()`, `evm_with_env()`, `create_executor()`, `executor_for_block()`, `create_block_builder()`, `builder_for_next_block()`, `executor()`, `batch_executor()`

### EthEvmConfig reference pattern (grounded — vendor)
**Grounded**: `vendor/reth/crates/ethereum/evm/src/lib.rs::EthEvmConfig`

```rust
pub struct EthEvmConfig<C = ChainSpec, EvmF = EthEvmFactory> {
    executor_factory: EthBlockExecutorFactory<RethReceiptBuilder, Arc<C>, EvmF>,
    block_assembler: EthBlockAssembler<C>,
}
```

This is the pattern `WhirlpoolEvmConfig` will follow.

### Three execution layers (grounded — vendor)
1. **EvmFactory** → creates individual `Evm` instances for single-transaction execution
2. **BlockExecutorFactory** → creates `Executor<DB>` for executing all transactions in a block
3. **BlockAssembler** → takes execution results and constructs a valid block (header, body)

**Grounded**: `vendor/reth/crates/evm/evm/src/execute.rs::Executor`, `::BlockAssembler`, `::BlockBuilder`

### [PROPOSED] WhirlpoolEvmConfig
```rust
pub struct WhirlpoolEvmConfig {
    executor_factory: EthBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EthEvmFactory>,
    block_assembler: EthBlockAssembler<ChainSpec>,
    chain_spec: Arc<ChainSpec>,
}

impl ConfigureEvm for WhirlpoolEvmConfig {
    type Primitives = EthPrimitives; // reuse Ethereum primitives initially
    type Error = Infallible;         // follow EthEvmConfig pattern
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = EthBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EthEvmFactory>;
    type BlockAssembler = EthBlockAssembler<ChainSpec>;
    // ... method implementations mirror EthEvmConfig
}
```

**Rationale**: Start with Ethereum-compatible primitives and chain spec. Diverge later for Sahara-specific features (custom precompiles, gas model, etc.) by swapping `EvmFactory` and `ChainSpec`.

## Core workflows

### Single block execution (proposed)
```pseudo
fn execute_block(config: &WhirlpoolEvmConfig, block: &Block, state: StateProvider) {
    let evm_env = config.evm_env(block.header())?;
    let ctx = config.context_for_block(block)?;
    let mut executor = config.create_executor(State::new(state));
    let result = executor.execute_one(&evm_env, &ctx, block)?;
    let bundle_state = executor.into_state();
    // result: BlockExecutionResult { receipts, gas_used, ... }
    // bundle_state: state changes to apply
}
```

### Block building for proposal (proposed)
```pseudo
fn build_next_block(config: &WhirlpoolEvmConfig, parent: &SealedHeader, attrs: NextBlockEnvAttributes, txs: Vec<Tx>, state: StateProvider) {
    let mut builder = config.builder_for_next_block(State::new(state), parent, attrs)?;
    for tx in txs {
        builder.execute_transaction(tx)?;
    }
    let outcome = builder.finish(state_provider)?;
    // outcome: BlockBuilderOutcome { block, execution_result, hashed_state, trie_updates }
}
```

## Open questions / TODOs

- ~~BLOCKER: Which `ChainSpec` to use?~~ **Resolved (round 3)**: Reuse reth's `ChainSpec` via `ChainSpecBuilder`. Sahara chain ID = `313_371` [PROPOSED], all hardforks through Cancun activated at genesis (block 0, timestamp 0), empty genesis allocation (no pre-funded accounts), 30M gas limit. Construction: `ChainSpec::builder().chain(Chain::from_id(313_371)).genesis(genesis).cancun_activated().build()`. Grounded on `vendor/reth/crates/chainspec/src/spec.rs::ChainSpecBuilder`. <!-- continuation round 3: B-001 resolved -->
- UNKNOWN: Whether custom precompiles are needed at this stage. If yes, `EvmFactory` needs customization.
- ~~UNKNOWN: State database implementation~~ — Resolved (round 2). `InMemoryStateDb` from `state` crate. See `state/README.md`. <!-- continuation round 2: B-002 resolved -->
