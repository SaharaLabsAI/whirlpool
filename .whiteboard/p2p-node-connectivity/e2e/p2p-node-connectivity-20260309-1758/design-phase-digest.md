# Design Phase Digest — Sub-Intent A: P2P Provider Completeness

## Summary
Design synthesis and finalization complete for Sub-Intent A (REQ-1, REQ-2, REQ-3). All agent-lane and review-lane artifacts produced. Verdict: PASS.

## Key Findings
- **3 bugs, 3 fixes**: validator seeding via oracle_handle in provider.rs build(), bootstrap threading into discovery::Config, channel metadata stored on CommonwareReceiver construction
- **Primary crate**: p2p-commonware (5 files touched: provider.rs, receiver.rs, sender.rs, lib.rs, traits.rs)
- **Integration crate**: whirlpool-node (main.rs — bootstrap + validator wiring only)
- **Stable boundary**: crates/p2p traits unchanged
- **No vendor changes**

## Files Produced
- agent/strategy.md, crates.md, workspace.md, domains.md, blockers.md (PASS)
- agent/crate-contracts/p2p-commonware.md, whirlpool-node.md
- agent/flows.md, tests.md (7 TST-* items), handoff.md
- agent/TASK_GEN_READY.md (READY)
- review/DESIGN.md (PASS), review/INDEX.md

## Implementation Order (from handoff)
1. receiver.rs — channel metadata fix (REQ-3)
2. provider.rs — validator seeding + bootstrap (REQ-1, REQ-2)
3. lib.rs — MultiplexReceiver alignment (REQ-3)
4. sender.rs + traits.rs — compatibility check
5. main.rs — node startup wiring (REQ-1, REQ-2)
6. Tests (TST-REQ1-001/002, TST-REQ2-001/002, TST-REQ3-001/002/003)
