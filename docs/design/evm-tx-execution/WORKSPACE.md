# Workspace — EVM Transaction Execution

## Workspace Members

```
crates/
├── app/                  # Application traits + EvmBlock type (stable, no changes)
├── app-evm/              # EVM execution wrapper (PRIMARY CHANGE TARGET)
├── state/                # In-memory state DB (SECONDARY CHANGE TARGET)
├── consensus/            # Consensus traits (out of scope)
├── consensus-simplex/    # Simplex BFT adapter (out of scope)
├── p2p/                  # P2P traits (out of scope)
├── p2p-commonware/       # P2P adapter (out of scope)
├── whirlpool-node/       # Node binary (deferred to Sub-Intent 3)
└── whirlpool-node-simple/ # Simple node binary (out of scope)
```

## Crate Dependency Graph (in-scope)

```
whirlpool-node (out of scope, consumes)
    ├── app-evm ← PRIMARY
    │   ├── app (traits, types)
    │   ├── state (InMemoryStateDb)
    │   ├── consensus (Block trait)
    │   ├── reth-evm (ConfigureEvm, BlockBuilder, BlockExecutor)
    │   ├── reth-evm-ethereum (EthEvmConfig, EthBlockAssembler)
    │   ├── reth-revm (revm integration)
    │   ├── reth-execution-types (BundleState, ExecutionOutcome)
    │   ├── reth-execution-errors (BlockExecutionError)
    │   ├── reth-primitives-traits (Header, SealedHeader)
    │   ├── reth-chainspec (ChainSpec)
    │   ├── reth-ethereum-primitives (EthPrimitives)
    │   ├── alloy-primitives, alloy-consensus, alloy-trie, alloy-eips
    │   └── revm
    └── state ← SECONDARY
        ├── alloy-primitives, alloy-genesis
        └── revm (Database, DatabaseRef, BundleState)
```

## Build / Read Entrypoints

| Purpose | Path |
|---|---|
| EVM execution logic | `crates/app-evm/src/executor.rs` |
| EVM configuration | `crates/app-evm/src/config.rs` |
| EVM errors | `crates/app-evm/src/error.rs` |
| State database | `crates/state/src/db.rs` |
| Application traits | `crates/app/src/traits.rs` |
| Block/result types | `crates/app/src/types.rs` |
| reth BlockBuilder API | `vendor/reth/crates/evm/evm/src/execute.rs` |
| reth ConfigureEvm | `vendor/reth/crates/evm/evm/src/lib.rs` |
| reth EthBlockAssembler | `vendor/reth/crates/ethereum/evm/src/build.rs` |
