# Task 02 Evidence: Scaffold WhirlpoolProvider + stub trait surface

## Summary

Created `WhirlpoolProvider` adapter struct implementing all 22+ reth provider
traits as noop stubs, following `NoopProvider` patterns from reth's storage-api.
Type-level integration test proves the provider satisfies `RpcModuleBuilder` bounds.

## Changes

### New files
- **`crates/rpc-eth/src/provider.rs`** (570 lines) — WhirlpoolProvider struct + all trait impls
- **`crates/rpc-eth/tests/provider_contract.rs`** — TST-1 type-level bounds test

### Modified files
- **`crates/rpc-eth/src/lib.rs`** — Added `pub mod provider;`
- **`crates/rpc-eth/Cargo.toml`** — Added deps needed by provider trait impls:
  alloy-eips, reth-db-api, reth-execution-types, reth-prune, reth-stages-api,
  reth-trie, revm-database

### WhirlpoolProvider struct
```rust
pub struct WhirlpoolProvider {
    state_db: Arc<RethStateDb>,
    chain_spec: Arc<ChainSpec>,
    canon_state_tx: broadcast::Sender<CanonStateNotification<EthPrimitives>>,
}
```

### Trait impls (all noop stubs)
| Category | Traits |
|----------|--------|
| Core | NodePrimitivesProvider, ChainSpecProvider (real), Clone, Debug |
| Block | BlockHashReader, BlockNumReader, BlockIdReader, HeaderProvider, BlockReader, BlockReaderIdExt |
| Tx/Receipt | TransactionsProvider, ReceiptProvider, ReceiptProviderIdExt |
| State | AccountReader, BytecodeReader, StateReader, StateProvider, StateProviderFactory |
| Trie/Proof | StateRootProvider, StorageRootProvider, StateProofProvider, HashedPostStateProvider |
| Misc | StageCheckpointReader, PruneCheckpointReader, ChangeSetReader, BlockBodyIndicesProvider |
| Subscriptions | CanonStateSubscriptions (noop broadcast channel) |

## Verification

- `cargo build -p rpc-eth`: **PASS**
- `cargo test -p rpc-eth --lib`: **PASS** (17/17 tests)
- `cargo test -p rpc-eth --test provider_contract`: **PASS** (1/1 test — TST-1)
- No vendor files modified
- All existing modules preserved

## Artifact Coverage
- TST-1: ✅ WhirlpoolProvider compiles against RpcModuleBuilder provider bounds

## Timestamp
2026-03-11T06:30:00Z
