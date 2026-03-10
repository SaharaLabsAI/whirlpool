## `03-add-peer-normalization-and-config-conversion`

> Finish the config module before touching startup so all parsing, validation, and normalization rules are proven in isolation.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `02-scaffold-node-config-contract` |
| **Wave** | 3 |
| **Complexity** | M |
| **Goal** | Add fail-fast bootstrap peer parsing and normalize `NodeArgs` into `NodeConfig` for all in-scope CLI fields |
| **Target Crate(s)** | `whirlpool-node` |
| **Requirements** | `REQ-4`, `REQ-5` |
| **Acceptance IDs** | `AC-B-3`, `AC-B-4`, `AC-B-6`, `AC-B-7` |
| **Tests** | `TST-REQ4-002`, `TST-REQ4-003`, `TST-REQ4-004` |

### Files to modify

- `crates/whirlpool-node/src/config.rs`

### Pre-task gate

- Task 02 completed with passing default and storage-helper tests.
- `NodeConfig`, `NodeArgs`, and nested config structs already exist in `crates/whirlpool-node/src/config.rs`.
- No startup wiring changes have been made yet; this task stays pure to config normalization and validation.

### TDD sequence

#### Phase 1 - Write failing unit tests first

1. Add `TST-REQ4-002` to prove `parse_bootstrap_peer("PUBKEY@HOST:PORT")` returns the exact `BootstrapPeer` tuple expected by `p2p_commonware`.
2. Add `TST-REQ4-003` as a malformed-input table covering missing `@`, empty segments, invalid hex, wrong key length, and invalid socket addresses.
3. Add `TST-REQ4-004` to prove `NodeArgs` normalizes every explicit flag into `NodeConfig`, merges `bootstrap_peers` and `dial_peers` into one ordered list, and converts `block_interval_ms` into `Duration`.
4. Run the crate tests and confirm the new cases fail before implementation.

```bash
nix develop --command cargo test -p whirlpool-node
```

#### Phase 2 - Implement parsing and normalization

5. Implement `parse_bootstrap_peer(input: &str) -> Result<BootstrapPeer, String>` with descriptive errors for each malformed segment class.
6. Wire repeatable `--bootstrap-peer` and `--dial-peer` parsing into `NodeArgs`, using fail-fast conversion so malformed peer inputs stop startup before the async runtime begins.
7. Implement `From<NodeArgs> for NodeConfig` or `TryFrom<NodeArgs>` if needed to preserve fail-fast semantics, while keeping the external contract aligned with the design docs.
8. Ensure `network.namespace` and `consensus.namespace` remain distinct consumers, and map `validator_seed` to `identity.seed` for later ed25519 derivation.

### Post-task gate

- `parse_bootstrap_peer` accepts valid `PUBKEY@HOST:PORT` inputs and rejects malformed input before runtime startup.
- `NodeArgs` to `NodeConfig` conversion preserves all in-scope fields and deterministic peer ordering.
- `TST-REQ4-002`, `TST-REQ4-003`, and `TST-REQ4-004` pass alongside Task 02 tests.
- Verification commands complete successfully:

```bash
nix develop --command cargo build
nix develop --command cargo test
```
