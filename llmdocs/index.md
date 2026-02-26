# Whirlpool — LLM Documentation Index

This index is the entry point for AI coding agents working in the Whirlpool repository.
Read this file first, then follow links to relevant sections based on your task.

## Project Summary

Whirlpool is a modular consensus framework for the Sahara Chain, built in Rust.
It uses a 3-layer architecture: abstract consensus traits → Simplex BFT adapter → node binary.
The vendor layer (commonware) is a git submodule under `vendor/` — **do not modify**.

## Reading Order

1. Start here (this file)
2. For project context → `overview/project-overview.md`
3. For your task's domain → pick from Architecture or Guides below
4. For code style → `reference/coding-conventions.md`

---

## Overview

| File | Description |
|------|-------------|
| [overview/project-overview.md](overview/project-overview.md) | Project purpose, v0 goals/non-goals, tech stack, workspace structure, 3-layer architecture, design principles, test coverage |

## Architecture (LLM Retrieval Maps)

Structured reference docs optimized for fast LLM lookup. Each contains type signatures, relationships, and design decisions.

| File | Description | Crate |
|------|-------------|-------|
| [architecture/consensus-traits.md](architecture/consensus-traits.md) | Core trait layer: Block, ConsensusApp, EventSink, ConsensusEngine, RunningEngine, ConsensusError. All public signatures, cross-trait relationships, mock implementations | `consensus` |
| [architecture/simplex-adapter.md](architecture/simplex-adapter.md) | Adapter bridge: CommonwareBlock, AppAdapter, CommonwareEngine, CommonwareConfig, Mailbox/MailboxActor, FinalizationSink. Sealed engine wiring, vendor trait mapping | `consensus-simplex` |
| [architecture/whirlpool-node.md](architecture/whirlpool-node.md) | Node binary: EmptyBlock (dual-trait conformance), EmptyBlockApp (5 verification rules). Pure business logic, delegates consensus wiring to consensus-simplex | `whirlpool-node` |
| [architecture/block-lifecycle.md](architecture/block-lifecycle.md) | End-to-end block data flow across all 3 layers: propose → verify → finalize. Cross-crate type mappings, event propagation chain, observability via atomics | Cross-crate |
| [crates/p2p-commonware.md](crates/p2p-commonware.md) | Commonware P2P bridge: MultiplexSender, MultiplexReceiver, channel routing, test infrastructure (MockCwReceiver), optional runtime handle design | `p2p-commonware` |

## Guides

Step-by-step instructions for common tasks and workflows.

| File | Description |
|------|-------------|
| [guides/implementing-consensus-traits.md](guides/implementing-consensus-traits.md) | How to implement Block, ConsensusApp, EventSink, and ConsensusEngine traits with code examples and mock references |
| [guides/wiring-simplex-adapter.md](guides/wiring-simplex-adapter.md) | How to wire CommonwareEngine using sealed API: define block type, implement ConsensusApp/EventSink, construct engine, configure CommonwareConfig |
| [guides/whirlpool-node-components.md](guides/whirlpool-node-components.md) | Understanding EmptyBlock dual-trait conformance, EmptyBlockApp verification rules, using CommonwareEngine API from consensus-simplex |
| [guides/block-lifecycle-walkthrough.md](guides/block-lifecycle-walkthrough.md) | Traces a block from proposal through verification to finalization across all 3 layers with exact method names |

## Reference

| File | Description |
|------|-------------|
| [reference/coding-conventions.md](reference/coding-conventions.md) | Rust edition 2021, formatting, naming, error handling, async patterns (`impl Future`, not `async_trait`), testing conventions |
| [reference/git-conventions.md](reference/git-conventions.md) | Conventional commits style, branch naming, PR workflow |

---

## Quick Lookup by Task

| If you need to... | Read |
|-------------------|------|
| Understand the project | `overview/project-overview.md` |
| Add a new block type | `architecture/consensus-traits.md` → `guides/implementing-consensus-traits.md` |
| Wire a new consensus engine | `architecture/simplex-adapter.md` → `guides/wiring-simplex-adapter.md` |
| Understand block flow | `architecture/block-lifecycle.md` → `guides/block-lifecycle-walkthrough.md` |
| Extend the node binary | `architecture/whirlpool-node.md` → `guides/whirlpool-node-components.md` |
| Complete wire.rs / main.rs | `architecture/whirlpool-node.md` (see STUB sections) |
| Check code style | `reference/coding-conventions.md` |
| Write a commit message | `reference/git-conventions.md` |

## Workspace → Documentation Map

| Crate | Source | Architecture Doc | Guide |
|-------|--------|-----------------|-------|
| `consensus` | `crates/consensus/src/` | `architecture/consensus-traits.md` | `guides/implementing-consensus-traits.md` |
| `consensus-simplex` | `crates/consensus-simplex/src/` | `architecture/simplex-adapter.md` | `guides/wiring-simplex-adapter.md` |
| `p2p-commonware` | `crates/p2p-commonware/src/` | `crates/p2p-commonware.md` | — |
| `whirlpool-node` | `crates/whirlpool-node/src/` | `architecture/whirlpool-node.md` | `guides/whirlpool-node-components.md` |
