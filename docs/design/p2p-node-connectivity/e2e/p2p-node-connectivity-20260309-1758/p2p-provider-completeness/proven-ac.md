# Proven Acceptance Criteria — P2P Provider Completeness

AC_VERSION: 1

## Acceptance Criteria

| ID | Criterion | REQ | Verification |
|----|-----------|-----|-------------|
| AC-1 | `CommonwareNetworkProviderBuilder::build()` applies `initial_validators` to `OracleHandle` before returning provider | REQ-1 | TST-REQ1-001 |
| AC-2 | `CommonwareNetworkProviderBuilder::build()` configures `discovery::Config` with `bootstrappers` | REQ-2 | TST-REQ2-001 |
| AC-3 | `CommonwareReceiver::recv()` emits `NetworkMessage` with the channel ID configured at construction (not hard-coded `Channel(0)`) | REQ-3 | TST-REQ3-001, TST-REQ3-002 |
| AC-4 | `whirlpool-node` startup populates builder with both validators and bootstrappers from config | REQ-1, REQ-2 | TST-REQ2-002 |
| AC-5 | `MultiplexReceiver` forwards already-tagged messages without overwriting channel metadata | REQ-3 | TST-REQ3-003 |

## QA Scenarios

| ID | Scenario | Covers | Expected Outcome |
|----|----------|--------|-----------------|
| QA-1 | Two nodes start with each other's addresses as bootstrappers and validators | AC-1, AC-2, AC-4 | Both nodes discover and connect to each other |
| QA-2 | Node starts with populated validators; consensus sends vote on VOTE channel | AC-1, AC-3, AC-5 | Vote arrives at peer with correct Channel tag |
| QA-3 | Multiplexed messages on VOTE, CERTIFICATE, RESOLVER channels arrive correctly | AC-3, AC-5 | Each message tagged with its origin channel, not Channel(0) |
| QA-4 | Node starts with empty validator list and empty bootstrappers | AC-1, AC-2 | No crash; node runs in isolated mode, oracle remains empty |
| QA-5 | Node receives message on unknown channel ID | AC-3 | Message dropped or error logged; no panic |

## Invariants

| ID | Statement | Source | Verification |
|----|-----------|--------|-------------|
| INV-1 | `crates/p2p` trait API (`PeerId`, `NetworkSender`, `NetworkReceiver`, `NetworkProvider`) unchanged | strategy.md | Compile-time: no modifications to p2p/src/traits.rs |
| INV-2 | `CommonwareReceiver` preserves Channel identity through `recv()` | crate-contracts/p2p-commonware.md | TST-REQ3-001, TST-REQ3-002 |
| INV-3 | Validator seeding occurs in `build()` before provider handoff | crate-contracts/p2p-commonware.md | TST-REQ1-001 |
| INV-4 | Bootstrap peers threaded into `discovery::Config` | crate-contracts/p2p-commonware.md | TST-REQ2-001 |
| INV-5 | `whirlpool-node/main.rs` is sole authority for startup validator/bootstrap selection | crate-contracts/whirlpool-node.md | Code review |
| INV-6 | Empty validator/bootstrap lists handled without panic | strategy.md | TST-REQ1-002 |
| INV-7 | Channel constants (`VOTE`/`CERTIFICATE`/`RESOLVER`) from `p2p` are canonical source | domains.md | Compile-time: imports from `p2p::types` |
