# Task 02: Add MempoolStore Trait

| Field | Value |
|---|---|
| Status | `completed` |
| Dependencies | `01-scaffold-mempool-mdbx` |
| Wave | 2 |
| Complexity | S (2 files) |
| Target Crate(s) | `mempool` |
| Migration Step | Step 2 |
| Change Type | CREATE |

## Pre-Task Gate

```bash
nix develop --command cargo check --workspace
```

Task 01 must be completed.

## Context

### Before
`mempool` exports `MempoolStore` as a concrete struct. No trait abstraction.

### After
`mempool` has a new `traits.rs` module with `MempoolStore` trait. The existing struct remains — coexistence until Task 07 removes it.

## What to Do

### Phase 1: Tests

Add trait object safety test (TN-002) to `crates/mempool/src/traits.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trait_is_object_safe() {
        fn _assert_object_safe(_: &dyn MempoolStore) {}
    }
}
```

### Phase 2: Implement

1. **Create `crates/mempool/src/traits.rs`**:
   ```rust
   use crate::error::MempoolError;

   /// Trait for mempool storage backends.
   pub trait MempoolStore: Send + Sync {
       /// Push a transaction into the mempool.
       fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError>;

       /// Drain all pending transactions, returning them in FIFO order.
       fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError>;
   }
   ```

2. **Update `crates/mempool/src/lib.rs`** — Add:
   ```rust
   pub mod traits;
   pub use traits::MempoolStore as MempoolStoreTrait;
   ```
   
   **IMPORTANT**: Keep the existing `pub use store::MempoolStore;` for now. The trait is exported as `MempoolStoreTrait` temporarily to avoid name collision. Task 07 removes the struct and renames.

### Phase 3: Consumers
No consumer changes.

## Rollback

```bash
rm crates/mempool/src/traits.rs
git checkout crates/mempool/src/lib.rs
```

## Must NOT Do

- Remove or modify the existing `MempoolStore` struct
- Remove or modify `store.rs` or `persistent.rs`
- Change any existing pub API
- Modify any other crate
- Modify vendor/

## References

- **MIGRATION**: Step 2 — "Add MempoolStore trait"
- **STRATEGY**: Interface design section
- **IMPACT**: MempoolStore trait design
- **TESTS**: TN-002 (trait object safety)

## Acceptance Criteria

- [ ] `crates/mempool/src/traits.rs` exists with `MempoolStore` trait
- [ ] Trait has `push` and `drain_pending` methods matching existing struct API
- [ ] Trait requires `Send + Sync`
- [ ] `MempoolStoreTrait` exported from lib.rs
- [ ] Existing `MempoolStore` struct still exported (no breaking change)
- [ ] TN-002 test passes
- [ ] `nix develop --command cargo check --workspace` passes
- [ ] `nix develop --command cargo test --workspace` passes

## Post-Task Gate

```bash
nix develop --command cargo check --workspace
nix develop --command cargo test --workspace
```

Both must pass. All existing tests still pass.

## Evidence

Record exit codes. Confirm TN-002 appears in test output.
