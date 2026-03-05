# WORKSPACE

## Workspace map (design-relevant)
- Root workspace members include `crates/app`, `crates/app-evm`, `crates/state`, `crates/state-memory`, `crates/consensus`, `crates/consensus-simplex`, `crates/whirlpool-node` (`Cargo.toml::[workspace].members`).
- In-scope crate set for this design: `app`, `whirlpool-node`.
- Dependency context crates: `app-evm`, `state`, `state-memory`.

## Current crate graph slice
```text
app (interface)
  ├─ traits: Application, TxSource
  └─ impl helper: InMemoryTxPool

app-evm (implementation)
  ├─ depends on app + state/state-memory
  └─ executes txs, updates state root/receipts root

whirlpool-node (node/binary)
  ├─ builds state_db + tx_pool
  ├─ constructs EvmApplication + ApplicationAdapter
  └─ starts CommonwareEngine and blocks forever
```

## Grounded runtime entrypoints
- Node process entrypoint: `crates/whirlpool-node/src/main.rs::main`.
- EVM execution entrypoints used by consensus cycle:
  - `crates/app-evm/src/executor.rs::EvmApplication::propose`
  - `crates/app-evm/src/executor.rs::EvmApplication::verify`
- State read/write backing:
  - `crates/state/src/traits.rs::StateDb`
  - `crates/state-memory/src/db.rs::InMemoryStateDb`

## [PROPOSED] workspace impact (implementation phase only)
- Keep crate boundaries unchanged.
- Add RPC dependencies to `crates/whirlpool-node/Cargo.toml` only:
  - `jsonrpsee` (`0.26` family, server + macros features)
  - method-type support crates as needed for ETH RPC signatures.
- Add node-local modules under `crates/whirlpool-node/src/` for RPC transport/handlers/context.

## Entrypoint placement decision
- **Grounded**: `main` currently starts engine then awaits forever.
- **[PROPOSED]**: start RPC server after `engine.start()` and before `pending::<()>().await` so both consensus and RPC run under one runtime without introducing new top-level processes.
