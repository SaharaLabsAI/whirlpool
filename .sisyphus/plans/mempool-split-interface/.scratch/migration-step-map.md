# Migration Step Map — mempool-split-interface

| Migration Step | Task(s) | Wave | Complexity | Files Touched | TestIDs | Split? |
|---|---|---|---|---|---|---|
| Step 1: Scaffold mempool-mdbx | 01-scaffold-mempool-mdbx | 1 | S (2 files) | Cargo.toml (workspace), crates/mempool-mdbx/Cargo.toml, crates/mempool-mdbx/src/lib.rs | — | No |
| Step 2: Add MempoolStore trait | 02-add-mempool-store-trait | 2 | S (2 files) | crates/mempool/src/traits.rs (new), crates/mempool/src/lib.rs | TN-002 | No |
| Step 3: Move store to mempool-mdbx | 03-move-store-impl | 3 | M (5 files) | crates/mempool-mdbx/src/store.rs (new), crates/mempool-mdbx/src/lib.rs, crates/mempool-mdbx/Cargo.toml, crates/mempool/src/error.rs | TB-001–TB-007, TN-001 | No |
| Step 4: Move PersistentTxPool | 04-move-persistent-txpool | 4 | M (4 files) | crates/mempool-mdbx/src/persistent.rs (new), crates/mempool-mdbx/src/lib.rs, crates/mempool-mdbx/Cargo.toml | TB-008–TB-010 | No |
| Step 5: Move integration tests | 05-move-integration-tests | 5 | S (2 files) | crates/mempool-mdbx/tests/integration.rs (new), crates/mempool/tests/integration.rs (delete) | TB-011–TB-016 | No |
| Step 6: Update whirlpool-node | 06-update-consumer | 6 | S (2 files) | crates/whirlpool-node/Cargo.toml, crates/whirlpool-node/src/main.rs | — | No |
| Step 7: Strip mempool interface | 07-strip-mempool-interface | 7 | M (5 files) | crates/mempool/src/lib.rs, crates/mempool/Cargo.toml, crates/mempool/src/store.rs (delete), crates/mempool/src/persistent.rs (delete), crates/mempool/tests/ (delete) | TN-002 | No |

## Ordering Justification
- Wave 1: Scaffold (no deps)
- Wave 2: Trait (depends on scaffold for compile target, but actually in mempool crate — still wave 2 for ordering)
- Wave 3: Store move (needs trait to impl against)
- Wave 4: PersistentTxPool (needs store in mempool-mdbx)
- Wave 5: Tests (needs all code in place)
- Wave 6: Consumer (needs mempool-mdbx fully working)
- Wave 7: Cleanup (after all consumers migrated)

All waves are strictly sequential — no parallelism possible (each depends on prior).
