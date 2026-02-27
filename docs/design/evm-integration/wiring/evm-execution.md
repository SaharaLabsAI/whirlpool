# Wiring: EVM Execution

## Wiring matrix

| Capability | Owning crate | Upstream deps | Downstream consumers | Public types | Config types | Config defaults | Trait interface | Default provider | Alternate providers | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| EVM configuration | `app-evm` [PROPOSED] | `reth-evm::ConfigureEvm`, `reth-chainspec::ChainSpec` | `app-evm::EvmApplication` [PROPOSED] | `WhirlpoolEvmConfig` [PROPOSED] | `ChainSpec` (vendor) | Sahara chain ID + mainnet params [PROPOSED] | `ConfigureEvm` (vendor) | `WhirlpoolEvmConfig` [PROPOSED] | `EthEvmConfig` (vendor, for testing) | `vendor/reth/crates/evm/evm/src/lib.rs::ConfigureEvm` |
| EVM env construction | `app-evm` [PROPOSED] | `reth-evm`, `reth-chainspec` | Block executor, block builder | `EvmEnv` (vendor) | `NextBlockEnvAttributes` (vendor) | — | `ConfigureEvm::evm_env()`, `::next_evm_env()` | `WhirlpoolEvmConfig` [PROPOSED] | — | `vendor/reth/crates/ethereum/evm/src/lib.rs::EthEvmConfig::evm_env` |
| Block execution | `app-evm` [PROPOSED] | `alloy-evm::BlockExecutorFactory`, `alloy-evm::EthBlockExecutorFactory` | `Application::verify()` [PROPOSED] | `BlockExecutionResult<Receipt>` (alloy-evm) | — | — | `Executor::execute_one()` (alloy-evm) | `EthBlockExecutorFactory` (alloy-evm, re-exported by reth-evm) | Custom `BlockExecutorFactory` impl | `vendor/reth/crates/evm/evm/src/execute.rs::Executor` |
| Block assembly | `app-evm` [PROPOSED] | `alloy-evm::BlockAssembler`, `reth-evm-ethereum::EthBlockAssembler` | `Application::propose()` [PROPOSED] | `Block` (reth primitives) | — | — | `BlockAssembler::assemble_block()` (alloy-evm) | `EthBlockAssembler` (reth-evm-ethereum) | Custom `BlockAssembler` impl | `vendor/reth/crates/ethereum/evm/src/build.rs::EthBlockAssembler` |
| Block building (combined execute+assemble) | `app-evm` [PROPOSED] | `reth-evm::BlockBuilder` | `Application::propose()` [PROPOSED] | `BlockBuilderOutcome` (vendor) | — | — | `BlockBuilder::execute_transaction()`, `::finish()` (vendor) | `BasicBlockBuilder` (vendor) | — | `vendor/reth/crates/evm/evm/src/execute.rs::BlockBuilder` |
| Receipt building | `app-evm` [PROPOSED] | `reth-evm-ethereum::RethReceiptBuilder` | Block executor | `Receipt` (reth primitives) | — | — | `ReceiptBuilder::build_receipt()` (vendor) | `RethReceiptBuilder` (vendor) | Custom `ReceiptBuilder` impl | `vendor/reth/crates/ethereum/evm/src/receipt.rs::RethReceiptBuilder` |
| EVM factory | `app-evm` [PROPOSED] | `alloy-evm::EthEvmFactory` | Block executor factory | `EthEvm` (vendor) | — | — | `EvmFactory::create_evm()` (vendor) | `EthEvmFactory` (vendor) | Custom `EvmFactory` (for custom precompiles) | `alloy_evm::EthEvmFactory` |

## Blockers

- **ChainSpec**: `WhirlpoolEvmConfig` needs `Arc<ChainSpec>`. Must decide: reth `ChainSpec::mainnet()` with overridden chain ID, or custom chain spec construction. Inspected: `vendor/reth/crates/ethereum/evm/src/lib.rs::EthEvmConfig::ethereum(chain_spec)` — takes any `C: EthChainSpec`.
- **State DB**: `Executor<DB>` and `BlockBuilder` require `DB: Database`. `app-evm` must either (a) be generic over `DB`, (b) define a concrete DB, or (c) take `Box<dyn StateProvider>`. Inspected: `vendor/reth/crates/revm/src/database.rs` — provides `StateProviderDatabase<SP>` as adapter.
