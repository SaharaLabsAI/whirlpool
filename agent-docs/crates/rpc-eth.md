# rpc-eth: Ethereum JSON-RPC Server (reth-backed)

## Summary
`rpc-eth` provides a full Ethereum-compatible JSON-RPC server for the Whirlpool node by wiring reth's production `reth-rpc` stack through three adapter types. Serves all standard `eth_*` methods over HTTP. EIP-4844 blob transactions are explicitly excluded (rejected at pool ingress).

Location: `crates/rpc/evm/`

## Dependency Boundaries
- `app`: `TxSource` trait (`app::traits`) — transaction submission bridge.
- `app-evm-state`: `RethStateDb` — persistent MDBX-backed state and block storage.
- `reth-rpc-builder`: `RpcModuleBuilder`, `TransportRpcModuleConfig`, `RpcServerConfig` — server assembly.
- `reth-rpc`: `EthApi` implementation wired through the builder.
- `reth-provider`: Provider traits (`BlockReader`, `HeaderProvider`, `TransactionsProvider`, `ReceiptProvider`, `StateProviderFactory`, `AccountReader`, `ChainSpecProvider`, `CanonStateSubscriptions`, `StageCheckpointReader`, etc.).
- `reth-transaction-pool`: `TransactionPool` trait and pool types.
- `reth-network-api`: `NetworkInfo`, `Peers`, `PeersInfo` traits.
- `reth-chainspec`: `ChainSpec`, `EthereumHardforks`.
- `app-evm-execution`: `WhirlpoolEvmConfig` for shared Whirlpool EVM/precompile configuration.
- `reth-consensus`: `NoopConsensus` — no consensus validation in RPC path.
- `reth-tasks`: `TaskExecutor` runtime passed to `RpcModuleBuilder::with_executor(...)`.
- `reth-tokio-util`: `EventSender` for RPC event fanout.
- `alloy-primitives`, `alloy-consensus`, `alloy-eips`: Ethereum transaction/header and EIP codec types.
- `thiserror`: Error derive macro for `RpcError`.

## Architecture

### Adapter Pattern
Three adapter types bridge Whirlpool's backend into reth's RPC trait requirements:

1. **WhirlpoolProvider** (`provider/mod.rs` + `provider/*.rs`): Wraps `Arc<RethStateDb>` + `Arc<ChainSpec>`. The provider is split by responsibility into submodules:
   - `provider/block.rs` — block/hash/header/body index readers
   - `provider/transactions.rs` — transaction/receipt providers
   - `provider/state.rs` — account/state/bytecode/provider-factory/proof/checkpoint surfaces
   - `provider/subscriptions.rs` — canon/fork-choice/persisted subscription traits
   Key implementations:
   - `BlockReaderIdExt`: `block_by_id`, `sealed_header_by_id`, `header_by_id` resolve `BlockId` (Latest/Number/Hash) to block numbers via `convert_block_number` → MDBX lookups.
   - `bytecode_by_hash`: reads from MDBX `Bytecodes` table, wraps `revm::state::Bytecode` → `reth_primitives_traits::Bytecode`.
   - `storage`: reads from MDBX `PlainStorageState` table.
  - `CanonStateSubscriptions` via `broadcast::channel`, plus `ForkChoiceSubscriptions` and `PersistedBlockSubscriptions` via `watch::channel`.
   - Noop stubs for traits not yet needed (e.g., `EvmEnvProvider`, `WithdrawalsProvider`).

2. **WhirlpoolTxPool** (`pool.rs`, ~380 lines): Wraps `Arc<dyn TxSource>`. Implements current `TransactionPool` trait surface (including `add_transactions_with_origins`, `prune_transactions`, and `AddressSet` sender reporting). Decodes incoming transactions via RLP, rejects EIP-4844 (Type-3) blob transactions at ingress with `PoolError::other`. Non-blob transactions are forwarded to `TxSource::push()`.

3. **WhirlpoolNetwork** (`network.rs`, ~159 lines): Implements `NetworkInfo + PeersInfo + Peers`. Returns static values (chain ID, no peers, not syncing). Suitable for standalone/single-node operation.

### Server Wiring (`server.rs`, 55 lines)
Uses `RpcModuleBuilder` pattern from reth:
```
RpcModuleBuilder::default()
  .with_provider(WhirlpoolProvider)
  .with_pool(WhirlpoolTxPool)
  .with_network(WhirlpoolNetwork)
  .with_executor(TaskExecutor::test())
  .with_evm_config(WhirlpoolEvmConfig)
  .with_consensus(NoopConsensus)
  → bootstrap_eth_api() → build() → RpcServerConfig::http().start()
```

