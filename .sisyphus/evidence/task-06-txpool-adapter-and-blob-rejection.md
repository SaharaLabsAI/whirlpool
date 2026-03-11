# Task 06 Evidence: TxPool Adapter and Blob Rejection

## Changes Made

### `crates/rpc-eth/src/pool.rs` (NEW)
- Created `WhirlpoolTxPool` struct wrapping `Arc<dyn TxSource>`
- Full `TransactionPool` impl with `type Transaction = EthPooledTransaction`
- Blob rejection: `insert_transaction()` checks `tx_type() == TxType::Eip4844`, returns `PoolError` with descriptive message
- Non-blob transactions: encoded via `encoded_2718()` and forwarded to `TxSource::push()`
- All other methods (pool queries, listeners, blob access) return empty/noop values matching reth's `NoopTransactionPool`

### `crates/rpc-eth/tests/pool_contract.rs` (NEW)
- TST-2a: `pool_satisfies_rpc_node_core_bounds` — type-level assertion WhirlpoolTxPool fits RpcModuleBuilder
- TST-2b: `blob_transactions_are_rejected` — EIP-4844 tx rejected with proper error message, TxSource not called
- TST-2c: `non_blob_transactions_are_forwarded` — EIP-1559 tx accepted, encoded bytes forwarded to TxSource

### `crates/rpc-eth/src/lib.rs`
- Added `pub mod pool;`

### `crates/rpc-eth/Cargo.toml`
- Added `app = { path = "../app" }` to [dependencies]
- Added `rand = "0.8"` and `reth-transaction-pool = { ... features = ["test-utils"] }` to [dev-dependencies]

## Verification

- `nix develop --command cargo build -p rpc-eth` — ✅ passes
- `nix develop --command cargo test -p rpc-eth` — ✅ 24/24 tests pass (17 eth_handler + 4 provider_contract + 3 pool_contract)

## Blob Rejection Behavior
- Type-3 (EIP-4844) transactions rejected at `insert_transaction()` ingress
- Error: `PoolError::other(hash, "blob transactions (EIP-4844) are not supported")`
- TxSource::push() never called for blob transactions
