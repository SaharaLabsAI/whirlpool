# TESTS — Mempool Interface/Implementation Split

## Broken Tests (all move to mempool-mdbx)

| ID | Test Name | Location | Breakage Type | Fix |
|---|---|---|---|---|
| TB-001 | `push_and_drain` | store.rs:L~85 | type_changed | Rename `MempoolStore` → `MdbxMempoolStore` in test |
| TB-002 | `drain_empty` | store.rs:L~100 | type_changed | Same rename |
| TB-003 | `drain_clears` | store.rs:L~110 | type_changed | Same rename |
| TB-004 | `persistence_across_reopen` | store.rs:L~120 | type_changed | Same rename |
| TB-005 | `fifo_ordering` | store.rs:L~135 | type_changed | Same rename |
| TB-006 | `multiple_push_drain_cycles` | store.rs:L~150 | type_changed | Same rename |
| TB-007 | `concurrent_push` | store.rs:L~165 | type_changed | Same rename |
| TB-008 | `txsource_trait_object` | persistent.rs:L~45 | import_path_changed | Update `use crate::` paths |
| TB-009 | `pending_drains` | persistent.rs:L~55 | import_path_changed | Update `use crate::` paths |
| TB-010 | `persistence` | persistent.rs:L~65 | import_path_changed | Update `use crate::` paths |
| TB-011 | `trait_object_coercion_across_crates` | integration.rs:L~8 | import_path_changed | `mempool::` → `mempool_mdbx::` |
| TB-012 | `restart_recovery_via_trait` | integration.rs:L~18 | import_path_changed | Same |
| TB-013 | `restart_after_drain_is_empty` | integration.rs:L~30 | import_path_changed | Same |
| TB-014 | `fifo_ordering_preserved` | integration.rs:L~42 | import_path_changed | Same |
| TB-015 | `fifo_ordering_survives_restart` | integration.rs:L~54 | import_path_changed | Same |
| TB-016 | `concurrent_push_via_trait_object` | integration.rs:L~66 | import_path_changed | Same |

## New Tests (recommended)

| ID | Test Name | Location | Purpose |
|---|---|---|---|
| TN-001 | `mdbx_store_implements_trait` | mempool-mdbx/src/store.rs | Compile-time check: `MdbxMempoolStore` implements `MempoolStore` |
| TN-002 | `trait_is_object_safe` | mempool/src/traits.rs | Compile-time check: `dyn MempoolStore` compiles |

## Migration-Test Alignment

| Migration Step | Tests Affected | Verification |
|---|---|---|
| Step 1 (scaffold) | None | `cargo build` |
| Step 2 (add trait) | None (additive) | `cargo build` |
| Step 3 (move store) | TB-001–TB-007, TN-001 | `cargo test -p mempool-mdbx` |
| Step 4 (move persistent) | TB-008–TB-010 | `cargo test -p mempool-mdbx` |
| Step 5 (move integration) | TB-011–TB-016 | `cargo test -p mempool-mdbx --test integration` |
| Step 6 (update node) | None (import path) | `cargo build -p whirlpool-node` |
| Step 7 (strip mempool) | TN-002 | `cargo test --workspace` |

## Test Helper Migration

- `new_store` helper in store.rs tests → moves to mempool-mdbx/src/store.rs tests
- `tempfile` dev-dep → moves to mempool-mdbx/Cargo.toml
