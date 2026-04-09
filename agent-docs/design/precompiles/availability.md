# Precompile Availability

## Answer
Whirlpool precompiles do not require an on-chain deployment transaction. They are available as soon as a node starts with the Whirlpool EVM configuration.

## Why
`whirlpool_precompiles(spec)` builds a `PrecompilesMap` from the built-in precompiles plus Whirlpool-owned entries in `crates/app/evm/precompiles/src/lib.rs:94-124`. `WhirlpoolEvmFactory` and `WhirlpoolEvmConfig::evm_with_env(...)` inject that map directly into `EthEvmBuilder` in `crates/app/evm/precompiles/src/lib.rs:142-163` and `crates/app/evm/app/src/config.rs:195-199`.

## Lifecycle model
- Genesis alloc and chain spec decide normal account/code/storage state; see `crates/app/evm/app/src/config.rs:29-94`.
- Precompiles are separate from genesis account deployment. They are runtime-registered execution hooks.
- Because the registry is attached when the EVM instance is created, the feature is present from the first executed block and in fresh `eth_call` contexts; see `crates/app/evm/app/src/config.rs:257-275`.

## Why not deploy them like contracts
- Deployment would make availability depend on chain history instead of node configuration.
- Deployment would add a bootstrapping problem for verification and RPC: historical replay and `eth_call` must see the same feature set as block production.
- Runtime registration keeps proposal and verification symmetric; see `crates/app/evm/app/src/executor.rs:761-800`.

## Operational implication
A custom precompile exists only on nodes running this Whirlpool binary/config seam. A node that does not install the Whirlpool registry will not expose the custom address.
