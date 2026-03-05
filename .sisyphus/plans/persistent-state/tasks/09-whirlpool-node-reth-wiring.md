## `09-whirlpool-node-reth-wiring`

> Replace `TestStateDb` runtime wiring with `RethStateDb` and enforce startup initialization sequence.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 9 |
| **Dependencies** | `08-consumer-fallible-migration-app-evm-rpc-eth` |
| **Goal** | Node boots on persistent backend and initializes genesis safely |
| **AC/INV** | `AC-7`, `AC-9`, `INV-6`, `INV-8` |

### Files to modify

- `crates/whirlpool-node/Cargo.toml`
- `crates/whirlpool-node/src/config.rs`
- `crates/whirlpool-node/src/main.rs`
- `crates/whirlpool-node/src/lib.rs`
- `crates/whirlpool-node/tests/rpc_integration.rs`
- `crates/whirlpool-node/tests/*` (add startup/genesis tests as needed)

### Work

1. Add `state-reth` dependency and any required DB argument/config imports.
2. Introduce `NodeStateDbConfig` and startup helper `build_state_db` with sequence: `create_db -> init_db -> with_genesis`.
3. Replace `Arc<RwLock<TestStateDb>>` with `Arc<RwLock<RethStateDb>>` in EVM and RPC wiring.
4. Add startup error mapping for invalid path/missing prerequisites/init failures.
5. Add/adjust node wiring tests for startup and genesis-first-run behavior (`TC-WN-I001`, `TC-WN-I002`, `TC-WN-I003`).

### Verification command

```bash
nix develop --command cargo test -p whirlpool-node
```
