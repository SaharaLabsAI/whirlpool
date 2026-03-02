# Workspace Map — EVM Block Production

## Build entrypoints

- `whirlpool-node` (binary): The main validator node that orchestrates block production.
- `app-evm` (library): Provides the EVM execution logic for block proposal and verification.
- `app` (library): Defines the core interfaces for block production (Application, TxSource).
- `state` (library): Provides the in-memory state database used by the node and EVM.

## Crate dependency graph

```text
whirlpool-node (bin)
├── app-evm (lib)
│   ├── app (lib)
│   │   └── consensus (lib)
│   ├── state (lib)
│   └── consensus (lib)
├── app (lib)
│   └── consensus (lib)
├── state (lib)
├── consensus-simplex (lib)
└── p2p-commonware (lib)
```

## Reading guide

1. **`app/traits.rs`**: Start here to understand the `Application` and `TxSource` interfaces that define how a consensus engine interacts with the state machine.
2. **`app-evm/executor.rs`**: Read this next to see how `EvmApplication` implements the `Application` trait and where transaction execution is integrated.
3. **`state/db.rs`**: Examine how `InMemoryStateDb` provides the underlying state storage and root hash calculation.
4. **`whirlpool-node/main.rs`**: Finally, see how all these components are wired together with the consensus engine in the main node entrypoint.

## Key configuration

- **Workspace**: The workspace uses `resolver = "2"` (Cargo.toml).
- **Features**: The node uses `tokio` with `full` features for the runtime (whirlpool-node/Cargo.toml).
- **Chain ID**: The EVM is configured for `SAHARA_CHAIN_ID` (app-evm/src/lib.rs::SAHARA_CHAIN_ID).
- **Consensus**: Uses `commonware-consensus` for orchestration (whirlpool-node/Cargo.toml).
