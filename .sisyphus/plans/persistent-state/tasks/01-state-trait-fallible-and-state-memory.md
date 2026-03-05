## `01-state-trait-fallible-and-state-memory`

> Migrate the `state` interface to a fallible trait contract and adapt `state-memory` to preserve behavior with `Infallible` errors.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 1 |
| **Dependencies** | none |
| **Goal** | Establish canonical fallible `StateDb` contract used by all downstream crates |
| **AC/INV** | `AC-3`, `AC-4`, `INV-1`, `INV-2` |

### Files to modify

- `crates/state/src/traits.rs`
- `crates/state/src/lib.rs`
- `crates/state-memory/src/db.rs`
- `crates/state-memory/src/lib.rs`

### Work

1. Add associated `type Error` to `StateDb` and convert all methods to `Result<_, Self::Error>`.
2. Keep method names and semantics unchanged; only add error channel.
3. Update `InMemoryStateDb` `StateDb` impl to use `type Error = core::convert::Infallible` and return `Ok(...)` values.
4. Keep `revm::Database`/`DatabaseRef` behavior intact while reconciling generic bounds needed by callers.
5. Add or update trait migration compile-time tests (`TC-ST-U001`, `TC-ST-U002`) in relevant test modules.

### Verification command

```bash
nix develop --command cargo test -p state && nix develop --command cargo test -p state-memory
```
