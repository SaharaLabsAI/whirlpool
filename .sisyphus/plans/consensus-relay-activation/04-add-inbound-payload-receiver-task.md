## `04-add-inbound-payload-receiver-task`

> Complete the relay data path by persisting valid inbound payloads into the shared verification cache before wiring the task into the full engine startup flow.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Prerequisites** | `03-activate-mailbox-payload-broadcast` |
| **Wave** | 4 |
| **Complexity** | M |
| **Goal** | Add an async payload receive loop that decodes inbound relay messages, validates digests, and stores accepted blocks in `BlockStore` |
| **Target Crate(s)** | `consensus-simplex` |
| **Requirements** | `REQ-6`, `REQ-8` |
| **Acceptance IDs** | `AC-C-4` |

### Files to modify

- `crates/consensus-simplex/src/engine.rs`
- Any crate-local helper module used by payload decoding/persistence
- `crates/consensus-simplex` async or unit tests covering payload receive behavior

### Pre-task gate

- Task 03 is complete and outbound payload message encoding exists in `consensus-simplex`.
- `engine.rs` does not yet persist inbound payload frames into the shared `BlockStore`.
- The chosen block encoding path can serialize and deserialize `A::Block` deterministically or the required local bounds/helpers are identified.

### Acceptance criteria

- `AC-C-4`: inbound payload receiver task stores received payloads in `BlockStore` only after decoding and digest validation succeed.

### Requirements covered

- `REQ-6`
- `REQ-8`

### Detailed implementation steps

1. Add a crate-local async helper in or near `crates/consensus-simplex/src/engine.rs` that owns the payload receiver loop and shared `BlockStore<A::Block>`.
2. For each inbound frame, decode `PayloadRelayMessage`, decode the enclosed block payload bytes into `A::Block`, recompute the block digest, and compare it with the envelope digest before storage.
3. On success, insert the decoded block into `BlockStore` under the validated digest so later verification can resolve it.
4. On malformed frame, payload decode failure, or digest mismatch, emit tracing/logging and continue the loop without panicking or terminating the engine.
5. Add tests for `TST-REQ6-003` that feed a deterministic payload message through the receiver task and assert the block is persisted under the expected digest; include at least one malformed or mismatched case if practical within the crate test harness.
6. Keep this task focused on the helper and tests; full engine startup ownership of the receiver task lands in Task 05.

### Test commands

```bash
nix develop --command cargo build
nix develop --command cargo test -p consensus-simplex
```

### Post-task gate

- The codebase contains a concrete payload receive loop that validates digest-to-block consistency before writing to `BlockStore`.
- Error handling drops malformed inputs safely and keeps the loop alive.
- Tests prove a valid inbound payload is persisted for later verification.
- Engine startup wiring changes are still limited to helper-level preparation until Task 05.
