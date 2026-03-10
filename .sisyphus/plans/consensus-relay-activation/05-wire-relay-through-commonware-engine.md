## `05-wire-relay-through-commonware-engine`

> Join the outbound and inbound relay pieces in `CommonwareEngine::start()` while preserving the vendor engine's exact three-channel startup contract and existing node integration boundary.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Prerequisites** | `04-add-inbound-payload-receiver-task` |
| **Wave** | 5 |
| **Complexity** | M |
| **Goal** | Extract the payload channel from `PerChannelNetwork`, pass payload egress into `Mailbox`, spawn the receive task, and keep node startup compatibility intact |
| **Target Crate(s)** | `consensus-simplex`, `whirlpool-node` |
| **Requirements** | `REQ-6`, `REQ-8` |
| **Acceptance IDs** | `AC-C-5`, `AC-C-6` |

### Files to modify

- `crates/consensus-simplex/src/engine.rs`
- `crates/consensus-simplex/src/mailbox.rs` if constructor wiring needs final adjustment
- `crates/whirlpool-node/src/main.rs` only if narrow compatibility updates are forced by upstream bounds or tests
- `crates/whirlpool-node` and/or `crates/consensus-simplex` integration tests for relay round-trip and single-node compatibility

### Pre-task gate

- Tasks 01 through 04 are complete.
- `network.start_per_channel()` now yields `payload`, but `CommonwareEngine::start()` is not yet consuming it end-to-end.
- The vendor simplex engine call boundary is still unchanged and must remain so.

### Acceptance criteria

- `AC-C-5`: the full path from proposal broadcast to remote payload availability for verification is wired through the engine.
- `AC-C-6`: single-node startup remains behaviorally valid with relay enabled.

### Requirements covered

- `REQ-6`
- `REQ-8`

### Detailed implementation steps

1. Update `CommonwareEngine::start()` to destructure `per_channel.payload` before passing the remaining `vote`, `cert`, and `resolver` pairs to the vendor simplex engine.
2. Construct `Mailbox` with the shared `BlockStore` and payload sender so proposal broadcast can use the same digest-indexed cache already shared with `MailboxActor` and `AppAdapter`.
3. Spawn the payload receive helper task with the payload receiver and shared `BlockStore`, ensuring its lifetime is tied to the running engine and that startup sequencing remains deterministic.
4. Keep the vendor call exactly `simplex::Engine::start(per_channel.vote, per_channel.cert, per_channel.resolver)` or its equivalent; do not add payload to the vendor interface.
5. Add or update end-to-end coverage for `TST-REQ8-001` so a deterministic multi-node scenario proves the relayed payload becomes available in the remote verification cache.
6. Add or update compatibility coverage for `TST-REQ8-002` so the existing single-node path still starts successfully and does not panic when no remote peers exist.
7. Touch `crates/whirlpool-node` only if compilation or test harness updates require narrow compatibility-preserving edits; payload relay logic must stay outside the node binary.

### Test commands

```bash
nix develop --command cargo build
nix develop --command cargo test -p consensus-simplex
nix develop --command cargo test -p whirlpool-node
```

### Post-task gate

- `CommonwareEngine::start()` owns both payload sender injection and payload receive task startup.
- The vendor engine still consumes only vote/cert/resolver channels.
- Deterministic relay round-trip coverage exists for remote verification availability.
- Single-node compatibility remains proven without moving relay logic into `whirlpool-node`.
