# eth-jsonrpc — Execution Plan

## Execution Order

### Wave 1 (sequential)
- [ ] Task 01: Add RPC dependencies to Cargo manifests [**S**] → [tasks/01-add-rpc-dependencies.md](tasks/01-add-rpc-dependencies.md)
- [ ] Task 02 (S1): RPC foundation modules and server bootstrap [**M**] → [tasks/02-rpc-foundation.md](tasks/02-rpc-foundation.md)
- [ ] Task 03 (S2): `eth_chainId` + `eth_gasPrice` handlers [**S**] → [tasks/03-chainid-gasprice.md](tasks/03-chainid-gasprice.md)
- [ ] Task 04 (S3): `eth_getBalance` + `eth_getTransactionCount` handlers [**M**] → [tasks/04-balance-nonce.md](tasks/04-balance-nonce.md)
- [ ] Task 05 (S4): `eth_sendRawTransaction` tx-pool ingress [**M**] → [tasks/05-send-raw-transaction.md](tasks/05-send-raw-transaction.md)
- [ ] Task 06 (S5): `eth_estimateGas` + `eth_getTransactionReceipt` [**M**] → [tasks/06-estimate-gas-receipt.md](tasks/06-estimate-gas-receipt.md)
- [ ] Task 07 (S6): main.rs wiring + alloy integration tests [**L**] → [tasks/07-main-wiring-alloy-e2e.md](tasks/07-main-wiring-alloy-e2e.md)

<!-- TASKS_START -->
1. [01-add-rpc-dependencies](tasks/01-add-rpc-dependencies.md)
2. [02-rpc-foundation](tasks/02-rpc-foundation.md)
3. [03-chainid-gasprice](tasks/03-chainid-gasprice.md)
4. [04-balance-nonce](tasks/04-balance-nonce.md)
5. [05-send-raw-transaction](tasks/05-send-raw-transaction.md)
6. [06-estimate-gas-receipt](tasks/06-estimate-gas-receipt.md)
7. [07-main-wiring-alloy-e2e](tasks/07-main-wiring-alloy-e2e.md)
<!-- TASKS_END -->

## Dependency Graph
```
Task 01 (deps)
  -> Task 02 (S1 foundation)
  -> Task 03 (S2 constants)
  -> Task 04 (S3 state reads)
  -> Task 05 (S4 tx ingress)
  -> Task 06 (S5 gas+receipt)
  -> Task 07 (S6 wiring+alloy e2e)
```

## Design Anchors
- RPC is implemented as modules inside `crates/whirlpool-node/src/rpc/` (not a separate crate).
- Required modules: `mod.rs`, `eth_api.rs`, `eth_handler.rs`, `context.rs`, `receipt_store.rs`, `server.rs`.
- Dependencies: `jsonrpsee = 0.26.0`, `alloy-primitives = 1.5.0`, `alloy-rpc-types = 1.4.3`.
- `EthRpcContext` carries Arc handles to tx pool, state DB, receipt store, chain id, and block height.
- Server starts after `engine.start()` in `crates/whirlpool-node/src/main.rs`.
- Receipt store is in-memory `HashMap<B256, TransactionReceipt>`.
- v1 constants: `eth_estimateGas = 21000`, `eth_gasPrice = 1 gwei`.
- Integration validation uses alloy `ProviderBuilder`.
