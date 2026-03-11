# Architecture Flows

## Flow 1: RPC Server Startup

```
whirlpool-node
    │
    ├── creates RpcConfig { state_db, tx_source, chain_id, bind_addr }
    │
    └── calls rpc_eth::start_rpc_server(config)
            │
            ├── Creates WhirlpoolProvider(config.state_db, chain_spec)
            ├── Creates WhirlpoolTxPool(config.tx_source)
            ├── Creates WhirlpoolNetwork(config.chain_id)
            ├── Gets EthEvmConfig::mainnet()
            ├── Gets NoopConsensus::default()
            │
            ├── RpcModuleBuilder::new(provider, pool, network, evm, consensus)
            │       .bootstrap_eth_api()
            │       .build(transport_config, eth_api, event_sender)
            │
            └── Returns ServerHandle
```

## Flow 2: eth_getBalance Request

```
HTTP Client → JSON-RPC: {"method": "eth_getBalance", "params": ["0x...", "latest"]}
    │
    └── reth EthApi::balance(address, block_id)
            │
            ├── Resolves block_id via WhirlpoolProvider::best_block_number()
            │       └── delegates to BlockStorage::get_latest_block_number()
            │
            ├── Gets StateProvider via WhirlpoolProvider::state_by_block_number(n)
            │       └── Returns RethStateDb (impl StateProvider via revm::DatabaseRef)
            │
            └── StateProvider::basic_account(address)
                    └── RethStateDb::get_account(address) → AccountInfo.balance
```

## Flow 3: eth_sendRawTransaction

```
HTTP Client → JSON-RPC: {"method": "eth_sendRawTransaction", "params": ["0xf8..."]}
    │
    └── reth EthApi::send_raw_transaction(bytes)
            │
            ├── Decodes bytes to TransactionSigned
            ├── Validates: reject Type-3 (blob) txs → error
            │
            └── WhirlpoolTxPool::add_external(tx)
                    └── tx_source.push(tx.encode()) → raw bytes into mempool
```

## Flow 4: eth_getBlockByNumber

```
HTTP Client → JSON-RPC: {"method": "eth_getBlockByNumber", "params": ["0x1", true]}
    │
    └── reth EthApi::block_by_number(1, full_txs=true)
            │
            ├── WhirlpoolProvider::block_by_number(1)
            │       └── BlockStorage::get_block_by_number(1) → EvmBlock
            │           └── convert::evm_block_to_reth_block(evm_block) → SealedBlock
            │
            └── reth converts SealedBlock → RPC Block response
                    └── Includes full tx objects (TransactionSigned → RpcTransaction)
```

## Flow 5: eth_blobBaseFee (excluded)

```
HTTP Client → JSON-RPC: {"method": "eth_blobBaseFee", "params": []}
    │
    └── reth EthApi::blob_base_fee()
            │
            └── Adapter contract enforces explicit unsupported-feature response
                    └── No blob fee calculation and no blob sidecar path
```
