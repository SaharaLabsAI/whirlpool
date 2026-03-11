# Crate Design: `rpc-eth`

## Planned Module Layout

```text
crates/rpc-eth/
├── src/
│   ├── lib.rs
│   ├── server.rs
│   ├── provider.rs
│   ├── pool.rs
│   ├── network.rs
│   └── convert.rs
```

- `server.rs`: compose and start reth RPC modules via `reth_rpc_builder::RpcModuleBuilder` and `reth_rpc_builder::RpcServerConfig`.
- `provider.rs`: `WhirlpoolProvider` implementing provider traits required by reth eth RPC build bounds.
- `pool.rs`: `WhirlpoolTxPool` implementing `reth_transaction_pool::TransactionPool` with `TxSource` bridging.
- `network.rs`: `WhirlpoolNetwork` implementing `reth_network_api::NetworkInfo` and `reth_network_api::Peers`.
- `convert.rs`: Whirlpool <-> reth/alloy conversion helpers used by adapters.

## Public Surface (Design Target)

- Preserve `rpc-eth` as the JSON-RPC integration crate consumed by `whirlpool-node`.
- Preserve a single startup entry point (`start_rpc_server`) as the node-facing API.
- Replace legacy internals (`context`, manual `EthApiServer` implementation, custom handler wiring) with reth module composition.

## Build-Bound Contract from `RpcModuleBuilder`

The adapter set must satisfy these grounded bounds in `reth_rpc_builder`:

- Provider:
  - `reth_storage_api::FullRpcProvider`
  - `reth_chain_state::CanonStateSubscriptions`
  - `reth_chain_state::PersistedBlockSubscriptions`
  - `reth_storage_api::AccountReader`
  - `reth_storage_api::ChangeSetReader`
- Pool:
  - `reth_transaction_pool::TransactionPool + Clone + 'static`
- Network:
  - `reth_network_api::NetworkInfo + reth_network_api::Peers + Clone + 'static`

## `WhirlpoolProvider` Design

### Role
Bridge `state-reth` + Whirlpool state/block traits into reth provider traits required by eth RPC wiring.

### Candidate backing handles
- `Arc<state_reth::RethStateDb>` for block/receipt/hash lookup and canonical chain reads
- chain spec handle (`Arc<reth_chainspec::ChainSpec>`) for `ChainSpecProvider`
- subscription handles needed for `CanonStateSubscriptions` / `PersistedBlockSubscriptions` (noop-style if no producer)

### Real-vs-Stub Trait Matrix (Normative)

| Trait | Coverage | Contract |
|---|---|---|
| `BlockHashReader` | Real | Backed by canonical hash lookup (`StateDb::get_block_hash` / block tables). |
| `BlockNumReader` | Real | Backed by latest canonical block number (`BlockStorage::get_latest_block_number`). |
| `HeaderProvider` | Real | Header reads from canonical block/header storage. |
| `BlockReader` | Real | `get_block_by_number` / `get_block_by_hash` backed reads. |
| `BlockReaderIdExt` | Real | Derived from block-number/hash resolvers. |
| `TransactionsProvider` | Real | Reads transaction payloads from stored block bodies. |
| `ReceiptProvider` | Real | Reads receipts from `BlockStorage::get_receipts_by_block`. |
| `StateProviderFactory` | Real | State/account/storage read views backed by Whirlpool DB. |
| `ChainSpecProvider` | Real | Returns configured chain spec used by node/EVM config. |
| `AccountReader` | Real | Account reads backed by state DB. |
| `NodePrimitivesProvider` | Real | Ethereum primitives binding (`reth_ethereum_primitives::EthPrimitives`). |
| `ChangeSetReader` | Stub | Deterministic empty/none semantics if historical changesets are unavailable. |
| `StageCheckpointReader` | Stub | Deterministic none/empty checkpoint semantics. |
| `PruneCheckpointReader` | Stub | Deterministic none semantics. |
| `HashedPostStateProvider` | Stub | Deterministic empty semantics. |
| `StateRootProvider` | Stub | Deterministic placeholder/unsupported semantics per trait contract. |
| `StorageRootProvider` | Stub | Deterministic placeholder/unsupported semantics per trait contract. |
| `StateProofProvider` | Stub | Deterministic unsupported/empty proof semantics. |
| `BlockBodyIndicesProvider` | Stub | Deterministic none semantics unless explicitly required by RPC path. |
| `CanonStateSubscriptions` | Stub/minimal | Non-panicking subscription surface; noop event stream allowed. |
| `PersistedBlockSubscriptions` | Stub/minimal | Non-panicking subscription surface; noop event stream allowed. |

Stub implementations must be explicit, deterministic, and non-panicking.

## `WhirlpoolTxPool` Design

### Role
Bridge `app::traits::TxSource` to reth pool contract used by eth RPC methods.

### Required behavior
- Submission path:
  - Accept raw tx from RPC (`eth_sendRawTransaction` path).
  - Decode/validate enough to enforce transaction-kind policy.
  - Forward accepted bytes to `TxSource::push`.
- Pending view:
  - Source pending transactions from `TxSource::pending()`.
  - Expose them in the `TransactionPool` pending APIs used by reth eth RPC paths.

### Blob policy (mandatory)
- Reject type-3 (blob) submissions at adapter boundary.
- Do not expose blob sidecar behavior.
- Return explicit unsupported-style errors for blob-specific operations.

### Non-critical pool surfaces
For broad `TransactionPool` APIs not backed by Whirlpool mempool capabilities, provide documented noop/minimal semantics consistent with reth noop patterns (never panic, deterministic outputs).

## `WhirlpoolNetwork` Design

### Role
Satisfy reth network trait requirements for RPC module build without taking over P2P ownership.

### Required traits
- `reth_network_api::NetworkInfo`
- `reth_network_api::Peers` (and thus `PeersInfo`)

### Behavior model
- Static/local metadata (`chain_id`, local address, syncing flags).
- Empty peer set and deterministic responses for peer queries/mutations.
- No runtime peer management ownership transferred into `rpc-eth`.

## Dependency Plan (`rpc-eth/Cargo.toml`)

Design target dependencies include:
- RPC/build: `reth-rpc`, `reth-rpc-builder`, `reth-rpc-eth-api`, `reth-rpc-eth-types`, `reth-rpc-server-types`
- Storage/provider: `reth-storage-api`, `reth-provider`, `reth-chain-state`
- Network: `reth-network-api`
- Pool: `reth-transaction-pool`
- Primitives/config: `reth-primitives-traits`, `reth-ethereum-primitives`, `reth-chainspec`, `reth-consensus`
- Existing Whirlpool crates: `state`, `state-reth`, `app`, `app-evm`

`jsonrpsee`/`async-trait` should remain only if directly needed after migration; otherwise they are expected to become transitive via reth RPC crates.
