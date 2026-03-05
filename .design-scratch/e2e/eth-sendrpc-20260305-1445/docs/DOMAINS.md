# DOMAINS

## Domain model

### D1. Node Runtime Orchestration
- **Grounded**: `whirlpool-node` owns process lifecycle and runtime composition (`crates/whirlpool-node/src/main.rs::main`).
- **Grounded**: Engine start and indefinite run loop already exist in the node binary (`crates/whirlpool-node/src/main.rs::main`).
- **[PROPOSED]**: RPC server lifecycle is an additional node-runtime responsibility, started after engine startup.

### D2. Transaction Ingress
- **Grounded**: `InMemoryTxPool` is thread-safe and stores raw EIP-2718 bytes with push/drain semantics (`crates/app/src/tx_source.rs::InMemoryTxPool`).
- **[PROPOSED]**: `eth_sendRawTransaction` writes into this pool and returns deterministic tx hash.

### D3. Account State Read Model
- **Grounded**: `StateDb` exposes account/storage/blockhash read methods (`crates/state/src/traits.rs::StateDb`).
- **Grounded**: Node wraps `InMemoryStateDb` in `Arc<RwLock<TestStateDb>>` (`crates/whirlpool-node/src/main.rs::main`).
- **[PROPOSED]**: `eth_getBalance` and `eth_getTransactionCount` read through node-held state DB lock.

### D4. Execution and Confirmation
- **Grounded**: `EvmApplication::propose` executes pending txs and commits bundle state (`crates/app-evm/src/executor.rs::EvmApplication::propose`).
- **Grounded**: Receipts roots and gas used are computed during execution (`crates/app-evm/src/executor.rs::EvmApplication::propose`).
- **[PROPOSED]**: Node-local receipt index maps tx hash -> receipt availability/status for RPC polling.

### D5. Chain Metadata
- **Grounded**: Chain id constant is `SAHARA_CHAIN_ID = 313_371` (`crates/app-evm/src/config.rs::SAHARA_CHAIN_ID`).
- **Grounded**: Finalized height is observable through atomic sink state (`crates/consensus-simplex/src/sink.rs::FinalizationSink`).
- **[PROPOSED]**: RPC context carries chain id and finalized height reference for response shaping.

## Wiring contracts

| Contract | Producer | Consumer | Type/Shape | Classification |
|---|---|---|---|---|
| Tx submission | RPC handler | `InMemoryTxPool` | raw `Vec<u8>` bytes | grounded + [PROPOSED] usage |
| Account balance read | state DB | RPC handler | `Address -> U256` | grounded + [PROPOSED] usage |
| Nonce read | state DB | RPC handler | `Address -> nonce` | grounded + [PROPOSED] usage |
| Chain id read | app-evm config | RPC handler | `u64` -> RPC `U64` | grounded + [PROPOSED] adaptation |
| Receipt lookup | node-local index | RPC handler | `B256 -> Option<Receipt>` | [PROPOSED] |
| Finalization signal | `FinalizationSink` atomic | RPC context | `u64` height | grounded + [PROPOSED] usage |

## Type layer map

| Layer | Grounded types | [PROPOSED] RPC-facing types | Boundary note |
|---|---|---|---|
| Runtime/internal | `Arc<RwLock<TestStateDb>>`, `Arc<InMemoryTxPool>`, `Arc<AtomicU64>` | `EthRpcContext` | Node-local only |
| Execution/internal | `Recovered<TransactionSigned>`, `ExecutionResult`, roots | none exposed directly | Keep hidden from transport |
| RPC transport | none today | `Address`, `Bytes`, `B256`, `U64`, `U256`, `Option<Receipt>` | Implemented via jsonrpsee trait |

## Boundary rules
- **Grounded**: `app` is interface crate exposing `Application` and `TxSource` contracts (`crates/app/src/traits.rs`).
- **Grounded**: Node binary currently composes all runtime dependencies (`crates/whirlpool-node/src/main.rs`).
- **[PROPOSED]**: No new public types are introduced into implementation crates that belong to existing interface domains.
- **[PROPOSED]**: RPC structs and helper indices remain node-private to avoid unnecessary interface expansion.
