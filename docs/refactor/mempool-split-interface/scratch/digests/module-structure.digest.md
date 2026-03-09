# Digest: Module Structure — mempool-split-interface

## Grounded Facts
- mempool facade: flat. lib.rs → 3 pub mods + 3 pub use re-exports. No nesting.
- store.rs: pub struct + 3 pub methods + 1 private fn. Private fields.
- persistent.rs: pub struct (private field: MempoolStore) + 1 pub fn + TxSource impl
- error.rs: pub enum + Display + Error + 2 From impls
- whirlpool-node: `use mempool::PersistentTxPool` only. No deep imports.

## [PROPOSED] Post-Split Structure
### mempool (interface)
```
lib.rs
├── pub mod error;       → MempoolError (keep, possibly rename Mdbx variant)
├── pub mod traits;      → MempoolStore trait (NEW — extracted from struct API)
├── pub use error::MempoolError;
└── pub use traits::MempoolStore;
```

### mempool-mdbx (implementation)
```
lib.rs
├── pub mod store;       → MdbxMempoolStore (impl MempoolStore)
├── pub mod persistent;  → PersistentTxPool (impl TxSource)
├── pub use store::MdbxMempoolStore;
└── pub use persistent::PersistentTxPool;
```

## Reference Pattern Match
- Follows state/state-memory exactly: interface = trait + error, impl = concrete types.
