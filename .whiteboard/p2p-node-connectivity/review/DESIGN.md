# Design Review Summary

## Verdict
- PASS for Sub-Intent C finalization.
- Scope remains correctly limited to `REQ-6`, `REQ-7`, and `REQ-8`.
- No blocking design gaps remain in the relay-activation pass.

## Reviewed Scope
- Primary implementation crate: `crates/consensus-simplex`
- Supporting additive crates:
  - `crates/p2p`
  - `crates/p2p-commonware`
- Compatibility boundary:
  - `crates/whirlpool-node`
- Verified source anchors:
  - `crates/consensus-simplex/src/mailbox.rs`
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/p2p/src/types.rs`
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/whirlpool-node/src/main.rs`
- Explicitly deferred:
  - any `vendor/**` modification
  - vendor simplex engine API redesign
  - any redesign of application finalization behavior
  - specialized payload gossip/backpressure policy beyond the additive dedicated channel

## Source-Grounded Findings
- `crates/consensus-simplex/src/mailbox.rs` currently wires `Mailbox` as both `Automaton` and `Relay`, but `broadcast(digest)` is a no-op, so application-level proposal payloads never leave the node.
- `crates/consensus-simplex/src/mailbox.rs` already stores locally created blocks into a shared `BlockStore`, so the required digest-to-block lookup surface already exists.
- `crates/consensus-simplex/src/engine.rs` already builds a shared `BlockStore` for mailbox and finalization integration, making it the correct place to also wire inbound payload persistence.
- `crates/p2p/src/types.rs` currently reserves only channels `0`, `1`, and `2`, matching vote/certificate/resolver traffic.
- `crates/p2p-commonware/src/provider.rs` currently registers exactly those three channels in `start_per_channel()`, so an additive payload channel is required to carry application payload bytes without disturbing vendor protocol streams.
- `crates/whirlpool-node/src/main.rs` already consumes `CommonwareEngine` as a sealed startup abstraction, so relay activation should remain internal to upstream crates rather than moving node-side logic into the binary.

## Decision Summary
- Add `Channel::PAYLOAD = Channel(3)` in `crates/p2p`.
- Extend `crates/p2p-commonware` `PerChannelNetwork` and `start_per_channel()` to register and expose a payload pair.
- Refactor `crates/consensus-simplex` `Mailbox` so `Relay::broadcast(digest)` looks up the cached block payload and sends it to `Recipients::All` over the payload path.
- Spawn a payload receive task in `CommonwareEngine::start()` that decodes inbound payload messages and stores validated blocks into the shared `BlockStore`.
- Keep vendor simplex engine integration unchanged at the call boundary: vote/certificate/resolver remain vendor-managed; payload is additive and application-owned.
- Preserve app/finalization compatibility by keeping `AppAdapter` and node startup semantics unchanged.

## Artifact Summary
- Agent-lane finalized outputs:
  - `docs/design/p2p-node-connectivity/agent/strategy.md`
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/consensus-simplex.md`
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/p2p.md`
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/p2p-commonware.md`
  - `docs/design/p2p-node-connectivity/agent/crate-contracts/whirlpool-node.md`
  - `docs/design/p2p-node-connectivity/agent/flows.md`
  - `docs/design/p2p-node-connectivity/agent/domains-wiring.md`
  - `docs/design/p2p-node-connectivity/agent/tests.md`
  - `docs/design/p2p-node-connectivity/agent/handoff.md`
- Supporting design inputs retained:
  - `docs/design/p2p-node-connectivity/agent/shared-intent-splits.md`
  - `docs/design/p2p-node-connectivity/agent/requirements.md`
- Review-lane outputs:
  - `docs/design/p2p-node-connectivity/review/DESIGN.md`
  - `docs/design/p2p-node-connectivity/review/INDEX.md`

## Review Checks
- Contract completeness: PASS
  - relay sender, payload receiver, channel registration, and compatibility boundaries are documented
  - preconditions, postconditions, failure handling, and no-vendor-change constraints are explicit
- Flow completeness: PASS
  - proposal-to-broadcast flow documented
  - payload receive-to-store flow documented
  - verification cache lookup relationship documented
  - startup wiring with additive payload task documented
- Testability: PASS
  - relay broadcast, payload persistence, channel alignment, end-to-end round-trip, and single-node compatibility all map to concrete `TST-*`
- Scope discipline: PASS
  - no source changes made in this design pass
  - no vendor redesign proposed
  - existing app/finalization flow preserved

## Residual Risks
- Exact block serialization/deserialization bounds may require a narrow tightening of local generic constraints in `crates/consensus-simplex`; this is an implementation detail, not a design blocker.
- The payload receive task must validate digest consistency carefully to avoid caching malformed blocks; this risk is already covered by explicit test contracts.
- Whether `NetworkProvider::start()` in `crates/p2p-commonware` also registers `PAYLOAD` for parity is optional for this sub-intent and does not block the dedicated per-channel design.
- These are implementation-time details only; the design remains PASS and execution-ready.
