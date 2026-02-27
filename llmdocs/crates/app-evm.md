# app-evm

## Purpose
EVM configuration and execution logic, integrating reth's EVM implementation into the Whirlpool application layer.

## Key Types
- `WhirlpoolEvmConfig`: Newtype wrapper around `reth_evm_ethereum::EthEvmConfig`, implementing the `ConfigureEvm` trait.
- `EvmApplication`: Struct that will implement the `Application` trait, holding the EVM configuration, state database, and transaction source.
- `EvmAppError`: Error type for EVM execution, with conversion to `ApplicationError`.

## Key Functions
- `build_sahara_chain_spec()`: Constructs the Sahara chain specification with Chain ID 313371, 30M gas limit, and Cancun hardforks activated.
- `WhirlpoolEvmConfig::new(chain_spec)`: Initializes the EVM configuration with a given chain specification.
- `build_header_from_evm_block(block)`: Converts an `EvmBlock` into an Ethereum `Header`.
- `build_sealed_header(block)`: Builds a sealed Ethereum header from an `EvmBlock` by computing its hash.

## Dependencies
- `reth-evm`: Core Reth EVM traits and configuration.
- `reth-chainspec`: Chain specification and hardfork configuration.
- `alloy-primitives`: Ethereum-compatible primitive types.
- `app`: Local application traits and types.

## Status
Partial. EVM configuration and header conversion helpers are complete. `EvmApplication` struct exists but its `Application` trait implementation is pending/incomplete.
