- Renamed error variant to storage-agnostic naming: `MempoolError::Mdbx` -> `MempoolError::Storage` in `mempool` while preserving `From<reth_libmdbx::Error>` conversion for existing `?` flows.
- Implemented `mempool::MempoolStoreTrait` for `MdbxMempoolStore` via delegation to inherent methods (`MdbxMempoolStore::push` and `MdbxMempoolStore::drain_pending`) to keep logic single-sourced.

- Added  dependency to  so the crate can implement  directly in the new .
- Re-exported  from  to preserve expected crate-level access pattern during parallel migration.
- Task 04 decision: Added app dependency to mempool-mdbx so the crate can implement app traits TxSource in persistent.rs.
- Task 04 decision: Re-exported PersistentTxPool from mempool-mdbx lib to preserve crate-level access during parallel migration.
