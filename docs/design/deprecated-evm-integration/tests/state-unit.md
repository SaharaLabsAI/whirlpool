# State Crate Unit Tests

<!-- continuation round 2: resolves B-002 -->

> Unit tests for `state` crate — `InMemoryStateDb` Database impl, commit, and state root.

Status: **PROPOSED** | Crate: `state`

## Database Trait Implementation

### `test_basic_returns_none_for_unknown_address`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Empty `InMemoryStateDb`, arbitrary `Address`
- **Assert**: `basic()` returns `Ok(None)` for unknown address. **[PROPOSED]**: `state::InMemoryStateDb::basic`.
- **Pseudo-code**:
```rust
#[test]
fn test_basic_returns_none_for_unknown_address() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    assert_eq!(db.basic(addr).unwrap(), None);
}
```

### `test_basic_returns_account_info`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with one account inserted
- **Assert**: `basic()` returns correct `AccountInfo` for known address. **[PROPOSED]**: `state::InMemoryStateDb::basic`.
- **Pseudo-code**:
```rust
#[test]
fn test_basic_returns_account_info() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    let info = AccountInfo { balance: U256::from(100), nonce: 1, ..Default::default() };
    db.insert_account(addr, info.clone());
    assert_eq!(db.basic(addr).unwrap(), Some(info));
}
```

### `test_storage_returns_zero_for_missing`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Empty `InMemoryStateDb`
- **Assert**: `storage()` returns `Ok(U256::ZERO)` for missing address/slot. **[PROPOSED]**: `state::InMemoryStateDb::storage`.
- **Pseudo-code**:
```rust
#[test]
fn test_storage_returns_zero_for_missing() {
    let mut db = InMemoryStateDb::new();
    let value = db.storage(Address::random(), U256::from(0)).unwrap();
    assert_eq!(value, U256::ZERO);
}
```

### `test_storage_returns_stored_value`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with account that has storage slot set
- **Assert**: `storage()` returns correct value. **[PROPOSED]**: `state::InMemoryStateDb::storage`.
- **Pseudo-code**:
```rust
#[test]
fn test_storage_returns_stored_value() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    db.insert_account_with_storage(addr, AccountInfo::default(), vec![(U256::from(1), U256::from(42))]);
    assert_eq!(db.storage(addr, U256::from(1)).unwrap(), U256::from(42));
}
```

### `test_code_by_hash_returns_default_for_missing`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Empty `InMemoryStateDb`
- **Assert**: `code_by_hash()` returns `Ok(Bytecode::default())` for unknown hash. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_code_by_hash_returns_default_for_missing() {
    let mut db = InMemoryStateDb::new();
    let code = db.code_by_hash(B256::random()).unwrap();
    assert_eq!(code, Bytecode::default());
}
```

### `test_block_hash_returns_zero_for_missing`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Empty `InMemoryStateDb`
- **Assert**: `block_hash()` returns `Ok(B256::ZERO)` for unknown block number. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_block_hash_returns_zero_for_missing() {
    let mut db = InMemoryStateDb::new();
    assert_eq!(db.block_hash(42).unwrap(), B256::ZERO);
}
```

### `test_block_hash_returns_inserted_hash`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with block hash inserted
- **Assert**: `block_hash()` returns the inserted hash. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_block_hash_returns_inserted_hash() {
    let mut db = InMemoryStateDb::new();
    let hash = B256::random();
    db.insert_block_hash(1, hash);
    assert_eq!(db.block_hash(1).unwrap(), hash);
}
```

## State Commitment

### `test_commit_creates_new_account`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Empty `InMemoryStateDb`, `BundleState` with one created account
- **Assert**: After `commit()`, `basic()` returns the new account. **[PROPOSED]**: `state::InMemoryStateDb::commit`.
- **Pseudo-code**:
```rust
#[test]
fn test_commit_creates_new_account() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    let bundle = create_bundle_with_new_account(addr, U256::from(1000), 0);
    db.commit(&bundle);
    let info = db.basic(addr).unwrap().expect("account should exist");
    assert_eq!(info.balance, U256::from(1000));
}
```

### `test_commit_updates_existing_account`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with account, `BundleState` changing balance
- **Assert**: After `commit()`, `basic()` returns updated balance. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_commit_updates_existing_account() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    db.insert_account(addr, AccountInfo { balance: U256::from(100), ..Default::default() });
    let bundle = create_bundle_with_changed_account(addr, U256::from(200), 1);
    db.commit(&bundle);
    let info = db.basic(addr).unwrap().unwrap();
    assert_eq!(info.balance, U256::from(200));
}
```

