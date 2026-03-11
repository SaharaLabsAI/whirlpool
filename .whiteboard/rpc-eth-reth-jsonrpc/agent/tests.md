# Test Contracts (QA Baseline)

The following protected tests define expected behavior for the synth design.

## Adapter Contract Tests

- TST-1: `WhirlpoolProvider` satisfies reth provider build bounds used by `RpcModuleBuilder` (`FullRpcProvider`, `CanonStateSubscriptions`, `PersistedBlockSubscriptions`, `AccountReader`, `ChangeSetReader`).

- TST-2: `WhirlpoolProvider` real read paths return data from Whirlpool-backed storage for core methods (chain tip/block/header/account/receipt/transaction lookups used by standard `eth_*`).

- TST-3: `WhirlpoolProvider` stub/noop traits are deterministic and non-panicking (empty/none or explicit unsupported contract, as documented).

- TST-4: `WhirlpoolTxPool` implements `TransactionPool` and bridges `TxSource::push` and `TxSource::pending` for submission/pending-read paths.

- TST-5: `WhirlpoolNetwork` implements both `NetworkInfo` and `Peers`, returning deterministic minimal values (no real peer set required).

## Server Wiring Tests

- TST-6: RPC server composes through reth `RpcModuleBuilder` + `bootstrap_eth_api` + `build` + `RpcServerConfig::http(...).start(...)` and accepts HTTP connections.

- TST-7: Method dispatch for standard `eth_*` requests goes through reth `EthApi` module path (not legacy custom handler path).

## Behavioral RPC Tests

- TST-8: `eth_chainId` returns the configured chain id from adapter-backed runtime configuration.

- TST-9: `eth_blockNumber` returns latest canonical number from adapter-backed block storage.

- TST-10: `eth_getBalance` and block lookup methods (`eth_getBlockByNumber`/`eth_getBlockByHash`) resolve through adapter-backed state/block reads.

- TST-11: `eth_sendRawTransaction` forwards accepted transactions through tx adapter to `TxSource`, and pending transaction RPC surfaces read from `TxSource::pending()`.

## Blob Exclusion Tests (Mandatory)

- TST-12: Blob behavior is explicitly unsupported:
  - `eth_blobBaseFee` returns unsupported-method/unsupported-feature style error contract.
  - Type-3 blob transaction submission is rejected at tx-pool adapter boundary.
  - No blob sidecar handling path is exposed.

## Traceability Map
- REQ-1 -> TST-6, TST-7
- REQ-2 -> TST-1, TST-2, TST-3
- REQ-3 -> TST-4, TST-11
- REQ-4 -> TST-5
- REQ-5 -> TST-12
- REQ-6 -> TST-12
- REQ-7 -> TST-6, TST-8, TST-9, TST-10, TST-11
