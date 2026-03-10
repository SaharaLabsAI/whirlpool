# consensus-relay-activation - Execution Plan

## TL;DR

| Field | Value |
|-------|-------|
| **Summary** | Complete Sub-Intent C by activating consensus payload relay across `p2p`, `p2p-commonware`, `consensus-simplex`, and compatible `whirlpool-node` startup wiring |
| **Deliverables** | Entry plan file, this index, and 6 serial task files covering payload channel reservation, transport exposure, outbound relay activation, inbound persistence, engine wiring, and final verification |
| **Effort** | 6 tasks, 6 waves, estimated M complexity overall |
| **Parallel** | none; transport and relay wiring have strict serial dependencies |
| **Critical Path** | `01` -> `02` -> `03` -> `04` -> `05` -> `06` |

## Context

### Original Request

Generate the `.sisyphus` execution plan for Sub-Intent C `consensus-relay-activation`, scoped only to `REQ-6`, `REQ-7`, and `REQ-8`.

### Grounding Summary

- `docs/design/p2p-node-connectivity/agent/strategy.md` fixes Sub-Intent C scope to additive relay activation without vendor changes or `Relay` trait redesign.
- `docs/design/p2p-node-connectivity/agent/handoff.md` fixes the implementation order: reserve payload channel, expose it in `p2p-commonware`, activate mailbox relay, wire payload receive in the engine, then verify end-to-end compatibility.
- `docs/design/p2p-node-connectivity/agent/crate-contracts/consensus-simplex.md` defines the relay contract: `Mailbox::broadcast(digest)` must read from shared `BlockStore`, send `PayloadRelayMessage` to `Recipients::All`, and fail safely.
- `docs/design/p2p-node-connectivity/agent/crate-contracts/p2p.md` and `docs/design/p2p-node-connectivity/agent/crate-contracts/p2p-commonware.md` require `PAYLOAD = 3` plus a fourth dedicated channel pair on `PerChannelNetwork`.
- `docs/design/p2p-node-connectivity/agent/tests.md` defines the required proof points: `TST-REQ6-001..003`, `TST-REQ7-001..002`, and `TST-REQ8-001..002`.
- The acceptance criteria for this sub-intent are `AC-C-1` through `AC-C-7`, covering channel reservation, transport registration, broadcast, inbound persistence, end-to-end relay, single-node compatibility, and cross-crate alignment.

### Scope Guardrails

- In scope: `crates/p2p/`, `crates/p2p-commonware/`, `crates/consensus-simplex/`, and narrow compatibility-preserving changes in `crates/whirlpool-node/` only if required by upstream contract tightening.
- Out of scope: `vendor/**`, any redesign of vote/certificate/resolver transport, any change to the vendor `Relay` trait or `simplex::Engine::start(...)`, and any redesign of finalization semantics.
- Every task must keep payload support strictly additive on channel `3` and preserve existing protocol channel IDs `0`, `1`, and `2`.

## Work Objectives

### Core Objective

Deliver an executable serial implementation plan for `REQ-6`, `REQ-7`, and `REQ-8` so consensus proposal payloads are broadcast over a dedicated payload channel, persisted on receipt into the shared `BlockStore`, and consumed by existing verification/finalization wiring without vendor edits.

### Deliverables

- Entry file: `.sisyphus/plans/consensus-relay-activation.md`
- Plan index: `.sisyphus/plans/consensus-relay-activation/INDEX.md`
- Task 01: `.sisyphus/plans/consensus-relay-activation/01-add-payload-channel-constant.md`
- Task 02: `.sisyphus/plans/consensus-relay-activation/02-extend-per-channel-network-with-payload.md`
- Task 03: `.sisyphus/plans/consensus-relay-activation/03-activate-mailbox-payload-broadcast.md`
- Task 04: `.sisyphus/plans/consensus-relay-activation/04-add-inbound-payload-receiver-task.md`
- Task 05: `.sisyphus/plans/consensus-relay-activation/05-wire-relay-through-commonware-engine.md`
- Task 06: `.sisyphus/plans/consensus-relay-activation/06-final-verification-and-cleanup.md`

