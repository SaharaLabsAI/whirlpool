# app-evm

## Purpose

Concrete EVM application crate that implements the `app::Application` trait using reth's EVM execution stack. Provides `WhirlpoolEvmConfig` (implementing `reth_evm::ConfigureEvm`) and `EvmApplication` (implementing `app::Application`). This crate bridges the abstract application layer with reth's battle-tested EVM execution, block building, and block assembly machinery.

## Public API at a glance (crate root exports)

[PROPOSED] — all items below are proposed; this crate does not yet exist.

```rust
// lib.rs
pub mod config;
pub mod executor;
pub mod error;

pub use config::WhirlpoolEvmConfig;
pub use executor::EvmApplication;
pub use error::EvmAppError;
```

## Modules

| Module | Responsibilities |
|---|---|
| `config` | `WhirlpoolEvmConfig` struct + `ConfigureEvm` impl — EVM environment construction, factory wiring |
| `executor` | `EvmApplication` struct + `Application` impl — block proposal, verification using EVM |
| `error` | `EvmAppError` — wraps reth execution errors |

## Types & traits (public contract)

### WhirlpoolEvmConfig [PROPOSED]

```rust
/// EVM configuration for Whirlpool / Sahara Chain.
/// Mirrors EthEvmConfig pattern but scoped to Sahara's chain spec.
pub struct WhirlpoolEvmConfig {
    chain_spec: Arc<ChainSpec>,
    executor_factory: EthBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EthEvmFactory>,
    block_assembler: EthBlockAssembler<ChainSpec>,
}

impl ConfigureEvm for WhirlpoolEvmConfig {
    type Primitives = EthPrimitives;
    type Error = Infallible;
    type NextBlockEnvCtx = NextBlockEnvAttributes;
    type BlockExecutorFactory = EthBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EthEvmFactory>;
    type BlockAssembler = EthBlockAssembler<ChainSpec>;

    fn block_executor_factory(&self) -> &Self::BlockExecutorFactory {
        &self.executor_factory
    }
    fn block_assembler(&self) -> &Self::BlockAssembler {
        &self.block_assembler
    }
    fn evm_env(&self, header: &Header) -> Result<EvmEnv<..>, Infallible> { ... }
    fn next_evm_env(&self, parent: &Header, attrs: NextBlockEnvAttributes) -> Result<EvmEnv<..>, Infallible> { ... }
    fn context_for_block(&self, block: &Block) -> Result<EthBlockExecutionCtx, Infallible> { ... }
    fn context_for_next_block(&self, parent: &Header, attrs: NextBlockEnvAttributes) -> Result<EthBlockExecutionCtx, Infallible> { ... }
}
```

**Grounded pattern**: mirrors `vendor/reth/crates/ethereum/evm/src/lib.rs::EthEvmConfig` exactly.

Construction:
```rust
impl WhirlpoolEvmConfig {
    /// Create with a chain spec
    pub fn new(chain_spec: Arc<ChainSpec>) -> Self { ... }

    /// Access the chain spec
    pub fn chain_spec(&self) -> &Arc<ChainSpec> { ... }
}
```

### EvmApplication [PROPOSED]

