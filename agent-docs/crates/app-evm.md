# app-evm

## Purpose
Pure EVM configuration and execution integration for Whirlpool applications.
Genesis `ChainSpec` construction now shares the native-token hard cap from the `native-token` crate.

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
- `state_root`, `commit`, `get_account`, and `insert_account` return `Result<_, Self::Error>`.

## Error Handling
- `EvmAppError::State(String)` — wraps database and state-related errors.
- `From<Infallible>`: trivial conversion for `InMemoryStateDb`.
- `From<state::StateError>`: generic state error conversion.
- `From<state_reth::RethStateError>`: persistent state error conversion.

## Execution Implementation
`EvmApplication` is now EVM-only. Mixed mem/personality transaction routing moved out to `app-composite` + `tx-dispatch`.
The executor uses `.map_err(Into::into)` on all `StateProvider` calls to convert into `EvmAppError`.

## EIP-1559 Base Fee
`EvmApplication::propose()` uses `calc_next_block_base_fee` from `reth-primitives-traits` to compute the `base_fee_per_gas` for each new block based on the parent block's gas usage and base fee. Genesis base fee defaults to 1 gwei (1_000_000_000).

## Fee Routing
- `DEFAULT_PROPOSER_FEE_RECIPIENT`: legacy fallback used only when no validator fee-recipient mapping exists in genesis.
- `VALIDATOR_FEE_RECIPIENTS_REGISTRY`: fixed genesis account whose storage maps validator ed25519 public keys to configured EVM fee-recipient addresses.
- `COMMUNITY_POOL_ADDRESS` (from `community-pool` crate): fixed account credited with each block's burned amount.
- `EvmApplication::propose_evm_transactions()` resolves the local proposer's fee recipient from the genesis registry, commits the execution bundle, then credits the community pool by `gas_used * base_fee_per_gas` before computing the block state root.
- `EvmApplication::verify_evm_transactions()` validates the block-carried proposer recipient against the genesis registry (when present) before replaying execution and burned-fee credit.

## Canonical Imports
- `app_evm::traits::StateProvider`
- `app_evm::build_sahara_chain_spec` / `app_evm::build_sahara_chain_spec_with_alloc`
- `app_evm::build_sahara_chain_spec_with_alloc_and_fee_recipients`
- `app_evm::try_build_sahara_chain_spec*`
- `app_evm::DEFAULT_PROPOSER_FEE_RECIPIENT`
- `app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY`
- `community_pool::COMMUNITY_POOL_ADDRESS`
- `native_token::validate_genesis_alloc`
- `state::traits::StateDb` (interface trait)
- `state_reth::RethStateDb` (persistent implementation)
- `state_memory::InMemoryStateDb` (test code only)

## Key Types
- `WhirlpoolEvmConfig`: wrapper for EVM configuration. Reads the genesis validator->recipient registry, tracks the local proposer public key, and resolves proposer fee recipients for proposal/verification.
- `EvmApplication`: application implementation that executes EVM-only blocks.
  - `pending_receipts: Arc<Mutex<Option<Vec<Receipt>>>>`: temporary storage for receipts between execution and persistence.
  - `last_proposed: Arc<Mutex<Option<(u64, EvmBlock, ExecutionResult, Vec<Receipt>)>>>`: cache for the most recent proposal at a given height; prevents duplicate mempool drain when simplex calls `propose()` multiple times for the same height.
  - `propose_evm_transactions(&self, parent, raw_txs, timestamp) -> Result<ProposedEvmPayload, EvmAppError>`: executes a candidate EVM tx list and returns included txs plus execution artifacts.
  - `verify_evm_transactions(&self, parent, block, raw_txs) -> Result<ExecutionResult, EvmAppError>`: replays only the EVM subset of a block.
  - `store_finalized_block(&self, block: &EvmBlock, storage: &dyn BlockStorage) -> Result<(), EvmAppError>`: persists block and receipts.
- `ProposedEvmPayload`: result of EVM-only proposal execution, including included transactions, inclusion outcomes, proposer public key, proposer fee recipient, receipts, and execution result.
- `EvmAppError`: EVM application error type.

## Public Functions
- `build_header_from_evm_block(block: &EvmBlock) -> Header`: converts internal block type to Ethereum header. Stores proposer public key in `extra_data`, proposer fee recipient in `beneficiary`, and sets `excess_blob_gas: Some(0)` / `blob_gas_used: Some(0)` for post-Cancun compatibility.
- `build_sahara_chain_spec() -> Arc<ChainSpec>`: builds the standard Sahara chain spec (chain ID 313371, Cancun-activated).
- `build_sahara_chain_spec_with_alloc(alloc: BTreeMap<Address, GenesisAccount>) -> Arc<ChainSpec>`: builds a Sahara chain spec with pre-funded genesis accounts. Used for integration tests requiring funded accounts.
- `build_sahara_chain_spec_with_alloc_and_fee_recipients(...) -> Arc<ChainSpec>`: builds a Sahara chain spec with pre-funded accounts plus genesis storage mapping validator public keys to EVM fee-recipient addresses.
- `try_build_sahara_chain_spec* -> Result<ChainSpec, NativeTokenError>`: fallible constructors that reject over-cap genesis allocs before building the spec.

## Native-Token Cap
- `native-token` is the canonical source of the 10 billion Sahara hard cap.
- `build_sahara_chain_spec*` validates `Genesis.alloc` balances against that cap after fee-recipient registry storage is injected.
- The fee-recipient registry account contributes zero token balance; only account balances count toward total supply.

## Status
Active. This crate remains the pure EVM execution layer used directly by `app-composite`, now with genesis-governed validator fee-recipient routing plus deterministic proposer verification.
