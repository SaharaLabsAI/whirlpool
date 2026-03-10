## `01-add-clap-derive-dependency`

> Establish the CLI parsing dependency first so later config work can introduce `NodeArgs` without mixing dependency setup into behavioral changes.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `none` |
| **Wave** | 1 |
| **Complexity** | S |
| **Goal** | Add `clap` derive support to `whirlpool-node` as the only dependency change required for `REQ-4`/`REQ-5` |
| **Target Crate(s)** | `whirlpool-node` |
| **Requirements** | `REQ-4`, `REQ-5` |
| **Acceptance IDs** | `AC-B-1` |

### Files to modify

- `crates/whirlpool-node/Cargo.toml`

### Pre-task gate

- `docs/design/p2p-node-connectivity/agent/TASK_GEN_READY.md` remains `READY`.
- `crates/whirlpool-node/Cargo.toml` does not already contain `clap = { version = "4.5", features = ["derive"] }`.
- No source files outside `crates/whirlpool-node` are queued for this sub-intent.

### What to do

1. Add `clap = { version = "4.5", features = ["derive"] }` to `crates/whirlpool-node/Cargo.toml`.
2. Keep the dependency local to `whirlpool-node`; do not promote it to a workspace dependency and do not add any other config library.
3. Do not modify `config.rs` or `main.rs` in this task.

### Post-task gate

- `crates/whirlpool-node/Cargo.toml` contains only the intended dependency addition.
- The repository still builds and tests before any config refactor starts.
- Verification commands complete successfully:

```bash
nix develop --command cargo build
nix develop --command cargo test
```