```rust
/// Concrete Application impl backed by reth EVM.
/// Generic over DB to allow different state providers.
/// DB is wrapped in Arc<RwLock<>> because Application trait methods take &self,
/// requiring interior mutability for state access. [PROPOSED]
pub struct EvmApplication<DB: Database + Clone> {
    evm_config: WhirlpoolEvmConfig,
    state_db: Arc<RwLock<DB>>,  // <!-- continuation round 2: Arc<RwLock> for &self compatibility -->
    // tx_source: impl TxSource, // BLOCKER: tx source interface TBD
}

impl<DB: Database + Clone + Send + Sync + 'static> Application for EvmApplication<DB> {
    type Block = EvmBlock;
    type ExecutionResult = ExecutionResult;
    type Error = EvmAppError;

    async fn genesis(&self) -> EvmBlock {
        // Produce genesis EvmBlock with empty state root
        EvmBlock {
            height: 0,
            parent_id: [0u8; 32],
            state_root: EMPTY_ROOT_HASH,
            ..Default::default()
        }
    }

    async fn propose(&self, parent: &EvmBlock, height: u64) -> Result<(EvmBlock, ExecutionResult), EvmAppError> {
        // 1. Construct next block env from parent
        let attrs = NextBlockEnvAttributes { timestamp, ... };
        // 2. Clone inner state for speculative execution (Arc<RwLock> → read → clone)
        let state_snapshot = self.state_db.read().unwrap().clone();
        let state = State::new(state_snapshot.clone());
        // 3. Execute transactions against snapshot
        let mut executor = self.evm_config.executor_factory().create_executor(state);
        for tx in self.pending_transactions() {
            executor.execute_transaction(tx)?;
        }
        // 4. Extract bundle, commit to snapshot, compute root
        let BlockExecutionOutput { state: bundle_state, result, .. } = executor.finish();
        let mut committed_snapshot = state_snapshot;
        committed_snapshot.commit(&bundle_state);
        let state_root = committed_snapshot.state_root();
        // 5. Assemble block with computed root
        let block = self.assemble_block(parent_header, result, state_root)?;
        // 6. Return block + bundle. Canonical commit happens on finalization only.
        Ok((block, ExecutionResult { state_root, bundle_state, .. }))
    }

    async fn verify(&self, parent: &EvmBlock, block: &EvmBlock) -> Result<ExecutionResult, EvmAppError> {
        // 1. Re-execute block.transactions against parent state snapshot
        let state_snapshot = self.state_db.read().unwrap().clone();
        let state = State::new(state_snapshot.clone());
        let evm_env = self.evm_config.evm_env(block_header)?;
        let ctx = self.evm_config.context_for_block(block_as_reth)?;
        let mut executor = self.evm_config.executor_factory().create_executor(state);
        let result = executor.execute_one(&evm_env, &ctx, block_as_reth)?;
        // 2. Compute state root on committed snapshot
        let BlockExecutionOutput { state: bundle_state, .. } = executor.finish();
        let mut committed_snapshot = state_snapshot;
        committed_snapshot.commit(&bundle_state);
        let computed_root = committed_snapshot.state_root();
        // 3. Compare state roots
        if computed_root != block.state_root {
            return Err(EvmAppError::StateRootMismatch { expected: block.state_root, computed: computed_root });
        }
        // 4. Return result + bundle. Canonical commit happens on finalization only.
        Ok(ExecutionResult { state_root: block.state_root, bundle_state, .. })
}
```

### EvmAppError [PROPOSED]

```rust
pub enum EvmAppError {
    /// Block execution failed
    Execution(BlockExecutionError),
    /// State root mismatch during verification
    StateRootMismatch { expected: [u8; 32], computed: [u8; 32] },
    /// State provider / database error
    State(String),
    /// Invalid block structure
    InvalidBlock(String),
}
```

## Config schema

### WhirlpoolEvmConfig construction [PROPOSED]

```rust
let chain_spec = Arc::new(build_sahara_chain_spec());

/// Build the Sahara / Whirlpool chain specification.
/// Uses reth's ChainSpecBuilder with all hardforks through Cancun activated at genesis.
/// [PROPOSED] — grounded on `vendor/reth/crates/chainspec/src/spec.rs::ChainSpecBuilder`
pub fn build_sahara_chain_spec() -> ChainSpec {
    // Chain ID: large random value to avoid collisions with public chains.
    // Range chosen per EIP-3770 / chainlist.org conventions for private/L2 chains.
    // TODO: register on chainlist.org once finalized.
    const SAHARA_CHAIN_ID: u64 = 313_371;  // [PROPOSED] — placeholder, must be finalized

    let genesis = Genesis {
        // Empty genesis — no pre-funded accounts.
        // Allocations can be added later via with_genesis() on InMemoryStateDb
        // or by extending this genesis config.
        alloc: Default::default(),
        difficulty: U256::ZERO,  // PoS chain — no PoW difficulty
        gas_limit: 30_000_000,   // 30M gas limit — standard for modern L2s
        timestamp: 0,            // genesis timestamp; overridden at actual chain launch
        extra_data: Bytes::default(),
        nonce: 0,
        mix_hash: B256::ZERO,
        coinbase: Address::ZERO,
        ..Default::default()
    };

    ChainSpec::builder()
        .chain(Chain::from_id(SAHARA_CHAIN_ID))
        .genesis(genesis)
        .cancun_activated()  // activates all prior forks (frontier→…→paris→shanghai→cancun) at block/timestamp 0
        .build()
}

let evm_config = WhirlpoolEvmConfig::new(chain_spec);
```

