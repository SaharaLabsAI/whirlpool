## `03-multiplex-receiver-alignment`

> Align the crate-level multiplex receiver with the repaired receiver contract so it forwards already-tagged messages without compensating remap logic.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `01-receiver-channel-contract`, `02-provider-build-seeding-and-bootstrap` |
| **Wave** | 3 |
| **Complexity** | S |
| **Goal** | Finish the `REQ-3` aggregate receive path by making `MultiplexReceiver` trust receiver-owned channel tagging |
| **Target Crate(s)** | `p2p-commonware` (aggregate receive path) |
| **Requirements** | `REQ-3` |
| **Tests** | `TST-REQ3-003` |

### Files to modify

- `crates/p2p-commonware/src/lib.rs`

### Mock Boundary

- Keep test coverage inside `crates/p2p-commonware` with deterministic receiver fixtures.
- Do not add new transport abstractions or modify channel constants in `crates/p2p`.

### What to do

#### Phase 1 - Write or update failing tests first

1. Add or update `MultiplexReceiver` tests in `crates/p2p-commonware/src/lib.rs` for `TST-REQ3-003`.
2. Verify that vote, certificate, and resolver messages emerge with the channels already attached by their `CommonwareReceiver` instances and are not rewritten by multiplex logic.

```bash
nix develop --command cargo test -p p2p-commonware multiplex
```

#### Phase 2 - Implement the aggregate-path alignment

3. Remove or simplify any channel-repair logic in `MultiplexReceiver::recv()` that assumes receiver outputs need to be rewritten.
4. Preserve the existing round-robin polling behavior while trusting the message returned by each underlying receiver.
5. Confirm the aggregate path remains compatible with the provider-side receiver construction introduced in Task 02.

```bash
nix develop --command cargo check -p p2p-commonware
```

### Acceptance Criteria

- `REQ-3` remains satisfied through the crate-level aggregate receive path.
- `TST-REQ3-003` passes using crate-local tests in `crates/p2p-commonware/src/lib.rs`.
- No new channel mapping or fallback logic is introduced outside the documented scope.

### Verification commands

```bash
nix develop --command cargo check -p p2p-commonware
nix develop --command cargo test -p p2p-commonware multiplex
```

Evidence: `.sisyphus/plans/p2p-provider-completeness/evidence/03-multiplex-receiver-alignment.log`
