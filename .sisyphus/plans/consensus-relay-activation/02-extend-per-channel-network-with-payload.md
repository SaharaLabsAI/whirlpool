## `02-extend-per-channel-network-with-payload`

> Expose the new payload transport as a fourth dedicated channel pair before touching consensus relay logic so the vendor-facing engine boundary stays unchanged while the application-level relay gets its own path.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Prerequisites** | `01-add-payload-channel-constant` |
| **Wave** | 2 |
| **Complexity** | M |
| **Goal** | Extend `PerChannelNetwork` and `start_per_channel()` to register and return the dedicated payload sender/receiver pair |
| **Target Crate(s)** | `p2p-commonware` |
| **Requirements** | `REQ-6`, `REQ-7` |
| **Acceptance IDs** | `AC-C-2`, `AC-C-7` |

### Files to modify

- `crates/p2p-commonware/src/provider.rs`
- `crates/p2p-commonware/src/traits.rs`
- `crates/p2p-commonware/src/lib.rs` if struct exports require adjustment
- `crates/p2p-commonware` transport tests that cover `start_per_channel()`

### Pre-task gate

- Task 01 is complete and `Channel::PAYLOAD = 3` is available from `p2p`.
- `PerChannelNetwork<S, R>` still exposes only `vote`, `cert`, and `resolver`.
- Existing per-channel tests are green before adding the fourth pair.

### Acceptance criteria

- `AC-C-2`: `p2p-commonware` registers the payload channel and exposes it on `PerChannelNetwork`.
- `AC-C-7`: payload registration uses channel `3` while preserving vote/certificate/resolver alignment.

### Requirements covered

- `REQ-6`
- `REQ-7`

### Detailed implementation steps

1. Extend `PerChannelNetwork<S, R>` with `pub payload: (S, R)` while preserving the existing `vote`, `cert`, `resolver`, and `network_handle` fields.
2. Update the matching trait and return-shape documentation in `crates/p2p-commonware/src/traits.rs` so the dedicated-channel API now promises four pairs instead of three.
3. In `start_per_channel()`, register `Channel::PAYLOAD.0` using the same quota and backlog settings already used for vote/certificate/resolver unless a concrete compile-time constraint requires shared helper refactoring.
4. Update the builder logic so it constructs and returns the payload sender/receiver pair alongside the existing three pairs, without changing the generic builder API shape.
5. Add or update deterministic transport coverage for `TST-REQ7-002` to prove one peer can send bytes over `.payload.0` and another can receive them from `.payload.1` intact.
6. Keep payload support transport-only; do not decode relay envelopes or introduce consensus block logic in this task.

### Test commands

```bash
nix develop --command cargo build
nix develop --command cargo test -p p2p-commonware
```

### Post-task gate

- `PerChannelNetwork` exposes four dedicated channel pairs.
- `start_per_channel()` registers channels `0`, `1`, `2`, and `3` without repurposing existing transport semantics.
- Transport tests prove the payload pair works and existing channel tests keep their original meaning.
- No files under `vendor/` or outside `crates/p2p-commonware` changed.
