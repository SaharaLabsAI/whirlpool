## `08-consumer-fallible-migration-app-evm-rpc-eth`

> Update `app-evm` and `rpc-eth` generic bounds and call sites to handle fallible `StateDb` methods.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 8 |
| **Dependencies** | `07-state-reth-tests` |
| **Goal** | Keep consumer crates implementation-agnostic while propagating StateDb errors correctly |
| **AC/INV** | `AC-5`, `AC-6`, `INV-7` |

### Files to modify

- `crates/app-evm/src/executor.rs`
- `crates/app-evm/src/traits.rs`
- `crates/app-evm/src/error.rs`
- `crates/app-evm/tests/*` (where trait assumptions changed)
- `crates/rpc-eth/src/context.rs`
- `crates/rpc-eth/src/eth_handler.rs`
- `crates/rpc-eth/src/eth_api.rs`
- `crates/rpc-eth/src/server.rs` (if error mapping surfaces here)

### Work

1. Update compile bounds and trait usage to accept `StateDb` methods returning `Result`.
2. Adjust EVM execution paths (`genesis`, `propose`, `verify`) to propagate or map state access errors.
3. Adjust RPC handlers to map `StateDb::Error` into JSON-RPC errors.
4. Update any affected unit/integration tests in both crates.

### Verification command

```bash
nix develop --command cargo test -p app-evm && nix develop --command cargo test -p rpc-eth
```
