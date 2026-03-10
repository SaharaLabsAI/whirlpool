# Digest: Test Coverage — mempool-split-interface

## Grounded Facts
- **16 total tests** across 3 locations. ALL will break due to path/type changes.
- store.rs: 7 unit tests (type_changed — test struct that becomes trait, must move to mempool-mdbx)
- persistent.rs: 3 unit tests (import_path_changed — move to mempool-mdbx)
- integration.rs: 6 tests (import_path_changed — move to mempool-mdbx)
- No other workspace tests reference mempool. No doc tests.
- All tests use `tempfile::TempDir`. Store tests also use `Arc`, `thread`, `BTreeSet`.

## [PROPOSED] Migration
- ALL 16 tests move to `mempool-mdbx` crate (unit tests in src, integration in tests/)
- Import paths change: `mempool::X` → `mempool_mdbx::X` or `crate::X`
- `new_store` test helper moves with store tests
- New trait-level tests can be added to `mempool` interface crate (optional, using mock)

## UNKNOWN
- Whether any CI or workspace-level test configuration needs updating
