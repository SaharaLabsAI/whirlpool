# Proof of Consensus Relay Activation (Sub-Intent C)

This document captures the acceptance criteria, local invariants, cross-sub-intent invariants, and QA scenarios for Sub-Intent C: Consensus Relay Activation.

## Acceptance Criteria

| ID | Description | Traceability | Verification Method |
|---|---|---|---|
| AC-C-1 | PAYLOAD channel constant (value=3) exists in p2p types | REQ-7 | Unit test `TST-REQ7-001` in `crates/p2p/src/types.rs` |
| AC-C-2 | p2p-commonware registers PAYLOAD channel and exposes it in PerChannelNetwork | REQ-7 | Unit test `TST-REQ7-002` in `crates/p2p-commonware/src/provider.rs` |
| AC-C-3 | Mailbox::broadcast(digest) looks up payload from BlockStore and sends via PAYLOAD sender | REQ-6 | Unit test `TST-REQ6-001` in `crates/consensus-simplex/src/mailbox.rs` |
| AC-C-4 | Inbound payload receiver task stores received payloads in BlockStore | REQ-6 | Unit test `TST-REQ6-003` in `crates/consensus-simplex/src/engine.rs` |
| AC-C-5 | End-to-end: propose on node A → broadcast → node B receives → verify succeeds | REQ-8 | Multi-node deterministic test `TST-REQ8-001` |
| AC-C-6 | Single-node backward compatibility: existing tests pass without peers | REQ-8 | Regression test `TST-REQ8-002` in `crates/whirlpool-node` |
| AC-C-7 | Channel constant alignment: p2p PAYLOAD=3 matches p2p-commonware registration | REQ-7 | Cross-crate constant value check in transport tests |

## Local Invariants

| ID | Description | Traceability | Verification Method |
|---|---|---|---|
| INV-C-1 | BlockStore is shared between Mailbox (write on propose) and relay (read on broadcast) | REQ-6 | Code audit of `CommonwareEngine::start` and `Mailbox::new` |
| INV-C-2 | BlockStore is shared between receiver task (write on inbound) and Automaton::verify (read) | REQ-6 | Code audit of `CommonwareEngine::start` and receiver task spawn |
| INV-C-3 | Relay broadcast is a no-op when no peers connected (graceful degradation) | REQ-8 | Unit test `TST-REQ8-002` and `TST-REQ6-001` with zero recipients |
| INV-C-4 | PAYLOAD channel uses same backlog/quota as vote/cert/resolver | REQ-7 | Code audit of `CommonwareNetworkProvider::start_per_channel` |
| INV-C-5 | Vendor engine's 3-channel interface unchanged (vote/cert/resolver) | REQ-7 | Compilation check: `simplex::Engine::start` call remains 3-argument |

## Cross-Sub-Intent Invariants

| ID | Description | Traceability | Verification Method |
|---|---|---|---|
| XINV-C-1 | p2p-commonware builder API from Sub-Intent A preserved (no breaking changes) | REQ-7 | Compilation of `whirlpool-node` using existing builder patterns |
| XINV-C-2 | NodeConfig from Sub-Intent B still drives all startup parameters | REQ-8 | Integration test ensuring `NodeConfig` values affect network setup |
| XINV-C-3 | PerChannelNetwork struct is backward-compatible (new field is additive) | REQ-7 | Compilation of existing code that ignores the `payload` field |

## QA Scenarios

| ID | Description | Traceability | Verification Method |
|---|---|---|---|
| QA-C-1 | Two-node consensus: both propose and relay, verify cross-node payload availability | REQ-8 | Multi-node simulation with dual proposers |
| QA-C-2 | Late joiner: node joins after blocks proposed, resolves via resolver channel | REQ-6 | Simulation where a third node joins after height 10 |
| QA-C-3 | Network partition: relay broadcast fails, node continues with local proposals | REQ-8 | Fault injection simulation dropping `PAYLOAD` channel traffic |
