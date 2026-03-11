# Domain Boundaries

## Primary Domain: `rpc-eth` (Adapter + Server Composition)

### Responsibilities
- Expose Ethereum JSON-RPC over HTTP for Whirlpool node runtime.
- Compose reth RPC modules (`EthApi` path) through `RpcModuleBuilder`.
- Own adapter boundaries that translate Whirlpool traits/data into reth RPC contracts.

### Non-responsibilities
- State persistence ownership (`state`, `state-reth`).
- Transaction execution semantics (`app-evm`).
- Consensus/finalization ownership (`consensus-*`).
- P2P networking ownership (`p2p-*`).

## Adapter Subdomains

### `WhirlpoolProvider` subdomain
- Input boundaries:
  - `state::StateDb`
  - `state::BlockStorage`
  - `state-reth::RethStateDb` storage substrate
- Output boundaries:
  - reth provider traits required by RPC build bounds
- Data ownership:
  - read-only projection for RPC methods
  - no state mutation ownership transfer

### `WhirlpoolTxPool` subdomain
- Input boundary: `app::traits::TxSource` (`push`, `pending`)
- Output boundary: `reth_transaction_pool::TransactionPool`
- Ownership notes:
  - transaction ingestion/pending view adapter only
  - advanced mempool management semantics remain out-of-domain for Whirlpool unless explicitly added later

### `WhirlpoolNetwork` subdomain
- Input boundary: node config/static metadata
- Output boundaries:
  - `reth_network_api::NetworkInfo`
  - `reth_network_api::Peers`
- Ownership notes:
  - no P2P session manager ownership in `rpc-eth`
  - RPC trait satisfaction only

## Integration Domain: `whirlpool-node`

### Current boundary (legacy)
`whirlpool-node` currently constructs `EthRpcContext` and calls `rpc::server::start_rpc_server(ctx, bind_addr)`.

### Planned boundary (design target)
`whirlpool-node` remains the startup owner and injects adapter dependencies required by new `start_rpc_server` composition path.

Expected boundary-level changes:
- drop direct dependency on legacy context/handler model
- inject adapter-ready handles (state/block source, tx source, chain/network config)
- retain node lifecycle behavior (threading, runtime, consensus startup order)

## Flow Ownership (High-Level)

1. RPC request enters reth HTTP server module.
2. reth `EthApi` handler dispatches method.
3. Method pulls data through one of the three adapters.
4. Adapter delegates into Whirlpool-owned storage/mempool traits.
5. Response serialization is owned by reth RPC stack.

## Contracted Behavior Boundary: Blob Exclusion
Blob/EIP-4844 behavior is intentionally outside this domain's supported feature set.

- `eth_blobBaseFee`: exposed by upstream API surface but contracted as unsupported for Whirlpool integration.
- Type-3 blob transaction submission: rejected at `WhirlpoolTxPool` boundary.
- No blob sidecar or blob-fee execution pipeline ownership is assumed by this domain.

## Failure and Error Boundary
- Adapter-level unsupported behavior should map to explicit JSON-RPC style errors (not silent fallbacks).
- Stub/noop trait surfaces must remain deterministic and non-panicking.
- Errors originating from Whirlpool backends should preserve enough context for operator debugging at RPC boundary logs.
