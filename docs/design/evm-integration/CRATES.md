# Crate Index

## Workspace members

| Crate | Path | Kind | Purpose | Status |
|---|---|---|---|---|
| `consensus` | `crates/consensus/` | lib | Abstract consensus traits: `Block`, `ConsensusApp`, `ConsensusEngine`, `ConsensusEvent` | Grounded |
| `consensus-simplex` | `crates/consensus-simplex/` | lib | Simplex BFT adapter bridging `ConsensusApp` to commonware-consensus | Grounded |
| `p2p` | `crates/p2p/` | lib | Abstract P2P networking traits | Grounded |
| `p2p-commonware` | `crates/p2p-commonware/` | lib | Commonware P2P adapter | Grounded |
| `whirlpool-node` | `crates/whirlpool-node/` | bin+lib | Node binary, `EmptyBlockApp`, `EmptyBlock`, node config | Grounded |
| `app` | `crates/app/` | lib | [PROPOSED] Abstract application traits for EVM-aware block execution | Proposed |
| `app-evm` | `crates/app-evm/` | lib | [PROPOSED] Concrete EVM application using reth-evm | Proposed |
| `state` | `crates/state/` | lib | [PROPOSED] In-memory EVM state database: `revm::Database` impl, `BundleState` commitment, state root computation | Proposed (round 2) |

## Vendor crates consumed (read-only)

| Crate | Path | Provides | Used by |
|---|---|---|---|
| `reth-evm` | `vendor/reth/crates/evm/evm/` | `ConfigureEvm`, `BlockExecutorFactory`, `BlockAssembler`, `Executor`, `BlockBuilder` traits | `app-evm` [PROPOSED] |
| `reth-evm-ethereum` | `vendor/reth/crates/ethereum/evm/` | `EthEvmConfig` reference impl, `EthBlockAssembler`, `RethReceiptBuilder` | `app-evm` [PROPOSED] (reference pattern) |
| `reth-revm` | `vendor/reth/crates/revm/` | revm wrapper, `State<DB>`, database glue | `app-evm` [PROPOSED] |
| `reth-execution-types` | `vendor/reth/crates/evm/execution-types/` | `ExecutionOutcome`, `BlockExecutionOutput`, `Chain` | `app-evm` [PROPOSED] |
| `reth-execution-errors` | `vendor/reth/crates/evm/execution-errors/` | `BlockExecutionError`, `BlockValidationError` | `app-evm` [PROPOSED] |
| `reth-primitives-traits` | `vendor/reth/crates/primitives-traits/` | `NodePrimitives`, `Block`, `Header` trait definitions | `app-evm` [PROPOSED] |
| `reth-chainspec` | `vendor/reth/crates/chainspec/` | `ChainSpec`, `EthChainSpec` | `app-evm` [PROPOSED] |
| `commonware-consensus` | `vendor/commonware/consensus/` | Simplex consensus engine | `consensus-simplex` |
| `commonware-runtime` | `vendor/commonware/runtime/` | Async runtime abstraction | `whirlpool-node` |
| `revm` | (cargo dependency) | `Database`, `DatabaseCommit`, `DatabaseRef` traits, `CacheDB<DB>`, `State<DB>` builder, `BundleState` | `state` [PROPOSED], `app-evm` [PROPOSED] |
