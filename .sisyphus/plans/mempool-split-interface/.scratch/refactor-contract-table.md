# Refactor Contract Table — mempool-split-interface

| # | Symbol | Current Path | Change Type | Target Path | Crate(s) Affected | TestIDs |
|---|---|---|---|---|---|---|
| S1 | `MempoolStore` (struct→trait) | `mempool::store::MempoolStore` | RESTRUCTURE | `mempool::traits::MempoolStore` (trait) | mempool | TN-002 |
| S2 | `MdbxMempoolStore` (renamed struct) | `mempool::store::MempoolStore` | MOVE+RENAME | `mempool_mdbx::store::MdbxMempoolStore` | mempool-mdbx | TB-001–TB-007, TN-001 |
| S3 | `PersistentTxPool` | `mempool::persistent::PersistentTxPool` | MOVE | `mempool_mdbx::persistent::PersistentTxPool` | mempool-mdbx | TB-008–TB-010, TB-011–TB-016 |
| S4 | `MempoolError` | `mempool::error::MempoolError` | MODIFY (rename variant) | `mempool::error::MempoolError` | mempool | — |
| S5 | `TxSource` impl | `mempool::persistent` | MOVE | `mempool_mdbx::persistent` | mempool-mdbx | TB-008–TB-010 |
| S6 | `From<reth_libmdbx::Error>` | `mempool::error` | DELETE+REPLACE | helper fn in `mempool_mdbx::store` | mempool, mempool-mdbx | — |
