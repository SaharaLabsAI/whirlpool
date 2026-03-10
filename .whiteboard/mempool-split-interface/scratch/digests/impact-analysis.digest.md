# Digest: Impact Analysis — mempool-split-interface

## Grounded Facts
- `MempoolStore` struct: 15+ call sites across 3 source files + tests. Only internal to mempool crate. **No external consumers.**
- `PersistentTxPool`: 12+ call sites. **One external consumer: whirlpool-node** (import + open + Arc<dyn TxSource>).
- `MempoolError`: internal only. Used as return types in store.rs + persistent.rs. **No external consumers.**
- `TxSource` impl: single impl block in persistent.rs. Consumed via trait object by whirlpool-node + integration tests.

## [PROPOSED] Changes
- `MempoolStore` struct → trait in `mempool`, impl `MdbxMempoolStore` in `mempool-mdbx`
- `PersistentTxPool` → moves to `mempool-mdbx`. whirlpool-node switches dep.
- `MempoolError` stays in `mempool` (shared). `mempool-mdbx` re-uses or wraps.

## Blast Radius
- **Small**: only mempool internals + whirlpool-node import path changes
- **Risk**: MempoolError's `From<reth_libmdbx::Error>` impl ties it to MDBX. May need split or the interface error can't have that variant.

## BLOCKER
- **BLK-001**: `MempoolError::Mdbx(String)` variant in interface crate. If `mempool` is storage-agnostic, it shouldn't know about MDBX. Options: (a) keep generic variant name like `Storage(String)`, (b) split error type, (c) use `From` adapter in impl crate. **Needs decision.**
