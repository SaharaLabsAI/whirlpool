# whirlpool-node Design Contract

## 1) Role and purpose
- Role: `binary` crate — top-level EVM consensus node.
- Purpose: wire all subsystems (consensus, EVM, P2P, state) and now JSON-RPC server.
- Scope in this design: add RPC server modules, wire shared state, spawn server alongside consensus engine.

## 2) Existing public API surface
- `pub mod config` — NAMESPACE, BLOCK_INTERVAL, VALIDATOR_SEED, BIND_ADDR constants
- Binary entry point: `main()` in `src/main.rs`

## 3) [PROPOSED] extensions

### New modules
- `src/rpc/mod.rs` — Module root, re-exports
- `src/rpc/eth_api.rs` — `#[rpc(server, namespace = "eth")]` trait definition (EthApiServer) with 7 methods
- `src/rpc/eth_handler.rs` — `EthApiHandler` struct implementing EthApiServer, holds `EthRpcContext`
- `src/rpc/context.rs` — `EthRpcContext` struct: `{ tx_pool: Arc<InMemoryTxPool>, state_db: Arc<RwLock<TestStateDb>>, receipt_store: Arc<RwLock<HashMap<B256, TransactionReceipt>>>, chain_id: u64, block_height: Arc<AtomicU64> }`
- `src/rpc/receipt_store.rs` — In-memory receipt storage indexed by tx hash
- `src/rpc/server.rs` — Server builder: `start_rpc_server(ctx, addr) -> ServerHandle`

### New config constants
- `RPC_BIND_ADDR: &str = "127.0.0.1:8545"` in `src/config.rs`

### New dependencies (Cargo.toml)
- jsonrpsee = { version = "0.26", features = ["server", "macros"] }
- alloy-primitives = { version = "1.5.0", features = ["map-foldhash"] }
- alloy-rpc-types = { version = "1.4.3", features = ["eth"] }
- serde = { version = "1", features = ["derive"] }
- serde_json = "1"

### main.rs wiring changes
1. Clone `state_db` Arc BEFORE passing to EvmApplication
2. Clone `tx_pool` Arc for RPC context
3. Create `receipt_store = Arc::new(RwLock::new(HashMap::new()))`
4. Create `block_height = Arc::new(AtomicU64::new(0))` (or share from FinalizationSink)
5. Construct `EthRpcContext`
6. After `engine.start()`: spawn RPC server task
7. Replace `pending::<()>().await` with select/join on both tasks

## 4) Consumers
- Integration tests in `tests/` directory use alloy client against the RPC server
- External clients (alloy, curl, etc.) connect via HTTP JSON-RPC

## 5) Migration notes
- No breaking changes to existing code paths
- Consensus engine startup is unchanged
- state_db sharing requires cloning Arc before EvmApplication::new() — minor reorder in main.rs
