# MIGRATION — Mempool Interface/Implementation Split

## Overview

7 migration steps. Each step leaves `cargo build --workspace` passing. Total estimated effort: ~1 hour.

---

## Step 1: Create `mempool-mdbx` crate scaffold

**Scope:** New crate creation + workspace registration  
**Prerequisite:** None  

### Changes
- [ ] Create `crates/mempool-mdbx/Cargo.toml` with deps: `mempool` (path), `app` (path), `reth-libmdbx` (vendor path). Dev: `tempfile`.
- [ ] Create `crates/mempool-mdbx/src/lib.rs` — empty, just a comment placeholder.
- [ ] Add `"crates/mempool-mdbx"` to workspace members in root `Cargo.toml`.

### Verification
```bash
nix develop --command cargo build --workspace
```

### Rollback
```bash
git checkout -- Cargo.toml && rm -rf crates/mempool-mdbx
```

---

## Step 2: Add `MempoolStore` trait to `mempool` interface

**Scope:** `crates/mempool/src/traits.rs` (new), `crates/mempool/src/lib.rs` (modified)  
**Prerequisite:** Step 1  

### Changes
- [ ] Create `crates/mempool/src/traits.rs`:
  ```rust
  use crate::MempoolError;

  /// Trait for mempool storage backends.
  pub trait MempoolStore: Send + Sync {
      fn push(&self, tx: Vec<u8>) -> Result<(), MempoolError>;
      fn drain_pending(&self) -> Result<Vec<Vec<u8>>, MempoolError>;
  }
  ```
- [ ] Add `pub mod traits;` and `pub use traits::MempoolStore as MempoolStoreTrait;` to `lib.rs`. (Temporary aliased re-export to avoid name conflict with existing `MempoolStore` struct.)

### Verification
```bash
nix develop --command cargo build --workspace
```

### Rollback
```bash
git checkout -- crates/mempool/src/lib.rs && rm crates/mempool/src/traits.rs
```

---

## Step 3: Move store implementation to `mempool-mdbx`

**Scope:** `crates/mempool-mdbx/src/store.rs` (new), `crates/mempool-mdbx/src/lib.rs` (modified)  
**Prerequisite:** Step 2  

### Changes
- [ ] Copy `crates/mempool/src/store.rs` → `crates/mempool-mdbx/src/store.rs`.
- [ ] Rename struct `MempoolStore` → `MdbxMempoolStore` in the new file.
- [ ] Update imports: `use crate::error::MempoolError` → `use mempool::MempoolError`.
- [ ] Replace `MempoolError::Mdbx(...)` construction with `MempoolError::Storage(...)` (will need Step 2.5 error rename — see note).
- [ ] Add `impl mempool::MempoolStore for MdbxMempoolStore` block delegating to existing methods.
- [ ] Update `crates/mempool-mdbx/src/lib.rs`: `pub mod store; pub use store::MdbxMempoolStore;`
- [ ] Move unit tests from old `store.rs` into new `store.rs` (within `#[cfg(test)]` module), updating type name.

**Note:** The error variant rename (`Mdbx` → `Storage`) and `From<reth_libmdbx::Error>` relocation should happen in this step to keep things atomic. Update `crates/mempool/src/error.rs`: rename `Mdbx(String)` → `Storage(String)`, update Display impl. Move `From<reth_libmdbx::Error>` to `mempool-mdbx/src/store.rs` as a helper fn `fn mdbx_err(e: reth_libmdbx::Error) -> MempoolError`.

### Verification
```bash
nix develop --command cargo build --workspace
nix develop --command cargo test -p mempool-mdbx
```

### Rollback
```bash
git checkout -- crates/mempool/src/error.rs crates/mempool/src/store.rs && rm -rf crates/mempool-mdbx/src/store.rs
```

---

## Step 4: Move PersistentTxPool to `mempool-mdbx`

**Scope:** `crates/mempool-mdbx/src/persistent.rs` (new), lib.rs updates  
**Prerequisite:** Step 3  

