# Shared Wiring Skeleton — evmblock-txsource

## Node Wiring (whirlpool-node/main.rs)

### Current
```rust
let tx_source = Arc::new(NoopTxSource);
let app = EvmApplication::new(evm_config, state_db, tx_source);
```

### [PROPOSED]
```rust
let tx_pool = Arc::new(InMemoryTxPool::new());
let app = EvmApplication::new(evm_config, state_db, tx_pool.clone());
// tx_pool handle retained for future tx submission (Sub-Intent 3: RPC)
```

## Dependency Changes

| Crate | Current deps | New deps |
|---|---|---|
| `app` | (none added) | (none — uses std::sync only) |
| `whirlpool-node` | `app` (for NoopTxSource) | `app` (for InMemoryTxPool instead) |
