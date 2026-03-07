# WORKSPACE

This document describes the workspace-level crate relationships and wiring for the persistent mempool implementation.

## Workspace Structure

```
whirlpool/
├── Cargo.toml                      (workspace root)
├── crates/
│   ├── app/                        [MODIFIED] Trait foundation
│   ├── app-evm/                    [UNCHANGED] Uses trait object
│   ├── consensus/                  [UNCHANGED] Trait definitions
│   ├── consensus-simplex/          [UNCHANGED] BFT adapter
│   ├── mempool/                    [NEW] Persistent tx pool
│   ├── p2p/                        [UNCHANGED] P2P traits
│   ├── p2p-commonware/             [UNCHANGED] P2P impl
│   ├── rpc-eth/                    [MODIFIED] Generified context
│   ├── state/                      [UNCHANGED] State traits
│   ├── state-memory/               [UNCHANGED] In-memory state
│   ├── state-reth/                 [UNCHANGED] Persistent state
│   └── whirlpool-node/             [MODIFIED] Wiring layer
└── testing/
    └── integration-tests/          [UNCHANGED] Test harness
```

## Crate Dependency Graph

```
                    ┌──────────────┐
                    │   consensus  │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  consensus-  │
                    │   simplex    │
                    └──────────────┘

                    ┌──────────────┐
                    │     state    │
                    └──────┬───────┘
                           │
             ┌─────────────┼─────────────┐
             │             │             │
      ┌──────▼──────┐  ┌──▼────────┐  ┌─▼──────────┐
      │ state-memory│  │state-reth │  │    p2p     │
      └─────────────┘  └───────────┘  └──┬─────────┘
                                          │
                                     ┌────▼────────┐
                                     │p2p-commonware│
                                     └─────────────┘

┌──────────────┐
│     app      │ ◄──────────────────────┐
└──────┬───────┘                        │
       │                                │
       ├────────────────┐               │
       │                │               │
┌──────▼───────┐   ┌────▼──────┐       │
│   mempool    │   │  app-evm  │       │
│    [NEW]     │   └─────┬─────┘       │
└──────────────┘         │             │
                         │             │
                  ┌──────▼───────┐     │
                  │   rpc-eth    │─────┘
                  └──────┬───────┘
                         │
                  ┌──────▼───────────────────────────┐
                  │      whirlpool-node              │
                  │  (wires all crates together)     │
                  └──────────────────────────────────┘
```

## Transaction Flow with Persistent Mempool

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER / RPC CLIENT                        │
└──────────────────────────────┬──────────────────────────────────┘
                               │ eth_sendRawTransaction
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│  rpc-eth::EthRpcContext                                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ tx_pool: Arc<dyn TxSource + Send + Sync>  [GENERIFIED]   │  │
│  └────────────────────────────┬──────────────────────────────┘  │
└───────────────────────────────┼─────────────────────────────────┘
                                │ .push(tx_bytes)
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│  mempool::PersistentTxPool [NEW]                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ store: Arc<Mutex<MempoolStore>>                          │  │
│  │   ├── MDBX Database (libmdbx-rs)                         │  │
│  │   ├── Table: pending_txs                                 │  │
│  │   │   └── Key: u64 (auto-increment)                      │  │
│  │   │   └── Value: Vec<u8> (EIP-2718 bytes)                │  │
│  │   └── Counter: next_id (AtomicU64)                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                │ (persisted to disk)
                                │
                ┌───────────────┴───────────────┐
                │ Node restart boundary         │
                │ (txs survive here)            │
                └───────────────┬───────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│  app-evm::EvmApplication                                         │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ tx_source: Arc<dyn TxSource + Send + Sync> [UNCHANGED]   │  │
│  └────────────────────────────┬──────────────────────────────┘  │
│                                │ .propose()                      │
│                                ▼                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ drain = tx_source.pending()  ← drains DB (FIFO order)    │  │
│  │ decode EIP-2718 → execute REVM → compute roots           │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│  consensus-simplex::ApplicationAdapter                           │
│  (wraps ConsensusApp trait)                                      │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│  Simplex BFT Consensus                                           │
└──────────────────────────────┬──────────────────────────────────┘
                               │ finalized event
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│  state-reth::RethStateDb                                         │
│  (state persistence - separate MDBX DB)                          │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow Diagram

### Push Transaction (RPC → Mempool)

