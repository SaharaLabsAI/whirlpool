# SKILL_DIGEST

## Grounded
- **Workspace**: 9 crates, Rust 2021, resolver 2. Cargo only via `nix develop --command`. — *Grounded (Cargo.toml)*
- **whirlpool-node bin**: Wires consensus engine + EVM app. No RPC server exists. Blocks on `pending::<()>().await`. — *Grounded (crates/whirlpool-node/src/main.rs)*
- **InMemoryTxPool**: `Arc<Mutex<Vec<Vec<u8>>>>`, `push(tx)` + `pending()` (drain). Already `Arc`-wrapped in main.rs. Thread-safe. — *Grounded (crates/app/src/tx_source.rs)*
- **StateDb**: `get_account(addr) -> Option<AccountInfo{balance: U256, nonce: u64, code_hash: B256}>`. State is `Arc<RwLock<TestStateDb>>`. — *Grounded (crates/state/src/traits.rs)*
- **No JSON-RPC deps**: Zero jsonrpsee/jsonrpc usage in workspace crates. — *Grounded (grep search)*
- **Receipts gap**: BlockExecutor::finish() generates receipts but DROPS them after computing receipts_root. — *Grounded (crates/app-evm/src/executor.rs)*
- **Chain ID**: SAHARA_CHAIN_ID = 313371 — *Grounded (crates/app-evm/src/config.rs)*
- **Design phase**: PASS verdict. 15 design docs produced. RPC as modules in whirlpool-node. jsonrpsee 0.26 + alloy. — *Grounded (design-phase-digest.md)*
- **Prove phase**: PASS verdict. 12 AC, 5 QA, 5 INV, 0 XINV. Auto-approved (0 challenges, 0 ungrounded blockers). — *Grounded (prove-phase-digest.md)*

## [PROPOSED]
- RPC modules inside whirlpool-node (not separate crate)
- EthRpcContext struct holding Arc refs to tx_pool, state_db, receipt_store, block_height, chain_id
- 7 eth_* methods via jsonrpsee proc macro
- Hardcoded gas price (1 gwei) and estimate (21000) for v1
- In-memory ReceiptStore (HashMap<B256, Receipt>)
- alloy integration tests

## Unknowns
- None remaining after exploration
