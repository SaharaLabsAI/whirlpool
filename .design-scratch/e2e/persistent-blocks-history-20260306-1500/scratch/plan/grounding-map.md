## app (crates/app/)

### Files
| Path | State | Key Symbols |
|------|-------|-------------|
| crates/app/Cargo.toml | existing | workspace deps: `consensus`, `commonware-consensus`, `commonware-codec`, `commonware-cryptography`, `sha2`, `bytes`, `thiserror`; dev `futures`. |
| crates/app/src/lib.rs | existing | exports `ApplicationAdapter`, `ApplicationError`, `InMemoryTxPool`, `NoopTxSource`, `EvmBlock`, `ExecutionResult`. |
| crates/app/src/types.rs | existing | `ExecutionResult` & `EvmBlock` structs plus `CoreBlock`, `CodecWrite/Read`, `EncodeSize`, `Digestible`, `Committable`, `Heightable`, `VendorBlock` impls; helper `compute_id`/`compute_digest`; tests for codec/trait behavior. |

### What Exists
- `EvmBlock` bundles height/parents/state/tx/receipt roots, gas/timestamp/transactions and implements consensus (CoreBlock) + codec/digest traits used by the runtime.
- `ExecutionResult` carries the four roots and gas/receipt metadata.
- `src/types.rs` already contains unit tests covering trait implementations, codec round-tripping, and `ExecutionResult` fields.

### What's Missing (gaps)
- Design calls for re-exporting `Receipt` from `alloy-consensus`; `src/lib.rs` currently only exposes `EvmBlock`/`ExecutionResult` so the re-export needs to be added.

## state (crates/state/)

### Files
| Path | State | Key Symbols |
|------|-------|-------------|
| crates/state/Cargo.toml | existing | deps: `revm`, `thiserror`, `alloy-genesis`. |
| crates/state/src/lib.rs | existing | re-exports `GenesisAccount`, `StateError`, `StateDb` trait. |
| crates/state/src/traits.rs | existing | `StateDb` trait with `fn new()`, `with_genesis(alloc)`, `state_root() -> Result<B256, Error>`, `commit(&BundleState)`, `get_account`, `get_code_by_hash`, `get_storage`, `get_block_hash`, `insert_account`, `insert_block_hash`. |
| crates/state/src/block_storage.rs | missing | (needs new `BlockStorage` trait/file as per design). |

### What Exists
- `StateDb` trait lays out constructors, state inspection helpers, and commit/insert helpers backed by `BundleState`.
- `StateError` exposes a simple `Internal(String)` variant compatible with `revm::database::DBErrorMarker`.
- No extra tests beyond those embedded in the trait modules.

### What's Missing (gaps)
- `block_storage.rs` is absent; design requires this crate to define the `BlockStorage` trait and export it for downstream consumers.
- No existing implementation of `BlockStorage` or glue code. 

## app-evm (crates/app-evm/)

### Files
| Path | State | Key Symbols |
|------|-------|-------------|
| crates/app-evm/Cargo.toml | existing | deps include `app`, `state`, `state-memory`, `state-reth`, `consensus`, numerous `reth-*` crates, `alloy-*`, `revm`, `thiserror`, `alloy-trie`; dev deps: `futures`, `tokio`. |
| crates/app-evm/src/lib.rs | existing | exposes `build_sahara_chain_spec`, `WhirlpoolEvmConfig`, `SAHARA_CHAIN_ID`, `EvmAppError`. |
| crates/app-evm/src/executor.rs | existing | `build_header_from_evm_block` (private) + `build_sealed_header`, `decode_transactions(raw: &[Vec<u8>]) -> Result<Vec<RecoveredTx>, EvmAppError>`, `EvmApplication<DB>` with `evm_config`, `state_db`, `tx_source`, `new`, `Application` impl (genesis/propose/verify), `RecoveredTx` alias, extensive tests for propose/verify behavior. |

### What Exists
- The executor can create genesis blocks, propose/verify blocks, and decode pending transactions via `decode_transactions` (returning `RecoveredTx`).
- `EvmApplication` wraps a `WhirlpoolEvmConfig`, `RwLock`ed state DB, and tx source, and its `Application` impl calls through to `revm`/`builder` logic, updating the canonical DB inside `propose` and `verify`.
- Tests demonstrate header conversion, transaction decoding, propose/verify flows, and error cases.

### What's Missing (gaps)
- `build_header_from_evm_block` is currently private even though the design requires it to be public for reuse elsewhere.
- `EvmApplication` does not track `pending_receipts` yet; no field or accessor exists to hold receipts that should be persisted when finalizing a block.
- There is no `store_finalized_block` helper; the design needs a method to persist finalized block data (including receipts) when consensus finalizes a block.

## state-reth (crates/state-reth/)

### Files
| Path | State | Key Symbols |
|------|-------|-------------|
| crates/state-reth/Cargo.toml | existing | deps: `state`, `revm`, `reth-db(+mdbx)`, `reth-db-api`, `reth-primitives-traits`, `reth-trie/db`, `reth-trie`, `reth-storage-errors`, `thiserror`, `alloy-genesis`, `alloy-primitives`, `tempfile`, `tracing`. |
| crates/state-reth/src/lib.rs | existing | modules `codec`, `db`, `error`, `init`, `tables`, `trie`; re-exports `RethStateDb`, `RethStateError`, `open_state_db`. |
| crates/state-reth/src/db.rs | existing | `RethStateDb` wraps `reth_db::DatabaseEnv`, exposes `open`, `inner`, implements `StateDb` trait (new/with_genesis/commit/getters/insert), implements `revm::DatabaseRef` + `revm::Database`, uses MDBX transactions and hashed/plain tables, commits bundles + bytecodes, and includes tests for accounts, storage, code, block hashes, state root, and revm behaviors. |
| crates/state-reth/src/block_storage.rs | missing | (needs file implementing `BlockStorage` trait for `RethStateDb`). |

