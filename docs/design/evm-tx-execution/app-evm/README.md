# Crate Contract — app-evm

## 1. Purpose

EVM configuration and block execution wrapper. Bridges reth/revm execution engine to the `app::Application` trait for the Sahara chain (chain ID 313371, 30M gas limit, Cancun hardfork).

**Primary change target** for this design: replace empty-block stubs in `propose()` and `verify()` with real EVM transaction execution.

## 2. Public API

| Symbol | Kind | Signature | Status |
|---|---|---|---|
| `SAHARA_CHAIN_ID` | const | `u64 = 313_371` | Grounded |
| `build_sahara_chain_spec()` | fn | `() -> ChainSpec` | Grounded |
| `WhirlpoolEvmConfig` | struct | `{ inner: EthEvmConfig }` | Grounded |
| `WhirlpoolEvmConfig::new` | method | `(Arc<ChainSpec>) -> Self` | Grounded |
| `StateProvider` | trait | `fn state_root(&self) -> B256` | Grounded |
| `EvmApplication<DB>` | struct | `{ evm_config, state_db: Arc<RwLock<DB>>, tx_source: Arc<dyn TxSource> }` | Grounded |
| `EvmApplication::propose` | method | `(&self, &EvmBlock, u64) -> Result<EvmBlock, ApplicationError>` | Grounded (stub) |
| `EvmApplication::verify` | method | `(&self, &EvmBlock, &EvmBlock) -> Result<(), ApplicationError>` | Grounded (stub) |
| `build_header_from_evm_block` | fn | `(&EvmBlock) -> Header` | Grounded |
| `build_sealed_header` | fn | `(&EvmBlock) -> SealedHeader` | Grounded |
| `EvmAppError` | enum | `Execution, StateRootMismatch, State, InvalidBlock` | Grounded |
| [PROPOSED] `decode_transactions` | fn | `(&[Vec<u8>]) -> Result<Vec<RecoveredTx>>` | Helper for tx decode + sender recovery |

## 3. Dependencies

**Internal**: `app` (Application trait, EvmBlock, TxSource), `state` (InMemoryStateDb), `consensus` (Block trait)

**Vendor**: reth-evm (ConfigureEvm, BlockBuilder, BasicBlockExecutor), reth-evm-ethereum (EthEvmConfig), reth-revm (State wrapper), reth-execution-types (BundleState), reth-primitives-traits (Header), reth-chainspec, alloy-primitives, alloy-consensus, alloy-trie, revm

## 4. Changes Required

### propose() — Replace empty-block stub

**Current** (Grounded: `crates/app-evm/src/executor.rs::EvmApplication::propose`):
Returns empty EvmBlock with `EMPTY_ROOT_HASH` for tx/receipts roots, 0 gas, timestamp = parent + 12.

**[PROPOSED]**:
1. Fetch raw txs via `self.tx_source.pending()`
2. Decode + recover senders via `TransactionSigned::decode_2718` + `try_recover`
3. Clone `InMemoryStateDb` for snapshot safety
4. Wrap clone in `reth_revm::State<DB>`
5. `evm_config.builder_for_next_block(&mut state, &parent_header, attrs)`
6. `builder.apply_pre_execution_changes()`
7. For each tx: `builder.execute_transaction(tx)` — skip failures
8. Extract `BundleState` via `state.take_bundle()`
9. Commit BundleState to canonical `InMemoryStateDb`
10. Compute `state_root()`, `tx_root`, `receipts_root` from results
11. Assemble and return `EvmBlock`

### verify() — Replace state-root-only check

**Current** (Grounded): Only checks `state_root` matches DB state.

**[PROPOSED]**:
1. Decode + recover txs from `block.transactions`
2. Clone state for isolation (do NOT commit to canonical)
3. Re-execute all txs via `BasicBlockExecutor::execute_one`
4. Compare computed `state_root`, `tx_root`, `receipts_root`, `gas_used` vs block
5. Return `Err(StateRootMismatch)` or `Err(InvalidBlock)` on mismatch

## 5. Test Seams

| Test | Type | Boundary |
|---|---|---|
| propose returns correct block fields after execution | Unit | Real EVM, mock TxSource |
| verify accepts valid block | Unit | Real EVM, real block from propose |
| verify rejects block with wrong state_root | Unit | Tampered block fields |
| tx decode handles invalid bytes gracefully | Unit | Pure function |
| propose skips invalid transactions | Unit | Mix of valid/invalid txs |
| propose + verify round-trip consistency | Integration | Full propose → verify cycle |