### Definition of Done

```bash
nix develop --command cargo build
nix develop --command cargo test -p p2p
nix develop --command cargo test -p p2p-commonware
nix develop --command cargo test -p consensus-simplex
nix develop --command cargo test -p whirlpool-node
```

### Must Have

- Strict serial execution with one task per wave.
- Explicit traceability to `REQ-6`, `REQ-7`, `REQ-8`, and relevant `AC-C-*` identifiers in every task.
- Pre-task and post-task gates for every task.
- Exact file lists per task.
- Payload relay work stays additive and preserves vendor boundaries.
- Final verification includes a scope audit confirming no `vendor/` modifications.

### Must NOT Have

- No source edits under `vendor/`.
- No task that renumbers or reuses `VOTE`, `CERTIFICATE`, or `RESOLVER`.
- No design-doc edits.
- No scope expansion beyond relay activation for Sub-Intent C.
- No parallel task waves.

## Execution Strategy

### Parallel Execution Waves

- Wave 1: Task 01 only
- Wave 2: Task 02 only
- Wave 3: Task 03 only
- Wave 4: Task 04 only
- Wave 5: Task 05 only
- Wave 6: Task 06 only

### Dependency Matrix

| Task | Depends On | Wave |
|------|------------|------|
| 01-add-payload-channel-constant | none | 1 |
| 02-extend-per-channel-network-with-payload | 01-add-payload-channel-constant | 2 |
| 03-activate-mailbox-payload-broadcast | 02-extend-per-channel-network-with-payload | 3 |
| 04-add-inbound-payload-receiver-task | 03-activate-mailbox-payload-broadcast | 4 |
| 05-wire-relay-through-commonware-engine | 04-add-inbound-payload-receiver-task | 5 |
| 06-final-verification-and-cleanup | 05-wire-relay-through-commonware-engine | 6 |

## Tasks

- [x] Task 1: Add PAYLOAD channel constant to p2p crate [**S**] -> [01-add-payload-channel-constant.md](01-add-payload-channel-constant.md)
- [x] Task 2: Extend PerChannelNetwork and expose payload transport [**M**] -> [02-extend-per-channel-network-with-payload.md](02-extend-per-channel-network-with-payload.md)
- [x] Task 3: Activate Mailbox payload broadcast [**M**] -> [03-activate-mailbox-payload-broadcast.md](03-activate-mailbox-payload-broadcast.md)
- [x] Task 4: Add inbound payload receiver task [**M**] -> [04-add-inbound-payload-receiver-task.md](04-add-inbound-payload-receiver-task.md)
- [x] Task 5: Wire relay through CommonwareEngine [**M**] -> [05-wire-relay-through-commonware-engine.md](05-wire-relay-through-commonware-engine.md)
- [x] Task 6: Final verification and cleanup [**S**] -> [06-final-verification-and-cleanup.md](06-final-verification-and-cleanup.md)

## Traceability Map

| Requirement | Acceptance Criteria | Planned Tasks |
|-------------|---------------------|---------------|
| `REQ-6` | `AC-C-3`, `AC-C-4` | `03`, `04`, `05`, `06` |
| `REQ-7` | `AC-C-1`, `AC-C-2`, `AC-C-7` | `01`, `02`, `06` |
| `REQ-8` | `AC-C-5`, `AC-C-6` | `04`, `05`, `06` |

## Scope

Only: `crates/p2p/`, `crates/p2p-commonware/`, `crates/consensus-simplex/`, `crates/whirlpool-node/`
No vendor modifications.

## DoD

- `nix develop --command cargo build` passes
- `nix develop --command cargo test -p p2p` passes
- `nix develop --command cargo test -p p2p-commonware` passes
- `nix develop --command cargo test -p consensus-simplex` passes
- `nix develop --command cargo test -p whirlpool-node` passes
- No vendor code modified

## Final Verification

The finishing gate is Task 06, which requires the full build plus package test matrix and a scope audit confirming payload relay stayed within the allowed crates, preserved channel alignment, and left `vendor/` untouched.
