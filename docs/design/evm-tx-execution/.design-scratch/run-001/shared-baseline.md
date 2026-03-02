# Shared Baseline — EVM Transaction Execution

## Design Intent

Replace empty-block stubs in `app-evm::EvmApplication::propose()` and `verify()` with real EVM transaction execution via reth's block executor. Commit resulting state changes to `state::InMemoryStateDb`. Produce correct `state_root`, `tx_root`, `receipts_root`, `gas_used` in `app::EvmBlock`.

## In-Scope Crates

| Crate | Role | Current State |
|---|---|---|
| `app-evm` | EVM config + execution wrapper | Empty-block stubs in propose/verify |
| `state` | In-memory EVM state DB | commit(BundleState) implemented, flat state_root |

## Out-of-Scope (boundary assumptions)

- `app` — trait definitions (Application, TxSource, EvmBlock) are stable; no changes expected
- `consensus` / `consensus-simplex` — block finalization trigger unchanged
- `whirlpool-node` — wiring changes deferred to Sub-Intent 3
- `p2p` / `p2p-commonware` — not involved
- Transaction source implementation — deferred to Sub-Intent 2 (TxSource trait already defined)
- MPT state root — out of scope (flat keccak256 stays)
- Disk persistence — out of scope

## Current Architecture (Grounded)

### app-evm::EvmApplication<DB>

```rust
// executor.rs
pub struct EvmApplication<DB> {
    evm_config: WhirlpoolEvmConfig,   // wraps EthEvmConfig
    state_db: Arc<RwLock<DB>>,         // InMemoryStateDb behind lock
    tx_source: Arc<dyn TxSource>,      // NoopTxSource currently
}
```

- `propose(parent: &EvmBlock, height: u64)` → Returns empty EvmBlock (TODO: execute txs)
- `verify(parent: &EvmBlock, block: &EvmBlock)` → Checks state_root match only
- `genesis()` → Returns empty EvmBlock with current state_root

### app-evm::WhirlpoolEvmConfig

- Wraps `EthEvmConfig`, delegates `ConfigureEvm` methods
- Chain spec: chain_id=313371, gas_limit=30M, Cancun hardfork
- Provides `block_executor_factory()`, `block_assembler()`, `evm_env()`, etc.

### state::InMemoryStateDb

- HashMap-based: accounts, bytecodes, block_hashes
- `commit(&BundleState)` — processes account changes (create/update/destroy), storage, contracts
- `state_root()` — flat keccak256 hash of all account data (NOT Merkle Patricia Trie)
- Implements `revm::Database` + `DatabaseRef`
- `with_genesis(alloc)` — initializes from genesis allocation

### app::Application Trait

```rust
pub trait Application: Send + Sync + 'static {
    type Block: Block;
    fn genesis(&self) -> Self::Block;
    fn propose(&self, parent: &Self::Block, height: u64) -> Result<Self::Block, ApplicationError>;
    fn verify(&self, parent: &Self::Block, block: &Self::Block) -> Result<(), ApplicationError>;
}
```

### app::EvmBlock

Fields: height, parent_id, state_root, tx_root, receipts_root, gas_used, timestamp, transactions (Vec<Vec<u8>>)

### app::ExecutionResult

Fields: state_root, receipts_root, gas_used, receipt_count

## Key Dependencies (Grounded)

- reth-evm, reth-evm-ethereum (block execution)
- reth-revm (EVM integration)
- reth-execution-types (ExecutionOutcome, BundleState)
- reth-execution-errors (BlockExecutionError)
- reth-primitives-traits (block primitives)
- reth-chainspec (chain specification)
- alloy-primitives 1.5.0, alloy-consensus 1.4.3, alloy-trie 0.9
- revm 34

## Known Blockers from Deprecated Design (reference)

- B-001: propose() produces empty blocks
- B-002: verify() doesn't re-execute transactions
- B-004: Finalize→commit ownership unclear (decision-gap)
- B-005: Snapshot/rollback orchestration undefined
