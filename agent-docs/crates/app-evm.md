# app-evm

## Purpose
Pure EVM configuration and execution integration for Whirlpool applications.
Genesis `ChainSpec` construction now shares the native-token hard cap from the `native-token` crate.
Genesis validator-registry encoding/decoding is shared from the `validators` crate.
Custom Whirlpool precompiles are now injected through the `evm-precompiles` crate.

## Interface/Implementation Split
- Interface module: `crates/evm/app/src/traits.rs`
  - `StateProvider`
- Implementation modules:
  - `crates/evm/app/src/config.rs`
  - `crates/evm/app/src/executor.rs`
  - `crates/evm/app/src/error.rs`

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
`EvmApplication` is EVM-only and now owns the canonical EVM transaction decode/recovery helpers used by its executor.
The executor uses `.map_err(Into::into)` on all `StateProvider` calls to convert into `EvmAppError`.

## EIP-1559 Base Fee
`EvmApplication::propose()` uses `calc_next_block_base_fee` from `reth-primitives-traits` to compute the `base_fee_per_gas` for each new block based on the parent block's gas usage and base fee. Genesis base fee defaults to 1 gwei (1_000_000_000).

## Fee Routing
- `DEFAULT_PROPOSER_FEE_RECIPIENT`: legacy fallback used only when no validator fee-recipient mapping exists in genesis.
- `VALIDATOR_FEE_RECIPIENTS_REGISTRY`: fixed genesis account whose storage maps validator ed25519 public keys to configured EVM fee-recipient addresses.
- `SIMPLEX_VALIDATORS_REGISTRY` (from `validators` crate): fixed genesis account whose storage encodes the ordered simplex validator list `{consensus_pubkey, ethereum_address}`.
- `COMMUNITY_POOL_ADDRESS` (from `community-pool` crate): fixed account credited with each block's burned amount.
- `EvmApplication::propose_evm_transactions()` resolves the local proposer's fee recipient from the genesis registry, commits the execution bundle, then credits the community pool by `gas_used * base_fee_per_gas` before computing the block state root.
- `EvmApplication::verify_evm_transactions()` validates the block-carried proposer recipient against the genesis registry (when present) before replaying execution and burned-fee credit.

## Canonical Imports
- `app_evm::traits::StateProvider`
- `app_evm::build_sahara_chain_spec` / `app_evm::build_sahara_chain_spec_with_alloc`
- `app_evm::build_sahara_chain_spec_with_alloc_and_fee_recipients`
- `app_evm::build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators`
- `app_evm::try_build_sahara_chain_spec*`
- `app_evm::try_simplex_validators_from_chain_spec`
- `app_evm::DEFAULT_PROPOSER_FEE_RECIPIENT`
- `app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY`
- `validators::SIMPLEX_VALIDATORS_REGISTRY`
- `community_pool::COMMUNITY_POOL_ADDRESS`
- `native_token::validate_genesis_alloc`
- `state::traits::StateDb` (interface trait)
- `state_reth::RethStateDb` (persistent implementation)
- `state_memory::InMemoryStateDb` (test code only)

## Key Types
- `WhirlpoolEvmConfig`: wrapper for EVM configuration. Reads genesis fee-recipient and simplex-validator registries, tracks the local proposer public key, resolves proposer fee recipients for proposal/verification, and injects precompiles with the decoded ordered validator list.
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
- `build_sahara_chain_spec_with_alloc_and_fee_recipients_and_validators(...) -> Arc<ChainSpec>`: builds a Sahara chain spec with fee-recipient registry data plus ordered simplex-validator registry entries.
- `try_simplex_validators_from_chain_spec(...) -> Result<Vec<ValidatorEntry>, ValidatorRegistryError>`: decodes ordered simplex-validator entries from genesis alloc storage.
- `try_build_sahara_chain_spec* -> Result<ChainSpec, NativeTokenError>`: fallible constructors that reject over-cap genesis allocs before building the spec.

## Precompile Wiring
- `crates/evm/app/src/config.rs` now composes `EthEvmConfig<ChainSpec, WhirlpoolEvmFactory>` internally.
- `WhirlpoolEvmConfig::evm_with_env(...)` injects `evm_precompiles::whirlpool_precompiles_with_validators(spec, decoded_simplex_validators)` through `EthEvmBuilder`.
- Proposal and verification still use the unchanged builder path in `crates/evm/app/src/executor.rs`; the precompile registry is attached at config/factory level rather than executor special-casing.
- `crates/evm/app/src/executor.rs` now includes a regression test that verifies block replay succeeds for a transaction that reaches a precompile through a small forwarding contract.

## Native-Token Cap
- `native-token` is the canonical source of the 10 billion Sahara hard cap.
- `build_sahara_chain_spec*` validates `Genesis.alloc` balances against that cap after fee-recipient registry storage is injected.
- The fee-recipient registry account contributes zero token balance; only account balances count toward total supply.

## Status
Active. This crate remains the pure EVM execution layer, now also owning the canonical EVM decode path and the config seam that injects Whirlpool custom precompiles while `app-composite`/`tx-dispatch` handle mixed mem routing.
