# app-evm

## Purpose
Pure EVM runtime/config/execution crate for Whirlpool.

## Location
`crates/app/execute/evm/app/`

## Ownership Boundary
`app-evm` now owns EVM behavior, not Sahara chain-spec construction.

### Owns
- `WhirlpoolEvmConfig`
- `EvmApplication`
- EVM tx decode/recovery helpers
- Fee-recipient runtime behavior and validation
- Constants:
  - `DEFAULT_PROPOSER_FEE_RECIPIENT`
  - `VALIDATOR_FEE_RECIPIENTS_REGISTRY`

### Does not own anymore
- `SAHARA_CHAIN_ID`
- `build_sahara_chain_spec*`
- `try_build_sahara_chain_spec*`
- public `try_simplex_validators_from_chain_spec`

Those live in `chainspec`.

## Key Runtime Notes
- `WhirlpoolEvmConfig` still derives proposer fee recipients from genesis storage at `VALIDATOR_FEE_RECIPIENTS_REGISTRY`.
- Precompile injection remains in `WhirlpoolEvmConfig::evm_with_env(...)` via `evm_precompiles::whirlpool_precompiles_with_validators(...)`.
- Epoch-boundary deterministic system-call handling now lives in `epoch_boundary.rs` and is shared between propose/verify paths.
- Fee routing behavior:
  - burned base fees are credited to `evm_precompiles::COMMUNITY_POOL_ADDRESS`
  - priority fees are credited to `evm_precompiles::FEE_POOL_PRECOMPILE_ADDRESS`
  - per-recipient claimable balances are stored in fee-pool precompile storage (`claimable_balance_slot`)
  - proposers withdraw later via fee-pool precompile `withdraw()`
- `suggested_fee_recipient` in execution env is now forced to fee-pool address; block header `proposer_fee_recipient` remains proposer metadata.
- `state::StateDb` writes are now used for claim-ledger slot updates via `insert_storage`.
- Block gas accounting now uses the final cumulative receipt gas (last receipt), avoiding sum-of-cumulative overcounting.
- On boundary heights, propose executes `advanceEpoch` as an internal system call before user tx execution; no synthetic boundary tx bytes are added to `block.transactions`.
- Reserved epoch namespace tx bytes in the user payload are treated as invalid protocol artifacts: propose excludes them and verify rejects blocks that contain them.

## Canonical Imports
- `app_evm::traits::StateProvider`
- `app_evm::WhirlpoolEvmConfig`
- `app_evm::EvmApplication`
- `app_evm::decode_evm_transaction`
- `app_evm::decode_evm_transactions`
- `app_evm::ProposedEvmPayload`
- `app_evm::DEFAULT_PROPOSER_FEE_RECIPIENT`
- `app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY`
- `chainspec::build_sahara_chain_spec*`
- `chainspec::try_build_sahara_chain_spec*`
- `chainspec::SAHARA_CHAIN_ID`
- `chainspec::try_simplex_validators_from_chain_spec`