<!-- continuation round 3: B-001 resolved -->
**Decision (round 3)**: `SAHARA_CHAIN_ID = 313_371` (placeholder, to be registered on chainlist.org). All hardforks through Cancun activated at genesis (block 0, timestamp 0). Empty genesis allocation. `build_sahara_chain_spec()` lives in `app-evm::config` module and is exported as a public function.

## Config defaults table

| Field | Type | Default | Source | Override path | Evidence |
|---|---|---|---|---|---|
| Chain ID | `u64` | `313_371` [PROPOSED] | `build_sahara_chain_spec()` | `ChainSpecBuilder::chain(Chain::from_id(...))` | `vendor/reth/crates/chainspec/src/spec.rs::ChainSpecBuilder` <!-- continuation round 3: B-001 resolved --> |
| Hardfork schedule | `ChainHardforks` | All through Cancun at genesis (block 0 / timestamp 0) | `build_sahara_chain_spec()` | `ChainSpecBuilder::cancun_activated()` | `vendor/reth/crates/chainspec/src/spec.rs::ChainSpecBuilder::cancun_activated` <!-- continuation round 3 --> |
| Genesis state | `Genesis` | Empty (`alloc: Default::default()`, `difficulty: U256::ZERO`, `gas_limit: 30_000_000`) | `build_sahara_chain_spec()` | `ChainSpecBuilder::genesis()` | `alloy_genesis::Genesis` <!-- continuation round 3 --> |
| EVM factory | `EthEvmFactory` | `EthEvmFactory` (ZST) | Hardcoded | — | `alloy_evm::EthEvmFactory` |
| Receipt builder | `RethReceiptBuilder` | `RethReceiptBuilder` (ZST) | Hardcoded | — | `vendor/reth/crates/ethereum/evm/src/receipt.rs::RethReceiptBuilder` |

## Provider interfaces & swap points

| Interface | Trait | Default impl | Swap point |
|---|---|---|---|
| EVM configuration | `ConfigureEvm` (vendor) | `WhirlpoolEvmConfig` | Replace struct to change EVM behavior |
| EVM factory | `EvmFactory` (vendor) | `EthEvmFactory` | Generic param on `WhirlpoolEvmConfig` for custom precompiles |
| Receipt building | `ReceiptBuilder` (vendor) | `RethReceiptBuilder` | Generic param on `EthBlockExecutorFactory` |
| Block assembly | `BlockAssembler` (vendor) | `EthBlockAssembler` | Associated type on `ConfigureEvm` |
| State database | `Database` (revm) | `InMemoryStateDb` from `state` crate [PROPOSED] | Generic `DB` param on `EvmApplication` — swap `InMemoryStateDb` for persistent backend | <!-- continuation round 2: B-002 resolved -->

## Feature flags & cfg

[PROPOSED]:
- `std` (default): enables rayon parallelism, full std features
- `test-utils`: mock state providers, test chain specs

## SemVer & stability

UNKNOWN — pre-1.0 workspace.

## Primary flows