### What Exists
- Persistent MDBX-backed `RethStateDb` provides genesis initialization, commit logic, account/storage/code fetches, block hash storage, and `StateDb` trait coverage.
- `RethStateDb` satisfies `revm::DatabaseRef`/`Database` by delegating to the `StateDb` methods inside read/write transactions.
- Unit tests in `db.rs` exercise inserts, commits, state roots, and revm `Database` behaviors.

### What's Missing (gaps)
- `block_storage.rs` is not present; per design this crate must host the `BlockStorage` implementation that likely wraps MDBX access and satisfies the new trait.
- No `BlockStorage` trait impl currently exists for `RethStateDb`.

## rpc-eth (crates/rpc-eth/)

### Files
| Path | State | Key Symbols |
|------|-------|-------------|
| crates/rpc-eth/Cargo.toml | existing | deps: `app`, `state`, `jsonrpsee(server, macros)`, `alloy-primitives`, `alloy-rpc-types(eth)`, `serde(derive)`, `async-trait`, `tracing`; dev deps: `state-memory`, `revm`, `tokio(full,test-util)`. |
| crates/rpc-eth/src/lib.rs | existing | modules `context`, `eth_api`, `eth_handler`, `receipt_store`, `server`. |
| crates/rpc-eth/src/context.rs | existing | `EthRpcContext<S: StateDb>` holds `tx_pool`, `state_db`, `receipt_store`, `chain_id`, `block_height`; constructor `new(tx_pool, state_db, chain_id)` builds `ReceiptStore` & zeroed height. |
| crates/rpc-eth/src/eth_api.rs | existing | `#[rpc(server, namespace = "eth")]` trait `EthApi` defining `chainId`, `gasPrice`, `getBalance`, `getTransactionCount`, `sendRawTransaction`, `estimateGas`, `getTransactionReceipt`. |
| crates/rpc-eth/src/eth_handler.rs | existing | `EthApiHandler<S>` holding `EthRpcContext<S>`, implements `EthApiServer` with hardcoded gas/gas price, balance/nonce lookups, tx pool push, receipt lookup, `validate_block_id` helper, tests covering RPC surface. |
| crates/rpc-eth/src/receipt_store.rs | existing | thread-safe `ReceiptStore` (RwLock<HashMap<B256, TransactionReceipt>>) with `insert`, `get`, `new`. |
| crates/rpc-eth/src/server.rs | existing | `start_rpc_server(ctx, addr)` builds `EthApiHandler`, starts `jsonrpsee` server. |

### What Exists
- RPC surface implements basic ETH methods (`chainId`, `gasPrice`, balance/nonce queries, raw tx submission, gas estimate, receipt lookup) backed by `EthRpcContext` holding tx pool, state, and a receipt cache.
- `EthApiHandler` runs with `StateDb`, validates `BlockId`, and uses `JsonRpsee` error handling; unit tests cover each method plus error cases.
- `ReceiptStore` caches finalized receipts in memory and is wired into `EthRpcContext`.
- RPC server startup (`start_rpc_server`) wires the handler into a `jsonrpsee` server. 

### What's Missing (gaps)
- `EthRpcContext` only accepts tx pool/state/chain id; design wants it parameterized over `BlockStorage` so block data/receipts can be routed through a persistent store.
- The RPC trait and handler currently expose no block-related endpoints; two new block query methods mentioned in the design are not implemented.

## whirlpool-node (crates/whirlpool-node/)

### Files
| Path | State | Key Symbols |
|------|-------|-------------|
| crates/whirlpool-node/Cargo.toml | existing | deps: `app`, `app-evm`, `rpc-eth`, `reth-revm`, `state`, `state-memory`, `state-reth`, `revm`, `alloy-primitives`, `consensus`, `consensus-simplex`, `p2p-commonware`, `commonware-cryptography`, `commonware-runtime`, `sha2`, `bytes`, `tokio`, `tracing`, `tracing-subscriber`. |
| crates/whirlpool-node/src/main.rs | existing | `height` counter + `FinalizationSink`, opens `state_reth::open_state_db`, constructs `WhirlpoolEvmConfig`, `InMemoryTxPool`, `EvmApplication`, wraps app in `ApplicationAdapter`, launches `CommonwareEngine`, builds `EthRpcContext::new(tx_pool, state_db, SAHARA_CHAIN_ID)`, starts RPC server, keeps runtime alive. |

### What Exists
- Entry point initializes tracing, persistent MDBX state, `EvmApplication`, consensus engine, and RPC server; `FinalizationSink` shared with consensus. 
- `EthRpcContext::new` is invoked with tx pool, shared state DB, and chain id before `start_rpc_server`. 
- No other node-level wrappers or persistence abstractions exist besides `FinalizationSink`. 

### What's Missing (gaps)
- Design requires a `PersistingFinalizationSink` wrapper that ensures finalized blocks/receipts are persisted; current main just instantiates `FinalizationSink` directly.
- `EthRpcContext` constructor is not yet updated to accept the new `BlockStorage` dependency; main still calls the old `new` signature.
