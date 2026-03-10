## `06-final-verification-and-cleanup`

> Close the sub-intent with a full build/test sweep and a scope audit that proves relay activation stayed additive, channel-aligned, and outside `vendor/`.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Prerequisites** | `05-wire-relay-through-commonware-engine` |
| **Wave** | 6 |
| **Complexity** | S |
| **Goal** | Verify the full relay activation stack, confirm requirement coverage, and audit the final change set for scope compliance |
| **Target Crate(s)** | `p2p`, `p2p-commonware`, `consensus-simplex`, `whirlpool-node` |
| **Requirements** | `REQ-6`, `REQ-7`, `REQ-8` |
| **Acceptance IDs** | `AC-C-1`, `AC-C-2`, `AC-C-3`, `AC-C-4`, `AC-C-5`, `AC-C-6`, `AC-C-7` |

### Files to modify

- Only the in-scope files already touched by Tasks 01-05 if cleanup is needed
- Test files in the same in-scope crates if final assertion tightening is required

### Pre-task gate

- Tasks 01 through 05 are complete and all focused crate tests are passing locally.
- No planned work remains outside the approved crate set.
- Any temporary debug scaffolding introduced during implementation is identified for removal.

### Acceptance criteria

- `AC-C-1` through `AC-C-7` are all demonstrably covered by code and tests.

### Requirements covered

- `REQ-6`
- `REQ-7`
- `REQ-8`

### Detailed implementation steps

1. Run the full verification matrix in serial so failures can be attributed cleanly:
   - `nix develop --command cargo build`
   - `nix develop --command cargo test -p p2p`
   - `nix develop --command cargo test -p p2p-commonware`
   - `nix develop --command cargo test -p consensus-simplex`
   - `nix develop --command cargo test -p whirlpool-node`
2. If any failing tests reveal relay regressions, fix them only within the allowed crate set and re-run the affected commands plus the full matrix.
3. Audit the final diff for scope compliance: no `vendor/` modifications, no hardcoded alternative channel IDs, and no accidental redesign of vote/certificate/resolver handling.
4. Confirm the final implementation still treats payload support as additive: vendor engine boundary unchanged, payload isolated on channel `3`, and single-node startup unaffected.
5. Record or verify requirement-to-test coverage for `TST-REQ6-001..003`, `TST-REQ7-001..002`, and `TST-REQ8-001..002` before declaring the plan complete.

### Test commands

```bash
nix develop --command cargo build
nix develop --command cargo test -p p2p
nix develop --command cargo test -p p2p-commonware
nix develop --command cargo test -p consensus-simplex
nix develop --command cargo test -p whirlpool-node
```

### Post-task gate

- Full build and package test matrix passes.
- All acceptance IDs `AC-C-1` through `AC-C-7` map to implemented behavior and test coverage.
- Scope audit confirms only allowed crates changed and `vendor/` is untouched.
- No temporary scaffolding, debug logging, or hidden hardcoded values remain.
