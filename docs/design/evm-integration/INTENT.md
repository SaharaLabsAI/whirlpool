# Design Intent

## Objective

Integrate EVM execution capability into the Whirlpool consensus framework by introducing two new crates:

- **`app`** — Abstract application trait that generalises `ConsensusApp` to support EVM-backed state transitions. Defines the `Application` trait (or extends `ConsensusApp`) so that block proposal and verification can delegate to an EVM executor.
- **`app-evm`** — Concrete EVM application implementation backed by reth's `ConfigureEvm` abstraction (`vendor/reth/crates/evm/`). Provides a `WhirlpoolEvmConfig` that implements `reth_evm::ConfigureEvm`, wires `BlockExecutorFactory` + `BlockAssembler`, and satisfies the `app` trait.

## Motivation

The current Whirlpool node uses `EmptyBlockApp` — a stateless consensus app that produces empty blocks with no transaction execution. To support Sahara Chain's EVM-compatible execution layer, we need:

1. An abstract application layer (`app`) that decouples consensus from execution details, allowing different execution backends (EVM, future VMs).
2. A concrete EVM backend (`app-evm`) that uses reth's battle-tested EVM execution stack to execute Ethereum-compatible transactions within Whirlpool's consensus-driven block lifecycle.

This mirrors the existing layering pattern: `consensus` (abstract) → `consensus-simplex` (concrete adapter). Here: `app` (abstract) → `app-evm` (concrete EVM backend).

## Scope

### In scope
- `crates/app/` — New crate with abstract application trait(s) for EVM-aware block proposal, execution, and verification
- `crates/app-evm/` — New crate implementing `app` traits using `reth-evm` and `reth-evm-ethereum`
- Integration points with existing `consensus::ConsensusApp` trait
- `crates/state/` — New crate providing concrete in-memory EVM state database implementing `revm::Database`, state commitment (`BundleState` application), and state root computation <!-- continuation round 2: resolves B-002 -->
- Block type design that carries EVM execution results (state root, receipts, etc.)
- Design of the `ConfigureEvm` implementation for Whirlpool (`WhirlpoolEvmConfig`)

### Out of scope
- ~~State storage / database layer~~ → Moved in-scope (round 2, see `crates/state/`). Persistent storage backends (RocksDB, MDBX) remain out of scope.
- Transaction pool / mempool
- RPC / JSON-RPC layer
- Network-level transaction propagation
- Modifying `vendor/` code

### Vendor crates consumed (read-only)
- `reth-evm` (`vendor/reth/crates/evm/evm/`) — Core `ConfigureEvm`, `BlockExecutorFactory`, `BlockAssembler` traits
- `reth-evm-ethereum` (`vendor/reth/crates/ethereum/evm/`) — Reference Ethereum implementation (`EthEvmConfig`)
- `reth-revm` (`vendor/reth/crates/revm/`) — revm wrapper, `State<DB>`, database glue
- `reth-execution-types` (`vendor/reth/crates/evm/execution-types/`) — `ExecutionOutcome`, `BlockExecutionOutput`
- `reth-execution-errors` (`vendor/reth/crates/evm/execution-errors/`) — `BlockExecutionError`

<!-- continuation round 2 -->
- `revm` (via `reth-revm` re-exports) — `Database`, `DatabaseCommit`, `DatabaseRef` traits, `CacheDB`, `State<DB>` builder

## Success criteria

1. `app` crate compiles and defines trait(s) that `EmptyBlockApp` could trivially implement (backwards-compatible abstraction)
2. `app-evm` crate compiles and provides a `WhirlpoolEvmConfig` that implements `reth_evm::ConfigureEvm`
3. `app-evm` can execute a block of EVM transactions given a state provider and produce execution results
4. The design preserves the existing consensus ↔ app boundary (`ConsensusApp` trait is not broken)
5. Clear wiring path from `ConsensusEngine` → `Application` → EVM executor → state updates
6. All proposed interfaces are grounded in evidence from existing reth EVM patterns
7. `state` crate compiles and provides an in-memory `Database` implementation that `EvmApplication` can use to execute blocks <!-- continuation round 2 -->
8. State root computation produces deterministic results for identical execution sequences <!-- continuation round 2 -->

## Grounding summary

### Existing consensus ↔ app boundary
- `consensus::ConsensusApp` trait (`crates/consensus/src/app.rs`): `genesis()`, `propose(parent, height)`, `verify(parent, block)` with associated `Block` type
- `consensus::Block` trait (`crates/consensus/src/block.rs`): `id()`, `parent_id()`, `height()` — identity-only, no execution data
- `EmptyBlockApp` (`crates/whirlpool-node/src/app.rs`): Stateless ZST implementing `ConsensusApp` for `EmptyBlock`
- `EmptyBlock` (`crates/whirlpool-node/src/block.rs`): `{height, parent_id}` — no txs, no state root

### Reth EVM abstraction stack
- `ConfigureEvm` trait (`vendor/reth/crates/evm/evm/src/lib.rs`): Central trait binding `NodePrimitives`, `BlockExecutorFactory`, `BlockAssembler`, and env construction
- `EthEvmConfig` (`vendor/reth/crates/ethereum/evm/src/lib.rs`): Reference implementation — pattern to follow
- Three execution layers: `EvmFactory` → single-tx execution, `BlockExecutorFactory` → block-level execution, `BlockAssembler` → block construction from results
- Key associated types flow: `ConfigureEvm::Primitives`, `::Error`, `::NextBlockEnvCtx`, `::BlockExecutorFactory`, `::BlockAssembler`

### Layering pattern in workspace
- Abstract traits crate → Concrete adapter crate (consensus → consensus-simplex). Same pattern applies: app → app-evm.
- Abstract traits crate → Concrete adapter crate also applies to state: `app-evm` is generic over `DB: Database` → `state` provides `InMemoryStateDb` as the concrete implementation. <!-- continuation round 2 -->