### Conversion Helpers (`convert.rs`)
- `decode_transaction(bytes) -> TransactionSigned`: RLP decode raw transaction bytes.
- `evmblock_to_header(EvmBlock) -> Header`: Maps internal `EvmBlock` fields to reth `Header`; projects canonical app `extra_data` back to RawEth bytes via `app::project_raw_eth_extra_data`.
- `evmblock_to_block(EvmBlock) -> SealedBlock`: Full block conversion with decoded transactions.

## Public API

```
pub struct RpcConfig {
    pub state_db: Arc<RethStateDb>,
    pub chain_spec: Arc<ChainSpec>,
    pub tx_source: Arc<dyn TxSource>,
    pub addr: SocketAddr,
}

pub enum RpcError { Server(Box<dyn Error + Send + Sync>) }

pub async fn start_rpc_server(config: RpcConfig) -> Result<(RpcServerHandle, SocketAddr), RpcError>
```

## Module Layout
- `lib.rs`: Public API (`RpcConfig`, `RpcError`, `start_rpc_server`) and hidden compatibility re-exports for contract tests.
- `provider/mod.rs`: `WhirlpoolProvider` root (constructor/shared helpers + submodule wiring).
- `provider/block.rs`: block/hash/header readers + block body index provider.
- `provider/transactions.rs`: transaction and receipt provider implementations.
- `provider/state.rs`: account/state/bytecode/provider-factory/proof/checkpoint implementations.
- `provider/subscriptions.rs`: canon/fork-choice/persisted subscription implementations.
- `pool.rs`: `WhirlpoolTxPool` with blob rejection.
- `network.rs`: `WhirlpoolNetwork` with static network info.
- `convert.rs`: `EvmBlock` ↔ reth type conversion helpers.
- `server.rs`: `start_rpc_server()` — RpcModuleBuilder-based server startup.

## Supported RPC Methods
All standard `eth_*` methods from `RpcModuleSelection::standard_modules()` are served, including:
- `eth_chainId`, `eth_blockNumber`, `eth_syncing`
- `eth_getBalance`, `eth_getTransactionCount`, `eth_getCode`, `eth_getStorageAt`
- `eth_getBlockByNumber`, `eth_getBlockByHash`
- `eth_getTransactionByHash`, `eth_getTransactionReceipt`
- `eth_sendRawTransaction` (bridges to TxSource)
- `eth_gasPrice`, `eth_estimateGas`, `eth_call`
- `net_version`, `net_peerCount`, `net_listening`
- `web3_clientVersion`, `web3_sha3`

**Not supported**: `eth_blobBaseFee` and blob-related methods (EIP-4844 excluded).

## Canonical Imports
- `rpc_eth::RpcConfig`
- `rpc_eth::RpcError`
- `rpc_eth::start_rpc_server`
- Internal (pub hidden): `rpc_eth::provider::WhirlpoolProvider`, `rpc_eth::pool::WhirlpoolTxPool`, `rpc_eth::network::WhirlpoolNetwork`, `rpc_eth::convert::{decode_transaction, evmblock_to_header, evmblock_to_block}`

## Test Files
- `tests/provider_contract.rs`: 4 tests — trait bounds, subscriptions, account reader, chain spec (TST-1).
- `tests/pool_contract.rs`: 3 tests — trait bounds, blob rejection, non-blob forwarding (TST-2).
- `tests/network_contract.rs`: 5 tests — chain ID, bounds, syncing, peers, status (TST-3).
- `tests/convert_tests.rs`: 5 tests — decode roundtrip, malformed bytes, header/block conversion.
- `tests/server_contract.rs`: 2 tests — server startup, eth_chainId response.
- **Total**: 19 tests.

## Key Design Notes
- `WhirlpoolProvider` constructor creates internal `broadcast::channel(16)` for canon state notifications and `watch::channel(None)` for safe/finalized/persisted block subscriptions.
- MDBX access pattern: `self.state_db.inner().tx().map_err(map_db_err)?` for read-only transactions.
- State tables used: `CanonicalHeaders`, `HeaderNumbers`, `Headers`, `BlockBodyIndices`, `Transactions`, `TransactionHashNumbers`, `TransactionBlocks`, `Receipts`, `HeaderTerminalDifficulties`, `PlainAccountState`, `PlainStorageState`, `Bytecodes`.
- On empty DB: `eth_blockNumber` returns 0 and `eth_getBalance` returns 0.
- RPC `eth_call` / estimation now share the same Whirlpool precompile registry as consensus execution by using `app_evm_execution::WhirlpoolEvmConfig` in `server.rs`.

## Status
Complete. Replaces the original hand-rolled JSON-RPC with reth's production RPC stack (Tasks 01-13), with legacy in-crate JSON-RPC handler/test scaffolding removed.
