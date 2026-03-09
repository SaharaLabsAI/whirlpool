## `01-receiver-channel-contract`

> Lock the receiver-owned channel contract first so all downstream provider and multiplex work can rely on `CommonwareReceiver` preserving real `p2p::Channel` values.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Dependencies** | none |
| **Wave** | 1 |
| **Complexity** | S |
| **Goal** | Implement the `REQ-3` receiver contract in `crates/p2p-commonware/src/receiver.rs` before any provider or node wiring changes |
| **Target Crate(s)** | `p2p-commonware` (primary implementation) |
| **Requirements** | `REQ-3` |
| **Tests** | `TST-REQ3-001`, `TST-REQ3-002` |

### Files to modify

- `crates/p2p-commonware/src/receiver.rs`

### Mock Boundary

- Use crate-local deterministic receiver/test doubles only.
- Do not modify `crates/p2p/**` or vendor Commonware transport code.

### What to do

#### Phase 1 - Write or update failing tests first

1. Add or update receiver-focused tests in `crates/p2p-commonware/src/receiver.rs` covering `TST-REQ3-001` and `TST-REQ3-002`.
2. Assert that `CommonwareReceiver::new(...)` requires the concrete `Channel` and that `recv()` preserves vote, certificate, and resolver channel identities alongside sender identity and payload bytes.

```bash
nix develop --command cargo test -p p2p-commonware receiver
```

#### Phase 2 - Implement the receiver contract

3. Change `CommonwareReceiver::new(...)` to accept the configured `p2p::Channel` and store it on the receiver struct.
4. Update `CommonwareReceiver::recv()` so every `NetworkMessage` uses the stored channel instead of any placeholder channel value.
5. Preserve the existing authenticated sender extraction and payload handling behavior while removing any `Channel(0)` fallback.

```bash
nix develop --command cargo check -p p2p-commonware
```

### Acceptance Criteria

- `REQ-3` is satisfied at the receiver boundary: `recv()` emits the configured `Channel::VOTE`, `Channel::CERTIFICATE`, or `Channel::RESOLVER` as appropriate.
- `TST-REQ3-001` and `TST-REQ3-002` pass in crate-local tests.
- The change remains confined to `crates/p2p-commonware/src/receiver.rs`.

### Verification commands

```bash
nix develop --command cargo check -p p2p-commonware
nix develop --command cargo test -p p2p-commonware receiver
```

Evidence: `.sisyphus/plans/p2p-provider-completeness/evidence/01-receiver-channel-contract.log`
