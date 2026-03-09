## `06-final-subintent-a-verification`

> Finish Sub-Intent A with the complete test pass requested by the design docs, validating that all scoped requirements hold together after the file-ordered implementation work is complete.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `01-receiver-channel-contract`, `02-provider-build-seeding-and-bootstrap`, `03-multiplex-receiver-alignment`, `04-sender-traits-compatibility-review`, `05-whirlpool-node-builder-wiring` |
| **Wave** | 6 |
| **Complexity** | M |
| **Goal** | Run the final Sub-Intent A verification sweep for `REQ-1`, `REQ-2`, and `REQ-3` across `p2p-commonware` and `whirlpool-node` |
| **Target Crate(s)** | `p2p-commonware`, `whirlpool-node` |
| **Requirements** | `REQ-1`, `REQ-2`, `REQ-3` |
| **Tests** | `TST-REQ1-001`, `TST-REQ1-002`, `TST-REQ2-001`, `TST-REQ2-002`, `TST-REQ3-001`, `TST-REQ3-002`, `TST-REQ3-003` |

### Files to modify

- `crates/p2p-commonware/src/provider.rs`
- `crates/p2p-commonware/src/receiver.rs`
- `crates/p2p-commonware/src/lib.rs`
- `crates/p2p-commonware/src/sender.rs`
- `crates/p2p-commonware/src/traits.rs`
- `crates/whirlpool-node/src/main.rs`

### Mock Boundary

- Do not widen scope beyond the finalized Sub-Intent A file set.
- Use only crate-local tests and standard cargo build/test verification under Nix.

### What to do

#### Phase 1 - Reconcile test coverage

1. Confirm every in-scope requirement has matching passing coverage: `REQ-1` -> `TST-REQ1-001`, `TST-REQ1-002`; `REQ-2` -> `TST-REQ2-001`, `TST-REQ2-002`; `REQ-3` -> `TST-REQ3-001`, `TST-REQ3-002`, `TST-REQ3-003`.
2. Add or finish any missing crate-local assertions inside the already-touched files so the test contracts in `docs/design/p2p-node-connectivity/agent/tests.md` are fully implemented.

```bash
nix develop --command cargo test -p p2p-commonware
nix develop --command cargo test -p whirlpool-node
```

#### Phase 2 - Run full verification

3. Run a full build verification for the workspace state produced by Tasks 01-05.
4. Run the full cargo test suite required by the request to ensure no scoped regression remains before handoff.
5. Record evidence for the final audit and confirm no files outside the documented Sub-Intent A scope were needed.

```bash
nix develop --command cargo build
nix develop --command cargo test
```

### Acceptance Criteria

- `REQ-1`, `REQ-2`, and `REQ-3` all have passing `TST-*` coverage in the scoped crates.
- `nix develop --command cargo build` succeeds.
- `nix develop --command cargo test` succeeds.
- No source changes extend into Sub-Intent B, Sub-Intent C, `crates/p2p/**`, or vendor code.

### Verification commands

```bash
nix develop --command cargo build
nix develop --command cargo test
```

Evidence: `.sisyphus/plans/p2p-provider-completeness/evidence/06-final-subintent-a-verification.log`
