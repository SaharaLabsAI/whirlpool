# Shared Domain Map — EVM Transaction Execution

## Domains

### 1. EVM Execution Domain
**Owner crate**: `app-evm`
**Responsibility**: Execute EVM transactions against state, produce execution results (receipts, gas used, state changes).

**Key entities**:
- `EvmApplication<DB>` — orchestrator: receives raw tx bytes, configures EVM, executes, returns block
- `WhirlpoolEvmConfig` — EVM configuration (chain spec, hardfork rules, ConfigureEvm delegation)
- reth `BasicBlockExecutor` / `BlockBuilder` — actual execution engine (from reth-evm)

**Boundaries**:
- IN: raw tx bytes from `TxSource` (via `app::TxSource` trait)
- IN: parent block header (from consensus via `Application::propose`)
- IN: state DB (from `state::InMemoryStateDb` via `Arc<RwLock<DB>>`)
- OUT: `EvmBlock` with filled fields (to consensus via `Application::propose`)
- OUT: `ExecutionResult` (to verify flow via `Application::verify`)
- OUT: `BundleState` (to state domain for commit)

### 2. State Management Domain
**Owner crate**: `state`
**Responsibility**: Maintain in-memory EVM state (accounts, storage, bytecodes), provide revm Database interface, compute state root, commit execution results.

**Key entities**:
- `InMemoryStateDb` — canonical state store
- `commit(&BundleState)` — apply execution results to state
- `state_root()` — compute deterministic hash of state

**Boundaries**:
- IN: `BundleState` from EVM execution (via commit)
- IN: read queries from EVM (via `Database` trait during execution)
- OUT: state root hash (via `StateProvider` trait / `state_root()`)
- OUT: account/storage/bytecode data (via `Database`/`DatabaseRef` during execution)

## Cross-Domain Wiring

| Source | Target | Interface | Data |
|---|---|---|---|
| EVM Execution → State Mgmt | `Database` trait | Account reads, storage reads during execution |
| EVM Execution → State Mgmt | `InMemoryStateDb::commit()` | BundleState after execution |
| State Mgmt → EVM Execution | `StateProvider::state_root()` | B256 hash for block assembly |
| Consensus (out-of-scope) → EVM Execution | `Application::propose/verify` | parent EvmBlock, height |
| EVM Execution → Consensus (out-of-scope) | `Application::propose` return | EvmBlock with execution results |
