### Crate Index with Purpose and Domain Ownership:

| Crate | Path | Domain | Purpose |
|---|---|---|---|
| `whirlpool-node` | `crates/whirlpool-node` | Block Production | Node binary, runtime wiring, consensus engine startup, TestStateDb bridge |
| `app-evm` | `crates/app-evm` | EVM Execution | Concrete Application impl (EvmApplication), EVM config, header conversion |
| `app` | `crates/app` | Application Layer | Application trait, TxSource, EvmBlock/ExecutionResult types, ApplicationAdapter bridge |
| `state` | `crates/state` | State Management | InMemoryStateDb, DbAccount, state commit, state root derivation |

### Per-Crate Public Contract Highlights:

**whirlpool-node** (crates/whirlpool-node/src/):
- `lib.rs`: pub mod app, block, config
- `config.rs`: NAMESPACE=b"sahara-chain-v0", BLOCK_INTERVAL=5s, BIND_ADDR="127.0.0.1:0", VALIDATOR_SEED=0u64
- `block.rs`: EmptyBlock{height:u64, parent_id:[u8;32]} — impls CoreBlock, CodecWrite/Read, Digestible, Committable, Heightable, VendorBlock. genesis(), new(height,parent_id), compute_id() via sha256.
- `app.rs`: EmptyBlockApp (zero-sized) — impl ConsensusApp for EmptyBlock: genesis at height 0, propose increments height, verify enforces 5 rules (height, parent_id, self-reference, genesis constraints)
- `main.rs`: TestStateDb(InMemoryStateDb) wrapper implementing StateProvider + revm::Database. main() wires: tracing, AtomicU64 height, FinalizationSink, tokio::Runner, ed25519 signer(VALIDATOR_SEED), CommonwareNetworkProviderBuilder, CommonwareConfig (leader_timeout=5s, notarization_timeout=5s, nullify_retry=500ms, activity_timeout=10, skip_timeout=5, mailbox_size=100, replay_buffer=100, write_buffer=100, epoch=0, fetch_timeout=5s, fetch_concurrent=4), Arc<RwLock<TestStateDb::new()>>, build_sahara_chain_spec(), WhirlpoolEvmConfig, Arc::new(NoopTxSource), EvmApplication::new(...), Arc::new(ApplicationAdapter::new(evm_app)), CommonwareEngine::new(...).start().
- INV-05 (Commit Atomicity): BLOCKER — no finalize-to-commit callback visible in node wiring

**app-evm** (crates/app-evm/src/):
- `lib.rs`: pub mod config, error, executor. Re-exports: SAHARA_CHAIN_ID, WhirlpoolEvmConfig, build_sahara_chain_spec, EvmAppError
- `config.rs`: SAHARA_CHAIN_ID=313_371. build_sahara_chain_spec() -> ChainSpec (chain=SAHARA_CHAIN_ID, gas_limit=30M, difficulty=ZERO, cancun_activated). WhirlpoolEvmConfig{inner:EthEvmConfig}, new(Arc<ChainSpec>), chain_spec(). impl ConfigureEvm: Primitives=EthPrimitives, Error=Infallible, delegates all to inner.
- `executor.rs`: StateProvider trait {fn state_root()->B256}. Helper fns: build_header_from_evm_block (gas_limit=30M, difficulty=ZERO), build_sealed_header. EvmApplication<DB>{evm_config, state_db:Arc<RwLock<DB>>, tx_source:Arc<dyn TxSource+Send+Sync>}. impl Application where DB:StateProvider+Clone+Send+Sync+'static: Block=EvmBlock, Result=ExecutionResult, Error=EvmAppError. genesis(): reads state_root, block 0 with EMPTY_ROOT_HASH. propose(): MVP empty block (timestamp=parent+12, empty txs, EMPTY_ROOT_HASH roots, gas=0). verify(): reads state_root, compares to block.state_root, returns StateRootMismatch on mismatch.
- `error.rs`: EvmAppError{Execution(String), StateRootMismatch{expected/computed:[u8;32]}, State(String), InvalidBlock(String)}. impl From<EvmAppError> for ApplicationError.
- INV-01 (Execution Visibility): BLOCKER in propose() — empty block, no tx execution
- INV-02 (Verification Integrity): BLOCKER in verify() — only checks state_root, no tx/receipt/gas replay
- INV-03 (Verification Read-Only): Grounded — verify uses read lock only
- INV-04 (Snapshot Safety): UNKNOWN/BLOCKER — no snapshot/rollback orchestration
- INV-06 (Root Consistency): BLOCKER — roots hardcoded to EMPTY_ROOT_HASH
- INV-07 (Proposal Determinism): Trivially satisfied for empty blocks, UNKNOWN for non-empty

