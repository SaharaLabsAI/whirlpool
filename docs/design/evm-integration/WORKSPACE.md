# Workspace Map

## Overview

Whirlpool is a modular consensus framework for Sahara Chain. The workspace follows a layered architecture:
- **Abstract trait crates** define contracts (consensus, app [PROPOSED])
- **Concrete adapter crates** implement those contracts for specific backends (consensus-simplex, app-evm [PROPOSED])
- **Node binary** wires everything together (whirlpool-node)

## Workspace structure

```
whirlpool/
├── Cargo.toml               # workspace root (resolver v2, excludes vendor/)
├── crates/
│   ├── consensus/            # Abstract consensus traits (Block, ConsensusApp, ConsensusEngine)
│   ├── consensus-simplex/    # Simplex BFT adapter (bridges to commonware-consensus)
│   ├── p2p/                  # Abstract P2P traits
│   ├── p2p-commonware/       # Commonware P2P adapter
│   ├── whirlpool-node/       # Node binary + EmptyBlockApp
│   ├── app/                  # [PROPOSED] Abstract application traits (EVM-aware)
│   └── app-evm/              # [PROPOSED] Concrete EVM application (reth-evm backed)
├── vendor/
│   ├── reth/                 # Reth Ethereum client (git submodule, read-only)
│   │   └── crates/
│   │       ├── evm/evm/      # reth-evm: ConfigureEvm, BlockExecutorFactory, BlockAssembler
│   │       ├── evm/execution-types/  # ExecutionOutcome, BlockExecutionOutput
│   │       ├── evm/execution-errors/ # BlockExecutionError
│   │       ├── ethereum/evm/ # reth-evm-ethereum: EthEvmConfig (reference impl)
│   │       ├── revm/         # reth-revm: revm wrapper, State<DB>
│   │       └── ...           # chainspec, primitives, etc.
│   └── commonware/           # Commonware consensus primitives (git submodule, read-only)
├── docs/
│   └── design/
│       └── evm-integration/  # This design doc set
└── llmdocs/                  # Auto-generated code documentation
```

## Crate dependency graph (post-integration)

```
                    ┌──────────────┐
                    │ whirlpool-   │
                    │    node      │
                    └──┬───┬───┬──┘
                       │   │   │
          ┌────────────┘   │   └────────────┐
          ▼                ▼                 ▼
   ┌──────────┐    ┌──────────────┐   ┌──────────┐
   │consensus-│    │  app-evm     │   │p2p-      │
   │ simplex  │    │  [PROPOSED]  │   │commonware│
   └────┬─────┘    └──┬───────┬──┘   └────┬─────┘
        │              │       │           │
        ▼              ▼       ▼           ▼
   ┌──────────┐   ┌────────┐ ┌─────┐  ┌──────┐
   │consensus │   │  app   │ │reth-│  │ p2p  │
   │          │   │[PROP.] │ │ evm │  │      │
   └──────────┘   └────────┘ └─────┘  └──────┘
                       │        │
                       │        ▼
                       │   ┌─────────────┐
                       │   │reth-evm-    │
                       │   │ethereum     │
                       │   │(reference)  │
                       │   └─────────────┘
                       │
                       ▼
                  ┌──────────┐
                  │consensus │
                  │(Block    │
                  │ trait)   │
                  └──────────┘
```

## Build entrypoints

- `cargo build` — all workspace members
- `cargo test` — all workspace tests
- Binary: `whirlpool-node` (`crates/whirlpool-node/src/main.rs`)
- All cargo commands via `nix develop --command cargo ...`

## Reading guide

1. Start with `consensus/src/{block.rs, app.rs}` for the abstract consensus ↔ app contract
2. Read `whirlpool-node/src/{block.rs, app.rs}` for the current EmptyBlock implementation
3. Study `vendor/reth/crates/evm/evm/src/lib.rs` for `ConfigureEvm` trait
4. Study `vendor/reth/crates/ethereum/evm/src/lib.rs` for `EthEvmConfig` reference implementation
5. This design proposes `app/` (abstract) and `app-evm/` (concrete) following the same pattern
