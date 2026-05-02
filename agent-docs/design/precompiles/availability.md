# Precompile Availability

## Answer
Whirlpool precompiles do not require an on-chain deployment transaction. They are available as soon as a node starts with the Whirlpool EVM configuration.

## Why
The canonical runtime path injects the Whirlpool precompile map through `WhirlpoolEvmConfig::evm_with_env(...)` and `EthEvmBuilder`; see `crates/precompiles/evm/src/lib.rs`, `crates/precompiles/evm/src/factory_api.rs`, and `crates/app/evm-execution/src/config/mod.rs`. Validator-aware constructor names such as `whirlpool_precompiles_with_validators(spec, validators)` and `WhirlpoolEvmFactory::with_validators(...)` remain compatibility entrypoints, but the validators precompile reads `SIMPLEX_VALIDATORS_REGISTRY` from runtime EVM state rather than a captured constructor snapshot.

## Lifecycle model
- Genesis alloc and chain spec decide normal account/code/storage state; see the `config/` module tree under `crates/app/evm-execution/src/config/` together with `crates/app/evm-execution/src/config/mod.rs`.
- Precompiles are separate from genesis account deployment. They are runtime-registered execution hooks.
- Because the precompile map is attached when the EVM instance is created, the feature is present from the first executed block and in fresh `eth_call` contexts; see `crates/app/evm-execution/src/config/mod.rs` and `crates/precompiles/evm/src/lib.rs`.

## Why not deploy them like contracts
- Deployment would make availability depend on chain history instead of node configuration.
- Deployment would add a bootstrapping problem for verification and RPC: historical replay and `eth_call` must see the same feature set as block production.
- Runtime registration keeps proposal and verification symmetric; see `crates/app/evm-execution/src/block_pipeline/mod.rs` together with `crates/app/evm-execution/src/block_pipeline/propose.rs` and `crates/app/evm-execution/src/block_pipeline/verify.rs`.

## Operational implication
A custom precompile exists only on nodes running this Whirlpool binary/config seam. A node that does not install the Whirlpool registry will not expose the custom address.
