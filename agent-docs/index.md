# Whirlpool — LLM Documentation Index

This index is the entry point for AI coding agents working in the Whirlpool repository.
Read this file first, then follow links to relevant sections based on your task.

## Project Summary

Whirlpool is a modular consensus framework for the Sahara Chain, built in Rust.
It uses a 3-layer architecture: abstract consensus traits -> Simplex BFT adapter -> node binary.
Canonical interface imports use `crate::traits::...` paths across crates after interface/implementation split refactoring.
EVM implementation crates are grouped under `crates/evm/`.
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
| [architecture/whirlpool-node.md](architecture/whirlpool-node.md) | Node library exports: `config` (CLI/TOML) and `node` lifecycle | `whirlpool-node` (lib) |
| [crates/whirlpool-node.md](crates/whirlpool-node.md) | EVM binary entrypoint and node configuration | `whirlpool-node` (bin) |
| [crates/app-composite.md](crates/app-composite.md) | Composite consensus application that classifies mixed tx streams and delegates execution to domain apps | `app-composite` |
| [crates/evm-precompiles.md](crates/evm-precompiles.md) | Workspace-owned registry/factory/example crate for Whirlpool custom EVM precompiles | `evm-precompiles` |
| [crates/community-pool.md](crates/community-pool.md) | Fixed community-pool account constant used by the current fee-accounting slice | `community-pool` |
| [crates/native-token.md](crates/native-token.md) | Canonical Sahara native-token hard cap and genesis allocation validation helpers | `native-token` |
| [crates/validators.md](crates/validators.md) | Ordered simplex validator-registry model and genesis-storage codec shared across node/app/precompile surfaces | `validators` |
| [crates/tx-dispatch.md](crates/tx-dispatch.md) | Mem-scoped mixed transaction classification across EVM and mem tx families | `tx-dispatch` |
| [crates/rpc-eth.md](crates/rpc-eth.md) | Ethereum JSON-RPC server: reth-backed adapters (WhirlpoolProvider, WhirlpoolTxPool, WhirlpoolNetwork), RpcConfig API, blob exclusion | `rpc-eth` |
| [crates/state-reth.md](crates/state-reth.md) | Persistent state implementation: RethStateDb, MDBX, state root | `state-reth` |
| [crates/mempool.md](crates/mempool.md) | Persistent transaction pool: MempoolStore, MDBX, FIFO ordering | `mempool` |

## Design

| File | Description |
|------|-------------|
| [design/index.md](design/index.md) | Design-level rationale docs with progressive-disclosure topics such as Whirlpool custom precompiles |

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
| Extend node/mempool | `architecture/whirlpool-node.md` -> `guides/whirlpool-node-components.md` + `crates/mempool.md` |
| Update EVM node tx sourcing | `crates/app.md` + `crates/whirlpool-node.md` + `crates/mempool.md` |
| Understand mixed EVM/mem tx routing | `crates/app-composite.md` + `crates/tx-dispatch.md` + `crates/app-evm.md` |
| Add/modify RPC methods | `crates/rpc-eth.md` + `crates/rpc-mem.md` |
| Understand why Whirlpool precompiles are runtime-installed | `design/precompiles/index.md` -> `design/precompiles/availability.md` -> `design/precompiles/wiring.md` |
| Check code style | `reference/coding-conventions.md` |
| Write a commit message | `reference/git-conventions.md` |
| Add/run e2e integration tests | `crates/integration-tests.md` |

## Workspace -> Documentation Map

| Crate | Source | Architecture Doc | Guide |
|-------|--------|------------------|-------|
| `consensus` | `crates/consensus/src/` | `architecture/consensus-traits.md` | `guides/implementing-consensus-traits.md` |
| `consensus-simplex` | `crates/consensus-simplex/src/` | `architecture/simplex-adapter.md` | `guides/wiring-simplex-adapter.md` |
| `p2p-commonware` | `crates/p2p-commonware/src/` | `crates/p2p-commonware.md` | — |
| `state` | `crates/state/src/` | `crates/state.md` | — |
| `state-memory` | `crates/mem/state/src/` | `crates/state-memory.md` | — |
| `state-reth` | `crates/evm/state/src/` | `crates/state-reth.md` | — |
| `app` | `crates/app/src/` | `crates/app.md` | — |
| `app-evm` | `crates/evm/app/src/` | `crates/app-evm.md` | — |
| `evm-precompiles` | `crates/evm/precompiles/src/` | `crates/evm-precompiles.md` | — |
| `app-composite` | `crates/mem/composite/src/` | `crates/app-composite.md` | — |
| `community-pool` | `crates/community-pool/src/` | `crates/community-pool.md` | — |
| `native-token` | `crates/native-token/src/` | `crates/native-token.md` | — |
| `validators` | `crates/validators/src/` | `crates/validators.md` | — |
| `mempool` | `crates/mempool/src/` | `crates/mempool.md` | — |
| `tx-dispatch` | `crates/mem/tx-dispatch/src/` | `crates/tx-dispatch.md` | — |
| `rpc-mem` | `crates/mem/rpc/src/` | `crates/rpc-mem.md` | — |
| `whirlpool-node` | `crates/node/src/` | `architecture/whirlpool-node.md` (lib) + `crates/whirlpool-node.md` (EVM bin) | `guides/whirlpool-node-components.md` |
| `rpc-eth` | `crates/evm/rpc/src/` | `crates/rpc-eth.md` | — |
| `integration-tests` | `testing/integration-tests/tests/` | `crates/integration-tests.md` | — |
