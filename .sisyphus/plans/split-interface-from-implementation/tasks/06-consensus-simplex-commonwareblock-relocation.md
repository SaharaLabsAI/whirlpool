## `06-consensus-simplex-commonwareblock-relocation`

> Relocate `CommonwareBlock` to `consensus-simplex::traits` and migrate internal bounds.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `05-app-evm-stateprovider-relocation` |
| **Wave** | 3 |
| **Complexity** | L |
| **Target Crate(s)** | `consensus-simplex` (both) |
| **Migration Step** | #6 |
| **Change Type** | move |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

This is a high-risk adapter step because generic bounds are dense in `adapter.rs` and `engine.rs`. Keep dual-path compatibility while moving trait and blanket impl together.

### What to do

#### Phase 1 - Update/create test expectations

1. Update `TB-008` tests to canonical `traits::CommonwareBlock` imports.
2. Add/refresh dual-path check `TN-007`.

```bash
cargo test -p consensus-simplex --lib
```

#### Phase 2 - Implement the change

3. Add `crates/consensus-simplex/src/traits.rs` with `CommonwareBlock` + blanket impl.
4. Update `crates/consensus-simplex/src/lib.rs` and `types.rs` for canonical path + compatibility export.
5. Update imports in `adapter.rs`, `engine.rs`, and tests.

```bash
cargo check -p consensus-simplex
```

### Rollback

```bash
git restore crates/consensus-simplex/src/lib.rs crates/consensus-simplex/src/types.rs crates/consensus-simplex/src/adapter.rs crates/consensus-simplex/src/engine.rs crates/consensus-simplex/src/tests.rs
rm -f crates/consensus-simplex/src/traits.rs
```

**Rollback dependencies**: revert this task before reverting task `05`.

### Mock Boundary

**Allowed to mock**: adapter test peers/fixtures.
**Must NOT mock**: trait bounds and blanket impl resolution.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 6
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-008`, `TN-007`)
- `docs/refactor/split-interface-implementation/consensus-simplex/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p consensus-simplex
cargo test -p consensus-simplex --lib
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/06-consensus-simplex-commonwareblock-relocation.log`
