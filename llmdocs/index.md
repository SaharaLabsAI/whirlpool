# Whirlpool — LLM Documentation Index

This index is the entry point for AI coding agents working in the Whirlpool repository.
Read this file first, then follow links to relevant sections based on your task.

## Project Summary

Whirlpool is a modular consensus framework for the Sahara Chain, built in Rust.
It uses a 3-layer architecture: abstract consensus traits -> Simplex BFT adapter -> node binary.
The vendor layer (commonware) is a git submodule under `vendor/` — **do not modify**.

## Reading Order

1. Start here (this file)
2. For project context -> `overview/project-overview.md`
3. For your task's domain -> pick from Architecture or Guides below
4. For code style -> `reference/coding-conventions.md`

---

## Overview

| File | Description |
|------|-------------|
| [overview/project-overview.md](overview/project-overview.md) | Project purpose, v0 goals/non-goals, tech stack, workspace structure, 3-layer architecture, design principles, test coverage |

## Architecture (LLM Retrieval Maps)

Structured reference docs optimized for fast LLM lookup. Each contains type signatures, relationships, and design decisions.

| File | Description | Crate |
|------|-------------|-------|
| [architecture/consensus-traits.md](architecture/consensus-traits.md) | Core trait layer: Block, ConsensusApp, EventSink, ConsensusEngine, RunningEngine, ConsensusError. Public signatures and trait relationships | `consensus` |
| [architecture/simplex-adapter.md](architecture/simplex-adapter.md) | Adapter bridge: CommonwareBlock, AppAdapter, CommonwareEngine, CommonwareConfig, Mailbox, FinalizationSink | `consensus-simplex` |
| [architecture/whirlpool-node.md](architecture/whirlpool-node.md) | Node library exports (EmptyBlock, EmptyBlockApp, config) used by EVM and non-EVM binaries | `whirlpool-node` (lib) |
| [crates/whirlpool-node.md](crates/whirlpool-node.md) | EVM binary wiring: `EvmApplication` + `InMemoryTxPool` tx source + Commonware engine startup | `whirlpool-node` (bin) |
| [crates/whirlpool-node-simple.md](crates/whirlpool-node-simple.md) | Non-EVM consensus binary using `EmptyBlockApp` | `whirlpool-node-simple` |

## Guides

Step-by-step instructions for common tasks and workflows.

| File | Description |
|------|-------------|
| [guides/implementing-consensus-traits.md](guides/implementing-consensus-traits.md) | Implementing Block, ConsensusApp, EventSink, and ConsensusEngine traits |
| [guides/wiring-simplex-adapter.md](guides/wiring-simplex-adapter.md) | Wiring CommonwareEngine via sealed API and CommonwareConfig |
| [guides/whirlpool-node-components.md](guides/whirlpool-node-components.md) | EmptyBlock conformance, EmptyBlockApp verification, and node extension points |
| [guides/block-lifecycle-walkthrough.md](guides/block-lifecycle-walkthrough.md) | End-to-end block lifecycle across all three layers |

## Reference

| File | Description |
|------|-------------|
| [reference/coding-conventions.md](reference/coding-conventions.md) | Rust 2021 style, naming, error handling, async patterns, tests |
| [reference/git-conventions.md](reference/git-conventions.md) | Conventional commits style, branch naming, PR workflow |

---

## Quick Lookup by Task

| If you need to... | Read |
|-------------------|------|
| Understand the project | `overview/project-overview.md` |
| Add a new block type | `architecture/consensus-traits.md` -> `guides/implementing-consensus-traits.md` |
| Wire a new consensus engine | `architecture/simplex-adapter.md` -> `guides/wiring-simplex-adapter.md` |
| Understand block flow | `architecture/block-lifecycle.md` -> `guides/block-lifecycle-walkthrough.md` |
| Extend the node binary | `architecture/whirlpool-node.md` -> `guides/whirlpool-node-components.md` |
| Update EVM node tx sourcing | `crates/app.md` + `crates/whirlpool-node.md` |
| Check code style | `reference/coding-conventions.md` |
| Write a commit message | `reference/git-conventions.md` |

## Workspace -> Documentation Map

| Crate | Source | Architecture Doc | Guide |
|-------|--------|------------------|-------|
| `consensus` | `crates/consensus/src/` | `architecture/consensus-traits.md` | `guides/implementing-consensus-traits.md` |
| `consensus-simplex` | `crates/consensus-simplex/src/` | `architecture/simplex-adapter.md` | `guides/wiring-simplex-adapter.md` |
| `p2p-commonware` | `crates/p2p-commonware/src/` | `crates/p2p-commonware.md` | — |
| `state` | `crates/state/src/` | `crates/state.md` | — |
| `app` | `crates/app/src/` | `crates/app.md` | — |
| `app-evm` | `crates/app-evm/src/` | `crates/app-evm.md` | — |
| `whirlpool-node` | `crates/whirlpool-node/src/` | `architecture/whirlpool-node.md` (lib) + `crates/whirlpool-node.md` (EVM bin) | `guides/whirlpool-node-components.md` |
| `whirlpool-node-simple` | `crates/whirlpool-node-simple/src/` | `crates/whirlpool-node-simple.md` | — |