### Changes
- [ ] Copy `crates/mempool/src/persistent.rs` → `crates/mempool-mdbx/src/persistent.rs`.
- [ ] Update imports: `use crate::MempoolStore` → `use crate::MdbxMempoolStore`, `use crate::MempoolError` → `use mempool::MempoolError`, keep `use app::traits::TxSource`.
- [ ] Update struct field: `store: MempoolStore` → `store: MdbxMempoolStore`.
- [ ] Update `::open()`: `MempoolStore::open` → `MdbxMempoolStore::open`.
- [ ] Update `crates/mempool-mdbx/src/lib.rs`: add `pub mod persistent; pub use persistent::PersistentTxPool;`
- [ ] Move unit tests from old `persistent.rs` into new file, updating imports.

### Verification
```bash
nix develop --command cargo build --workspace
nix develop --command cargo test -p mempool-mdbx
```

### Rollback
```bash
git checkout -- crates/mempool/src/persistent.rs && rm crates/mempool-mdbx/src/persistent.rs
```

---

## Step 5: Move integration tests to `mempool-mdbx`

**Scope:** `crates/mempool-mdbx/tests/integration.rs` (new)  
**Prerequisite:** Step 4  

### Changes
- [ ] Copy `crates/mempool/tests/integration.rs` → `crates/mempool-mdbx/tests/integration.rs`.
- [ ] Update imports: `use mempool::PersistentTxPool` → `use mempool_mdbx::PersistentTxPool`.
- [ ] Keep `use app::traits::TxSource` unchanged.
- [ ] Delete `crates/mempool/tests/integration.rs`.

### Verification
```bash
nix develop --command cargo test -p mempool-mdbx --test integration
```

### Rollback
```bash
git checkout -- crates/mempool/tests/integration.rs && rm -rf crates/mempool-mdbx/tests/
```

---

## Step 6: Update `whirlpool-node` dependency

**Scope:** `crates/whirlpool-node/Cargo.toml`, `crates/whirlpool-node/src/main.rs`  
**Prerequisite:** Step 5  

### Changes
- [ ] In `crates/whirlpool-node/Cargo.toml`: replace `mempool` dep with `mempool-mdbx` (path = "../mempool-mdbx").
- [ ] In `crates/whirlpool-node/src/main.rs`: `use mempool::PersistentTxPool` → `use mempool_mdbx::PersistentTxPool`.

### Verification
```bash
nix develop --command cargo build -p whirlpool-node
nix develop --command cargo test --workspace
```

### Rollback
```bash
git checkout -- crates/whirlpool-node/Cargo.toml crates/whirlpool-node/src/main.rs
```

---

## Step 7: Strip `mempool` to interface-only

**Scope:** `crates/mempool/src/` cleanup, `crates/mempool/Cargo.toml` dep removal  
**Prerequisite:** Step 6  

### Changes
- [ ] Delete `crates/mempool/src/store.rs` (code now lives in mempool-mdbx).
- [ ] Delete `crates/mempool/src/persistent.rs` (code now lives in mempool-mdbx).
- [ ] Delete `crates/mempool/tests/` directory (if not already deleted in Step 5).
- [ ] Update `crates/mempool/src/lib.rs`:
  ```rust
  pub mod error;
  pub mod traits;

  pub use error::MempoolError;
  pub use traits::MempoolStore;
  ```
- [ ] Update `crates/mempool/Cargo.toml`: remove `app` and `reth-libmdbx` deps. Remove `tempfile` dev-dep. Keep only `thiserror` (or std) if used for error derive.
- [ ] Remove the `From<reth_libmdbx::Error>` impl from `error.rs` (already moved in Step 3).
- [ ] Rename the trait re-export from `MempoolStoreTrait` back to `MempoolStore` (the struct is gone, no conflict).

### Verification
```bash
nix develop --command cargo build --workspace
nix develop --command cargo test --workspace
```

### Rollback
```bash
git checkout -- crates/mempool/
```
