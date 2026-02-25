# Whirlpool — Project Overview

## Purpose

Whirlpool is a minimal Rust workspace that builds a single-node blockchain binary. Its primary function is to finalize deterministic empty blocks at a fixed 5-second cadence using the Simplex BFT consensus engine from the Commonware vendor library.

## Goals (v0)

- Validate consensus integration with the Commonware Simplex BFT engine
- Provide a simple, deterministic chain for testing and development
- Demonstrate the three-layer consensus architecture (traits → adapter → binary)

## Non-Goals (v0)

- No transaction execution or mempool
- No dynamic validator sets (single hard-coded validator)
- No RPC server
- No persistent storage (uses temp directory)
- No config files or CLI argument parsing

## Tech Stack

- **Language**: Rust (edition 2021)
- **Build**: Cargo workspace with Nix flake devShell
- **Consensus**: Commonware Simplex BFT (vendor submodule)
- **Testing**: cargo-nextest, deterministic runtime patterns
- **Linker**: mold (Linux, for fast linking)

## Workspace Structure

```
whirlpool/
├── crates/
│   ├── consensus/           # Layer 1: Abstract consensus traits
│   ├── consensus-simplex/   # Layer 2: Simplex BFT adapter
│   └── whirlpool-node/      # Layer 3: Binary entrypoint
├── vendor/
│   └── commonware/          # Vendor BFT engine (git submodule, read-only)
├── docs/                    # Design documents
├── agents/                  # AI agent workflow conventions
├── flake.nix                # Nix dev environment
└── Cargo.toml               # Workspace manifest
```

## Architecture (3-layer)

```
┌─────────────────────────────────────────┐
│  whirlpool-node (binary)                │
│  EmptyBlock, EmptyBlockApp,             │
│  FinalizationSink, Mailbox, wire.rs     │
├─────────────────────────────────────────┤
│  consensus-simplex (adapter)            │
│  AppAdapter, CommonwareEngine,          │
│  CommonwareBlock, CommonwareConfig      │
├─────────────────────────────────────────┤
│  consensus (traits)                     │
│  Block, ConsensusApp, EventSink,        │
│  ConsensusEngine, RunningEngine         │
├─────────────────────────────────────────┤
│  vendor: commonware (submodule)         │
│  Simplex BFT, P2P, Runtime, Codec      │
└─────────────────────────────────────────┘
```

**Layer 1 (consensus)**: Runtime-agnostic trait definitions. No vendor dependencies.

**Layer 2 (consensus-simplex)**: Bridges abstract traits to Commonware's Simplex engine via AppAdapter pattern.

**Layer 3 (whirlpool-node)**: Concrete implementations (EmptyBlock, EmptyBlockApp) and the binary that wires and runs the consensus engine.

## Key Design Principles

1. **Minimal core surface**: consensus crate has zero vendor dependencies — only trait definitions
2. **Adapter isolation**: All vendor-specific code lives in consensus-simplex
3. **Dual-trait conformance**: Block types must satisfy both abstract traits and vendor traits
4. **Native async**: Uses `impl Future` return types (not `#[async_trait]` macro)
5. **Atomic status**: Cross-thread state via `Arc<AtomicU64>` / `Arc<AtomicBool>` (no locks)

## Test Coverage

- **consensus**: 7 tests (trait impls, mock engine lifecycle)
- **consensus-simplex**: 8 tests (blanket impls, adapter type bounds, engine start/shutdown)
- **whirlpool-node**: 32 tests (block, app, sink, mailbox)
- **Total**: 48 tests, all passing
