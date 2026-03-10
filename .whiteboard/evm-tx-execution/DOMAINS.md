# Domains — EVM Transaction Execution

## Domain Model

### Domain 1: EVM Execution

**Owner crate**: `app-evm`

**Responsibility**: Receive raw transaction bytes, configure the EVM environment, execute transactions against state, produce execution results (receipts, gas used, state diff), and assemble EVM blocks.

**Key Entities** (Grounded):

| Entity | Location | Role |
|---|---|---|
| `EvmApplication<DB>` | `crates/app-evm/src/executor.rs` | Orchestrator: holds evm_config + state_db + tx_source |
| `WhirlpoolEvmConfig` | `crates/app-evm/src/config.rs` | Wraps `EthEvmConfig`, delegates `ConfigureEvm` |
| `EvmAppError` | `crates/app-evm/src/error.rs` | Error types: Execution, StateRootMismatch, State, InvalidBlock |
| `Application` trait | `crates/app/src/traits.rs` | Interface: genesis(), propose(), verify() |
| `EvmBlock` | `crates/app/src/types.rs` | Block type with height, parent_id, state_root, tx_root, receipts_root, gas_used, timestamp, transactions |
| `ExecutionResult` | `crates/app/src/types.rs` | Result struct: state_root, receipts_root, gas_used, receipt_count |

**Key Entities** [PROPOSED]:

| Entity | Role |
|---|---|
| `reth_revm::State<DB>` | Wraps InMemoryStateDb for reth executor compatibility |
| `BasicBlockExecutor` | Batch block execution for verify path (from `vendor/reth/crates/evm/evm/src/execute.rs`) |
| `BlockBuilder` | Incremental block building for propose path (from `vendor/reth/crates/evm/evm/src/execute.rs`) |
| `TransactionSigned` | Decoded/recovered transaction (from alloy/reth primitives) |

**Operations**:
- **propose**: fetch txs → decode → build block via reth BlockBuilder → extract BundleState → return EvmBlock
- **verify**: decode block txs → re-execute via BasicBlockExecutor → compare results → return Ok/Err
- **genesis**: return empty block with initial state_root (Grounded, no changes needed)

### Domain 2: State Management

**Owner crate**: `state`

**Responsibility**: Maintain in-memory EVM state (accounts, storage, bytecodes), provide revm Database interface for EVM execution reads, commit execution results (BundleState), and compute deterministic state root.

**Key Entities** (Grounded):

| Entity | Location | Role |
|---|---|---|
| `InMemoryStateDb` | `crates/state/src/db.rs` | HashMap-based state store: accounts, bytecodes, block_hashes |
| `DbAccount` | `crates/state/src/db.rs` | Account record: info (AccountInfo) + storage (HashMap<U256,U256>) |
| `StateError` | `crates/state/src/error.rs` | Error type implementing DBErrorMarker |

**Operations**:
- **commit**: Apply `BundleState` to state (account create/update/destroy, storage changes, bytecode insertion). Grounded: `crates/state/src/db.rs::InMemoryStateDb::commit`
- **state_root**: Compute flat keccak256 over sorted account data. Grounded: `crates/state/src/db.rs::InMemoryStateDb::state_root`
- **Database reads**: Provide account info, storage, bytecodes, block hashes to EVM during execution. Grounded: `Database` + `DatabaseRef` impls
- **insert_block_hash**: Record block number → hash mapping. Grounded: `crates/state/src/db.rs::InMemoryStateDb::insert_block_hash`

**[PROPOSED] Operations**:
- **snapshot**: Clone state before speculative execution (propose). Clone-based approach for MVP.
- **swap**: Replace canonical state with successfully committed snapshot after finalization.

## Boundary Contracts

### EVM Execution ↔ State Management

| Direction | Interface | Data | Grounded/Proposed |
|---|---|---|---|
| Execution reads State | `revm::Database` trait | `basic_block_hash`, `basic_account_info`, `basic_code_by_hash`, `basic_storage` | Grounded |
| Execution reads State | `revm::DatabaseRef` trait | Ref-counted read variants | Grounded |
| Execution commits to State | `InMemoryStateDb::commit(&BundleState)` | Account changes, storage diffs, bytecodes | Grounded |
| State provides root to Execution | `StateProvider::state_root() -> B256` | Flat keccak256 hash | Grounded |
| Execution inserts block hash | `InMemoryStateDb::insert_block_hash(u64, B256)` | Block number and hash | Grounded |

### EVM Execution ↔ Consensus (out-of-scope boundary)

| Direction | Interface | Data | Notes |
|---|---|---|---|
| Consensus calls Execution | `Application::propose(&parent, height)` | Parent EvmBlock, target height | Via ApplicationAdapter |
| Consensus calls Execution | `Application::verify(&parent, &block)` | Parent + candidate EvmBlock | Via ApplicationAdapter |
| Execution returns to Consensus | `Result<EvmBlock>` / `Result<()>` | Assembled block or validation result | ExecutionResult dropped by adapter |

### EVM Execution ↔ TxSource (out-of-scope boundary)

| Direction | Interface | Data | Notes |
|---|---|---|---|
| Execution reads TxSource | `TxSource::pending() -> Vec<Vec<u8>>` | Raw transaction bytes | Currently NoopTxSource; real impl deferred to Sub-Intent 2 |

## Domain Invariants

1. **State consistency**: After `commit(bundle)`, `state_root()` MUST reflect all changes in the bundle (Grounded: tested in `crates/state/src/db.rs` tests)
2. **Deterministic execution**: Given the same parent state + transactions, execution MUST produce identical `state_root`, `tx_root`, `receipts_root`, `gas_used` (required for propose/verify agreement)
3. **Verify isolation**: `verify()` MUST NOT mutate canonical state [PROPOSED] — verifier operates on snapshot/clone
4. **Block field correctness**: `tx_root` = Ethereum-standard transaction trie root; `receipts_root` = Ethereum-standard receipt trie root [PROPOSED]
