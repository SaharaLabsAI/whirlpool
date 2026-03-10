# Test Contracts

## Scope
- Sub-Intent C only: `REQ-6`, `REQ-7`, and `REQ-8`.
- Primary test targets:
  - `crates/consensus-simplex/src/mailbox.rs`
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/p2p/src/types.rs`
  - `crates/p2p-commonware/src/provider.rs`
  - `crates/whirlpool-node/src/main.rs`
- Read-only vendor boundary under test-by-consumption:
  - `vendor/**`

## Test Strategy
- Framework:
  - crate-local unit tests in `crates/consensus-simplex`
  - focused transport tests in `crates/p2p` and `crates/p2p-commonware`
  - compatibility regression tests in `crates/whirlpool-node`
- Assertion style:
  - exact channel constant assertions
  - exact `BlockStore` state assertions by digest
  - deterministic send/receive assertions for payload channel transport
  - explicit backward-compatibility assertions for single-node startup
- Mocking approach:
  - prefer fake sender / in-memory sender capture for relay broadcast unit tests
  - use existing deterministic Commonware runtime tests for per-channel transport coverage
  - do not mock or alter vendor simplex internals
- Failure philosophy:
  - relay activation must fail safely on malformed or missing payload data
  - channel alignment regressions must be caught by exact-value tests
  - single-node behavior must remain valid after multi-node relay enablement

## Requirement Traceability
- `REQ-6` -> `TST-REQ6-001`, `TST-REQ6-002`, `TST-REQ6-003`
- `REQ-7` -> `TST-REQ7-001`, `TST-REQ7-002`
- `REQ-8` -> `TST-REQ8-001`, `TST-REQ8-002`

## Unit Tests

### `TST-REQ6-001` Mailbox relay broadcast sends cached payload to all peers
- Requirement: `REQ-6`
- Target file:
  - `crates/consensus-simplex/src/mailbox.rs`
- Test type: unit test with fake payload sender
- Setup:
  - create shared `BlockStore<TestBlock>`
  - insert a deterministic `TestBlock` under its digest
  - construct `Mailbox` with a fake sender that records outbound bytes and recipients
  - call `broadcast(digest)`
- Assertions:
  - exactly one outbound send occurs
  - recipients equal `Recipients::All`
  - outbound message is sent on the payload relay path
  - decoded payload envelope carries the expected digest and payload bytes
- Failure caught:
  - relay remains a no-op or sends the wrong payload bytes

### `TST-REQ6-002` Mailbox relay broadcast is safe when digest is absent
- Requirement: `REQ-6`
- Target file:
  - `crates/consensus-simplex/src/mailbox.rs`
- Test type: unit test
- Setup:
  - create empty `BlockStore<TestBlock>`
  - construct `Mailbox` with fake sender
  - call `broadcast(missing_digest)`
- Assertions:
  - no panic occurs
  - no outbound send is recorded
- Failure caught:
  - relay crashes or emits garbage when the local cache misses

### `TST-REQ6-003` Payload receive task stores valid inbound payload in `BlockStore`
- Requirement: `REQ-6`
- Target files:
  - `crates/consensus-simplex/src/engine.rs`
  - any helper module used for payload receive decoding
- Test type: unit test or focused async task test
- Setup:
  - construct a valid `PayloadRelayMessage` for a deterministic block
  - feed it through a fake or test receiver into the payload receive task
  - share an initially empty `BlockStore<TestBlock>`
- Assertions:
  - block is inserted under the expected digest
  - stored block equals the decoded input block
- Failure caught:
  - inbound payloads are received but never persisted for verification

## Transport and Alignment Tests

### `TST-REQ7-001` Channel constants remain aligned and additive
- Requirement: `REQ-7`
- Target file:
  - `crates/p2p/src/types.rs`
- Test type: unit test
- Assertions:
  - `Channel::VOTE == Channel(0)`
  - `Channel::CERTIFICATE == Channel(1)`
  - `Channel::RESOLVER == Channel(2)`
  - `Channel::PAYLOAD == Channel(3)`
- Failure caught:
  - accidental renumbering or aliasing of existing protocol channels

### `TST-REQ7-002` `start_per_channel()` returns and carries payload channel traffic
- Requirement: `REQ-7`
- Target file:
  - `crates/p2p-commonware/src/provider.rs`
- Test type: deterministic integration test
- Setup:
  - start two providers in the deterministic runtime
  - obtain `PerChannelNetwork` for each side
  - send bytes over `peer_0.payload.0`
  - read bytes from `peer_1.payload.1`
- Assertions:
  - payload bytes arrive intact
  - vote/certificate/resolver channel tests still pass unchanged
  - `PerChannelNetwork` exposes all four pairs
- Failure caught:
  - payload channel not registered, miswired, or breaking existing per-channel setup

## End-to-End and Compatibility Tests

### `TST-REQ8-001` Relay round-trip makes remote payload available for verification
- Requirement: `REQ-8`
- Target files:
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/p2p-commonware/src/provider.rs`
- Test type: end-to-end multi-node deterministic test
- Setup:
  - start two consensus-engine instances sharing the real payload channel transport
  - have node A propose a deterministic block
  - allow relay broadcast and payload receive task to run
  - trigger or observe node B verification against the relayed digest
- Assertions:
  - node B stores the relayed block in its `BlockStore`
  - node B can resolve the digest during verification
  - no vendor code change is required to complete the round-trip
- Failure caught:
  - relay path works in isolation but not through the full engine wiring

### `TST-REQ8-002` Single-node startup remains behaviorally valid with relay active
- Requirement: `REQ-8`
- Target files:
  - `crates/consensus-simplex/src/engine.rs`
  - `crates/whirlpool-node/src/main.rs`
- Test type: compatibility regression test
- Setup:
  - start the existing single-node path with no remote peers
- Assertions:
  - engine startup succeeds
  - no payload relay panic occurs when there are zero remote peers
  - existing finalization wiring and node startup tests continue to pass
- Failure caught:
  - relay activation introduces a regression in the current local-dev path

## Suggested Test Layout
- `crates/consensus-simplex/src/mailbox.rs` test module:
  - `tst_req6_001_broadcast_sends_cached_payload_to_all_peers`
  - `tst_req6_002_broadcast_missing_digest_is_noop`
- `crates/consensus-simplex/src/engine.rs` or helper-module tests:
  - `tst_req6_003_payload_receive_stores_block_in_block_store`
  - `tst_req8_001_relay_round_trip_populates_remote_verification_cache`
  - `tst_req8_002_single_node_startup_remains_valid`
- `crates/p2p/src/types.rs` test module:
  - `tst_req7_001_channel_constants_are_aligned`
- `crates/p2p-commonware/src/provider.rs` tests:
  - `tst_req7_002_start_per_channel_exposes_payload_pair`

## Completion Criteria
- Every in-scope requirement maps to at least one concrete `TST-*`.
- Relay broadcast, payload receive/store, and full round-trip are all covered.
- Channel constant alignment is tested explicitly.
- One test explicitly proves single-node backward compatibility after relay activation.
