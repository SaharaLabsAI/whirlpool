## `04-rewire-startup-through-node-config`

> Once config normalization is stable, thread the config object through startup so all builder, storage, RPC, and consensus inputs come from one source of truth.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `03-add-peer-normalization-and-config-conversion` |
| **Wave** | 4 |
| **Complexity** | M |
| **Goal** | Parse CLI at process start, construct `NodeConfig`, and wire its values through the Commonware builder and remaining startup consumers for `REQ-5` |
| **Target Crate(s)** | `whirlpool-node` |
| **Requirements** | `REQ-4`, `REQ-5` |
| **Acceptance IDs** | `AC-B-1`, `AC-B-2`, `AC-B-5`, `AC-B-6`, `AC-B-7` |
| **Tests** | `TST-REQ5-001`, `TST-REQ5-002` |

### Files to modify

- `crates/whirlpool-node/src/main.rs`
- `crates/whirlpool-node/src/config.rs` (only if small supporting exports or cleanup are required)
- `crates/whirlpool-node/tests/startup_config.rs` or `crates/whirlpool-node/src/main.rs` test module

### Pre-task gate

- Task 03 completed with passing unit tests for defaults, storage helpers, peer parsing, and config normalization.
- `NodeArgs` can be parsed and normalized before runtime startup.
- No task has yet changed `p2p_commonware`; startup must consume the existing builder API as-is.

### TDD sequence

#### Phase 1 - Write failing startup-wiring tests first

1. Add `TST-REQ5-001` to prove no-arg startup remains backwards compatible: seed `0`, `data/{runtime,state,mempool}`, `127.0.0.1:8545`, network namespace `b"whirlpool-dev"`, and ephemeral listen/dial addresses.
2. Add `TST-REQ5-002` to prove a fully customized config reaches the Commonware builder inputs: `.listen_addr(...)`, `.dialable_addr(...)`, `.bootstrappers(...)`, `.max_message_size(...)`, and `.initial_validators(...)`, while RPC, storage, and consensus consumers also read from the same config object.
3. Prefer extracting a testable startup-assembly helper from `main.rs` rather than using runtime or socket-heavy integration tests.
4. Run the crate tests and confirm the new startup coverage fails before implementation.

```bash
nix develop --command cargo test -p whirlpool-node
```

#### Phase 2 - Implement startup rewiring

5. Parse `NodeArgs` before constructing `commonware_runtime::tokio::Runner` and convert them into `NodeConfig`.
6. Build runtime storage from `config.storage.runtime_dir()`, state DB from `config.storage.state_dir()`, and mempool DB from `config.storage.mempool_dir()`.
7. Derive the signer from `config.identity.seed`, derive `validators = vec![signer.public_key()]`, and pass config-owned values into `CommonwareNetworkProviderBuilder::new(...).listen_addr(...).dialable_addr(...).bootstrappers(...).max_message_size(...).initial_validators(...)`.
8. Bind JSON-RPC to `config.rpc.bind_addr`, pass `config.consensus.namespace` into `consensus_simplex::CommonwareConfig`, and use `config.consensus.block_interval` everywhere the current code uses the fixed five-second interval for in-scope startup timing.
9. Preserve tracing initialization and `oracle_handle` lifetime behavior.

### Post-task gate

- `main.rs` no longer owns hardcoded in-scope startup literals for listen address, dialable address, validator seed, RPC address, storage directories, namespace selection, or max message size.
- `TST-REQ5-001` and `TST-REQ5-002` pass, proving both zero-arg compatibility and custom config propagation.
- Existing Task 02/03 unit tests remain green.
- Verification commands complete successfully:

```bash
nix develop --command cargo build
nix develop --command cargo test
```
