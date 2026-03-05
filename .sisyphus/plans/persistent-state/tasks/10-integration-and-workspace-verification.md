## `10-integration-and-workspace-verification`

> Run cross-crate integration closure and full workspace gates after all crate-level migrations complete.

| Field | Value |
|---|---|
| **Status** | `[ ]` |
| **Wave** | 10 |
| **Dependencies** | `09-whirlpool-node-reth-wiring` |
| **Goal** | Validate end-to-end feature readiness and merge gates |
| **AC/INV/QA** | `AC-8`, `AC-9`, `AC-11`, `AC-12`, `INV-3`, `INV-4`, `INV-5`, `INV-8`, `QA-1`, `QA-2` |

### Files to modify/create

- `crates/app-evm/tests/cross_crate_flows.rs`
- `crates/whirlpool-node/tests/rpc_integration.rs`
- `crates/state-reth/tests/persistence.rs`
- Optional: `crates/state-reth/tests/stress_large_state.rs` (if QA-2 encoded as test)

### Work

1. Add/complete end-to-end flows (`TC-CC-I001`..`TC-CC-I006`) ensuring persistence/restart/RPC correctness.
2. Verify restart persistence and state root consistency across reopen.
3. Add or run large-state persistence verification path for QA-2 (test or documented manual procedure).
4. Execute full workspace build and test gates after crate-level suite is green.
5. Record final AC/INV/QA evidence mapping for merge readiness.

### Verification command

```bash
nix develop --command cargo build && nix develop --command cargo test
```