### 1. Propose block [PROPOSED]
```pseudo
EvmApplication::propose(parent, height)
  1. Construct NextBlockEnvAttributes { timestamp: now(), gas_limit, ... }
  2. evm_config.builder_for_next_block(state, parent_header, attrs)
     → creates BasicBlockBuilder with Executor + Assembler
  3. For each pending tx: builder.execute_transaction(tx)
     → EvmFactory::create_evm → execute tx → accumulate receipts
  4. builder.finish(state_provider)
     → compute state_root via trie
     → BlockAssembler::assemble_block(input)
     → return BlockBuilderOutcome { block, execution_result, hashed_state, trie_updates }
  5. Convert to (EvmBlock, ExecutionResult)
```

### 2. Verify block [PROPOSED]
```pseudo
EvmApplication::verify(parent, block)
  1. evm_config.evm_env(block_header) → EvmEnv
  2. evm_config.context_for_block(block) → EthBlockExecutionCtx
  3. evm_config.create_executor(State::new(db)) → BasicBlockExecutor
  4. executor.execute_one(&evm_env, &ctx, block)
     → executes all txs, produces BlockExecutionResult
  5. Compute state root from executor.into_state()
  6. Compare state_root vs block.state_root
  7. Return ExecutionResult or StateRootMismatch error
```

### 3. WhirlpoolEvmConfig construction [PROPOSED]
```pseudo
WhirlpoolEvmConfig::new(chain_spec)
  1. Create EthEvmFactory (ZST)
  2. Create RethReceiptBuilder (ZST)
  3. Create EthBlockExecutorFactory::new(receipt_builder, chain_spec.clone(), evm_factory)
  4. Create EthBlockAssembler::new(chain_spec.clone())
  5. Return WhirlpoolEvmConfig { executor_factory, block_assembler }
```

## API omissions report

- **Transaction pool interface**: Not defined. `EvmApplication::propose()` needs a `pending_transactions()` source. Out of scope — recommend defining a `TxSource` trait in `app` crate.
- **State persistence**: `EvmApplication` does not persist state changes after execution via the `Application` trait return value. The `ExecutionResult` returned is a summary (hashes + gas). The full `BundleState` (state diff) is held internally by `EvmApplication` and must be committed to the state DB upon finalization. The node binary's `EventSink` handler triggers this commitment by calling a dedicated `commit_state()` method on `EvmApplication` (or by re-executing the finalized block against the state DB). See architecture/consensus-app-bridge.md for the full flow.
- **State provider factory**: `EvmApplication` takes a single `DB` but real execution needs state at specific block heights. A `StateProviderFactory` (as in reth) may be needed.

<!-- continuation round 2: B-002 state database resolved -->

- **State database resolved**: `EvmApplication<DB>` is instantiated with `DB = InMemoryStateDb` from the `state` crate. The `state` crate provides `revm::Database` + `Clone` impl, `commit(&BundleState)` for state commitment, and `state_root() -> B256` for block header computation. See `state/README.md` and `wiring/state-storage.md` for full contract.
- **Updated propose/verify flow**: After execution, `EvmApplication` calls `state_db.commit(&bundle)` then `state_db.state_root()` to get the new root. During verify, state is cloned before execution; if roots match the clone is promoted to canonical; if not, the clone is discarded.
- **Genesis initialization**: `InMemoryStateDb::with_genesis(alloc)` called at node startup to populate initial state. Depends on ChainSpec resolution (B-001).

## Open questions / TODOs

- ~~BLOCKER: Chain spec (chain ID, genesis, hardforks)~~ — Resolved (round 3). `SAHARA_CHAIN_ID = 313_371`, all hardforks through Cancun at genesis, empty allocation. See `build_sahara_chain_spec()` in Config schema section.
- ~~BLOCKER: State database~~ — Resolved (round 2). `InMemoryStateDb` from `state` crate provides `Database + Clone`. See `state/README.md`.
- BLOCKER: Transaction source — how `propose()` gets pending txs.
- UNKNOWN: Whether `WhirlpoolEvmConfig` should diverge from `EthEvmConfig` initially or start as a thin wrapper.
- UNKNOWN: Custom precompile requirements (if any, `EvmFactory` needs customization).
- UNKNOWN: Whether `EvmBlock` ↔ reth `Block` conversion is lossless or lossy.
