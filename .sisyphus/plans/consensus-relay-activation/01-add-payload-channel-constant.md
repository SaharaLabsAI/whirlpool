## `01-add-payload-channel-constant`

> Reserve the additive payload channel first so every downstream transport registration, relay send, and alignment test has a stable cross-crate constant to target.

| Field | Value |
|-------|-------|
| **Status** | `[ ]` |
| **Prerequisites** | `none` |
| **Wave** | 1 |
| **Complexity** | S |
| **Goal** | Add `Channel::PAYLOAD = Channel(3)` in `p2p` without disturbing the existing protocol channel IDs |
| **Target Crate(s)** | `p2p` |
| **Requirements** | `REQ-7` |
| **Acceptance IDs** | `AC-C-1`, `AC-C-7` |

### Files to modify

- `crates/p2p/src/types.rs`
- `crates/p2p/src/lib.rs` if export plumbing is required

### Pre-task gate

- `crates/p2p/src/types.rs` still exposes only `VOTE`, `CERTIFICATE`, and `RESOLVER`.
- No downstream code has started depending on a different payload channel number.
- Scope remains limited to additive channel reservation in `crates/p2p`.

### Acceptance criteria

- `AC-C-1`: `Channel::PAYLOAD` exists and equals `Channel(3)`.
- `AC-C-7`: existing channel IDs remain aligned as `0`, `1`, and `2` for vote/certificate/resolver.

### Requirements covered

- `REQ-7`

### Detailed implementation steps

1. Add `pub const PAYLOAD: Channel = Channel(3);` alongside the existing channel constants in `crates/p2p/src/types.rs`.
2. Preserve the exact values for `VOTE`, `CERTIFICATE`, and `RESOLVER`; do not reorder or renumber the existing constants in a way that obscures the stable mapping.
3. If `types.rs` exports are indirectly surfaced through `lib.rs`, update that file only as needed to keep the new constant reachable without changing the crate API shape.
4. Add or update the channel-alignment unit test for `TST-REQ7-001` so the crate asserts the full `0/1/2/3` mapping explicitly.
5. Avoid any sender, receiver, or trait redesign in this task.

### Test commands

```bash
nix develop --command cargo build
nix develop --command cargo test -p p2p
```

### Post-task gate

- `crates/p2p/src/types.rs` contains exactly one additive constant for payload routing.
- Channel alignment tests cover `VOTE`, `CERTIFICATE`, `RESOLVER`, and `PAYLOAD` explicitly.
- No files outside `crates/p2p` changed.
- Build and crate test commands pass before transport registration work begins.
