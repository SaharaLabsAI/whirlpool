# Shared Test Coverage — mempool-split-interface

## Summary
| Location | Test Count | Will Break | Classification |
| --- | --- | --- | --- |
| `crates/mempool/src/store.rs` | 7 | yes | `type_changed` (struct → trait/impl split) |
| `crates/mempool/src/persistent.rs` | 3 | yes | `import_path_changed` (type now lives in `mempool-mdbx`) |
| `crates/mempool/tests/integration.rs` | 6 | yes | `import_path_changed` (same type but new crate) |
| Doc tests | 0 | no | `indirect` (no runnable examples referencing the moved symbols) |
| Other crates | 0 | no | `indirect` (no workspace tests reference the mempool types) |

## Unit Tests — mempool/src/store.rs
Everything here depends on `MempoolStore` as a concrete struct. After the split it becomes the `mempool::MempoolStore` trait and the MDBX-backed implementation is `mempool_mdbx::MdbxMempoolStore`, so each test will need to move to the implementation crate and target the new type.
- `test_push_and_drain` — classification: `type_changed`. Plan: move into `mempool-mdbx`, reuse the `new_store` helper updated to call `MdbxMempoolStore::open` so the push/drain/equality assertions continue testing the concrete store.
- `test_drain_empty` — classification: `type_changed`. Plan: keep in sync with the other store tests in `mempool-mdbx` and ensure the empty-drain behavior stays covered on `MdbxMempoolStore`.
- `test_drain_clears` — classification: `type_changed`. Plan: the same migration path as the other store tests, verifying clear-after-drain semantics against the concrete MDBX implementation.
- `test_persistence_across_reopen` — classification: `type_changed`. Plan: move to the new crate so it can reopen `MdbxMempoolStore` and validate persistence without keeping the interface crate dependent on MDBX.
- `test_fifo_ordering` — classification: `type_changed`. Plan: migrate with the other store tests, letting `MdbxMempoolStore` continue to guard FIFO ordering.
- `test_multiple_push_drain_cycles` — classification: `type_changed`. Plan: include in the implementation crate’s store tests to cover repeated cycles on `MdbxMempoolStore`.
- `test_concurrent_push` — classification: `type_changed`. Plan: move to `mempool-mdbx` and keep the concurrency assertions targeted at `MdbxMempoolStore`.

## Unit Tests — mempool/src/persistent.rs
These tests exercise the `PersistentTxPool` implementation, which is relocating to `mempool-mdbx::PersistentTxPool`. The trait object usage stays the same, but the crate that exposes the type changes.
- `test_txsource_trait_object` — classification: `import_path_changed`. Plan: move the test into `mempool-mdbx` (or another crate that users of the implementation already depend on) and import `PersistentTxPool` from `mempool_mdbx` so the trait-object coercion stays validated without the interface crate depending on MDBX.
- `test_pending_drains` — classification: `import_path_changed`. Plan: keep this in the implementation crate to continue verifying that draining behaves consistently when the type lives in `mempool-mdbx`.
- `test_persistence` — classification: `import_path_changed`. Plan: migrate into `mempool-mdbx` so persistence/reopen coverage remains tied to the concrete implementation.

## Integration Tests — mempool/tests/integration.rs
All six tests here import `mempool::PersistentTxPool`, so they will fail once the type moves to `mempool-mdbx`. The full test module should be relocated to the new crate (or to a crate that depends on the implementation) so it can keep exercising `PersistentTxPool` without forcing the interface crate to depend on MDBX.
- `trait_object_coercion_across_crates` — classification: `import_path_changed`. Plan: port the test to `mempool-mdbx/tests` and update the imports to the new crate while keeping the trait-object assertions intact.
- `restart_recovery_via_trait` — classification: `import_path_changed`. Plan: keep it alongside the other integration scenarios in the implementation crate so restart recovery remains verified.
- `restart_after_drain_is_empty` — classification: `import_path_changed`. Plan: move with the rest of the integration suite to keep the drain durability regression covered by `mempool-mdbx`.
- `fifo_ordering_preserved` — classification: `import_path_changed`. Plan: port to `mempool-mdbx` so FIFO ordering is still asserted against the concrete pool.
- `fifo_ordering_survives_restart` — classification: `import_path_changed`. Plan: keep the test after moving to the implementation crate so FIFO ordering across restarts still passes.
- `concurrent_push_via_trait_object` — classification: `import_path_changed`. Plan: move to `mempool-mdbx` and retain the concurrent trait-object push assertions there.

## Tests in Other Crates
- No workspace tests outside `crates/mempool` currently refer to `PersistentTxPool` or `MempoolStore`. The only other reference is the runtime initialization in `crates/whirlpool-node/src/main.rs`, which is not a test; its compile-time dependency will need to switch to `mempool-mdbx`, but no tests break (`indirect`).

## Doc Tests
- None of the `//!` or `///` comments in `crates/mempool` expose runnable Rust code that references `MempoolStore` or `PersistentTxPool`, so there are currently no doc tests that need to migrate alongside the split.

## Test Utilities & Patterns
- `store.rs` exposes the `new_store` helper that creates a `TempDir`, points it at `mdbx/`, and opens the store; this helper (and its `Arc`/`BTreeSet` helpers) will need to move with the tests to `mempool-mdbx` so the same fixtures remain available.
- Both `store` and `persistent` tests rely on `tempfile::TempDir` to isolate on-disk databases, and `persistent` tests wrap the pool in `Arc<dyn TxSource>` to assert trait-object behavior; those patterns can be preserved in the implementation crate’s tests.
- The integration suite uses `Arc<dyn TxSource>`, `TempDir`, and straightforward push/pending sequences; after migration these helpers stay the same but reference the relocated `PersistentTxPool` type.

## Test Configuration
- All tests use `tempfile::TempDir` (workspace dev dependency) to create clean MDBX directories under `mdbx/`; this dependency will move to `mempool-mdbx` alongside the tests so they can continue creating temporary environments.
- `store` unit tests use `std::sync::{Arc, AtomicU64, Ordering}`, `std::thread`, and `std::collections::BTreeSet` to verify concurrency and ordering guarantees; these specifics remain relevant for any tests that continue to target the concrete MDBX store.
- `persistent` and integration tests keep depending on `app::traits::TxSource` for trait-object assertions and `Vec<u8>` payloads for predictable FIFO validation.
