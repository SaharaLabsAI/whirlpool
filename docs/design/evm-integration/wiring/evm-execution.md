# Wiring: EVM Execution

## Wiring matrix

| Capability | Owning crate | Upstream deps | Downstream consumers | Public types | Config types | Config defaults | Trait interface | Default provider | Alternate providers | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| EVM configuration | `app-evm` [PROPOSED] | `reth-evm::ConfigureEvm`, `reth-chainspec::ChainSpec` | `app-evm::EvmApplication` [PROPOSED] | `WhirlpoolEvmConfig` [PROPOSED] | `ChainSpec` (vendor) | Sahara chain ID `313_371` + Cancun-activated genesis [PROPOSED] <!-- continuation round 3: B-001 resolved --> | `ConfigureEvm` (vendor) | `WhirlpoolEvmConfig` [PROPOSED] | `EthEvmConfig` (vendor, for testing) | `vendor/reth/crates/evm/evm/src/lib.rs::ConfigureEvm` |
| EVM env construction | `app-evm` [PROPOSED] | `reth-evm`, `reth-chainspec` | Block executor, block builder | `EvmEnv` (vendor) | `NextBlockEnvAttributes` (vendor) | — | `ConfigureEvm::evm_env()`, `::next_evm_env()` | `WhirlpoolEvmConfig` [PROPOSED] | — | `vendor/reth/crates/ethereum/evm/src/lib.rs::EthEvmConfig::evm_env` |
| Block execution | `app-evm` [PROPOSED] | `alloy-evm::BlockExecutorFactory`, `alloy-evm::EthBlockExecutorFactory` | `Application::verify()` [PROPOSED] | `BlockExecutionResult<Receipt>` (alloy-evm) | — | — | `Executor::execute_one()` (alloy-evm) | `EthBlockExecutorFactory` (alloy-evm, re-exported by reth-evm) | Custom `BlockExecutorFactory` impl | `vendor/reth/crates/evm/evm/src/execute.rs::Executor` |
| Block assembly | `app-evm` [PROPOSED] | `alloy-evm::BlockAssembler`, `reth-evm-ethereum::EthBlockAssembler` | `Application::propose()` [PROPOSED] | `Block` (reth primitives) | — | — | `BlockAssembler::assemble_block()` (alloy-evm) | `EthBlockAssembler` (reth-evm-ethereum) | Custom `BlockAssembler` impl | `vendor/reth/crates/ethereum/evm/src/build.rs::EthBlockAssembler` |
| Block building (combined execute+assemble) | `app-evm` [PROPOSED] | `reth-evm::BlockBuilder` | `Application::propose()` [PROPOSED] | `BlockBuilderOutcome` (vendor) | — | — | `BlockBuilder::execute_transaction()`, `::finish()` (vendor) | `BasicBlockBuilder` (vendor) | — | `vendor/reth/crates/evm/evm/src/execute.rs::BlockBuilder` |
| Receipt building | `app-evm` [PROPOSED] | `reth-evm-ethereum::RethReceiptBuilder` | Block executor | `Receipt` (reth primitives) | — | — | `ReceiptBuilder::build_receipt()` (vendor) | `RethReceiptBuilder` (vendor) | Custom `ReceiptBuilder` impl | `vendor/reth/crates/ethereum/evm/src/receipt.rs::RethReceiptBuilder` |
| EVM factory | `app-evm` [PROPOSED] | `alloy-evm::EthEvmFactory` | Block executor factory | `EthEvm` (vendor) | — | — | `EvmFactory::create_evm()` (vendor) | `EthEvmFactory` (vendor) | Custom `EvmFactory` (for custom precompiles) | `alloy_evm::EthEvmFactory` |

## Blockers

- ~~**ChainSpec**~~: **Resolved (round 3)**. `build_sahara_chain_spec()` constructs a `ChainSpec` with chain ID `313_371`, all hardforks through Cancun at genesis (block 0 / timestamp 0), empty genesis allocation, 30M gas limit. See `app-evm/README.md` for full construction code. <!-- continuation round 3: B-001 resolved -->
- ~~**State DB**~~: **Resolved (round 2)**. `Executor<DB>` and `BlockBuilder` use `DB = InMemoryStateDb` from `state` crate. See `state/README.md`. <!-- continuation round 2: B-002 resolved -->
