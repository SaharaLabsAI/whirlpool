## `03-state-reth-core-db-tables-codec`

> Implement persistent storage primitives in `state-reth`: DB lifecycle helpers, table adapters, and revm/reth codec conversions.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 3 |
| **Dependencies** | `02-state-reth-scaffold` |
| **Goal** | Enable reliable account/storage/code/block-hash I/O building blocks |
| **AC/INV** | `AC-1`, `INV-6` |

### Files to modify

- `crates/state-reth/src/init.rs`
- `crates/state-reth/src/tables.rs`
- `crates/state-reth/src/codec.rs`
- `crates/state-reth/src/error.rs`
- `crates/state-reth/src/db.rs`

### Work

1. Implement `create_db` and `init_db` for MDBX open/create and required table initialization.
2. Implement table helper APIs for:
   - accounts: `PlainAccountState`
   - storage: `PlainStorageState` dupsort cursor
   - bytecode: `Bytecodes`
   - block hash mapping: `CanonicalHeaders`
3. Implement codec conversions between revm and reth storage model types.
4. Wire low-level error mapping into `RethStateError`.
5. Ensure db/table helper surface compiles independently before trait impl completion.

### Verification command

```bash
nix develop --command cargo build -p state-reth
```
