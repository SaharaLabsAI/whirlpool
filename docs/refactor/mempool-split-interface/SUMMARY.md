# SUMMARY — Mempool Interface/Implementation Split

## What

Split the `mempool` crate into two: an interface crate (`mempool`) defining the `MempoolStore` trait and error types, and an implementation crate (`mempool-mdbx`) containing the concrete MDBX-backed storage and `PersistentTxPool` adapter.

## Why

The current `mempool` crate couples its public interface with a heavy vendor dependency (`reth-libmdbx`). Any crate needing mempool semantics must pull in MDBX, even if it only needs the trait contract. This violates the dependency inversion principle and blocks future alternative backends (in-memory for tests, RocksDB, etc.).

The workspace already has a proven pattern for this: `state` (interface) / `state-memory` (impl) / `state-reth` (impl). This refactoring brings `mempool` into alignment.

## How

**Scaffolding approach in 7 atomic steps**, each leaving the workspace compilable:

1. **Scaffold** — Create empty `mempool-mdbx` crate, register in workspace.
2. **Trait** — Add `MempoolStore` trait to `mempool` with `push()` and `drain_pending()` methods.
3. **Move store** — Copy `MempoolStore` struct to `mempool-mdbx` as `MdbxMempoolStore`, implement the new trait, rename error variant `Mdbx` → `Storage`.
4. **Move adapter** — Copy `PersistentTxPool` to `mempool-mdbx`, update internal references.
5. **Move tests** — All 16 tests relocate to `mempool-mdbx` with import path updates.
6. **Update consumer** — `whirlpool-node` switches from `mempool` to `mempool-mdbx` dependency.
7. **Strip interface** — Remove implementation files from `mempool`, drop `reth-libmdbx` dependency.

## Key Decisions

| # | Decision | Rationale |
|---|---|---|
| 1 | `MempoolStore` trait excludes `open()` | Constructors are impl-specific, not trait methods. Matches `StateDb` pattern. |
| 2 | `PersistentTxPool` stays concrete (not generic) | KISS. Generics are additive if needed later. |
| 3 | `MempoolError::Mdbx` → `MempoolError::Storage` | Storage-agnostic naming for the interface crate. Already `String`-typed, no type dep. |
| 4 | No `From` orphan impls | `mempool-mdbx` constructs `MempoolError::Storage(e.to_string())` directly. |
| 5 | `mempool` drops `app` dependency | Trait doesn't reference `TxSource`. Only `mempool-mdbx` needs `app`. |

## Risk Assessment

**Overall: LOW.** Small codebase (~250 lines), single external consumer (`whirlpool-node`), well-tested (16 tests), proven pattern to follow. The only non-trivial decision was the error variant rename, which is resolved.

## Blockers

**None active.** BLK-001 (error variant naming) was resolved: rename to `Storage(String)`.
