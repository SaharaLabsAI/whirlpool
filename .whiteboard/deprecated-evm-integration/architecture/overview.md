# Architecture Overview

## Subsystem map

```
┌─────────────────────────────────────────────────────────────┐
│                      whirlpool-node                          │
│  (wires everything, runs consensus engine)                   │
├──────────┬──────────────┬──────────┬───────────────────────┤
│consensus-│   app-evm    │  state   │ p2p-                  │
│simplex   │  [PROPOSED]  │[PROPOSED]│ commonware            │
│(BFT      │(EVM exec     │(in-mem   │(network               │
│ adapter) │ backend)     │ state DB)│ adapter)              │
├──────────┼──────────────┼──────────┼───────────────────────┤
│consensus │    app       │   revm   │  p2p                  │
│(abstract │  [PROPOSED]  │(Database │(abstract              │
│ traits)  │(abstract     │ trait,   │ traits)               │
│          │ app trait)   │ cargo)   │                       │
├──────────┴──────────────┴──────────┴───────────────────────┤
│                     Vendor (read-only)                        │
│  reth-evm │ reth-evm-ethereum │ reth-revm │ commonware       │
└─────────────────────────────────────────────────────────────┘
```

## Flow index

| Flow | Summary | File |
|---|---|---|
| Block proposal | Consensus triggers → app proposes → EVM executes → block assembled | architecture/block-proposal.md |
| Block verification | Consensus triggers → app verifies → EVM re-executes → root compared | architecture/block-verification.md |
| Consensus ↔ app bridge | How ApplicationAdapter bridges Application to ConsensusApp | architecture/consensus-app-bridge.md |
| Node startup wiring | How whirlpool-node wires EVM app into consensus engine | architecture/node-startup.md |

## Key invariants

1. **Consensus is execution-agnostic**: `ConsensusApp` trait does not change. All EVM-specific logic lives behind `Application` → `ApplicationAdapter`.
2. **Vendor code is read-only**: All reth/commonware crates are consumed as dependencies, never modified.
3. **Block identity is deterministic**: `EvmBlock::compute_id()` must be deterministic from block contents (same as `EmptyBlock` pattern).
4. **State root is authoritative**: Block verification succeeds iff re-execution produces the same state root as the proposed block.

<!-- continuation round 2 -->
5. **State root is deterministic**: Same genesis + same transaction sequence + same commit order = identical `state_root()`. [PROPOSED — `state` crate]
6. **State commitment is post-execution**: `BundleState` is committed to `InMemoryStateDb` only after successful execution, never during. Clone-based snapshots ensure failed executions don't corrupt canonical state. [PROPOSED — `state` crate]

## Glossary

| Term | Definition |
|---|---|
| ConfigureEvm | Reth trait binding EVM environment, executor factory, and block assembler. Entry point for all EVM configuration. |
| BlockExecutorFactory | Creates block-level executors that process all transactions in a block. |
| BlockAssembler | Constructs a valid block (header + body) from execution results. |
| EvmFactory | Creates individual EVM instances for single-transaction execution. |
| BlockBuilder | Combined execute+assemble workflow: execute txs one by one, then finish() to get the assembled block. |
| NextBlockEnvAttributes | CL-provided attributes for the next block: timestamp, fee recipient, randao, gas limit, etc. |
| EthBlockExecutionCtx | Contextual data for Ethereum block execution: parent hash, beacon root, withdrawals, etc. |
| NodePrimitives | Reth trait that bundles Block, Receipt, and other type families for a chain. |
| EvmBlock | [PROPOSED] Whirlpool block type carrying both consensus identity and EVM execution data. |
| ApplicationAdapter | [PROPOSED] Adapter that wraps `Application` to satisfy `ConsensusApp`. |
| InMemoryStateDb | [PROPOSED] HashMap-based `revm::Database` implementation. Provides `commit()` and `state_root()`. (round 2) |
| BundleState | Reth/revm type representing state diff from block execution — changed accounts, storage, and contracts. |
| DbAccount | [PROPOSED] Per-account state: `AccountInfo` + `HashMap<U256, U256>` storage. (round 2) |

## Implementation slices

