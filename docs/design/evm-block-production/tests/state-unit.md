# state Unit Test Contracts

| Test name | Test case ID | Label | Invariant ref | Preconditions | Actions | Assertions / Oracle | Mock/Stub requirements | Priority | Status |
|---|---|---|---|---|---|---|---|---|---|
| `state_root_empty_db_is_keccak_empty` | `STATE-U-001` | `[GROUNDED]` | INV-06 | Fresh `InMemoryStateDb::new()`. | Call `state_root()`. | Returns `KECCAK_EMPTY`. | None. | High | Active |
| `state_root_is_deterministic_for_identical_contents` | `STATE-U-002` | `[GROUNDED]` | INV-06, INV-07 | Two DB instances with identical account/storage contents. | Compute `state_root()` for both DBs. | Roots are identical. | Deterministic fixtures for account and storage maps. | High | Active |
| `commit_applies_account_and_storage_effects` | `STATE-U-003` | `[GROUNDED]` | INV-05, INV-06 | DB initialized; `BundleState` includes account updates and storage mutations. | Call `commit(&bundle)`, then read account/storage via `DatabaseRef`. | Updated values are visible and consistent with bundle effects. | BundleState fixture builder. | High | Active |
| `commit_destroyed_account_is_removed` | `STATE-U-004` | `[GROUNDED]` | INV-05 | DB has existing account; bundle marks account destroyed. | Call `commit(&bundle)` and query account. | Account is absent after commit. | Bundle fixture with `AccountStatus::Destroyed`. | High | Active |
| `clone_provides_independent_snapshot_for_speculative_paths` | `STATE-U-005` | `[GROUNDED]` | INV-04 | Canonical DB exists; clone taken before speculative changes. | Mutate clone via `commit`, compare canonical vs clone state. | Canonical DB remains unchanged; clone reflects speculative mutations. | None. | High | Active |
| `failed_speculative_execution_leaves_canonical_snapshot_identical` | `STATE-U-006` | `[PROPOSED]` | INV-04 | Snapshot orchestration around propose/verify failure path is defined. | Inject execution failure after snapshot creation and before canonical commit. | Canonical pre-snapshot and post-failure state are byte-identical by agreed snapshot boundary. | Requires cross-crate snapshot orchestration seam (currently undocumented). | High | Blocked |
| `finalize_commit_is_applied_exactly_once` | `STATE-U-007` | `[PROPOSED]` | INV-05 | Finalization callback path to `commit` exists and is idempotent-aware. | Replay same finalized block signal twice. | State effects applied once; second signal is no-op or explicit duplicate rejection. | Requires finalize->commit integration contract (missing). | High | Blocked |

## Pseudo-code outlines

```rust
// STATE-U-002 [GROUNDED]
let db1 = fixture_db_with_accounts();
let db2 = fixture_db_with_accounts();
assert_eq!(db1.state_root(), db2.state_root());
```

```rust
// STATE-U-005 [GROUNDED]
let canonical = fixture_db();
let before = canonical.state_root();
let mut clone = canonical.clone();
clone.commit(&bundle_update());
assert_eq!(canonical.state_root(), before);
assert_ne!(clone.state_root(), before);
```

```rust
// STATE-U-007 [PROPOSED] BLOCKER
on_finalized(block);
on_finalized(block);
assert_commit_applied_once(block.id());
```
