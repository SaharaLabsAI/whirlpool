# INTENT

## Objective
Design (no implementation) how to add an Ethereum JSON-RPC server to `whirlpool-node` so an alloy client can submit and verify basic ETH transfers in integration tests.

## Scope
- In scope crates: `app`, `whirlpool-node`.
- Runtime integration point: node binary lifecycle in `crates/whirlpool-node/src/main.rs::main`.
- Required RPC methods:
  - `eth_chainId`
  - `eth_getBalance(address, block_id?)`
  - `eth_getTransactionCount(address, block_id?)`
  - `eth_estimateGas(tx_request, block_id?)`
  - `eth_gasPrice`
  - `eth_sendRawTransaction(bytes)`
  - `eth_getTransactionReceipt(hash)`

## Grounded facts
- **Grounded**: The node currently has no RPC server and waits indefinitely after starting consensus (`crates/whirlpool-node/src/main.rs::main`).
- **Grounded**: Node owns shareable handles required for RPC (`Arc<InMemoryTxPool>`, `Arc<RwLock<TestStateDb>>`) (`crates/whirlpool-node/src/main.rs::main`).
- **Grounded**: `InMemoryTxPool` is thread-safe and supports raw tx push + drain (`crates/app/src/tx_source.rs::InMemoryTxPool`).
- **Grounded**: Sahara chain id constant is defined as `313_371` (`crates/app-evm/src/config.rs::SAHARA_CHAIN_ID`).
- **Grounded**: Vendor reth uses `jsonrpsee` macro-based namespaces and version `0.26.0` (`vendor/reth/examples/node-custom-rpc/src/main.rs`, `vendor/reth/Cargo.toml`).

## [PROPOSED] design target
- Add node-local `eth` RPC module(s) in `whirlpool-node`, started after consensus engine startup.
- Keep existing 3-layer architecture intact: no RPC coupling into `consensus` traits or simplex adapter.
- Implement a minimal receipt tracking/index strategy suitable for transfer integration tests.

## Success criteria
- SC-01: A node-local `eth` namespace design exists with all 7 required methods and typed request/response contracts (validated by `TC-001`..`TC-008`).
- SC-02: RPC lifecycle is explicitly wired into node runtime without changing consensus trait boundaries (validated by `TC-009`).
- SC-03: `eth_sendRawTransaction` design routes raw bytes into shared tx pool and returns tx hash deterministically (validated by `TC-006`).
- SC-04: `eth_getBalance` and `eth_getTransactionCount` read account state via node-held state DB handle (validated by `TC-002`, `TC-003`).
- SC-05: `eth_estimateGas` has a defined behavior for transfer requests and error path boundaries (validated by `TC-004`).
- SC-06: `eth_getTransactionReceipt` has explicit pending/confirmed semantics for polling clients (validated by `TC-007`, `TC-008`).
- SC-07: End-to-end alloy-provider transfer flow is test-contracted from send to confirmation + balance delta (validated by `TC-009`, `TC-010`).

## Assumptions
- **[PROPOSED]** Initial implementation targets integration-test correctness over full Ethereum RPC parity.
- **[PROPOSED]** Block tags unsupported by current state model beyond `latest`/`pending` can return explicit RPC errors instead of silent fallback.
- **[PROPOSED]** Receipt data may be synthesized from execution-observable data available at node layer until persistent receipt storage exists.

## Out of scope
- Full Ethereum RPC surface beyond the 7 required methods.
- Historical state queries requiring archival/state-versioned DB.
- P2P transaction gossip behavior changes.
- Any source-code implementation in this design pass.
