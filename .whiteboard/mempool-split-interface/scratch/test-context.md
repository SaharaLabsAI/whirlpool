# Test Context — mempool-split-interface

## Test Inventory (16 total)

| Location | Count | Breakage Type | Destination |
|---|---|---|---|
| `store.rs` unit tests | 7 | type_changed | `mempool-mdbx/src/store.rs` |
| `persistent.rs` unit tests | 3 | import_path_changed | `mempool-mdbx/src/persistent.rs` |
| `integration.rs` | 6 | import_path_changed | `mempool-mdbx/tests/integration.rs` |

## Migration Strategy
- **All 16 tests move to mempool-mdbx.** They test concrete MDBX behavior, not the trait interface.
- Import changes: `use crate::MempoolStore` → `use crate::MdbxMempoolStore` (store tests), `use mempool::PersistentTxPool` → `use mempool_mdbx::PersistentTxPool` (integration)
- `new_store` helper moves with store tests.
- `tempfile` dev-dep moves to `mempool-mdbx`.

## New Tests (optional, recommended)
- `mempool` interface crate: trait-level tests are not strictly needed (trait has no default methods), but could add a compile-test ensuring trait is object-safe.
- `mempool-mdbx`: could add test that `MdbxMempoolStore` implements `MempoolStore` trait (compile-time check via type annotation).

## Coverage Gap Analysis
- No gaps: all existing behavior is tested, all tests move with the impl.
- Post-split verification: `cargo test --workspace` must pass with all 16 tests in new location.
