## `05-whirlpool-node-builder-wiring`

> Update the node integration boundary only after the provider contract is finalized so startup wiring feeds validators and bootstrappers into the builder without bypassing provider-owned initialization.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | `02-provider-build-seeding-and-bootstrap`, `04-sender-traits-compatibility-review` |
| **Wave** | 5 |
| **Complexity** | M |
| **Goal** | Complete the `whirlpool-node` integration work for `REQ-1` and `REQ-2` while preserving existing defaults and startup flow |
| **Target Crate(s)** | `whirlpool-node` (integration) |
| **Requirements** | `REQ-1`, `REQ-2` |
| **Tests** | `TST-REQ2-002` |

### Files to modify

- `crates/whirlpool-node/src/main.rs`

### Mock Boundary

- Keep verification at startup wiring or construction level.
- Do not add CLI/config surface, do not call `oracle_handle.update_validators(...)` directly from `main.rs`, and do not alter consensus startup behavior beyond provider input wiring.

### What to do

#### Phase 1 - Write or update failing tests first

1. Add or update startup wiring coverage in `crates/whirlpool-node/src/main.rs` for `TST-REQ2-002`.
2. Assert that `CommonwareNetworkProviderBuilder` receives both `initial_validators(...)` and `bootstrappers(...)` before `.build(...)` while namespace, max message size, and ephemeral listen/dial defaults remain unchanged.

```bash
nix develop --command cargo test -p whirlpool-node
```

#### Phase 2 - Implement node builder wiring

3. Reuse the already-derived startup validator set in `crates/whirlpool-node/src/main.rs` and convert it into the Commonware public-key form expected by `initial_validators(...)`.
4. Materialize the bootstrap peer list for this pass and pass it into `CommonwareNetworkProviderBuilder::bootstrappers(...)`.
5. Preserve oracle-handle lifetime management and existing startup defaults, and keep provider-side seeding centralized by not calling `oracle_handle.update_validators(...)` directly from `main.rs`.

```bash
nix develop --command cargo check -p whirlpool-node
```

### Acceptance Criteria

- `REQ-1`: node startup passes the initial validator set into the provider builder.
- `REQ-2`: node startup passes bootstrap peers into the provider builder without forcing behavior outside current defaults.
- `TST-REQ2-002` passes, demonstrating that validators and bootstrappers are provided together before build.

### Verification commands

```bash
nix develop --command cargo check -p whirlpool-node
nix develop --command cargo test -p whirlpool-node
```

Evidence: `.sisyphus/plans/p2p-provider-completeness/evidence/05-whirlpool-node-builder-wiring.log`