### Slice 1: `app` crate scaffold
- **Goal**: Establish the abstract application trait so downstream crates can depend on it.
- **Crates touched**: `app` (new), `Cargo.toml` (workspace)
- **New types**: `Application` trait, `ApplicationError` enum, `ExecutionResult` struct, `EvmBlock` struct
- **New interfaces**: `Application` trait with `genesis()`, `propose()`, `verify()`
- **Pseudo-code**: Define trait + types. Implement `consensus::Block` for `EvmBlock`. Implement commonware codec traits.
- **Acceptance**: `cargo build` succeeds, `app` crate exports `Application` trait, `EvmBlock` implements `consensus::Block`.

### Slice 2: `ApplicationAdapter` + ConsensusApp bridge
- **Goal**: Bridge `Application` impls to `ConsensusApp` so existing consensus engine works unchanged.
- **Crates touched**: `app`
- **New types**: `ApplicationAdapter<A>` struct
- **New interfaces**: `impl ConsensusApp for ApplicationAdapter<A: Application>`
- **Pseudo-code**: Delegate genesis/propose/verify, discard ExecutionResult at boundary, map errors.
- **Acceptance**: Unit test showing `ApplicationAdapter<MockApp>` satisfies `ConsensusApp` bounds.

### Slice 3: `WhirlpoolEvmConfig` (ConfigureEvm impl)
- **Goal**: Provide a concrete `ConfigureEvm` implementation for Sahara Chain.
- **Crates touched**: `app-evm` (new), `Cargo.toml` (workspace)
- **New types**: `WhirlpoolEvmConfig` struct
- **New interfaces**: `impl ConfigureEvm for WhirlpoolEvmConfig`
- **Deps**: `reth-evm`, `reth-evm-ethereum`, `reth-chainspec`, `alloy-evm`
- **Pseudo-code**: Mirror `EthEvmConfig::new(chain_spec)` pattern. Wire `EthBlockExecutorFactory` + `EthBlockAssembler`.
- **Acceptance**: `WhirlpoolEvmConfig::new(chain_spec)` compiles. `block_executor_factory()` and `block_assembler()` return valid references.

### Slice 4: `EvmApplication` (Application impl)
- **Goal**: Implement `Application` trait using `WhirlpoolEvmConfig` for EVM-backed block execution.
- **Crates touched**: `app-evm`
- **New types**: `EvmApplication<DB>` struct, `EvmAppError` enum
- **New interfaces**: `impl Application for EvmApplication<DB>`
- **Pseudo-code**: `propose()` uses `builder_for_next_block()` + `execute_transaction()` + `finish()`. `verify()` uses `create_executor()` + `execute_one()` + state root comparison.
- **Acceptance**: Unit test executing a simple tx against in-memory state.

### Slice 5: Node wiring
- **Goal**: Wire `EvmApplication` into `whirlpool-node` alongside existing `EmptyBlockApp`.
- **Crates touched**: `whirlpool-node`
- **Changed interfaces**: `main.rs` — add EVM app construction path (feature-gated or config-driven)
- **Pseudo-code**: `let evm_config = WhirlpoolEvmConfig::new(chain_spec); let app = EvmApplication::new(evm_config, state_db); let adapter = ApplicationAdapter::new(app); engine.start(adapter);`
- **Acceptance**: Node starts with EVM app, produces genesis block, can propose empty EVM blocks.

<!-- continuation round 2 -->

### Slice 0.5: `state` crate scaffold (round 2 — B-002 resolution)
- **Goal**: Provide `InMemoryStateDb` so `EvmApplication<DB>` has a concrete `DB` type.
- **Crates touched**: `state` (new), `Cargo.toml` (workspace)
- **New types**: `InMemoryStateDb`, `DbAccount`, `StateError`
- **New interfaces**: `impl Database for InMemoryStateDb`, `impl DatabaseRef for InMemoryStateDb`, `commit(&mut self, &BundleState)`, `state_root(&self) -> B256`, `with_genesis(alloc) -> Self`
- **Deps**: `revm` (Database trait, types), `alloy-primitives` (Address, B256, U256)
- **Acceptance**: `cargo build` succeeds. `InMemoryStateDb` satisfies `Database + Clone`. Unit tests for basic/storage/code_by_hash/block_hash lookups, commit round-trip, state root determinism.
