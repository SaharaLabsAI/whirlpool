# Digest: Dependency Graph — mempool-split-interface

## Grounded Facts
- 12 workspace members. Only `whirlpool-node` depends on `mempool`.
- mempool deps: `app` (path, for TxSource), `reth-libmdbx` (vendor path). Dev: `tempfile`.
- No feature flags. Only `#[cfg(test)]` conditional compilation.
- No circular dependency risk in proposed split.

## [PROPOSED] Post-Split Dependency Graph
```
whirlpool-node → mempool-mdbx → mempool → app (TxSource trait)
                              → reth-libmdbx (vendor)
```
- `mempool` (interface): deps on `app` only (for TxSource re-export or reference). **No reth-libmdbx.**
- `mempool-mdbx`: deps on `mempool` + `reth-libmdbx` + `app`. Dev: `tempfile`.
- `whirlpool-node`: switches from `mempool` → `mempool-mdbx`.

## Key Insight
- `mempool` interface crate MIGHT not need `app` dep at all if it only defines `MempoolStore` trait + `MempoolError`. TxSource is in `app`, not re-exported by mempool currently.
