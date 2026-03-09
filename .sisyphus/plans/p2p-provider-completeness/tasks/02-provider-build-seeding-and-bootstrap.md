## `02-provider-build-seeding-and-bootstrap`

> Update the provider assembly point after the receiver contract lands so the builder becomes the single place that applies validator seeding, preserves bootstrap discovery inputs, and passes explicit channel tags into receiver construction.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `01-receiver-channel-contract` |
| **Wave** | 2 |
| **Complexity** | M |
| **Goal** | Complete the provider-owned runtime assembly for `REQ-1`, `REQ-2`, and the provider call-site portion of `REQ-3` |
| **Target Crate(s)** | `p2p-commonware` (primary implementation) |
| **Requirements** | `REQ-1`, `REQ-2`, `REQ-3` |
| **Tests** | `TST-REQ1-001`, `TST-REQ1-002`, `TST-REQ2-001`, `TST-REQ3-001`, `TST-REQ3-002` |

### Files to modify

- `crates/p2p-commonware/src/provider.rs`

### Mock Boundary

- Use deterministic crate-local builder/runtime test scaffolding.
- Treat `crates/p2p` as a stable interface boundary and do not modify `vendor/commonware/**`.

### What to do

#### Phase 1 - Write or update failing tests first

1. Add or update provider-focused tests in `crates/p2p-commonware/src/provider.rs` for `TST-REQ1-001`, `TST-REQ1-002`, and `TST-REQ2-001`.
2. Extend provider-side receive-path coverage so `TST-REQ3-001` and `TST-REQ3-002` verify that `CommonwareReceiver::new(...)` is called with `Channel::VOTE`, `Channel::CERTIFICATE`, and `Channel::RESOLVER`.
3. Assert that non-empty `initial_validators(...)` are applied before build returns, empty validator input skips seeding without failing, and supplied bootstrappers flow unchanged into `discovery::Config::local(...)`.

```bash
nix develop --command cargo test -p p2p-commonware provider
```

#### Phase 2 - Implement provider build behavior

4. Update `CommonwareNetworkProviderBuilder::build(context)` in `crates/p2p-commonware/src/provider.rs` to thread builder-owned bootstrappers directly into `discovery::Config::local(...)`.
5. Apply `oracle_handle.update_validators(epoch, validators.clone()).await` inside `build(context)` before returning whenever `initial_validators` is present and non-empty.
6. Preserve the empty-validator path by skipping the oracle update when no validator keys are supplied.
7. Update the provider start path so all `CommonwareReceiver::new(...)` call sites pass explicit `Channel::VOTE`, `Channel::CERTIFICATE`, and `Channel::RESOLVER` values.

```bash
nix develop --command cargo check -p p2p-commonware
```

### Acceptance Criteria

- `REQ-1`: provider build seeds validators before provider handoff and keeps empty-validator startup legal.
- `REQ-2`: provider build preserves supplied bootstrappers into Commonware discovery config without reinterpretation.
- `REQ-3`: provider receiver construction assigns the canonical `p2p` channel constants at each lane.
- `TST-REQ1-001`, `TST-REQ1-002`, `TST-REQ2-001`, `TST-REQ3-001`, and `TST-REQ3-002` pass.

### Verification commands

```bash
nix develop --command cargo check -p p2p-commonware
nix develop --command cargo test -p p2p-commonware provider
```

Evidence: `.sisyphus/plans/p2p-provider-completeness/evidence/02-provider-build-seeding-and-bootstrap.log`
