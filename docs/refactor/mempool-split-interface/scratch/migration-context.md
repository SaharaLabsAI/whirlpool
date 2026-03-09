# Migration Context — mempool-split-interface

## Dependency Topology (post-split)
```
app ← mempool(interface) ← mempool-mdbx ← whirlpool-node
                          ↑
                    reth-libmdbx (vendor)
```

## Migration Order Constraints
1. Create `mempool-mdbx` crate first (can initially duplicate, then remove from mempool)
2. Or: Transform `mempool` in-place first, then extract impl to new crate
3. **Recommended**: Scaffolding approach — create mempool-mdbx, move code, update mempool to interface-only, update consumers. Each step compiles.

## Compilability Invariant
Every migration step MUST leave `cargo build --workspace` passing. Steps:
1. Create empty `mempool-mdbx` crate, add to workspace → compiles
2. Add `MempoolStore` trait to `mempool` (alongside struct) → compiles  
3. Move struct+impl to `mempool-mdbx` as `MdbxMempoolStore`, impl trait → compiles (mempool-mdbx builds)
4. Move `PersistentTxPool` to `mempool-mdbx` → compiles
5. Move tests to `mempool-mdbx` → compiles
6. Update `whirlpool-node` dep → compiles
7. Clean up mempool: remove store.rs, persistent.rs, reth-libmdbx dep → compiles

## Error Type Decision
- `MempoolError` stays in `mempool` interface.
- `From<reth_libmdbx::Error>` impl moves to `mempool-mdbx` (requires `MempoolError` variant to accept String, which it already does via `Mdbx(String)`).
- Rename `Mdbx` → `Storage` for cleanliness. `From<io::Error>` stays in interface (generic).
