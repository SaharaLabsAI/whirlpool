# Design Context Payload

## Intent
Enable `whirlpool-node` to produce real EVM blocks with transaction execution, replacing the current MVP that only produces empty blocks. This covers the full lifecycle from transaction ingestion to consensus finalization.

## Output root
docs/design/evm-block-production/

## Crate index
| Crate | Path | Purpose | Key public types | Domains |
|---|---|---|---|---|
| `whirlpool-node` | `crates/whirlpool-node` | Node binary and orchestration | `config::VALIDATOR_SEED`, `main` | Block Production |
| `app` | `crates/app` | Abstract application interfaces | `Application`, `TxSource`, `EvmBlock`, `ExecutionResult`, `ApplicationAdapter` | Application Layer |
| `app-evm` | `crates/app-evm` | Concrete EVM execution engine | `EvmApplication`, `WhirlpoolEvmConfig`, `StateProvider` | EVM Execution |
| `state` | `crates/state` | In-memory EVM state database | `InMemoryStateDb`, `DbAccount`, `StateError` | State Management |

## Domain map
| Domain | Summary | Owning crates | Key entrypoints | Evidence |
|---|---|---|---|---|
| Block Production | Orchestration of block lifecycle from proposal to finalization | `whirlpool-node` | `main.rs` (ConsensusEngine instantiation) | `crates/whirlpool-node/src/main.rs` |
| Application Layer | Generic traits for connecting consensus to execution logic | `app` | `Application` trait | `crates/app/src/traits.rs::Application` |
| EVM Execution | Execution of EVM transactions and block validation | `app-evm` | `EvmApplication::propose`, `EvmApplication::verify` | `crates/app-evm/src/executor.rs::EvmApplication` |
| State Management | Storage and commitment of EVM state changes | `state` | `InMemoryStateDb::commit`, `InMemoryStateDb::state_root` | `crates/state/src/db.rs::InMemoryStateDb` |

## Wiring skeleton
| Domain | Capability | Owning crate | Trait interface | Provider | Evidence |
|---|---|---|---|---|---|
| Block Production | Consensus Orchestration | `consensus-simplex` | `ConsensusEngine` | `CommonwareEngine` | `crates/whirlpool-node/src/main.rs::CommonwareEngine` |
| Application Layer | Consensus to App Bridge | `app` | `ConsensusApp` | `ApplicationAdapter` | `crates/app/src/adapter.rs::ApplicationAdapter` |
| EVM Execution | Block Proposal/Verification | `app-evm` | `Application` | `EvmApplication` | `crates/app-evm/src/executor.rs::EvmApplication` |
| EVM Execution | EVM Configuration | `app-evm` | `ConfigureEvm` (reth) | `WhirlpoolEvmConfig` | `crates/app-evm/src/config.rs::WhirlpoolEvmConfig` |
| State Management | State Access | `state` | `Database` (revm) | `InMemoryStateDb` | `crates/state/src/db.rs::InMemoryStateDb` |
| State Management | State Root Provider | `app-evm` | `StateProvider` | `TestStateDb` (in node) | `crates/whirlpool-node/src/main.rs::TestStateDb` |

## Key flows (index only)
| Flow | Trigger | Crates involved | Summary |
|---|---|---|---|
| Block Proposal | Consensus Engine | `whirlpool-node`, `app`, `app-evm`, `state` | Engine calls `propose`, `EvmApplication` fetches txs from `TxSource`, executes them, and builds `EvmBlock`. |
| Block Verification | Consensus Engine | `whirlpool-node`, `app`, `app-evm`, `state` | Engine calls `verify`, `EvmApplication` re-executes txs and validates `state_root`. |
| State Commitment | Consensus Engine (Finalization) | `state` | After consensus finalization, the winning block's state changes are committed to the database. |

## Grounded vs Proposed summary
- Grounded contracts: `ConsensusApp` (`consensus`), `Block` (`consensus`), `Application` (`app`), `TxSource` (`app`), `Database` (`revm`), `InMemoryStateDb` (`state`), `EvmApplication` (`app-evm`).
- Proposed contracts: Actual transaction execution logic inside `EvmApplication::propose` and `EvmApplication::verify`. A non-noop `TxSource` implementation (e.g., `MempoolTxSource`).

## Open blockers
- BLOCKER: `EvmApplication::propose` is a stub: `// MVP: Empty block execution (no transaction processing)` (`crates/app-evm/src/executor.rs:95`).
- BLOCKER: `EvmApplication::verify` does not re-execute transactions, only checks `state_root` against current DB (`crates/app-evm/src/executor.rs:130`).
- BLOCKER: No concrete `TxSource` implementation that provides actual transactions (`crates/app/src/traits.rs:27`).
- BLOCKER: `InMemoryStateDb::state_root` uses a simplified manual hash instead of a real Merkle Patricia Trie (MPT), though noted as out-of-scope for immediate intent (`crates/state/src/db.rs:105`).
