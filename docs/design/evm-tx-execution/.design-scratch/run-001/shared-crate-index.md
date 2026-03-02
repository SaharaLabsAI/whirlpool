# Shared Crate Index — EVM Transaction Execution

## app-evm (crates/app-evm)

### Purpose
EVM configuration and block execution wrapper. Bridges reth/revm execution engine to the `app::Application` trait.

### Public API Surface

| Symbol | Kind | Signature |
|---|---|---|
| `SAHARA_CHAIN_ID` | const | `u64 = 313_371` |
| `build_sahara_chain_spec()` | fn | `() -> ChainSpec` |
| `WhirlpoolEvmConfig` | struct | `{ inner: EthEvmConfig }` |
| `WhirlpoolEvmConfig::new` | method | `(chain_spec: Arc<ChainSpec>) -> Self` |
| `WhirlpoolEvmConfig::chain_spec` | method | `(&self) -> &ChainSpec` |
| `StateProvider` | trait | `state_root(&self) -> B256` |
| `EvmApplication<DB>` | struct | `{ evm_config, state_db: Arc<RwLock<DB>>, tx_source: Arc<dyn TxSource + Send + Sync> }` |
| `EvmApplication::new` | method | `(evm_config, state_db, tx_source) -> Self` |
| `EvmApplication::genesis` | method | `(&self) -> impl Future<Output=Result<EvmBlock, ApplicationError>>` |
| `EvmApplication::propose` | method | `(&self, parent: &EvmBlock, height: u64) -> impl Future<Output=Result<EvmBlock, ApplicationError>>` |
| `EvmApplication::verify` | method | `(&self, parent: &EvmBlock, block: &EvmBlock) -> impl Future<Output=Result<(), ApplicationError>>` |
| `build_header_from_evm_block` | fn | `(&EvmBlock) -> Header` |
| `build_sealed_header` | fn | `(&EvmBlock) -> SealedHeader` |
| `EvmAppError` | enum | `Execution(String), StateRootMismatch{expected,computed}, State(String), InvalidBlock(String)` |

### Trait Impls
- `WhirlpoolEvmConfig: ConfigureEvm` (delegates to EthEvmConfig)
- `EvmApplication<DB>: Application` where DB: Database + DatabaseRef + StateProvider + Send + Sync
- `EvmAppError: From<EvmAppError> for ApplicationError`

### Current Stubs (to be replaced)
- `propose()`: returns empty EvmBlock with EMPTY_ROOT_HASH for tx/receipts roots, 0 gas, timestamp=parent+12
- `verify()`: only checks state_root matches DB, no re-execution

### Key Dependencies
reth-evm, reth-evm-ethereum, reth-revm, reth-execution-types, reth-execution-errors, reth-primitives-traits, reth-chainspec, reth-ethereum-primitives, alloy-consensus, alloy-eips, alloy-genesis, alloy-primitives, alloy-trie, revm

---

## state (crates/state)

### Purpose
In-memory EVM state database implementing revm's Database trait.

### Public API Surface

| Symbol | Kind | Signature |
|---|---|---|
| `InMemoryStateDb` | struct | `{ accounts: HashMap<Address,DbAccount>, bytecodes: HashMap<B256,Bytecode>, block_hashes: HashMap<u64,B256> }` |
| `InMemoryStateDb::new` | method | `() -> Self` |
| `InMemoryStateDb::with_genesis` | method | `(alloc: &BTreeMap<Address,GenesisAccount>) -> Self` |
| `InMemoryStateDb::commit` | method | `(&mut self, bundle: &BundleState) -> ()` |
| `InMemoryStateDb::state_root` | method | `(&self) -> B256` |
| `InMemoryStateDb::insert_block_hash` | method | `(&mut self, number: u64, hash: B256) -> ()` |
| `StateError` | struct | (unit) |
| `DbAccount` | struct | `{ info: AccountInfo, storage: HashMap<U256, U256> }` |

### Trait Impls
- `InMemoryStateDb: Database<Error=StateError>` — basic_block_hash, basic_account_info, basic_code_by_hash, basic_storage
- `InMemoryStateDb: DatabaseRef<Error=StateError>` — ref versions
- `StateError: core::error::Error + DBErrorMarker`

### commit() Semantics
Iterates `bundle.state`: destroy → clear account+storage; create/update → update nonce/balance/code_hash + storage changes. Iterates `bundle.contracts`: insert bytecodes. Does NOT process reverts, logs, or metadata.

### state_root() Algorithm
Flat keccak256 over lexicographically sorted accounts (address || nonce || balance || code_hash + sorted storage keys/values). Returns KECCAK_EMPTY when empty. NOT Merkle Patricia Trie compatible.

### Gaps / Limitations
- No bytecode cleanup on account destroy
- No revert support (BundleState reverts ignored)
- Flat state root incompatible with canonical Ethereum
- No snapshot/restore capability
