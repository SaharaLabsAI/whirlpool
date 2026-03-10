## `02-scaffold-node-config-contract`

> Replace the constant-only config module with the normalized startup shape while keeping the crate buildable between tasks.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `01-add-clap-derive-dependency` |
| **Wave** | 2 |
| **Complexity** | M |
| **Goal** | Introduce the `NodeConfig` hierarchy, CLI struct shell, defaults, and storage helpers required by `REQ-4` without rewiring startup yet |
| **Target Crate(s)** | `whirlpool-node` |
| **Requirements** | `REQ-4` |
| **Acceptance IDs** | `AC-B-1`, `AC-B-5` |
| **Tests** | `TST-REQ4-001`, `TST-REQ4-005` |

### Files to modify

- `crates/whirlpool-node/src/config.rs`

### Pre-task gate

- Task 01 is complete and `clap` derive is available in `whirlpool-node`.
- `crates/whirlpool-node/src/config.rs` is still the sole startup-config module for the crate.
- `crates/whirlpool-node/src/main.rs` still depends on legacy config constants, so this task must preserve a compiling intermediate state.

### TDD sequence

#### Phase 1 - Write failing unit tests first

1. Add unit coverage for `TST-REQ4-001` and `TST-REQ4-005` inside `crates/whirlpool-node/src/config.rs`.
2. Assert that `NodeConfig::default()` preserves the current hardcoded startup defaults and that `StorageConfig::{runtime_dir,state_dir,mempool_dir}` derive deterministic sub-paths from `data_dir`.
3. Run the crate tests to confirm the new coverage fails before implementation.

```bash
nix develop --command cargo test -p whirlpool-node
```

#### Phase 2 - Implement the config contract scaffold

4. Add `BootstrapPeer` type alias, `NodeArgs`, `NodeConfig`, `NetworkConfig`, `IdentityConfig`, `RpcConfig`, `StorageConfig`, and `ConsensusStartupConfig` exactly as defined by the crate contract.
5. Implement `Default for NodeConfig` with the documented legacy values: network namespace `b"whirlpool-dev"`, localhost ephemeral listen and dialable addresses, empty bootstrap peers, max message size `1_048_576`, validator seed `0`, RPC bind `127.0.0.1:8545`, `data` storage root, consensus namespace `sahara-chain-v0`, and block interval `Duration::from_secs(5)`.
6. Implement `StorageConfig::{runtime_dir,state_dir,mempool_dir}`.
7. Keep `main.rs` compiling after this task. If compatibility shims are temporarily necessary, keep them confined to `crates/whirlpool-node/src/config.rs` and mark them for removal in Task 05.

### Post-task gate

- `crates/whirlpool-node/src/config.rs` owns the canonical startup shape and default values.
- `TST-REQ4-001` and `TST-REQ4-005` pass.
- The crate remains buildable even though `main.rs` is not rewired yet.
- Verification commands complete successfully:

```bash
nix develop --command cargo build
nix develop --command cargo test
```