```
RPC Client
    │
    └─► rpc-eth::send_raw_transaction(bytes)
            │
            └─► ctx.tx_pool.push(bytes)  [trait method]
                    │
                    └─► PersistentTxPool::push(bytes)
                            │
                            ├─► next_id = counter.fetch_add(1)
                            ├─► tx.put(next_id, bytes)
                            └─► tx.commit()
                                    │
                                    └─► MDBX disk write
                                            (fsync on commit)
```

### Drain Pending (Consensus → Mempool)

```
EvmApplication::propose()
    │
    └─► tx_source.pending()  [trait method]
            │
            └─► PersistentTxPool::pending()
                    │
                    └─► MempoolStore::drain_pending()
                            │
                            ├─► tx = begin_write_txn()
                            ├─► cursor = open_cursor(pending_txs)
                            ├─► txs = cursor.iter().collect()  [FIFO order]
                            ├─► cursor.delete_all()
                            └─► tx.commit()
                                    │
                                    └─► return Vec<Vec<u8>>
```

### Crash Recovery (Startup)

```
whirlpool-node::main()
    │
    └─► PersistentTxPool::open(path)?
            │
            └─► MempoolStore::open(path)?
                    │
                    ├─► env = libmdbx::Environment::open(path)
                    ├─► db = env.open_db(None)  [default DB]
                    ├─► tx = begin_read_txn()
                    ├─► max_key = cursor.last().map(|k| k + 1).unwrap_or(0)
                    └─► counter = AtomicU64::new(max_key)
                            │
                            └─► All txs remain in DB
                                (available to next pending() call)
```

## Wiring Changes in `whirlpool-node`

### Before (Current)

```rust
// main.rs
let tx_pool = Arc::new(InMemoryTxPool::new());

let evm_app = EvmApplication::new(
    tx_pool.clone(),  // Arc<InMemoryTxPool>
    // ...
);

let rpc_ctx = EthRpcContext::new(
    tx_pool,  // Arc<InMemoryTxPool> [concrete type]
    // ...
);
```

### After (Persistent)

```rust
// main.rs
let mempool_path = persistent_storage_dir.join("mempool");
let tx_pool: Arc<dyn TxSource + Send + Sync> = Arc::new(
    PersistentTxPool::open(&mempool_path)
        .context("Failed to open persistent mempool")?
);

let evm_app = EvmApplication::new(
    tx_pool.clone(),  // Arc<dyn TxSource + Send + Sync>
    // ...
);

let rpc_ctx = EthRpcContext::new(
    tx_pool,  // Arc<dyn TxSource + Send + Sync> [trait object]
    // ...
);
```

## Storage Layout

```
persistent_storage_dir/
├── state/                  (state-reth MDBX database)
│   └── data.mdb
├── blocks/                 (block storage)
│   └── ...
├── receipts/               (receipt storage)
│   └── ...
└── mempool/                [NEW] (mempool MDBX database)
    └── data.mdb            (separate MDBX environment)
        ├── pending_txs     (table: u64 → Vec<u8>)
        └── metadata        (counter, schema version)
```

## Key Architectural Properties

1. **Separation of Concerns**: Mempool persistence isolated in dedicated crate, orthogonal to state/block persistence
2. **Trait-Based Abstraction**: `TxSource` trait enables polymorphic tx pool implementations (in-memory, persistent, future variants)
3. **Zero Consensus Impact**: Consensus layer unchanged — only wiring layer modified
4. **Independent Databases**: Each MDBX database has separate directory, no shared tables or schema
5. **Backward Compatibility**: `InMemoryTxPool` remains in `app` crate for tests and fallback scenarios
6. **Crash Safety**: MDBX ACID properties ensure no partial writes; all txs in DB are recoverable on startup

## Migration Path

### Development Phase
1. Implement trait extension in `app` → verify existing code compiles
2. Implement `mempool` crate in parallel → verify tests pass
3. Generify `rpc-eth` → verify RPC tests pass with `InMemoryTxPool`
4. Wire `PersistentTxPool` in `whirlpool-node` → verify integration tests pass

### Deployment
- Fresh nodes: Start with empty mempool DB, normal operation
- Existing nodes: In-memory state lost on upgrade (acceptable — mempool is transient by nature)
- Rollback: Change wiring back to `InMemoryTxPool`, remove `mempool` dependency

## Build Verification

After each phase:
```bash
nix develop --command cargo build -p app
nix develop --command cargo build -p mempool
nix develop --command cargo build -p rpc-eth
nix develop --command cargo build -p whirlpool-node
nix develop --command cargo build  # full workspace
nix develop --command cargo test   # all tests
```
