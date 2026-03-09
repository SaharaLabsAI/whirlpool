## `04-sender-traits-compatibility-review`

> Review the remaining `p2p-commonware` support modules after the provider and multiplex fixes land, limiting this step to compile-fix or test-alignment changes only if required.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `02-provider-build-seeding-and-bootstrap`, `03-multiplex-receiver-alignment` |
| **Wave** | 4 |
| **Complexity** | S |
| **Goal** | Keep `crates/p2p-commonware/src/sender.rs` and `crates/p2p-commonware/src/traits.rs` compatible with the finalized provider/receiver contract without expanding scope |
| **Target Crate(s)** | `p2p-commonware` (support modules) |
| **Requirements** | `REQ-3` compatibility only |
| **Tests** | compatibility review only; retain coverage from prior tasks |

### Files to modify

- `crates/p2p-commonware/src/sender.rs`
- `crates/p2p-commonware/src/traits.rs`

### Mock Boundary

- No new mocks or new behavior belong here.
- Restrict any edits to import normalization, compile fixes, or crate-local test maintenance required by earlier tasks.

### What to do

#### Phase 1 - Confirm compatibility coverage first

1. Re-run the focused `p2p-commonware` checks established in Tasks 01-03 to determine whether `sender.rs` or `traits.rs` need follow-up edits.
2. If tests or compile checks fail because of imports, trait bounds, or compatibility glue in these files, add or update the minimal crate-local assertions needed to keep the prior `REQ-*` and `TST-*` coverage stable.

```bash
nix develop --command cargo check -p p2p-commonware
nix develop --command cargo test -p p2p-commonware
```

#### Phase 2 - Apply compatibility-only fixes if needed

3. Adjust `crates/p2p-commonware/src/sender.rs` only if the finalized channel-preservation path requires compile-fix alignment while keeping send routing unchanged.
4. Adjust `crates/p2p-commonware/src/traits.rs` only if local trait imports or `PerChannelNetwork` references need normalization to match the finalized provider/receiver wiring.
5. Do not introduce validator seeding, bootstrap logic, or any new public API surface in this task.

```bash
nix develop --command cargo check -p p2p-commonware
```

### Acceptance Criteria

- `crates/p2p-commonware/src/sender.rs` continues routing by caller-provided channel with no behavior redesign.
- `crates/p2p-commonware/src/traits.rs` remains the canonical local import surface with compatibility fixes only.
- `REQ-1`, `REQ-2`, and `REQ-3` coverage from prior tasks remains green after any support-file adjustments.

### Verification commands

```bash
nix develop --command cargo check -p p2p-commonware
nix develop --command cargo test -p p2p-commonware
```

Evidence: `.sisyphus/plans/p2p-provider-completeness/evidence/04-sender-traits-compatibility-review.log`
