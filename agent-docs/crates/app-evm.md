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
- Fee routing behavior remains unchanged:
  - burned base fees are credited to `evm_precompiles::COMMUNITY_POOL_ADDRESS`
  - proposer priority fees accrue to proposer fee recipient.

## Canonical Imports
- `app_evm::traits::StateProvider`
- `app_evm::WhirlpoolEvmConfig`
- `app_evm::EvmApplication`
- `app_evm::decode_evm_transaction`
- `app_evm::decode_evm_transactions`
- `app_evm::DEFAULT_PROPOSER_FEE_RECIPIENT`
- `app_evm::VALIDATOR_FEE_RECIPIENTS_REGISTRY`
- `chainspec::build_sahara_chain_spec*`
- `chainspec::try_build_sahara_chain_spec*`
- `chainspec::SAHARA_CHAIN_ID`
- `chainspec::try_simplex_validators_from_chain_spec`
