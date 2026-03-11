# app-evm

## Purpose
EVM configuration and execution integration for Whirlpool applications.

## Interface/Implementation Split
- Interface module: `crates/app-evm/src/traits.rs`
  - `StateProvider`
- Implementation modules:
  - `crates/app-evm/src/config.rs`
  - `crates/app-evm/src/executor.rs`
  - `crates/app-evm/src/error.rs`

## Trait Boundary
- `StateProvider` is now defined in `app_evm::traits`.
- `type Error`: fallible operations associated error type.
- Blanket impl delegates to `state::traits::StateDb`.
- `state_root` and `commit` return `Result<_, Self::Error>`.

## Error Handling
- `EvmAppError::State(String)` — wraps database and state-related errors.
- `From<Infallible>`: trivial conversion for `InMemoryStateDb`.
- `From<state::StateError>`: generic state error conversion.
- `From<state_reth::RethStateError>`: persistent state error conversion.

## Execution Implementation
The `EvmApplication` executor uses `.map_err(Into::into)` on all `StateProvider` calls to convert into `EvmAppError`.

## EIP-1559 Base Fee
`EvmApplication::propose()` uses `calc_next_block_base_fee` from `reth-primitives-traits` to compute the `base_fee_per_gas` for each new block based on the parent block's gas usage and base fee. Genesis base fee defaults to 1 gwei (1_000_000_000).

## Canonical Imports
- `app_evm::traits::StateProvider`
- `app_evm::build_sahara_chain_spec` / `app_evm::build_sahara_chain_spec_with_alloc`
- `state::traits::StateDb` (interface trait)
- `state_reth::RethStateDb` (persistent implementation)
- `state_memory::InMemoryStateDb` (test code only)

## Key Types
- `WhirlpoolEvmConfig`: wrapper for EVM configuration.
- `EvmApplication`: application implementation that executes EVM blocks.
  - `pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>`: temporary storage for receipts between execution and persistence.
  - `last_proposed: Arc<Mutex<Option<(u64, EvmBlock, ExecutionResult, Vec<Receipt>)>>>`: cache for the most recent proposal at a given height; prevents duplicate mempool drain when simplex calls `propose()` multiple times for the same height.
  - `store_finalized_block(&self, block: &EvmBlock, storage: &dyn BlockStorage) -> Result<(), EvmAppError>`: persists block and receipts.
- `EvmAppError`: EVM application error type.

## Public Functions
- `build_header_from_evm_block(block: &EvmBlock) -> Header`: converts internal block type to Ethereum header. Sets `excess_blob_gas: Some(0)` and `blob_gas_used: Some(0)` for post-Cancun compatibility.
- `build_sahara_chain_spec() -> Arc<ChainSpec>`: builds the standard Sahara chain spec (chain ID 313371, Cancun-activated).
- `build_sahara_chain_spec_with_alloc(alloc: BTreeMap<Address, GenesisAccount>) -> Arc<ChainSpec>`: builds a Sahara chain spec with pre-funded genesis accounts. Used for integration tests requiring funded accounts.

## Status
Complete. Traits are stable and execution/persistence logic is implemented.
