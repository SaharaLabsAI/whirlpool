## `01-consensus-traits-boundary`

> Create canonical `consensus::traits` boundary and keep legacy paths compiling.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | none |
| **Wave** | 1 |
| **Complexity** | M |
| **Target Crate(s)** | `consensus` (both) |
| **Migration Step** | #1 |
| **Change Type** | move |

### Pre-Task Gate

```bash
cargo check --workspace
```

### Context

This is the first foundation step and establishes canonical trait locations for consensus interfaces. Compatibility exports must remain so downstream crates can migrate later without breakage.

**Before state**: traits are spread across `app.rs`, `block.rs`, `event.rs`, `engine.rs`.
**After state**: canonical trait surface exists in `crates/consensus/src/traits.rs`.

### What to do

#### Phase 1 - Update/create test expectations

1. Update compile tests in `crates/consensus/src/{app.rs,engine.rs}` to assert canonical imports from `consensus::traits`.
2. Add/adjust dual-path compile checks in `crates/consensus/src/lib.rs` for `TB-001`, `TB-002`, `TN-001`.

```bash
cargo test -p consensus --features mock
```

#### Phase 2 - Implement the change

3. Add `crates/consensus/src/traits.rs` exposing `ConsensusApp`, `Block`, `EventSink`, `ConsensusEngine`.
4. Update `crates/consensus/src/lib.rs` to expose `pub mod traits;` and canonical re-exports.
5. Keep compatibility exports from `app.rs`, `block.rs`, `event.rs`, `engine.rs`.

```bash
cargo check -p consensus
```

#### Phase 3 - Update consumers (if applicable)

6. Spot-update internal imports to prefer `crate::traits::*` where safe; keep external compatibility unchanged.

```bash
cargo check --workspace
```

### Rollback

```bash
git restore crates/consensus/src/lib.rs crates/consensus/src/app.rs crates/consensus/src/block.rs crates/consensus/src/event.rs crates/consensus/src/engine.rs
rm -f crates/consensus/src/traits.rs
```

**Rollback dependencies**: none.

### Mock Boundary

**Allowed to mock**: local consensus mock feature types.
**Must NOT mock**: trait signatures/associated types.

### References

- `docs/refactor/split-interface-implementation/MIGRATION.md` Step 1
- `docs/refactor/split-interface-implementation/TESTS.md` (`TB-001`, `TB-002`, `TN-001`)
- `docs/refactor/split-interface-implementation/consensus/CHANGES.md`

### Acceptance Criteria

```bash
cargo check -p consensus
cargo test -p consensus --features mock
```

Evidence: `.sisyphus/plans/split-interface-from-implementation/evidence/01-consensus-traits-boundary.log`