### `test_commit_destroys_account`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with account, `BundleState` with selfdestruct
- **Assert**: After `commit()`, `basic()` returns `None`. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_commit_destroys_account() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    db.insert_account(addr, AccountInfo::default());
    let bundle = create_bundle_with_destroyed_account(addr);
    db.commit(&bundle);
    assert_eq!(db.basic(addr).unwrap(), None);
}
```

### `test_commit_applies_storage_changes`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with account, `BundleState` with storage writes
- **Assert**: After `commit()`, `storage()` returns new values. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_commit_applies_storage_changes() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    db.insert_account(addr, AccountInfo::default());
    let bundle = create_bundle_with_storage(addr, vec![(U256::from(1), U256::from(99))]);
    db.commit(&bundle);
    assert_eq!(db.storage(addr, U256::from(1)).unwrap(), U256::from(99));
}
```

### `test_commit_inserts_new_bytecode`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `BundleState` with new contract bytecode
- **Assert**: After `commit()`, `code_by_hash()` returns the bytecode. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_commit_inserts_new_bytecode() {
    let mut db = InMemoryStateDb::new();
    let code = Bytecode::new_raw(Bytes::from(vec![0x60, 0x00]));
    let code_hash = keccak256(&code.original_bytes());
    let bundle = create_bundle_with_bytecode(code_hash, code.clone());
    db.commit(&bundle);
    assert_eq!(db.code_by_hash(code_hash).unwrap(), code);
}
```

## State Root

### `test_state_root_deterministic`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Two `InMemoryStateDb` instances with identical state
- **Assert**: Both produce the same `state_root()`. **[PROPOSED]**: `state::InMemoryStateDb::state_root`.
- **Pseudo-code**:
```rust
#[test]
fn test_state_root_deterministic() {
    let mut db1 = InMemoryStateDb::new();
    let mut db2 = InMemoryStateDb::new();
    let addr = Address::random();
    let info = AccountInfo { balance: U256::from(100), ..Default::default() };
    db1.insert_account(addr, info.clone());
    db2.insert_account(addr, info);
    assert_eq!(db1.state_root(), db2.state_root());
}
```

### `test_state_root_changes_after_commit`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb`, commit a `BundleState`
- **Assert**: `state_root()` before and after commit are different. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_state_root_changes_after_commit() {
    let mut db = InMemoryStateDb::new();
    let root_before = db.state_root();
    let bundle = create_bundle_with_new_account(Address::random(), U256::from(1), 0);
    db.commit(&bundle);
    assert_ne!(db.state_root(), root_before);
}
```

### `test_state_root_empty_db`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Empty `InMemoryStateDb`
- **Assert**: `state_root()` returns a known constant (empty state root). **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_state_root_empty_db() {
    let db = InMemoryStateDb::new();
    let root = db.state_root();
    assert_ne!(root, B256::ZERO); // empty state still has a defined root
    assert_eq!(root, InMemoryStateDb::new().state_root()); // and it's consistent
}
```

## Clone Semantics

### `test_clone_produces_independent_snapshot`
- **Layer**: unit
- **Crate**: `state`
- **Input**: `InMemoryStateDb` with state, clone it, modify clone
- **Assert**: Original is unchanged after modifying clone. **[PROPOSED]**.
- **Pseudo-code**:
```rust
#[test]
fn test_clone_produces_independent_snapshot() {
    let mut db = InMemoryStateDb::new();
    let addr = Address::random();
    db.insert_account(addr, AccountInfo { balance: U256::from(100), ..Default::default() });
    let mut clone = db.clone();
    let bundle = create_bundle_with_changed_account(addr, U256::from(200), 1);
    clone.commit(&bundle);
    // Original unchanged
    assert_eq!(db.basic(addr).unwrap().unwrap().balance, U256::from(100));
    // Clone updated
    assert_eq!(clone.basic(addr).unwrap().unwrap().balance, U256::from(200));
}
```

## Genesis

### `test_with_genesis_populates_accounts`
- **Layer**: unit
- **Crate**: `state`
- **Input**: Genesis allocation with 2 accounts (one with balance, one with code)
- **Assert**: Both accounts accessible via `basic()`, code accessible via `code_by_hash()`. **[PROPOSED]**: `state::InMemoryStateDb::with_genesis`.
- **Pseudo-code**:
```rust
#[test]
fn test_with_genesis_populates_accounts() {
    let addr1 = Address::random();
    let addr2 = Address::random();
    let alloc = hashmap! {
        addr1 => GenesisAccount { balance: U256::from(1_000_000), ..Default::default() },
        addr2 => GenesisAccount { balance: U256::from(0), code: Some(vec![0x60, 0x00].into()), ..Default::default() },
    };
    let db = InMemoryStateDb::with_genesis(alloc);
    assert_eq!(db.basic(addr1).unwrap().unwrap().balance, U256::from(1_000_000));
    assert!(db.basic(addr2).unwrap().is_some());
}
```
