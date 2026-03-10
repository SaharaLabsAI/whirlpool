## `03-activate-mailbox-payload-broadcast`

> Replace the mailbox relay no-op only after the payload transport exists, so outbound consensus relay can stay a small additive lookup-and-send path over the shared `BlockStore`.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Prerequisites** | `02-extend-per-channel-network-with-payload` |
| **Wave** | 3 |
| **Complexity** | M |
| **Goal** | Give `Mailbox` access to shared payload egress and implement `broadcast(digest)` as a safe block lookup plus `Recipients::All` send |
| **Target Crate(s)** | `consensus-simplex` |
| **Requirements** | `REQ-6`, `REQ-7` |
| **Acceptance IDs** | `AC-C-3` |

### Files to modify

- `crates/consensus-simplex/src/mailbox.rs`
- Any crate-local helper module introduced for payload relay message encoding if needed
- `crates/consensus-simplex` unit tests for mailbox relay behavior

### Pre-task gate

- Task 02 is complete and `per_channel.payload` is available from the transport layer.
- `crates/consensus-simplex/src/mailbox.rs` still contains the current no-op `Relay::broadcast()` baseline.
- The shared `BlockStore` contract used by `MailboxActor::remember_block()` remains unchanged.

### Acceptance criteria

- `AC-C-3`: `Mailbox::broadcast(digest)` looks up the payload from `BlockStore` and sends it through the payload sender to all peers.

### Requirements covered

- `REQ-6`
- `REQ-7`

### Detailed implementation steps

1. Update `Mailbox::new(...)` so mailbox instances receive the mailbox command sender plus shared `BlockStore<B>` and an optional payload sender or typed adapter compatible with the new payload transport.
2. Add the relay-specific mailbox fields needed to share block lookup state and outbound send capability across clones, preserving backward compatibility by allowing the sender dependency to be absent or no-op when not wired.
3. Introduce the crate-local `PayloadRelayMessage` envelope or equivalent helper required to serialize the digest together with the full block payload bytes.
4. Implement `Relay::broadcast(&mut self, digest)` so it loads the block from `BlockStore`, encodes the message, and sends it to `Recipients::All` over the payload path.
5. Handle absent digests, serialization failures, and send failures with tracing/logging plus early return; do not panic and do not touch vote/certificate/resolver channels.
6. Add mailbox-focused tests covering `TST-REQ6-001` and `TST-REQ6-002`, including a fake sender assertion that exactly one outbound payload message is produced for the happy path and none for the missing-digest path.

### Test commands

```bash
nix develop --command cargo build
nix develop --command cargo test -p consensus-simplex
```

### Post-task gate

- `Relay::broadcast()` is no longer a no-op when a payload sender is configured.
- The broadcast path reads from the shared `BlockStore` and targets `Recipients::All` only on the payload relay path.
- Missing digest and send/encode failure cases remain non-panicking.
- Mailbox unit tests cover both successful broadcast and guarded no-send behavior.