**app** (crates/app/src/):
- `lib.rs`: pub mod adapter, error, traits, types. Re-exports: ApplicationAdapter, ApplicationError, Application, NoopTxSource, TxSource, EvmBlock, ExecutionResult
- `traits.rs`: Application trait (Send+Sync+Clone+'static): assoc types Block:consensus::Block, Result:Clone+Send, Error:Error+Send+Sync. Methods: genesis()->Block, propose(parent,height)->Result<(Block,Result),Error>, verify(parent,block)->Result<Result,Error>. TxSource trait: pending()->Vec<Vec<u8>>. NoopTxSource: returns empty vec.
- `adapter.rs`: ApplicationAdapter<A:Application<Block=EvmBlock>>{inner:A}. new(app), inner(). impl ConsensusApp: Block=EvmBlock, genesis delegates, propose maps Ok((block,_))->Some(block) / Err->None, verify maps Ok->Ok(()) / Err->ConsensusError::InvalidBlock(err.to_string()).
- `types.rs`: ExecutionResult{state_root, receipts_root, gas_used, receipt_count}. EvmBlock{height, parent_id, state_root, transactions_root, receipts_root, gas_used, timestamp, transactions:Vec<Vec<u8>>}. compute_id() sha256(height+parent_id+state_root+transactions_root). Impls: CoreBlock, Codec, Digestible, Committable, Heightable, VendorBlock.
- `error.rs`: ApplicationError{Execution(String), Verification(String), State(String)}

**state** (crates/state/src/):
- `lib.rs`: pub mod db, error. Re-exports: DbAccount, InMemoryStateDb, StateError
- `db.rs`: DbAccount{info:AccountInfo, storage:HashMap<U256,U256>}. InMemoryStateDb{accounts:HashMap<Address,DbAccount>, bytecodes:HashMap<B256,Bytecode>, block_hashes:HashMap<u64,B256>}. impl Default. Methods: new(), with_genesis(HashMap<Address,GenesisAccount>), commit(&mut self, &BundleState) — handles destroy/update/create/storage/bytecodes, state_root()->B256 (sorted accounts+storage, keccak256, KECCAK_EMPTY for empty), insert_block_hash(). impl DatabaseRef (Error=StateError), impl Database (delegates to DatabaseRef).
- `error.rs`: StateError::Internal(String). impl DBErrorMarker for StateError.
- INV-03 (Verification Read-Only): Grounded — DatabaseRef is &self
- INV-04 (Snapshot Safety): Clone is derived on InMemoryStateDb
- INV-05 (Commit Atomicity): commit() method exists, but finalize-to-commit integration absent
- INV-06 (Root Consistency): state_root() is deterministic (sorted keccak256)

### Provider/Swap-Point Hints from Wiring:

| Swap Point | Trait | Current Provider | Location |
|---|---|---|---|
| TxSource | `app::TxSource` | `NoopTxSource` | `crates/whirlpool-node/src/main.rs` |
| StateProvider | `app_evm::executor::StateProvider` | `TestStateDb` | `crates/whirlpool-node/src/main.rs` |
| Database | `revm::Database` | `TestStateDb` (delegates to InMemoryStateDb) | `crates/whirlpool-node/src/main.rs` |
| ConfigureEvm | `reth_evm::ConfigureEvm` | `WhirlpoolEvmConfig` (wraps EthEvmConfig) | `crates/app-evm/src/config.rs` |
| ConsensusApp | `consensus::ConsensusApp` | `ApplicationAdapter<EvmApplication<TestStateDb>>` | `crates/whirlpool-node/src/main.rs` |

### Key Flow Entry Points:

| Flow | Entry | Crates | Status |
|---|---|---|---|
| Block Proposal | ConsensusApp::propose → ApplicationAdapter::propose → EvmApplication::propose | whirlpool-node, app, app-evm, state | BLOCKER: empty blocks only |
| Block Verification | ConsensusApp::verify → ApplicationAdapter::verify → EvmApplication::verify | whirlpool-node, app, app-evm, state | BLOCKER: state_root check only |
| Genesis | ConsensusApp::genesis → ApplicationAdapter::genesis → EvmApplication::genesis | whirlpool-node, app, app-evm, state | Grounded |
| State Commitment | InMemoryStateDb::commit (exists but no finalize trigger) | state | BLOCKER: no finalize callback |
| Runtime Bootstrap | main() → engine.start() | whirlpool-node | Grounded |

### Explicit Unknowns and Blocker Candidates:

- BLOCKER: EvmApplication::propose() is MVP stub — no tx execution (INV-01, INV-06, INV-07)
- BLOCKER: EvmApplication::verify() only checks state_root — no tx/receipt/gas replay (INV-02)
- BLOCKER: Only NoopTxSource exists — no real tx ingress (INV-01)
- BLOCKER: No finalize-to-commit callback in ConsensusApp or node wiring (INV-05)
- UNKNOWN/BLOCKER: Snapshot/rollback orchestration not explicit in runtime wiring (INV-04)
- UNKNOWN: Tx decode/validation path from Vec<u8> to executable tx not in scope
- UNKNOWN: Deterministic tx ordering policy for non-empty proposals (INV-07)
- NOTE: state_root uses simplified hash not MPT (documented as out of scope)

### Invariant Reference Table (INV-01 through INV-07):

| INV | Name | Primary Crate(s) | Status | Evidence |
|---|---|---|---|---|
| INV-01 | Execution Visibility | app-evm (propose) | BLOCKER | MVP empty block stub |
| INV-02 | Verification Integrity | app-evm (verify) | BLOCKER | state_root check only |
| INV-03 | Verification Read-Only | app-evm (verify), state | Grounded | read lock + DatabaseRef |
| INV-04 | Snapshot Safety | state, app-evm | UNKNOWN/BLOCKER | Clone exists but no orchestration |
| INV-05 | Commit Atomicity | state, whirlpool-node | BLOCKER | commit() exists, no finalize trigger |
| INV-06 | Root Consistency | app-evm, state | BLOCKER | hardcoded EMPTY_ROOT_HASH in propose |
| INV-07 | Proposal Determinism | app-evm | Trivially OK / UNKNOWN | empty blocks trivial, non-empty TBD |

