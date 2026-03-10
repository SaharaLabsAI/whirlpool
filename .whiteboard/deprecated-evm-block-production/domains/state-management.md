# State Management

## Definition
State Management owns canonical EVM state storage, deterministic root derivation, and bundle application for account/storage/code changes. The current implementation is an in-memory database (`state::InMemoryStateDb`) consumed by EVM execution and node wiring, with no persisted backend in the scoped code.

## Derived crates
| Crate | Role in domain | Grounded evidence |
|---|---|---|
| `state` | Owns state data model and database behavior | `crates/state/src/db.rs::InMemoryStateDb`, `crates/state/src/db.rs::DbAccount`, `crates/state/src/lib.rs` |
| `app-evm` | Consumes state root through provider and reads DB during app lifecycle | `crates/app-evm/src/executor.rs::EvmApplication::genesis`, `crates/app-evm/src/executor.rs::EvmApplication` (`state_db` field) |
| `whirlpool-node` | Wires node-local DB wrapper and provides `StateProvider`/`revm::Database` bridge | `crates/whirlpool-node/src/main.rs::TestStateDb`, `crates/whirlpool-node/src/main.rs::impl StateProvider for TestStateDb`, `crates/whirlpool-node/src/main.rs::impl revm::Database for TestStateDb` |

## Key public contracts
| Contract | Kind | Purpose | Evidence |
|---|---|---|---|
| `state::InMemoryStateDb` | Struct | Canonical in-memory state store for accounts, bytecodes, and block hashes | `crates/state/src/db.rs::InMemoryStateDb` |
| `state::DbAccount` | Struct | Per-account state payload (`AccountInfo` + storage map) | `crates/state/src/db.rs::DbAccount` |
| `InMemoryStateDb::with_genesis` | API | Loads genesis alloc into state | `crates/state/src/db.rs::InMemoryStateDb::with_genesis` |
| `InMemoryStateDb::commit` | API | Applies `revm::database::BundleState` changes atomically per call | `crates/state/src/db.rs::InMemoryStateDb::commit` |
| `InMemoryStateDb::state_root` | API | Derives deterministic hash over sorted account/storage entries | `crates/state/src/db.rs::InMemoryStateDb::state_root` |
| `impl revm::DatabaseRef for InMemoryStateDb` | Trait impl | Read interface consumed by EVM/revm callers | `crates/state/src/db.rs::impl DatabaseRef for InMemoryStateDb` |
| `impl revm::Database for InMemoryStateDb` | Trait impl | Mutable DB interface forwarding to `DatabaseRef` methods | `crates/state/src/db.rs::impl Database for InMemoryStateDb` |
| `app_evm::executor::StateProvider` (consumed) | Trait seam | Exposes `state_root()` to execution flow | `crates/whirlpool-node/src/main.rs::impl StateProvider for TestStateDb` |

## Core workflows
1. **Genesis load**: `with_genesis` converts `GenesisAccount` fields into `DbAccount` entries and indexes bytecode hashes for code lookup (`crates/state/src/db.rs::InMemoryStateDb::with_genesis`).
2. **Proposal/verify read path**: execution path reads root via `StateProvider::state_root()` (node wrapper delegates to `InMemoryStateDb::state_root`) and reads account/code/storage through `revm::Database` delegation (`crates/whirlpool-node/src/main.rs`).
3. **State transition application**: `commit` iterates `BundleState`, handles account destruction/update, updates storage slots, and stores contract bytecodes (`crates/state/src/db.rs::InMemoryStateDb::commit`).
4. **Root derivation**: `state_root` sorts accounts and storage keys before hashing to produce deterministic output; empty DB returns `KECCAK_EMPTY` (`crates/state/src/db.rs::InMemoryStateDb::state_root`).
5. **Consensus finalization linkage**: runtime wiring exposes `FinalizationSink` height tracking but no in-scope finalize callback to call `InMemoryStateDb::commit` is visible (`crates/consensus/src/app.rs::ConsensusApp` has only `genesis/propose/verify`; `crates/whirlpool-node/src/main.rs`).

## Open questions / TODOs
- **INV-01 impact (Execution Visibility)**: BLOCKER. Non-empty tx effects cannot surface through state yet in the wired runtime because `NoopTxSource` returns empty txs and `EvmApplication::propose` builds an empty block (`crates/whirlpool-node/src/main.rs`, `crates/app/src/traits.rs::NoopTxSource`, `crates/app-evm/src/executor.rs::EvmApplication::propose`).
- **INV-02 impact (Verification Integrity)**: BLOCKER. Current verify path checks only `state_root`; verification of `transactions_root`, `receipts_root`, and gas accounting is missing in scoped code (`crates/app-evm/src/executor.rs::EvmApplication::verify`).
- **INV-03 impact (Verification Read-Only)**: grounded for current minimal path; verify takes read lock and queries root only. TODO: preserve this property once full tx re-execution is added (`crates/app-evm/src/executor.rs::EvmApplication::verify`).
- **INV-04 impact (Snapshot Safety)**: UNKNOWN/BLOCKER. `InMemoryStateDb` supports `Clone` and `commit`, but no explicit runtime snapshot/rollback coordinator is wired (`crates/state/src/db.rs`, `crates/whirlpool-node/src/main.rs`).
- **INV-05 impact (Commit Atomicity)**: BLOCKER. `commit` exists, but finalize-to-commit integration is not visible through current consensus app contract and node wiring (`crates/state/src/db.rs::InMemoryStateDb::commit`, `crates/consensus/src/app.rs::ConsensusApp`, `crates/whirlpool-node/src/main.rs`).
- **INV-06 impact (Root Consistency)**: BLOCKER for non-empty blocks because proposal path currently hardcodes empty execution artifacts (`crates/app-evm/src/executor.rs::EvmApplication::propose`).
- **INV-07 impact (Proposal Determinism)**: currently trivial for empty-block behavior; UNKNOWN for non-empty tx ordering/policy because no tx selection policy contract is present (`crates/app/src/traits.rs::TxSource`, `crates/app-evm/src/executor.rs::EvmApplication::propose`).
