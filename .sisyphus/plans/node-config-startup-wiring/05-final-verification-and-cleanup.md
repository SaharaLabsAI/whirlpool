## `05-final-verification-and-cleanup`

> Close the sub-intent by removing any temporary compatibility glue, auditing scope, and proving the full workspace still builds and tests through the required `nix develop` entrypoints.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `04-rewire-startup-through-node-config` |
| **Wave** | 5 |
| **Complexity** | S |
| **Goal** | Finalize `REQ-4`/`REQ-5` implementation with a no-stray-literals audit and full build/test verification |
| **Target Crate(s)** | `whirlpool-node` |
| **Requirements** | `REQ-4`, `REQ-5` |
| **Acceptance IDs** | `AC-B-1`, `AC-B-2`, `AC-B-3`, `AC-B-4`, `AC-B-5`, `AC-B-6`, `AC-B-7` |

### Files to modify

- `crates/whirlpool-node/src/config.rs` (only if temporary compatibility shims remain)
- `crates/whirlpool-node/src/main.rs` (only if final literal cleanup is still needed)
- `crates/whirlpool-node/tests/startup_config.rs` or `crates/whirlpool-node/src/main.rs` test module (only if assertions need final tightening)

### Pre-task gate

- Task 04 completed with passing startup-wiring tests.
- All in-scope behavior is implemented, leaving only cleanup and final proof work.
- Any temporary constant shims or transitional helpers are explicitly identified before editing.

### What to do

1. Remove any temporary compatibility constants or transitional helpers that were kept only to preserve compilation between earlier tasks.
2. Audit `crates/whirlpool-node/src/main.rs` and `crates/whirlpool-node/src/config.rs` against the handoff checklist so the startup source of truth lives in `NodeConfig` and no hidden in-scope literals remain.
3. Confirm the final implementation still respects `INV-B-5`, `INV-B-6`, and `INV-B-7`: no `p2p-commonware` API changes, CLI/config parsing completes before runtime initialization, and network versus consensus namespaces remain distinct.
4. Run the full required verification commands.

### Post-task gate

- Only `whirlpool-node` files changed for this sub-intent.
- No temporary compatibility glue remains unless it is part of the intentional final contract.
- `REQ-4` and `REQ-5` are fully covered by passing `TST-REQ4-001..005` and `TST-REQ5-001..002`.
- Final definition-of-done commands complete successfully:

```bash
nix develop --command cargo build
nix develop --command cargo test
```
